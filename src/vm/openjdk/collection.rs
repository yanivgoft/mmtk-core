use super::super::Collection;
use ::plan::{MutatorContext, ParallelCollector};
use ::util::OpaquePointer;
use plan::{Plan, SelectedPlan};
use plan::collector_context::CollectorContext;
use plan::plan::CONTROL_COLLECTOR_CONTEXT;

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

    fn block_for_gc(tls: *mut c_void) {
        println!("Block for GC");
        unsafe {
            ((*UPCALLS).block_for_gc)();
        }
    }

    unsafe fn spawn_worker_thread<T: ParallelCollector>(tls: *mut c_void, ctx: *mut T) {
        // if ctx == 0 as *mut T {
        ((*UPCALLS).spawn_collector_thread)(tls, ctx as usize as _);
        // } else {
        //     let tls = tls as usize;
        //     let ctx = ctx as usize;
        //     ::std::thread::spawn(move || {
        //         let worker_instance = &mut *(ctx as *mut <SelectedPlan as Plan>::CollectorT);
        //         worker_instance.init(tls as _);
        //         worker_instance.run(tls as _);
        //     });
        // }
    }

    fn prepare_mutator<T: MutatorContext>(tls: *mut c_void, m: &T) {
        // unimplemented!()
    }
}