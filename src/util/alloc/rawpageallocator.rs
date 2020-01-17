use ::util::{Address, ObjectReference};
use super::allocator::{align_allocation_no_fill, fill_alignment_gap, MIN_ALIGNMENT};

use ::util::alloc::{allocator, Allocator};
use ::util::heap::PageResource;
use ::util::alloc::linear_scan::LinearScan;
use ::util::alloc::dump_linear_scan::DumpLinearScan;

use ::vm::*;

use std::marker::PhantomData;

use libc::{memset, c_void};

use ::policy::space::Space;
use util::conversions::bytes_to_pages;
use ::util::constants::BYTES_IN_ADDRESS;
use policy::rawpagespace::RawPageSpace;
use util::heap::FreeListPageResource;

// const BYTES_IN_PAGE: usize = 1 << 12;
// const BLOCK_SIZE: usize = 8 * BYTES_IN_PAGE;
// const BLOCK_MASK: usize = BLOCK_SIZE - 1;

// const REGION_LIMIT_OFFSET: isize = 0;
// const NEXT_REGION_OFFSET: isize = REGION_LIMIT_OFFSET + BYTES_IN_ADDRESS as isize;
// const DATA_END_OFFSET: isize = NEXT_REGION_OFFSET + BYTES_IN_ADDRESS as isize;

use ::util::constants::*;

type PR = FreeListPageResource<RawPageSpace>;

#[repr(C)]
#[derive(Debug)]
pub struct RawPageAllocator {
    pub tls: *mut c_void,
    space: Option<&'static RawPageSpace>
}

impl Allocator<PR> for RawPageAllocator {
    fn get_space(&self) -> Option<&'static RawPageSpace> {
        self.space
    }

    fn get_tls(&self) -> *mut c_void {
        // println!("get_tls({:?})", self as *const _);
        self.tls
    }

    fn alloc(&mut self, size: usize, align: usize, offset: isize) -> Address {
        let cell: Address = self.alloc_slow(size, align, offset);
        allocator::align_allocation_no_fill(cell, align, offset)
    }

    fn alloc_slow(&mut self, size: usize, align: usize, offset: isize) -> Address {
        self.alloc_slow_inline(size, align, offset)
    }

    fn alloc_slow_once(&mut self, size: usize, align: usize, offset: isize) -> Address {
        let header = BYTES_IN_ADDRESS * 2; // HashSet is used instead of DoublyLinkedList
        let maxbytes = allocator::get_maximum_aligned_size(size + header, align, allocator::MIN_ALIGNMENT);
        let pages = ::util::conversions::bytes_to_pages_up(maxbytes);
        let cell = self.space.unwrap().acquire(self.tls, pages);
        if !cell.is_zero() {
            // VMMemory::unprotect(cell, pages << LOG_BYTES_IN_PAGE);
            VMMemory::zero(cell, pages << LOG_BYTES_IN_PAGE);
            unsafe { (cell + BYTES_IN_ADDRESS).store(pages) };
            cell + BYTES_IN_ADDRESS * 2
        } else {
            cell
        }
    }
}

impl RawPageAllocator {
    pub fn new(tls: *mut c_void, space: Option<&'static RawPageSpace>) -> Self {
        Self {
            tls,
            space,
        }
    }
}
