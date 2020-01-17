use util::Address;
use super::allocator::{align_allocation_no_fill, fill_alignment_gap};
use ::util::alloc::Allocator;
use ::util::heap::FreeListPageResource;
use policy::markregionspace::*;
use libc::c_void;

type PR = FreeListPageResource<MarkRegionSpace>;

#[repr(C)]
#[derive(Debug)]
pub struct MarkRegionAllocator {
    pub tls: *mut c_void,
    pub space: &'static MarkRegionSpace,
    cursor: Address,
    limit: Address,
}

impl MarkRegionAllocator {
    pub fn reset(&mut self) {
        self.cursor = unsafe { Address::zero() };
        self.limit = unsafe { Address::zero() };
    }
}

impl Allocator<PR> for MarkRegionAllocator {
    fn get_space(&self) -> Option<&'static MarkRegionSpace> {
        Some(self.space)
    }

    fn alloc(&mut self, bytes: usize, align: usize, offset: isize) -> Address {
        let start = align_allocation_no_fill(self.cursor, align, offset);
        let end = start + bytes;
        if end <= self.limit {
            fill_alignment_gap(self.cursor, start);
            self.cursor = end;
            start
        } else {
            self.alloc_slow_inline(bytes, align, offset)
        }
    }

    fn alloc_slow_once(&mut self, bytes: usize, align: usize, offset: isize) -> Address {
        match self.space.get_new_region(self.tls) {
            Some(a) => {
                assert!(!a.is_zero());
                self.cursor = a;
                self.limit = a + MarkRegionSpace::BYTES_IN_REGION;
                self.alloc(bytes, align, offset)
            },
            None => unsafe { Address::zero() },
        }
    }

    fn get_tls(&self) -> *mut c_void {
        self.tls
    }
}

impl MarkRegionAllocator {
    pub fn new(tls: *mut c_void, space: &'static MarkRegionSpace) -> Self {
        Self {
            tls,
            space,
            cursor: unsafe { Address::zero() },
            limit: unsafe { Address::zero() },
        }
    }
}
