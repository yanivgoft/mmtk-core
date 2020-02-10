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
    base: Address,
}



impl PageResource for FreeListPageResource {
    unsafe fn unsafe_common(&self) -> *mut CommonPageResource {
        &self.common as *const _ as usize as *mut _
    }

    fn alloc_pages(&self, reserved_pages: usize, required_pages: usize, zeroed: bool, space: &impl Space, tls: *mut ::libc::c_void) -> Option<Address> {
        let mut freelist = self.freelist.lock().unwrap();
        match self.alloc_pages_impl(reserved_pages, required_pages, zeroed, false, tls, &mut freelist) {
            Some((rtn, new_chunk)) => {
                space.grow_space(rtn, ::util::conversions::pages_to_bytes(required_pages), new_chunk);
                Some(rtn)
            },
            _ => None
        }
    }

    fn release_pages(&self, first: Address) -> usize {
        debug_assert!(::util::conversions::is_page_aligned(first));
        let page_index = self.get_page_index(first);
        let mut freelist = self.freelist.lock().unwrap();
        let pages = freelist.get_size(page_index);
        self.common.reserved.fetch_sub(pages, Ordering::Relaxed);
        self.common.committed.fetch_sub(pages, Ordering::Relaxed);
        let (freed, coalesced_size) = freelist.dealloc(page_index);
        // Try free this chunk
        if coalesced_size + self.common.metadata_pages_per_region == PAGES_IN_CHUNK {
            let chunk = ::util::conversions::chunk_align(first, true);
            let first_unit = self.get_page_index(chunk);
            if self.common.metadata_pages_per_region > 0 {
                let (_, total) = freelist.dealloc(first_unit);
                debug_assert!(total == PAGES_IN_CHUNK);
            }
            freelist.remove(first_unit, PAGES_IN_CHUNK);
            self.release_contiguous_chunks(chunk);
        }
        freed
    }

    fn release_all(&self) {
        unimplemented!()
    }

    fn new_contiguous(start: Address, bytes: usize, metadata_pages_per_region: usize, space_descriptor: usize) -> Self {
        let mut freelist = Freelist::new(Some(LOG_BYTES_IN_CHUNK - LOG_BYTES_IN_PAGE as usize));
        let count = ::util::conversions::bytes_to_pages(bytes);
        freelist.insert_free(0, count);
        Self {
            common: CommonPageResource {
                reserved: AtomicUsize::new(0),
                committed: AtomicUsize::new(0),
                space_descriptor,
                metadata_pages_per_region,
                memory: SpaceMemoryMeta::Contiguous { start, extent: bytes },
            },
            freelist: Mutex::new(freelist),
            base: start,
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
            freelist: Mutex::new(Freelist::new(Some(LOG_BYTES_IN_CHUNK - LOG_BYTES_IN_PAGE as usize))),
            base: ::util::heap::layout::heap_layout::VM_MAP.heap_range.0,
        }
    }
}

impl FreeListPageResource {
    #[inline(always)]
    fn alloc_pages_impl(&self, reserved_pages: usize, required_pages: usize, zeroed: bool, is_retrial: bool, tls: *mut ::libc::c_void, freelist: &mut Freelist) -> Option<(Address, bool)> {
        match freelist.alloc(required_pages) {
            Some(page_index) => {
                self.commit_pages(reserved_pages, required_pages, tls);
                let page_address = self.get_page_address(page_index);
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
                        let page_index = self.get_page_index(chunk_start);
                        let pages = (required_chunks << LOG_BYTES_IN_CHUNK) >> LOG_BYTES_IN_PAGE;
                        freelist.insert_free(page_index, pages);
                        freelist.alloc_from(page_index, self.common.metadata_pages_per_region).unwrap();
                        self.alloc_pages_impl(reserved_pages, required_pages, zeroed, true, tls, freelist).map(|(a, _)| (a, true))
                    },
                    _ => return None
                }
            }
        }
    }

    fn get_page_index(&self, page: Address) -> usize {
        (page - self.base) >> LOG_BYTES_IN_PAGE
    }

    fn get_page_address(&self, index: usize) -> Address {
        self.base + (index << LOG_BYTES_IN_PAGE)
    }
}