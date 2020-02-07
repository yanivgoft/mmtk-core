use ::policy::markregionspace::MarkRegionSpace;
use policy::rawpagespace::RawPageSpace;
use ::util::alloc::{MarkRegionAllocator, RawPageAllocator};
use ::plan::mutator_context::MutatorContext;
use ::plan::Phase;
use ::util::{Address, ObjectReference};
use ::util::alloc::Allocator;
use ::plan::Allocator as AllocationType;

use libc::c_void;

#[repr(C)]
pub struct MarkRegionMutator {
    mr: MarkRegionAllocator,
    vs: RawPageAllocator,
}

impl MutatorContext for MarkRegionMutator {
    fn collection_phase(&mut self, tls: *mut c_void, phase: &Phase, primary: bool) {
        match phase {
            &Phase::PrepareStacks => {
                self.mr.reset();
                // self.vs.reset();
            }
            &Phase::Prepare => {
                self.mr.reset();
                // self.vs.reset();
            }
            &Phase::Release => {
                self.mr.reset();
                // self.vs.reset();
            }
            _ => {
                panic!("Per-mutator phase not handled!")
            }
        }
    }

    fn alloc(&mut self, size: usize, align: usize, offset: isize, allocator: AllocationType) -> Address {
        trace!("MutatorContext.alloc({}, {}, {}, {:?})", size, align, offset, allocator);
        let a = if size <= MarkRegionSpace::BYTES_IN_REGION {
            let a = self.mr.alloc(size, align, offset);
            // let o = unsafe { a.to_object_reference() };
            // assert!(super::PLAN.space.contains(o));
            // unsafe {
            //     assert!(!PLAN.versatile_space.in_space(a.to_object_reference()));
            // assert!(PLAN.space.in_space(a.to_object_reference()));
            // }
            a
        } else {
            self.vs.alloc(size, align, offset)
        };
        // assert!(a < ::util::heap::layout::vm_layout_constants::HEAP_END);
        a
    }

    fn alloc_slow(&mut self, size: usize, align: usize, offset: isize, allocator: AllocationType) -> Address {
        trace!("MutatorContext.alloc_slow({}, {}, {}, {:?})", size, align, offset, allocator);
        // match allocator {
        //     AllocationType::Default => self.mr.alloc(size, align, offset),
        //     _ => unreachable!()
        // }
        unimplemented!()
    }

    fn post_alloc(&mut self, refer: ObjectReference, type_refer: ObjectReference, bytes: usize, allocator: AllocationType) {
    }

    fn get_tls(&self) -> *mut c_void {
        self.mr.tls
    }
}

impl MarkRegionMutator {
    pub fn new(tls: *mut c_void, vs: &'static RawPageSpace, space: &'static MarkRegionSpace) -> Self {
        Self {
            mr: MarkRegionAllocator::new(tls, space),
            vs: RawPageAllocator::new(tls, vs)
        }
    }
}