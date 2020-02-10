use ::util::address::Address;
use ::policy::space::Space;
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fmt::Debug;
use util::heap::layout::heap_layout::VM_MAP;

static CUMULATIVE_COMMITTED: AtomicUsize = AtomicUsize::new(0);

pub trait PageResource: Sized + 'static + Debug {
    fn new_contiguous(start: Address, bytes: usize, metadata_pages_per_region: usize, space_descriptor: usize) -> Self;
    fn new_discontiguous(metadata_pages_per_region: usize, space_descriptor: usize) -> Self;

    fn reserve_pages(&self, pages: usize) -> usize {
        let adj_pages = pages;//self.adjust_for_metadata(pages);
        self.common().reserved.fetch_add(adj_pages, Ordering::Relaxed);
        adj_pages
    }

    fn clear_request(&self, reserved_pages: usize) {
        self.common().reserved.fetch_sub(reserved_pages, Ordering::Relaxed);
    }

    fn commit_pages(&self, reserved_pages: usize, actual_pages: usize, _tls: *mut ::libc::c_void) {
        let delta = actual_pages - reserved_pages;
        self.common().reserved.fetch_add(delta, Ordering::Relaxed);
        self.common().committed.fetch_add(actual_pages, Ordering::Relaxed);
    }

    fn alloc_pages(&self, reserved_pages: usize, required_pages: usize, zeroed: bool, space: &impl Space, tls: *mut ::libc::c_void) -> Option<Address>;
    fn release_pages(&self, first: Address) -> usize;
    fn release_all(&self);

    fn allocate_contiguous_chunks(&self, chunks: usize) -> Option<Address> {
        match self.common().memory {
            SpaceMemoryMeta::Discontiguous { ref head } => {
                let mut head = head.write().unwrap();
                match VM_MAP.allocate_contiguous_chunks(chunks, self.common().space_descriptor, *head) {
                    Some(chunk_start) => {
                        *head = chunk_start;
                        Some(chunk_start)
                    },
                    _ => {
                        None
                    },
                }
            }
            _ => unreachable!()
        }
    }

    fn release_contiguous_chunks(&self, chunk: Address) {
        debug_assert!(chunk == ::util::conversions::chunk_align(chunk, true));
        match self.common().memory {
            SpaceMemoryMeta::Discontiguous { ref head } => {
                if chunk == *head.read().unwrap() {
                    let mut head = head.write().unwrap();
                    *head = VM_MAP.get_next_contiguous_region(chunk).unwrap_or(unsafe { Address::zero() });
                }
                VM_MAP.release_contiguous_chunks(chunk);
            }
            _ => unreachable!()
        }
    }

    fn reserved_pages(&self) -> usize {
        self.common().reserved.load(Ordering::Relaxed)
    }

    fn committed_pages(&self) -> usize {
        self.common().committed.load(Ordering::Relaxed)
    }

    unsafe fn unsafe_common(&self) -> *mut CommonPageResource;
    fn common(&self) -> &'static CommonPageResource {
        unsafe { &*self.unsafe_common() }
    }
    fn common_mut(&mut self) -> &'static mut CommonPageResource {
        unsafe { &mut *self.unsafe_common() }
    }
}

pub fn cumulative_committed_pages() -> usize {
    CUMULATIVE_COMMITTED.load(Ordering::Relaxed)
}

#[derive(Debug)]
pub struct CommonPageResource {
    pub reserved: AtomicUsize,
    pub committed: AtomicUsize,
    pub space_descriptor: usize,
    pub metadata_pages_per_region: usize,
    pub memory: SpaceMemoryMeta,
}

#[derive(Debug)]
pub enum SpaceMemoryMeta {
    Contiguous { start: Address, extent: usize },
    Discontiguous { head: RwLock<Address> },
}
