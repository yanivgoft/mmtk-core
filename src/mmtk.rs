use crate::plan::Plan;
use crate::plan::SelectedPlan;
use crate::plan::phase::PhaseManager;
use crate::util::heap::layout::heap_layout::VMMap;
use crate::util::heap::layout::heap_layout::Mmapper;

use std::default::Default;
use util::reference_processor::{Semantics, ReferenceProcessors};
use util::options::{UnsafeOptionsWrapper, Options};
use std::sync::atomic::{Ordering, AtomicBool};

use util::statistics::stats::STATS;
use util::OpaquePointer;

// TODO: remove this singleton at some point to allow multiple instances of MMTK
// This helps refactoring.
lazy_static!{
    // I am not sure if we should include these mmappers as part of MMTk struct.
    // The considerations are:
    // 1. We need VMMap and Mmapper to create spaces. It is natural that the mappers are not
    //    part of MMTK, as creating MMTK requires these mappers. We could use Rc/Arc for these mappers though.
    // 2. These mmappers are possibly global across multiple MMTk instances, as they manage the
    //    entire address space.
    // TODO: We should refactor this when we know more about how multiple MMTK instances work.
    pub static ref VM_MAP: VMMap = VMMap::new();
    pub static ref MMAPPER: Mmapper = Mmapper::new();

    // TODO: We should change how the VM instantiates MMTK instance and passes options.
    // This is a temporary mutable options processor, as the current API requires mutation on options.
    // However, I would suggest that options should not be mutable - the VM would give us all the options together
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
        STATS.lock().unwrap().start_all();
    }

    pub fn harness_end(&self) {
        STATS.lock().unwrap().stop_all();
        self.inside_harness.store(false, Ordering::SeqCst);
    }
}