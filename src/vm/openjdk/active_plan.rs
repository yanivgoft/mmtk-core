use ::plan::{Plan, SelectedPlan};
use super::super::ActivePlan;
use ::util::OpaquePointer;
use libc::c_void;
use vm::OpenJDK;

pub struct VMActivePlan<> {}

impl ActivePlan<OpenJDK> for VMActivePlan {
    fn global() -> &'static SelectedPlan<OpenJDK> {
        &::mmtk::SINGLETON.plan
    }

    unsafe fn collector(tls: OpaquePointer) -> &'static mut <SelectedPlan<OpenJDK> as Plan<OpenJDK>>::CollectorT {
        unimplemented!()
    }

    unsafe fn is_mutator(tls: OpaquePointer) -> bool {
        // FIXME
        true
    }

    unsafe fn mutator(tls: OpaquePointer) -> &'static mut <SelectedPlan<OpenJDK> as Plan<OpenJDK>>::MutatorT {
        unimplemented!()
    }

    fn collector_count() -> usize {
        unimplemented!()
    }

    fn reset_mutator_iterator() {
        unimplemented!()
    }

    fn get_next_mutator() -> Option<&'static mut <SelectedPlan<OpenJDK> as Plan<OpenJDK>>::MutatorT> {
        unimplemented!()
    }
}