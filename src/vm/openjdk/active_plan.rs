use ::plan::{Plan, SelectedPlan};
use super::super::ActivePlan;
use ::util::OpaquePointer;
use super::UPCALLS;
use libc::c_void;

pub struct VMActivePlan<> {}

impl ActivePlan for VMActivePlan {
    unsafe fn collector(tls: *mut c_void) -> &'static mut <SelectedPlan as Plan>::CollectorT {
        let c = ((*UPCALLS).active_collector)(tls);
        assert!(c != 0 as *mut c_void);
        unsafe { ::std::mem::transmute(c) }
    }

    unsafe fn is_mutator(tls: OpaquePointer) -> bool {
        // FIXME
        true
    }

    unsafe fn mutator(tls: OpaquePointer) -> &'static mut <SelectedPlan as Plan>::MutatorT {
        unimplemented!()
    }

    fn collector_count() -> usize {
        unimplemented!()
    }

    fn reset_mutator_iterator() {
        unsafe {
            ((*UPCALLS).reset_mutator_iterator)();
        }
    }

    fn get_next_mutator() -> Option<&'static mut <SelectedPlan as Plan>::MutatorT> {
        unsafe {
            let c = ((*UPCALLS).get_next_mutator)();
            ::std::mem::transmute(c)
        }
    }
}