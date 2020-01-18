use ::plan::{Plan, SelectedPlan};
use ::util::OpaquePointer;
use libc::c_void;
use vm::VMBinding;

pub trait ActivePlan<VM: VMBinding> {
    // TODO: I don't know how this can be implemented when we have multiple MMTk instances.
    // This function is used by space and phase to refer to the current plan.
    // Possibly we should remove the use of this function, and remove this function?
    fn global() -> &'static SelectedPlan { &::mmtk::SINGLETON.plan }
    unsafe fn collector(tls: OpaquePointer) -> &'static mut <SelectedPlan as Plan>::CollectorT;
    unsafe fn is_mutator(tls: OpaquePointer) -> bool;
    unsafe fn mutator(tls: OpaquePointer) -> &'static mut <SelectedPlan as Plan>::MutatorT;
    fn collector_count() -> usize;
    fn reset_mutator_iterator();
    fn get_next_mutator() -> Option<&'static mut <SelectedPlan as Plan>::MutatorT>;
}