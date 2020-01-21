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

use ::vm::{Collection, VMCollection};

#[cfg(feature = "jikesrvm")]
use ::vm::jikesrvm::JTOC_BASE;

#[cfg(feature = "openjdk")]
use ::vm::openjdk::UPCALLS;

use ::util::{Address, ObjectReference};

use ::plan::selected_plan;
use self::selected_plan::SelectedPlan;

use ::plan::Allocator;
use util::constants::LOG_BYTES_IN_PAGE;
use util::heap::layout::vm_layout_constants::HEAP_START;
use util::heap::layout::vm_layout_constants::HEAP_END;
use util::OpaquePointer;
use crate::mmtk::SINGLETON;
use crate::mmtk::OPTIONS_PROCESSOR;
use util::opaque_pointer::UNINITIALIZED_OPAQUE_POINTER;
use vm::VMBinding;

#[no_mangle]
#[cfg(any(feature = "jikesrvm", feature = "openjdk"))]
pub extern fn start_control_collector(tls: OpaquePointer) {
    SINGLETON.plan.common().control_collector_context.run(tls);
}

#[no_mangle]
#[cfg(not(any(feature = "jikesrvm", feature = "openjdk")))]
pub extern fn start_control_collector(tls: OpaquePointer) {
    panic!("Cannot call start_control_collector when not building for JikesRVM or OpenJDK");
}

#[cfg(any(feature = "jikesrvm", feature = "openjdk"))]
pub unsafe fn gc_init(heap_size: usize) {
    ::util::logger::init().unwrap();
    SINGLETON.plan.gc_init(heap_size, &SINGLETON.vm_map);
}

#[no_mangle]
#[cfg(not(any(feature = "jikesrvm", feature = "openjdk")))]
pub unsafe extern fn gc_init(heap_size: usize) {
    ::util::logger::init().unwrap();
    SINGLETON.plan.gc_init(heap_size, &SINGLETON.vm_map);
    SINGLETON.plan.common().initialized.store(true, Ordering::SeqCst);
    thread::spawn(|| {
        SINGLETON.plan.common().control_collector_context.run(UNINITIALIZED_OPAQUE_POINTER )
    });
}

#[no_mangle]
pub extern fn bind_mutator(tls: OpaquePointer) -> *mut c_void {
    SelectedPlan::bind_mutator(&SINGLETON.plan, tls)
}

pub unsafe fn alloc<VM: VMBinding>(mutator: *mut c_void, size: usize,
             align: usize, offset: isize, allocator: Allocator) -> *mut c_void {
    let local = &mut *(mutator as *mut <SelectedPlan<VM> as Plan<VM>>::MutatorT);
    local.alloc(size, align, offset, allocator).as_usize() as *mut c_void
}

#[inline(never)]
pub unsafe fn alloc_slow<VM: VMBinding>(mutator: *mut c_void, size: usize,
                  align: usize, offset: isize, allocator: Allocator) -> *mut c_void {
    let local = &mut *(mutator as *mut <SelectedPlan<VM> as Plan<VM>>::MutatorT);
    local.alloc_slow(size, align, offset, allocator).as_usize() as *mut c_void
}

pub fn post_alloc<VM: VMBinding>(mutator: *mut c_void, refer: ObjectReference, type_refer: ObjectReference,
                         bytes: usize, allocator: Allocator) {
    let local = unsafe {&mut *(mutator as *mut <SelectedPlan<VM> as Plan<VM>>::MutatorT)};
    local.post_alloc(refer, type_refer, bytes, allocator);
}

pub unsafe fn mmtk_malloc<VM: VMBinding>(size: usize) -> *mut c_void {
    alloc::<VM>(null_mut(), size, 1, 0, Allocator::Default)
}

#[no_mangle]
pub extern fn mmtk_free(_ptr: *const c_void) {}

#[no_mangle]
pub extern fn will_never_move(object: ObjectReference) -> bool {
    SINGLETON.plan.will_never_move(object)
}

#[no_mangle]
pub unsafe extern fn is_valid_ref(val: ObjectReference) -> bool {
    SINGLETON.plan.is_valid_ref(val)
}

#[cfg(feature = "sanity")]
pub unsafe fn report_delayed_root_edge<VM: VMBinding>(trace_local: *mut c_void, addr: *mut c_void) {
    use ::util::sanity::sanity_checker::SanityChecker;
    if SINGLETON.plan.common().is_in_sanity() {
        report_delayed_root_edge_inner::<SanityChecker>(trace_local, addr)
    } else {
        report_delayed_root_edge_inner::<<SelectedPlan<VM> as Plan<VM>>::TraceLocalT>(trace_local, addr)
    }
}
#[cfg(not(feature = "sanity"))]
pub unsafe fn report_delayed_root_edge<VM: VMBinding>(trace_local: *mut c_void, addr: *mut c_void) {
    report_delayed_root_edge_inner::<<SelectedPlan<VM> as Plan<VM>>::TraceLocalT>(trace_local, addr)
}
unsafe fn report_delayed_root_edge_inner<T: TraceLocal>(trace_local: *mut c_void, addr: *mut c_void) {
    trace!("report_delayed_root_edge with trace_local={:?}", trace_local);
    let local = &mut *(trace_local as *mut T);
    local.report_delayed_root_edge(Address::from_usize(addr as usize));
    trace!("report_delayed_root_edge returned with trace_local={:?}", trace_local);
}

#[cfg(feature = "sanity")]
pub unsafe fn will_not_move_in_current_collection<VM: VMBinding>(trace_local: *mut c_void, obj: *mut c_void) -> bool {
    use ::util::sanity::sanity_checker::SanityChecker;
    if SINGLETON.plan.common().is_in_sanity() {
        will_not_move_in_current_collection_inner::<SanityChecker>(trace_local, obj)
    } else {
        will_not_move_in_current_collection_inner::<<SelectedPlan<VM> as Plan<VM>>::TraceLocalT>(trace_local, obj)
    }
}
#[cfg(not(feature = "sanity"))]
pub unsafe fn will_not_move_in_current_collection<VM: VMBinding>(trace_local: *mut c_void, obj: *mut c_void) -> bool {
    will_not_move_in_current_collection_inner::<<SelectedPlan<VM> as Plan<VM>>::TraceLocalT>(trace_local, obj)
}
unsafe fn will_not_move_in_current_collection_inner<T: TraceLocal>(trace_local: *mut c_void, obj: *mut c_void) -> bool {
    trace!("will_not_move_in_current_collection({:?}, {:?})", trace_local, obj);
    let local = &mut *(trace_local as *mut T);
    let ret = local.will_not_move_in_current_collection(Address::from_usize(obj as usize).to_object_reference());
    trace!("will_not_move_in_current_collection returned with trace_local={:?}", trace_local);
    ret
}

#[cfg(feature = "sanity")]
pub unsafe fn process_interior_edge<VM: VMBinding>(trace_local: *mut c_void, target: *mut c_void, slot: *mut c_void, root: bool) {
    use ::util::sanity::sanity_checker::SanityChecker;
    if SINGLETON.plan.common().is_in_sanity() {
        process_interior_edge_inner::<SanityChecker>(trace_local, target, slot, root)
    } else {
        process_interior_edge_inner::<<SelectedPlan<VM> as Plan<VM>>::TraceLocalT>(trace_local, target, slot, root)
    }
    trace!("process_interior_root_edge returned with trace_local={:?}", trace_local);
}
#[cfg(not(feature = "sanity"))]
pub unsafe fn process_interior_edge<VM: VMBinding>(trace_local: *mut c_void, target: *mut c_void, slot: *mut c_void, root: bool) {
    process_interior_edge_inner::<<SelectedPlan<VM> as Plan<VM>>::TraceLocalT>(trace_local, target, slot, root)
}
unsafe fn process_interior_edge_inner<T: TraceLocal>(trace_local: *mut c_void, target: *mut c_void, slot: *mut c_void, root: bool) {
    trace!("process_interior_edge with trace_local={:?}", trace_local);
    let local = &mut *(trace_local as *mut T);
    local.process_interior_edge(Address::from_usize(target as usize).to_object_reference(),
                                Address::from_usize(slot as usize), root);
    trace!("process_interior_root_edge returned with trace_local={:?}", trace_local);
}

pub unsafe fn start_worker<VM: VMBinding>(tls: OpaquePointer, worker: *mut c_void) {
    let worker_instance = &mut *(worker as *mut <SelectedPlan<VM> as Plan<VM>>::CollectorT);
    worker_instance.init(tls);
    worker_instance.run(tls);
}

pub unsafe fn enable_collection<VM: VMBinding>(tls: OpaquePointer) {
    (&mut *SINGLETON.plan.common().control_collector_context.workers.get()).init_group(&SINGLETON, tls);
    VM::VMCollection::spawn_worker_thread::<<SelectedPlan<VM> as Plan<VM>>::CollectorT>(tls, null_mut()); // spawn controller thread
    SINGLETON.plan.common().initialized.store(true, Ordering::SeqCst);
}

#[no_mangle]
pub extern fn process(name: *const c_char, value: *const c_char) -> bool {
    let name_str: &CStr = unsafe { CStr::from_ptr(name) };
    let value_str: &CStr = unsafe { CStr::from_ptr(value) };
    let option = &OPTIONS_PROCESSOR;
    unsafe {
        option.process(name_str.to_str().unwrap(), value_str.to_str().unwrap())
    }
}

#[no_mangle]
#[cfg(feature = "openjdk")]
pub extern fn used_bytes() -> usize {
    SINGLETON.plan.get_pages_used() << LOG_BYTES_IN_PAGE
}


#[no_mangle]
pub extern fn free_bytes() -> usize {
    SINGLETON.plan.get_free_pages() << LOG_BYTES_IN_PAGE
}


#[no_mangle]
#[cfg(not(feature = "openjdk"))]
pub extern fn used_bytes() -> usize {
    panic!("Cannot call used_bytes when not building for OpenJDK");
}

#[no_mangle]
pub extern fn starting_heap_address() -> *mut c_void {
    HEAP_START.as_usize() as *mut c_void
}

#[no_mangle]
pub extern fn last_heap_address() -> *mut c_void {
    HEAP_END.as_usize() as *mut c_void
}

#[no_mangle]
pub extern fn total_bytes() -> usize {
    SINGLETON.plan.get_total_pages() << LOG_BYTES_IN_PAGE
}

#[no_mangle]
#[cfg(feature = "openjdk")]
pub extern fn openjdk_max_capacity() -> usize {
    SINGLETON.plan.get_total_pages() << LOG_BYTES_IN_PAGE
}

#[no_mangle]
#[cfg(not(feature = "openjdk"))]
pub extern fn openjdk_max_capacity() -> usize {
    panic!("Cannot call max_capacity when not building for OpenJDK");
}

#[no_mangle]
#[cfg(feature = "openjdk")]
pub extern fn executable() -> bool {
    true
}

#[no_mangle]
#[cfg(not(feature = "openjdk"))]
pub extern fn executable() -> bool {
    panic!("Cannot call executable when not building for OpenJDK")
}

#[no_mangle]
#[cfg(feature = "sanity")]
pub unsafe extern fn scan_region(){
    ::util::sanity::memory_scan::scan_region(&SINGLETON.plan);
}

pub unsafe fn trace_get_forwarded_referent<VM: VMBinding>(trace_local: *mut c_void, object: ObjectReference) -> ObjectReference{
    let local = &mut *(trace_local as *mut <SelectedPlan<VM> as Plan<VM>>::TraceLocalT);
    local.get_forwarded_reference(object)
}

pub unsafe fn trace_get_forwarded_reference<VM: VMBinding>(trace_local: *mut c_void, object: ObjectReference) -> ObjectReference{
    let local = &mut *(trace_local as *mut <SelectedPlan<VM> as Plan<VM>>::TraceLocalT);
    local.get_forwarded_reference(object)
}

pub unsafe fn trace_is_live<VM: VMBinding>(trace_local: *mut c_void, object: ObjectReference) -> bool{
    let local = &mut *(trace_local as *mut <SelectedPlan<VM> as Plan<VM>>::TraceLocalT);
    local.is_live(object)
}

pub unsafe fn trace_retain_referent<VM: VMBinding>(trace_local: *mut c_void, object: ObjectReference) -> ObjectReference{
    let local = &mut *(trace_local as *mut <SelectedPlan<VM> as Plan<VM>>::TraceLocalT);
    local.retain_referent(object)
}

pub fn handle_user_collection_request<VM: VMBinding>(tls: OpaquePointer) {
    SINGLETON.plan.handle_user_collection_request(tls, false);
}

#[no_mangle]
pub extern fn is_mapped_object(object: ObjectReference) -> bool {
    SINGLETON.plan.is_mapped_object(object)
}

#[no_mangle]
pub extern fn is_mapped_address(address: Address) -> bool {
    SINGLETON.plan.is_mapped_address(address)
}

#[no_mangle]
pub extern fn modify_check(object: ObjectReference) {
    SINGLETON.plan.modify_check(object);
}

#[no_mangle]
pub unsafe extern fn add_weak_candidate(reff: *mut c_void, referent: *mut c_void) {
    SINGLETON.reference_processors.add_weak_candidate(
        Address::from_mut_ptr(reff).to_object_reference(),
        Address::from_mut_ptr(referent).to_object_reference());
}

#[no_mangle]
pub unsafe extern fn add_soft_candidate(reff: *mut c_void, referent: *mut c_void) {
    SINGLETON.reference_processors.add_soft_candidate(
        Address::from_mut_ptr(reff).to_object_reference(),
        Address::from_mut_ptr(referent).to_object_reference());
}

#[no_mangle]
pub unsafe extern fn add_phantom_candidate(reff: *mut c_void, referent: *mut c_void) {
    SINGLETON.reference_processors.add_phantom_candidate(
        Address::from_mut_ptr(reff).to_object_reference(),
        Address::from_mut_ptr(referent).to_object_reference());
}

#[no_mangle]
pub extern fn harness_begin(tls: OpaquePointer) {
    SINGLETON.harness_begin(tls);
}

#[no_mangle]
pub extern fn harness_end() {
    SINGLETON.harness_end();
}
