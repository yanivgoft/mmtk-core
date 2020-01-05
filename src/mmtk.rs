use crate::plan::Plan;
use crate::plan::SelectedPlan;
use crate::plan::phase::PhaseManager;
use crate::util::heap::layout::heap_layout::VMMap;

use std::sync::Arc;

// TODO: remove this singleton at some point to allow multiple instances of MMTK
// This helps refactoring.
lazy_static!{
    pub static ref VM_MAP: VMMap = VMMap::new();
    pub static ref SINGLETON: MMTK = MMTK::new(&VM_MAP);
}

pub struct MMTK {
    pub plan: SelectedPlan,
    pub phase_manager: PhaseManager,
    pub vm_map: &'static VMMap,
}

impl MMTK {
    pub fn new(vm_map: &'static VMMap) -> Self {
        MMTK {
            plan: SelectedPlan::new(vm_map),
            phase_manager: PhaseManager::new(),
            vm_map
        }
    }
}