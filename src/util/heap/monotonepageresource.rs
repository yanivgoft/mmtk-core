use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;
use ::util::address::Address;
use ::policy::space::Space;
use ::util::heap::pageresource::CommonPageResource;
use ::util::heap::layout::vm_layout_constants::LOG_BYTES_IN_CHUNK;
use util::constants::*;
use super::layout::heap_layout::VM_MAP;
use super::PageResource;
use std::sync::atomic::Ordering;



#[derive(Debug)]
pub struct MonotonePageResource {
    common: CommonPageResource,
    alloc_chunk: Mutex<(Address, Address)>,
}

impl PageResource for MonotonePageResource {
    unsafe fn unsafe_common(&self) -> *mut CommonPageResource {
        &self.common as *const _ as usize as *mut _
    }

    fn alloc_pages(&self, reserved_pages: usize, required_pages: usize, zeroed: bool, space: &impl Space, tls: *mut ::libc::c_void) -> Option<Address> {
        let mut alloc_chunk = self.alloc_chunk.lock().unwrap();
        let bytes = required_pages << LOG_BYTES_IN_PAGE;
        let addr = if alloc_chunk.0 + bytes > alloc_chunk.1 {
            let required_chunks = ::policy::space::required_chunks(required_pages);
            let chunk_start = match self.allocate_contiguous_chunks(required_chunks) {
                Some(c) => c,
                _ => return None,
            };
            let chunk_limit = chunk_start + (required_chunks << LOG_BYTES_IN_CHUNK);
            let data_start = chunk_start + (self.common.metadata_pages_per_region << LOG_BYTES_IN_PAGE);
            *alloc_chunk = (data_start + (required_pages << LOG_BYTES_IN_PAGE), chunk_limit);
            data_start
        } else {
            let a = alloc_chunk.0;
            alloc_chunk.0 = alloc_chunk.0 + bytes;
            space.grow_space(a, ::util::conversions::pages_to_bytes(required_pages), false);
            a
        };
        if zeroed {
            unsafe { ::std::ptr::write_bytes::<u8>(addr.to_ptr_mut(), 0, required_pages << LOG_BYTES_IN_PAGE) }
        }
        self.commit_pages(reserved_pages, required_pages, tls);
        Some(addr)
    }

    fn release_pages(&self, _first: Address) -> usize {
        unimplemented!()
    }

    fn release_all(&self) {
        self.common.reserved.store(0, Ordering::Relaxed);
        self.common.committed.store(0, Ordering::Relaxed);
        let mut head_discontiguous_region = self.common.head_discontiguous_region.lock().unwrap();
        VM_MAP.free_all_chunks(*head_discontiguous_region);
        *head_discontiguous_region = unsafe { Address::zero() };
        *self.alloc_chunk.lock().unwrap() = (unsafe { Address::zero() }, unsafe { Address::zero() });
    }
}

impl MonotonePageResource {
    pub fn new_contiguous(_start: Address, _bytes: usize,  _metadata_pages_per_region: usize) -> Self {
        unimplemented!()
    }

    pub fn new_discontiguous(metadata_pages_per_region: usize, space_descriptor: usize) -> Self {
        Self {
            common: CommonPageResource {
                reserved: AtomicUsize::new(0),
                committed: AtomicUsize::new(0),
                contiguous: false,
                growable: true,
                space_descriptor,
                metadata_pages_per_region,
                head_discontiguous_region: Mutex::new(unsafe { Address::zero() }),
            },
            alloc_chunk: Mutex::new((unsafe { Address::zero() }, unsafe { Address::zero() })),
        }
    }
}