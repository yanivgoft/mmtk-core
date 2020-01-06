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
use util::reference_processor::{Semantics, ReferenceProcessors};

// TODO: remove this singleton at some point to allow multiple instances of MMTK
// This helps refactoring.
lazy_static!{
    // possible global
    pub static ref VM_MAP: VMMap = VMMap::new();
    pub static ref MMAPPER: Mmapper = Mmapper::new();

    // mmtk instance
    pub static ref SINGLETON: MMTK = MMTK::new(&VM_MAP, &MMAPPER);
}

pub struct MMTK {
    pub plan: SelectedPlan,
    pub phase_manager: PhaseManager,
    pub vm_map: &'static VMMap,
    pub mmapper: &'static Mmapper,

    pub reference_processors: ReferenceProcessors
}

impl MMTK {
    pub fn new(vm_map: &'static VMMap, mmapper: &'static Mmapper) -> Self {
        MMTK {
            plan: SelectedPlan::new(vm_map, mmapper),
            phase_manager: PhaseManager::new(),
            vm_map,
            mmapper,
            reference_processors: ReferenceProcessors::new(),
        }
    }
}