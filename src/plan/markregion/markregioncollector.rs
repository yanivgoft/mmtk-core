use super::MarkRegionTraceLocal;
use super::super::ParallelCollectorGroup;
use super::super::ParallelCollector;
use super::super::CollectorContext;
use super::super::TraceLocal;
use super::super::Phase;
use super::super::Allocator;
use plan::phase;
use super::PLAN;
use libc::c_void;
use vm::*;

use ::util::{Address, ObjectReference};

pub struct MarkRegionCollector {
    pub tls: *mut c_void,
    trace: MarkRegionTraceLocal,
    last_trigger_count: usize,
    worker_ordinal: usize,
    group: Option<&'static ParallelCollectorGroup<MarkRegionCollector>>,
}

impl<'a> CollectorContext for MarkRegionCollector {
    fn new() -> Self {
        Self {
            tls: 0 as *mut c_void,
            trace: MarkRegionTraceLocal::new(&PLAN.trace),
            last_trigger_count: 0,
            worker_ordinal: 0,
            group: None,
        }
    }

    fn init(&mut self, tls: *mut c_void) {
        self.tls = tls;
    }

    fn alloc_copy(&mut self, original: ObjectReference, bytes: usize, align: usize, offset: isize, allocator: Allocator) -> Address {
        unimplemented!();
    }

    fn run(&mut self, tls: *mut c_void) {
        loop {
            self.park();
            self.collect();
        }
    }

    fn collection_phase(&mut self, tls: *mut c_void, phase: &Phase, primary: bool) {
        println!("Collector {:?}", phase);
        match phase {
            &Phase::Prepare => {}
            &Phase::StackRoots => {
                trace!("Computing thread roots");
                VMScanning::compute_thread_roots(&mut self.trace, self.tls);
                trace!("Thread roots complete");
            }
            &Phase::Roots => {
                trace!("Computing global roots");
                VMScanning::compute_global_roots(&mut self.trace, self.tls);
                trace!("Computing static roots");
                VMScanning::compute_static_roots(&mut self.trace, self.tls);
                trace!("Finished static roots");
                // if super::ss::SCAN_BOOT_IMAGE {
                    trace!("Scanning boot image");
                    VMScanning::compute_bootimage_roots(&mut self.trace, self.tls);
                    trace!("Finished boot image");
                // }
            }
            &Phase::SoftRefs => {
                // if primary {
                //     // FIXME Clear refs if noReferenceTypes is true
                //     scan_soft_refs(&mut self.trace, self.tls)
                // }
            }
            &Phase::WeakRefs => {
                // if primary {
                //     // FIXME Clear refs if noReferenceTypes is true
                //     scan_weak_refs(&mut self.trace, self.tls)
                // }
            }
            &Phase::Finalizable => {
                if primary {
                    // FIXME
                }
            }
            &Phase::PhantomRefs => {
                // if primary {
                //     // FIXME Clear refs if noReferenceTypes is true
                //     scan_phantom_refs(&mut self.trace, self.tls)
                // }
            }
            &Phase::ForwardRefs => {
                // if primary && SelectedConstraints::NEEDS_FORWARD_AFTER_LIVENESS {
                //     forward_refs(&mut self.trace)
                // }
            }
            &Phase::ForwardFinalizable => {
                if primary {
                    // FIXME
                }
            }
            &Phase::Complete => {
                debug_assert!(self.trace.is_empty());
            }
            &Phase::Closure => {
                self.trace.complete_trace();
                debug_assert!(self.trace.is_empty());
            }
            &Phase::Release => {
                debug_assert!(self.trace.is_empty());
                self.trace.release();
                debug_assert!(self.trace.is_empty());
            }
            _ => { panic!("Per-collector phase not handled") }
        }
    }

    fn get_tls(&self) -> *mut c_void {
        self.tls
    }
}

impl ParallelCollector for MarkRegionCollector {
    type T = MarkRegionTraceLocal;

    fn park(&mut self) {
        self.group.unwrap().park(self);
    }

    fn collect(&self) {
        phase::begin_new_phase_stack(self.tls, (phase::Schedule::Complex, ::plan::plan::COLLECTION.clone()));
    }

    fn get_current_trace(&mut self) -> &mut MarkRegionTraceLocal {
        &mut self.trace
    }

    fn parallel_worker_count(&self) -> usize {
        self.group.unwrap().active_worker_count()
    }

    fn parallel_worker_ordinal(&self) -> usize {
        self.worker_ordinal
    }

    fn rendezvous(&self) -> usize {
        self.group.unwrap().rendezvous()
    }

    fn get_last_trigger_count(&self) -> usize {
        self.last_trigger_count
    }

    fn set_last_trigger_count(&mut self, val: usize) {
        self.last_trigger_count = val;
    }

    fn increment_last_trigger_count(&mut self) {
        self.last_trigger_count += 1;
    }

    fn set_group(&mut self, group: *const ParallelCollectorGroup<Self>) {
        self.group = Some ( unsafe {&*group} );
    }

    fn set_worker_ordinal(&mut self, ordinal: usize) {
        self.worker_ordinal = ordinal;
    }
}