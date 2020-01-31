use ::util::Address;
use ::util::alloc::{allocator, Allocator};
use libc::c_void;
use policy::rawpagespace::RawPageSpace;

#[repr(C)]
#[derive(Debug)]
pub struct RawPageAllocator {
    pub tls: *mut c_void,
    space: &'static RawPageSpace,
}

impl Allocator<RawPageSpace> for RawPageAllocator {
    fn get_space(&self) -> Option<&'static RawPageSpace> {
        Some(self.space)
    }

    fn get_tls(&self) -> *mut c_void {
        self.tls
    }

    fn alloc(&mut self, size: usize, align: usize, offset: isize) -> Address {
        let cell: Address = self.alloc_slow(size, align, offset);
        allocator::align_allocation(cell, align, offset, allocator::MIN_ALIGNMENT, true)
    }

    fn alloc_slow(&mut self, size: usize, align: usize, offset: isize) -> Address {
        self.alloc_slow_inline(size, align, offset)
    }

    fn alloc_slow_once(&mut self, size: usize, align: usize, _offset: isize) -> Address {
        let max_bytes = allocator::get_maximum_aligned_size(size, align, allocator::MIN_ALIGNMENT);
        let pages = ::util::conversions::bytes_to_pages_up(max_bytes);
        self.space.alloc(self.tls, pages).unwrap_or(unsafe { Address::zero() })
    }
}

impl RawPageAllocator {
    pub fn new(tls: *mut c_void, space: &'static RawPageSpace) -> Self {
        Self { tls, space }
    }
}
