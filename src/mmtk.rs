use crate::plan::Plan;
use crate::plan::SelectedPlan;
use crate::plan::phase::PhaseManager;
use crate::util::heap::layout::heap_layout::VMMap;
use crate::util::heap::layout::heap_layout::Mmapper;
use crate::util::ReferenceProcessor;
use crate::util::ObjectReference;
use crate::util::OpaquePointer;
use crate::plan::TraceLocal;

use std::sync::Arc;
use std::default::Default;
use util::reference_processor::{Semantics, ReferenceProcessors};
use util::options::{UnsafeOptionsWrapper, Options};
use std::sync::atomic::{Ordering, AtomicBool};

use util::statistics::stats::STATS;

// TODO: remove this singleton at some point to allow multiple instances of MMTK
// This helps refactoring.
lazy_static!{
    // possible global
    pub static ref VM_MAP: VMMap = VMMap::new();
    pub static ref MMAPPER: Mmapper = Mmapper::new();

    // This is a temporary mutable options processor, as the given API requires mutating on options.
    // However, I would suggest that options should not be mutable - the VM would give us all the options
    // (possibly as a string), we parse it and use those to instantiate an MMTK instance.
    // This processor is temporary to store options while receiving process() call from the VM.
    pub static ref OPTIONS_PROCESSOR: UnsafeOptionsWrapper = UnsafeOptionsWrapper::new(Options::default());
    // mmtk instance
    pub static ref SINGLETON: MMTK = MMTK::new(&VM_MAP, &MMAPPER, &OPTIONS_PROCESSOR);
}

pub struct MMTK {
    pub plan: SelectedPlan,
    pub phase_manager: PhaseManager,
    pub vm_map: &'static VMMap,
    pub mmapper: &'static Mmapper,
    pub reference_processors: ReferenceProcessors,
    pub options: &'static Options,

    inside_harness: AtomicBool,
}

impl MMTK {
    pub fn new(vm_map: &'static VMMap, mmapper: &'static Mmapper, options: &'static Options) -> Self {
        MMTK {
            plan: SelectedPlan::new(vm_map, mmapper, options),
            phase_manager: PhaseManager::new(),
            vm_map,
            mmapper,
            reference_processors: ReferenceProcessors::new(),
            options,
            inside_harness: AtomicBool::new(false),
        }
    }

    pub fn harness_begin(&self, tls: OpaquePointer) {
        // FIXME Do a full heap GC if we have generational GC
        self.plan.handle_user_collection_request(tls, true);
        self.inside_harness.store(true, Ordering::SeqCst);
        STATS.start_all();
    }

    pub fn harness_end(&self) {
        STATS.stop_all();
        self.inside_harness.store(false, Ordering::SeqCst);
    }
}