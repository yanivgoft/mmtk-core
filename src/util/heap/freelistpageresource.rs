use std::sync::{Mutex, RwLock};
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use util::address::Address;
use util::heap::pageresource::{CommonPageResource, SpaceMemoryMeta};
use util::heap::layout::vm_layout_constants::*;
use util::heap::layout::freelist::Freelist;
use util::constants::*;
use policy::space::Space;
use super::PageResource;



#[derive(Debug)]
pub struct FreeListPageResource {
    common: CommonPageResource,
    freelist: Mutex<Freelist>,
}



impl PageResource for FreeListPageResource {
    unsafe fn unsafe_common(&self) -> *mut CommonPageResource {
        &self.common as *const _ as usize as *mut _
    }

    fn alloc_pages(&self, reserved_pages: usize, required_pages: usize, zeroed: bool, space: &impl Space, tls: *mut ::libc::c_void) -> Option<Address> {
        match self.alloc_pages_aux(reserved_pages, required_pages, zeroed, false, tls) {
            Some((rtn, new_chunk)) => {
                space.grow_space(rtn, ::util::conversions::pages_to_bytes(required_pages), new_chunk);
                Some(rtn)
            },
            _ => None
        }
    }

    fn release_pages(&self, first: Address) -> usize {
        debug_assert!(::util::conversions::is_page_aligned(first));
        let page_index = ::util::conversions::bytes_to_pages(first.as_usize());
        let mut freelist = self.freelist.lock().unwrap();
        let pages = freelist.get_size(page_index);
        self.common.reserved.fetch_sub(pages, Ordering::Relaxed);
        self.common.committed.fetch_sub(pages, Ordering::Relaxed);
        let freed = freelist.dealloc(page_index);
        // Try free this chunk
        if self.common.metadata_pages_per_region > 0 {
            let chunk = ::util::conversions::chunk_align(first, true);
            let first_unit = ::util::conversions::bytes_to_pages(chunk.as_usize());
            if freelist.get_coalescable_size(first_unit) == PAGES_IN_CHUNK {
                freelist.dealloc(first_unit);
                freelist.remove(first_unit, PAGES_IN_CHUNK);
                self.free_chunk(&mut freelist, chunk);
            }
        } else if freed == PAGES_IN_CHUNK {
            self.free_chunk(&mut freelist, ::util::conversions::chunk_align(first, true));
        }
        freed
    }

    fn release_all(&self) {
        unimplemented!()
    }

    fn new_contiguous(start: Address, bytes: usize, metadata_pages_per_region: usize, space_descriptor: usize) -> Self {
        let mut freelist = Freelist::new();
        let page_index = ::util::conversions::bytes_to_pages(start.as_usize());
        let count = ::util::conversions::bytes_to_pages(bytes);
        freelist.insert_free(page_index, count);
        Self {
            common: CommonPageResource {
                reserved: AtomicUsize::new(0),
                committed: AtomicUsize::new(0),
                space_descriptor,
                metadata_pages_per_region,
                memory: SpaceMemoryMeta::Contiguous { start, extent: bytes },
            },
            freelist: Mutex::new(freelist),
        }
    }

    fn new_discontiguous(metadata_pages_per_region: usize, space_descriptor: usize) -> Self {
        Self {
            common: CommonPageResource {
                reserved: AtomicUsize::new(0),
                committed: AtomicUsize::new(0),
                space_descriptor,
                metadata_pages_per_region,
                memory: SpaceMemoryMeta::Discontiguous { head: RwLock::new(unsafe { Address::zero() }) },
            },
            freelist: Mutex::new(Freelist::new()),
        }
    }
}

impl FreeListPageResource {
    fn alloc_pages_aux(&self, reserved_pages: usize, required_pages: usize, zeroed: bool, is_retrial: bool, tls: *mut ::libc::c_void) -> Option<(Address, bool)> {
        let mut freelist = self.freelist.lock().unwrap();
        match freelist.alloc(required_pages) {
            Some(page_index) => {
                self.commit_pages(reserved_pages, required_pages, tls);
                let page_address = unsafe { Address::from_usize(page_index << LOG_BYTES_IN_PAGE) };
                if zeroed {
                    unsafe { ::std::ptr::write_bytes::<u8>(page_address.to_ptr_mut(), 0, required_pages << LOG_BYTES_IN_PAGE) }
                }
                Some((page_address, false))
            },
            _ => {
                if let SpaceMemoryMeta::Contiguous {..} = self.common().memory {
                    return None;
                }
                if is_retrial {
                    return None;
                }
                let required_chunks = ::policy::space::required_chunks(required_pages);
                match self.allocate_contiguous_chunks(required_chunks) {
                    Some(chunk_start) => {
                        let page_index = chunk_start.as_usize() >> LOG_BYTES_IN_PAGE;
                        let pages = (required_chunks << LOG_BYTES_IN_CHUNK) >> LOG_BYTES_IN_PAGE;
                        freelist.insert_free(page_index, pages);
                        freelist.alloc_from(page_index, self.common.metadata_pages_per_region).unwrap();
                        ::std::mem::drop(freelist);
                        self.alloc_pages_aux(reserved_pages, required_pages, zeroed, true, tls).map(|(a, _)| (a, true))
                    },
                    _ => return None
                }
            }
        }
    }

    fn free_chunk(&self, freelist: &mut Freelist, chunk: Address) {
        let first_unit = ::util::conversions::bytes_to_pages(chunk.as_usize());
        freelist.remove(first_unit, PAGES_IN_CHUNK);
        self.release_contiguous_chunks(chunk);
    }
}