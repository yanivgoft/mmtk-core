use ::policy::copyspace2::CopySpace;
use ::util::alloc::{BumpAllocator, RawPageAllocator};
use ::policy::rawpagespace::RawPageSpace;
use ::plan::mutator_context::MutatorContext;
use ::plan::Phase;
use ::util::{Address, ObjectReference};
use ::util::alloc::Allocator;
use ::plan::Allocator as AllocationType;
use ::plan::plan;
use ::vm::{Collection, VMCollection};
use ::util::heap::MonotonePageResource;
use super::PLAN;

use libc::c_void;

#[repr(C)]
pub struct SSMutator {
    ss: BumpAllocator<MonotonePageResource<CopySpace>>,
    vs: RawPageAllocator,
}

impl MutatorContext for SSMutator {
    fn collection_phase(&mut self, _tls: *mut c_void, phase: &Phase, _primary: bool) {
        match phase {
            &Phase::PrepareStacks => {
                if !plan::stacks_prepared() {
                    VMCollection::prepare_mutator(self.ss.tls, self);
                }
                self.flush_remembered_sets();
            }
            &Phase::Prepare => {}
            &Phase::Release => {
                // rebind the allocation bump pointer to the appropriate semispace
                self.ss.rebind(Some(PLAN.tospace()));
            }
            _ => {
                panic!("Per-mutator phase not handled!")
            }
        }
    }

    fn alloc(&mut self, size: usize, align: usize, offset: isize, allocator: AllocationType) -> Address {
        match allocator {
            AllocationType::Default => self.ss.alloc(size, align, offset),
            _ => self.vs.alloc(size, align, offset),
        }
    }

    fn alloc_slow(&mut self, _size: usize, _align: usize, _offset: isize, _allocator: AllocationType) -> Address {
        unimplemented!()
    }

    fn post_alloc(&mut self, _refer: ObjectReference, _type_refer: ObjectReference, _bytes: usize, _allocator: AllocationType) {
    }

    fn get_tls(&self) -> *mut c_void {
        debug_assert!(self.ss.tls == self.vs.tls);
        self.ss.tls
    }
}

impl SSMutator {
    pub fn new(tls: *mut c_void, space: &'static CopySpace, versatile_space: &'static RawPageSpace) -> Self {
        SSMutator {
            ss: BumpAllocator::new(tls, Some(space)),
            vs: RawPageAllocator::new(tls, versatile_space),
        }
    }
}