use std::ptr::null_mut;
use libc::c_void;
use libc::c_char;

use std::ffi::CStr;
use std::{str, thread};

use std::sync::atomic::Ordering;

use plan::Plan;
use ::plan::MutatorContext;
use ::plan::TraceLocal;
use ::plan::CollectorContext;
use ::plan::ParallelCollectorGroup;

use ::vm::Collection;

use ::util::{Address, ObjectReference};

use ::plan::selected_plan;
use self::selected_plan::SelectedPlan;

use ::plan::Allocator;
use util::constants::LOG_BYTES_IN_PAGE;
use util::heap::layout::vm_layout_constants::HEAP_START;
use util::heap::layout::vm_layout_constants::HEAP_END;
use util::OpaquePointer;
use vm::VMBinding;
use mmtk::MMTK;
use util::handle::MMTKHandle;
use self::selected_plan::{SelectedMutator, SelectedTraceLocal, SelectedCollector};

pub fn start_control_collector<VM: VMBinding>(mmtk: &MMTK<VM>, tls: OpaquePointer) {
    mmtk.plan.common().control_collector_context.run(tls);
}

pub fn gc_init<VM: VMBinding>(mmtk: &MMTK<VM>, heap_size: usize) {
    ::util::logger::init().unwrap();
    mmtk.plan.gc_init(heap_size, &mmtk.vm_map);
    mmtk.plan.common().initialized.store(true, Ordering::SeqCst);

    // TODO: We should have an option so we know whether we should spawn the controller.
//    thread::spawn(|| {
//        SINGLETON.plan.common().control_collector_context.run(UNINITIALIZED_OPAQUE_POINTER )
//    });
}

pub fn bind_mutator<VM: VMBinding>(mmtk: &'static MMTK<VM>, tls: OpaquePointer) -> MMTKHandle<SelectedMutator<VM>> {
    SelectedPlan::bind_mutator(&mmtk.plan, tls)
}

pub fn alloc<VM: VMBinding>(mutator: MMTKHandle<SelectedMutator<VM>>, size: usize,
             align: usize, offset: isize, allocator: Allocator) -> Address {
    unsafe { mutator.as_mut() }.alloc(size, align, offset, allocator)
}

#[inline(never)]
pub fn alloc_slow<VM: VMBinding>(mutator: MMTKHandle<SelectedMutator<VM>>, size: usize,
                  align: usize, offset: isize, allocator: Allocator) -> Address {
    unsafe { mutator.as_mut() }.alloc_slow(size, align, offset, allocator)
}

pub fn post_alloc<VM: VMBinding>(mutator: MMTKHandle<SelectedMutator<VM>>, refer: ObjectReference, type_refer: ObjectReference,
                         bytes: usize, allocator: Allocator) {
    unsafe { mutator.as_mut() }.post_alloc(refer, type_refer, bytes, allocator);
}

pub fn will_never_move<VM: VMBinding>(mmtk: &MMTK<VM>, object: ObjectReference) -> bool {
    mmtk.plan.will_never_move(object)
}

pub fn is_valid_ref<VM: VMBinding>(mmtk: &MMTK<VM>, val: ObjectReference) -> bool {
    mmtk.plan.is_valid_ref(val)
}

#[cfg(feature = "sanity")]
pub fn report_delayed_root_edge<VM: VMBinding>(mmtk: &MMTK<VM>, trace_local: MMTKHandle<SelectedTraceLocal<VM>>, addr: Address) {
    use ::util::sanity::sanity_checker::SanityChecker;
    if mmtk.plan.common().is_in_sanity() {
        let sanity_checker = unsafe { trace_local as MMTKHandle<SanityChecker<VM>> };
        unsafe { sanity_checker.as_mut() }.report_delayed_root_edge(addr);
    } else {
        unsafe { trace_local.as_mut() }.report_delayed_root_edge(addr)
    }
}
#[cfg(not(feature = "sanity"))]
pub fn report_delayed_root_edge<VM: VMBinding>(_: &MMTK<VM>, trace_local: MMTKHandle<SelectedTraceLocal<VM>>, addr: Address) {
    unsafe { trace_local.as_mut() }.report_delayed_root_edge(addr);
}

#[cfg(feature = "sanity")]
pub fn will_not_move_in_current_collection<VM: VMBinding>(mmtk: &MMTK<VM>, trace_local: MMTKHandle<SelectedTraceLocal<VM>>, obj: ObjectReference) -> bool {
    use ::util::sanity::sanity_checker::SanityChecker;
    if mmtk.plan.common().is_in_sanity() {
        let sanity_checker = unsafe { trace_local as MMTKHandle<SanityChecker<VM>> };
        unsafe { sanity_checker.as_mut() }.will_not_move_in_current_collection(obj)
    } else {
        unsafe { trace_local.as_mut() }.will_not_move_in_current_collection(obj)
    }
}
#[cfg(not(feature = "sanity"))]
pub fn will_not_move_in_current_collection<VM: VMBinding>(_: &MMTK<VM>, trace_local: MMTKHandle<SelectedTraceLocal<VM>>, obj: ObjectReference) -> bool {
    unsafe { trace_local.as_mut() }.will_not_move_in_current_collection(obj)
}

#[cfg(feature = "sanity")]
pub fn process_interior_edge<VM: VMBinding>(mmtk: &MMTK<VM>, trace_local: MMTKHandle<SelectedTraceLocal<VM>>, target: ObjectReference, slot: Address, root: bool) {
    use ::util::sanity::sanity_checker::SanityChecker;
    if mmtk.plan.common().is_in_sanity() {
        let sanity_checker = unsafe { trace_local as MMTKHandle<SanityChecker<VM>> };
        unsafe { sanity_checker.as_mut() }.process_interior_edge(target, slot, root)
    } else {
        unsafe { trace_local.as_mut() }.process_interior_edge(target, slot, root)
    }
}
#[cfg(not(feature = "sanity"))]
pub fn process_interior_edge<VM: VMBinding>(_: &MMTK<VM>, trace_local: MMTKHandle<SelectedTraceLocal<VM>>, target: ObjectReference, slot: Address, root: bool) {
    unsafe { trace_local.as_mut() }.process_interior_edge(target, slot, root)
}

pub fn start_worker<VM: VMBinding>(tls: OpaquePointer, worker: MMTKHandle<SelectedCollector<VM>>) {
    let worker_instance = unsafe { worker.as_mut() };
    worker_instance.init(tls);
    worker_instance.run(tls);
}

pub fn enable_collection<VM: VMBinding>(mmtk: &'static MMTK<VM>, tls: OpaquePointer) {
    unsafe {
        { (&mut *mmtk.plan.common().control_collector_context.workers.get()) }.init_group(mmtk, tls);
        { VM::VMCollection::spawn_worker_thread::<<SelectedPlan<VM> as Plan<VM>>::CollectorT>(tls, null_mut()); }// spawn controller thread
        mmtk.plan.common().initialized.store(true, Ordering::SeqCst);
    }
}

pub fn process<VM: VMBinding>(mmtk: &'static MMTK<VM>, name: *const c_char, value: *const c_char) -> bool {
    let name_str: &CStr = unsafe { CStr::from_ptr(name) };
    let value_str: &CStr = unsafe { CStr::from_ptr(value) };
    let option = &mmtk.options;
    unsafe {
        option.process(name_str.to_str().unwrap(), value_str.to_str().unwrap())
    }
}

pub fn used_bytes<VM: VMBinding>(mmtk: &MMTK<VM>) -> usize {
    mmtk.plan.get_pages_used() << LOG_BYTES_IN_PAGE
}

pub fn free_bytes<VM: VMBinding>(mmtk: &MMTK<VM>) -> usize {
    mmtk.plan.get_free_pages() << LOG_BYTES_IN_PAGE
}

pub fn starting_heap_address() -> Address {
    HEAP_START
}

pub fn last_heap_address() -> Address {
    HEAP_END
}

pub fn total_bytes<VM: VMBinding>(mmtk: &MMTK<VM>) -> usize {
    mmtk.plan.get_total_pages() << LOG_BYTES_IN_PAGE
}

#[cfg(feature = "sanity")]
pub fn scan_region<VM: VMBinding>(mmtk: &MMTK<VM>){
    ::util::sanity::memory_scan::scan_region(&mmtk.plan);
}

pub fn trace_get_forwarded_referent<VM: VMBinding>(trace_local: MMTKHandle<SelectedTraceLocal<VM>>, object: ObjectReference) -> ObjectReference {
    unsafe { trace_local.as_mut() }.get_forwarded_reference(object)
}

pub fn trace_get_forwarded_reference<VM: VMBinding>(trace_local: MMTKHandle<SelectedTraceLocal<VM>>, object: ObjectReference) -> ObjectReference {
    unsafe { trace_local.as_mut() }.get_forwarded_reference(object)
}

pub fn trace_is_live<VM: VMBinding>(trace_local: MMTKHandle<SelectedTraceLocal<VM>>, object: ObjectReference) -> bool{
    unsafe { trace_local.as_mut() }.is_live(object)
}

pub fn trace_retain_referent<VM: VMBinding>(trace_local: MMTKHandle<SelectedTraceLocal<VM>>, object: ObjectReference) -> ObjectReference {
    unsafe { trace_local.as_mut() }.retain_referent(object)
}

pub fn handle_user_collection_request<VM: VMBinding>(mmtk: &MMTK<VM>, tls: OpaquePointer) {
    mmtk.plan.handle_user_collection_request(tls, false);
}

pub fn is_mapped_object<VM: VMBinding>(mmtk: &MMTK<VM>, object: ObjectReference) -> bool {
    mmtk.plan.is_mapped_object(object)
}

pub fn is_mapped_address<VM: VMBinding>(mmtk: &MMTK<VM>, address: Address) -> bool {
    mmtk.plan.is_mapped_address(address)
}

pub fn modify_check<VM: VMBinding>(mmtk: &MMTK<VM>, object: ObjectReference) {
    mmtk.plan.modify_check(object);
}

pub fn add_weak_candidate<VM: VMBinding>(mmtk: &MMTK<VM>, reff: ObjectReference, referent: ObjectReference) {
    mmtk.reference_processors.add_weak_candidate::<VM>(reff, referent);
}

pub fn add_soft_candidate<VM: VMBinding>(mmtk: &MMTK<VM>, reff: ObjectReference, referent: ObjectReference) {
    mmtk.reference_processors.add_soft_candidate::<VM>(reff, referent);
}

pub fn add_phantom_candidate<VM: VMBinding>(mmtk: &MMTK<VM>, reff: ObjectReference, referent: ObjectReference) {
    mmtk.reference_processors.add_phantom_candidate::<VM>(reff, referent);
}

pub fn harness_begin<VM: VMBinding>(mmtk: &MMTK<VM>, tls: OpaquePointer) {
    mmtk.harness_begin(tls);
}

pub fn harness_end<VM: VMBinding>(mmtk: &MMTK<VM>) {
    mmtk.harness_end();
}
