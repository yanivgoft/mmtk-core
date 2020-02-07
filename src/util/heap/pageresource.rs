use ::util::address::Address;
use ::policy::space::Space;
use ::vm::{ActivePlan, VMActivePlan};

use std::marker::PhantomData;
use std::sync::{Mutex, MutexGuard};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fmt::Debug;
use std::ops::Deref;
use libc::c_void;
use util::constants::*;
use util::heap::layout::vm_layout_constants::*;
use util::heap::layout::freelist::*;
use util::heap::layout::heap_layout::VM_MAP;
use std::collections::HashMap;

static CUMULATIVE_COMMITTED: AtomicUsize = AtomicUsize::new(0);

pub trait PageResource: Sized + 'static + Debug {
    fn reserve_pages(&self, pages: usize) -> usize {
        let adj_pages = pages;//self.adjust_for_metadata(pages);
        self.common().reserved.fetch_add(adj_pages, Ordering::Relaxed);
        adj_pages
    }

    fn clear_request(&self, reserved_pages: usize) {
        self.common().reserved.fetch_sub(reserved_pages, Ordering::Relaxed);
    }

    fn commit_pages(&self, reserved_pages: usize, actual_pages: usize, tls: *mut ::libc::c_void) {
        let delta = actual_pages - reserved_pages;
        self.common().reserved.fetch_add(delta, Ordering::Relaxed);
        self.common().committed.fetch_add(actual_pages, Ordering::Relaxed);
    }

    fn alloc_pages(&self, reserved_pages: usize, required_pages: usize, zeroed: bool, space: &impl Space, tls: *mut ::libc::c_void) -> Option<Address>;
    fn release_pages(&self, first: Address) -> usize;
    fn release_all(&self);

    fn allocate_contiguous_chunks(&self, chunks: usize) -> Option<Address> {
        let mut head_discontiguous_region = self.common().head_discontiguous_region.lock().unwrap();
        match VM_MAP.allocate_contiguous_chunks(chunks, self.common().space_descriptor, *head_discontiguous_region) {
            Some(chunk_start) => {
                *head_discontiguous_region = chunk_start;
                Some(chunk_start)
            },
            _ => {
                None
            },
        }
    }

    fn release_discontiguous_chunks(&mut self, chunk: Address) {
        debug_assert!(chunk == ::util::conversions::chunk_align(chunk, true));
        let mut head_discontiguous_region = self.common().head_discontiguous_region.lock().unwrap();
        if chunk == *head_discontiguous_region {
            *head_discontiguous_region = VM_MAP.get_next_contiguous_region(chunk).unwrap_or(unsafe { Address::zero() });
        }
        VM_MAP.release_contiguous_chunks(chunk);
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
    pub contiguous: bool,
    pub growable: bool,
    pub space_descriptor: usize,
    pub metadata_pages_per_region: usize,
    pub head_discontiguous_region: Mutex<Address>,
}
