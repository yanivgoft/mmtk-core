use super::super::Collection;
use ::plan::{MutatorContext, ParallelCollector};

pub struct VMCollection {}

impl Collection for VMCollection {
    fn stop_all_mutators(_thread_id: usize) {
        unimplemented!();
    }

    fn resume_mutators(_thread_id: usize) {
        unimplemented!();
    }

    fn block_for_gc(_thread_id: usize) {
        unimplemented!();
    }

    unsafe fn spawn_worker_thread<T: ParallelCollector>(_thread_id: usize, _ctx: *mut T) {
        unimplemented!();
    }

    fn prepare_mutator<T: MutatorContext>(_thread_id: usize, _m: &T) {
        unimplemented!()
    }
}