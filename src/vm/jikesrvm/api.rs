use vm::jikesrvm::JTOC_BASE;
use mm::memory_manager;
use libc::c_void;
use util::{Address, OpaquePointer, ObjectReference};
use vm::jikesrvm::JikesRVM;
use plan::Allocator;

#[no_mangle]
pub unsafe extern fn jikesrvm_gc_init(jtoc: *mut c_void, heap_size: usize) {
    JTOC_BASE = Address::from_mut_ptr(jtoc);
    ::vm::jikesrvm::BOOT_THREAD
        = OpaquePointer::from_address(::vm::jikesrvm::collection::VMCollection::thread_from_id(1));
    memory_manager::gc_init(heap_size);
    debug_assert!(54 == ::vm::jikesrvm::JikesRVM::test(44));
    debug_assert!(112 == ::vm::jikesrvm::JikesRVM::test2(45, 67));
    debug_assert!(731 == ::vm::jikesrvm::JikesRVM::test3(21, 34, 9, 8));
}

#[no_mangle]
pub unsafe extern fn alloc(mutator: *mut c_void, size: usize,
                           align: usize, offset: isize, allocator: Allocator) -> *mut c_void {
    memory_manager::alloc::<JikesRVM>(mutator, size, align, offset, allocator)
}

#[no_mangle]
pub unsafe extern fn alloc_slow(mutator: *mut c_void, size: usize,
                                align: usize, offset: isize, allocator: Allocator) -> *mut c_void {
    memory_manager::alloc_slow::<JikesRVM>(mutator, size, align, offset, allocator)
}

#[no_mangle]
pub unsafe extern fn post_alloc(mutator: *mut c_void, refer: ObjectReference, type_refer: ObjectReference,
                                bytes: usize, allocator: Allocator) {
    memory_manager::post_alloc::<JikesRVM>(mutator, refer, type_refer, bytes, allocator)
}

#[no_mangle]
pub unsafe extern fn mmtk_malloc(size: usize) -> *mut c_void {
    memory_manager::mmtk_malloc::<JikesRVM>(size)
}

#[no_mangle]
pub unsafe extern fn report_delayed_root_edge(trace_local: *mut c_void, addr: *mut c_void) {
    memory_manager::report_delayed_root_edge::<JikesRVM>(trace_local, addr)
}

#[no_mangle]
pub unsafe extern fn will_not_move_in_current_collection(trace_local: *mut c_void, obj: *mut c_void) -> bool {
    memory_manager::will_not_move_in_current_collection::<JikesRVM>(trace_local, obj)
}

#[no_mangle]
pub unsafe extern fn process_interior_edge(trace_local: *mut c_void, target: *mut c_void, slot: *mut c_void, root: bool) {
    memory_manager::process_interior_edge::<JikesRVM>(trace_local, target, slot, root)
}

#[no_mangle]
pub unsafe extern fn start_worker(tls: OpaquePointer, worker: *mut c_void) {
    memory_manager::start_worker::<JikesRVM>(tls, worker)
}

#[no_mangle]
pub unsafe extern fn enable_collection(tls: OpaquePointer) {
    memory_manager::enable_collection::<JikesRVM>(tls)
}

#[no_mangle]
pub unsafe extern fn trace_get_forwarded_referent(trace_local: *mut c_void, object: ObjectReference) -> ObjectReference{
    memory_manager::trace_get_forwarded_referent::<JikesRVM>(trace_local, object)
}

#[no_mangle]
pub unsafe extern fn trace_get_forwarded_reference(trace_local: *mut c_void, object: ObjectReference) -> ObjectReference{
    memory_manager::trace_get_forwarded_reference::<JikesRVM>(trace_local, object)
}

#[no_mangle]
pub unsafe extern fn trace_is_live(trace_local: *mut c_void, object: ObjectReference) -> bool{
    memory_manager::trace_is_live::<JikesRVM>(trace_local, object)
}

#[no_mangle]
pub unsafe extern fn trace_retain_referent(trace_local: *mut c_void, object: ObjectReference) -> ObjectReference{
    memory_manager::trace_retain_referent::<JikesRVM>(trace_local, object)
}

#[no_mangle]
pub extern fn handle_user_collection_request(tls: OpaquePointer) {
    memory_manager::handle_user_collection_request::<JikesRVM>(tls);
}

#[no_mangle]
pub unsafe extern fn add_weak_candidate(reff: *mut c_void, referent: *mut c_void) {
    memory_manager::add_weak_candidate::<JikesRVM>(reff, referent)
}

#[no_mangle]
pub unsafe extern fn add_soft_candidate(reff: *mut c_void, referent: *mut c_void) {
    memory_manager::add_soft_candidate::<JikesRVM>(reff, referent)
}

#[no_mangle]
pub unsafe extern fn add_phantom_candidate(reff: *mut c_void, referent: *mut c_void) {
    memory_manager::add_phantom_candidate::<JikesRVM>(reff, referent)
}
