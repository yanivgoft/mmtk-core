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

use util::statistics::stats::Stats;
use vm::VMBinding;
use std::marker::PhantomData;

// TODO: remove this singleton at some point to allow multiple instances of MMTK
// This helps refactoring.
lazy_static!{
    // possible global across multiple MMTk instances
    pub static ref VM_MAP: VMMap = VMMap::new();
    pub static ref MMAPPER: Mmapper = Mmapper::new();

    // This is a temporary mutable options processor, as the given API requires mutating on options.
    // However, I would suggest that options should not be mutable - the VM would give us all the options in one go
    // (possibly as a string), we parse it and use those to instantiate an MMTK instance.
    // This processor is temporary to store options while receiving process() call from the VM.
    pub static ref OPTIONS_PROCESSOR: UnsafeOptionsWrapper = UnsafeOptionsWrapper::new(Options::default());
}

#[cfg(feature = "jikesrvm")]
use vm::JikesRVM;
#[cfg(feature = "jikesrvm")]
lazy_static!{
    pub static ref SINGLETON: MMTK<JikesRVM> = MMTK::new(&VM_MAP, &MMAPPER, &OPTIONS_PROCESSOR);
}

pub struct MMTK<VM: VMBinding> {
    pub plan: SelectedPlan,
    pub phase_manager: PhaseManager,
    pub vm_map: &'static VMMap,
    pub mmapper: &'static Mmapper,
    pub reference_processors: ReferenceProcessors,
    pub options: &'static Options,

    inside_harness: AtomicBool,

    // FIXME: Delete this before merging
    p: PhantomData<VM>
}

impl<VM: VMBinding> MMTK<VM> {
    pub fn new(vm_map: &'static VMMap, mmapper: &'static Mmapper, options: &'static Options) -> Self {
        let plan = SelectedPlan::new(vm_map, mmapper, options);
        let phase_manager = PhaseManager::new(&plan.common().stats);
        MMTK {
            plan,
            phase_manager,
            vm_map,
            mmapper,
            reference_processors: ReferenceProcessors::new(),
            options,
            inside_harness: AtomicBool::new(false),
            p: PhantomData,
        }
    }

    pub fn harness_begin(&self, tls: OpaquePointer) {
        // FIXME Do a full heap GC if we have generational GC
        self.plan.handle_user_collection_request(tls, true);
        self.inside_harness.store(true, Ordering::SeqCst);
        self.plan.common().stats.start_all();
    }

    pub fn harness_end(&self) {
        self.plan.common().stats.stop_all();
        self.inside_harness.store(false, Ordering::SeqCst);
    }
}