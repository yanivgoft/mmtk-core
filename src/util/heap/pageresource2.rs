use std::sync::{Mutex, MutexGuard};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use super::layout::freelist::*;
use super::layout::heap_layout::VM_MAP;
use super::layout::vm_layout_constants::{LOG_BYTES_IN_CHUNK, BYTES_IN_CHUNK};
use util::alloc::embedded_meta_data::LOG_BYTES_IN_REGION;
use util::Address;
use util::constants::*;

#[derive(Debug)]
pub struct PageResource {
    space_descriptor: usize,
    freelist: Mutex<Freelist>,
    reserved: AtomicUsize,
    committed: AtomicUsize,
    // contiguous: bool,
    // growable: bool,
    chunks: Mutex<HashMap<Address, usize>>,
    metadata_pages_per_region: usize,
}



impl PageResource {
    pub fn new_discontiguous(metadata_pages_per_region: usize, space_descriptor: usize) -> Self {
        assert!(metadata_pages_per_region == 0);
        Self {
            freelist: Mutex::new(Freelist::new()),
            reserved: AtomicUsize::new(0),
            committed: AtomicUsize::new(0),
            chunks: Mutex::new(HashMap::new()),
            metadata_pages_per_region,
            space_descriptor,
        }
    }

    pub fn reserve_pages(&self, pages: usize) -> usize {
        let adj_pages = pages;//self.adjust_for_metadata(pages);
        self.reserved.fetch_add(adj_pages, Ordering::Relaxed);
        adj_pages
    }

    pub fn clear_request(&self, reserved_pages: usize) {
        self.reserved.fetch_sub(reserved_pages, Ordering::Relaxed);
    }

    pub fn commit_pages(&self, reserved_pages: usize, actual_pages: usize, tls: *mut ::libc::c_void) {
        let delta = actual_pages - reserved_pages;
        self.reserved.fetch_add(delta, Ordering::Relaxed);
        self.committed.fetch_add(actual_pages, Ordering::Relaxed);
    }

    pub fn alloc_pages(&self, reserved_pages: usize, required_pages: usize, zeroed: bool, tls: *mut ::libc::c_void) -> Option<Address> {
        let mut freelist = self.freelist.lock().unwrap();
        match freelist.alloc(required_pages) {
            Some(page_index) => {
                self.commit_pages(reserved_pages, required_pages, tls);
                let page_address = unsafe { Address::from_usize(page_index << LOG_BYTES_IN_PAGE) };
                // if zeroed {
                    unsafe { ::std::ptr::write_bytes::<u8>(page_address.to_ptr_mut(), 0, required_pages << LOG_BYTES_IN_PAGE) }
                // }
                // println!("PR::alloc_pages {} -> ({:?}, {:?})", required_pages, page_address, page_address + (required_pages << LOG_BYTES_IN_PAGE));
                Some(page_address)
            },
            _ => {
                match self.allocate_contiguous_chunks(required_pages, freelist) {
                    Some(chunk_start) => {
                        self.alloc_pages(reserved_pages, required_pages, zeroed, tls)
                    },
                    _ => None
                }
            }
        }
    }

    pub fn release_pages(&self, first: Address) -> usize {
        debug_assert!(::util::conversions::is_page_aligned(first));
        let page_index = ::util::conversions::bytes_to_pages(first.as_usize());
        self.freelist.lock().unwrap().dealloc(page_index)
    }

    pub fn release_all(&self) {

        self.reserved.store(0, Ordering::Relaxed);
        self.committed.store(0, Ordering::Relaxed);

        for (start, n_chunks) in self.chunks.lock().unwrap().drain() {
            VM_MAP.release_contiguous_chunks(start);
        }
        self.freelist.lock().unwrap().reset()
    }

    fn allocate_contiguous_chunks(&self, pages: usize, mut freelist: MutexGuard<Freelist>) -> Option<Address> {
        // println!("PR::allocate_contiguous_chunks");
        let required_chunks = ::policy::space::required_chunks(pages);
        // println!("PR::allocate_contiguous_chunks -> {}", required_chunks);
        match VM_MAP.allocate_contiguous_chunks(required_chunks, self.space_descriptor) {
            Some(chunk_start) => {
                // println!("allocated -> {:?} {:?} {:?}", chunk_start, chunk_start + (1usize << LOG_BYTES_IN_REGION), chunk_start + BYTES_IN_CHUNK);
                self.chunks.lock().unwrap().insert(chunk_start, required_chunks);
                let page_index = chunk_start.as_usize() >> LOG_BYTES_IN_PAGE;
                let pages = (required_chunks << LOG_BYTES_IN_REGION) >> LOG_BYTES_IN_PAGE;
                // println!("pages = {}", pages);
                freelist.insert_free(page_index, pages);
                // println!("PR::allocate_contiguous_chunks -> {:?}", chunk_start);
                Some(chunk_start)
            },
            _ => {
                // println!("PR::allocate_contiguous_chunks -> None");    
                None
            },
        }
    }

    pub fn reserved_pages(&self) -> usize {
        self.reserved.load(Ordering::Relaxed)
    }

    pub fn committed_pages(&self) -> usize {
        self.committed.load(Ordering::Relaxed)
    }
}