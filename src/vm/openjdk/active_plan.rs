use ::plan::{Plan, SelectedPlan};
use super::super::ActivePlan;
use ::util::OpaquePointer;
use super::UPCALLS;
use libc::c_void;
use std::sync::Mutex;

pub struct VMActivePlan<> {}

impl ActivePlan for VMActivePlan {
    unsafe fn collector(tls: OpaquePointer) -> &'static mut <SelectedPlan as Plan>::CollectorT {
        let c = ((*UPCALLS).active_collector)(tls);
        assert!(c != 0 as *mut c_void);
        unsafe { ::std::mem::transmute(c) }
    }

    unsafe fn is_mutator(tls: OpaquePointer) -> bool {
        ((*UPCALLS).is_mutator)(tls)
    }

    unsafe fn mutator(tls: OpaquePointer) -> &'static mut <SelectedPlan as Plan>::MutatorT {
        let m = ((*UPCALLS).get_mmtk_mutator)(tls);
        ::std::mem::transmute(m)
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
        let _guard = MUTATOR_ITERATOR_LOCK.lock().unwrap();
        unsafe {
            let c = ((*UPCALLS).get_next_mutator)();
            ::std::mem::transmute(c)
        }
    }
}

lazy_static! {
    pub static ref MUTATOR_ITERATOR_LOCK: Mutex<()> = Mutex::new(());
}
