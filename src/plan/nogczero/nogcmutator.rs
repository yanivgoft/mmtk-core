use ::policy::rawpagespace::RawPageSpace;
use ::policy::immortalspace::ImmortalSpace;
use ::util::alloc::BumpAllocator;
use ::util::alloc::RawPageAllocator;
use ::plan::mutator_context::MutatorContext;
use ::plan::Phase;
use ::util::{Address, ObjectReference};
use ::util::alloc::Allocator;
use ::plan::Allocator as AllocationType;
use ::util::heap::MonotonePageResource;
use super::PLAN;

use libc::c_void;

#[repr(C)]
pub struct NoGCMutator {
    rp: RawPageAllocator,
    // vs: BumpAllocator<MonotonePageResource<ImmortalSpace>>,
}

impl MutatorContext for NoGCMutator {
    fn collection_phase(&mut self, tls: *mut c_void, phase: &Phase, primary: bool) {
        unimplemented!();
    }

    fn alloc(&mut self, size: usize, align: usize, offset: isize, allocator: AllocationType) -> Address {
        trace!("MutatorContext.alloc({}, {}, {}, {:?})", size, align, offset, allocator);
        match allocator {
            AllocationType::Default => self.rp.alloc(size, align, offset),
            _ => unreachable!()//self.vs.alloc(size, align, offset)
        }
    }

    fn alloc_slow(&mut self, size: usize, align: usize, offset: isize, allocator: AllocationType) -> Address {
        trace!("MutatorContext.alloc_slow({}, {}, {}, {:?})", size, align, offset, allocator);
        match allocator {
            AllocationType::Default => self.rp.alloc(size, align, offset),
            _ => unreachable!()//self.vs.alloc(size, align, offset)
        }
        // self.nogc.alloc_slow(size, align, offset)
    }

    fn post_alloc(&mut self, refer: ObjectReference, type_refer: ObjectReference, bytes: usize, allocator: AllocationType) {
        // println!("Allocated {:?} ", refer);
        PLAN.space.initialize_header(refer);
    }

    fn get_tls(&self) -> *mut c_void {
        self.rp.tls
    }
}

impl NoGCMutator {
    pub fn new(tls: *mut c_void, space: &'static RawPageSpace) -> Self {
        NoGCMutator {
            rp: RawPageAllocator::new(tls, Some(space)),
        }
    }
}