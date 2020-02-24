use super::super::Collection;
use ::plan::{MutatorContext, ParallelCollector};
use ::util::OpaquePointer;
use plan::{Plan, SelectedPlan};
use plan::collector_context::CollectorContext;

use super::UPCALLS;

use libc::c_void;

pub struct VMCollection {}

impl Collection for VMCollection {
    fn stop_all_mutators(tls: OpaquePointer) {
        unsafe {
            ((*UPCALLS).stop_all_mutators)(tls);
        }
    }

    fn resume_mutators(tls: OpaquePointer) {
        unsafe {
            ((*UPCALLS).resume_mutators)(tls);
        }
    }

    fn collect_work() {
        // unsafe {
        //     ((*UPCALLS).collect_work)();
        // }
    }

    fn block_for_gc(tls: OpaquePointer) {
        unsafe {
            ((*UPCALLS).block_for_gc)();
        }
    }

    unsafe fn spawn_worker_thread<T: ParallelCollector>(tls: OpaquePointer, ctx: *mut T) {
        ((*UPCALLS).spawn_collector_thread)(tls, ctx as usize as _);
    }

    fn prepare_mutator<T: MutatorContext>(tls: OpaquePointer, m: &T) {
        // unimplemented!()
    }
}