use vm::openjdk::{OpenJDK, UPCALLS, OpenJDK_Upcalls};
use mm::memory_manager;
use libc::c_void;
use plan::Allocator;
use std::ptr::null_mut;
use util::{ObjectReference, OpaquePointer};

#[no_mangle]
pub unsafe extern fn openjdk_gc_init(calls: *const OpenJDK_Upcalls, heap_size: usize) {
    UPCALLS = calls;
    memory_manager::gc_init(heap_size);
}

#[no_mangle]
pub unsafe extern fn alloc(mutator: *mut c_void, size: usize,
                    align: usize, offset: isize, allocator: Allocator) -> *mut c_void {
    memory_manager::alloc::<OpenJDK>(mutator, size, align, offset, allocator)
}

#[no_mangle]
pub unsafe extern fn alloc_slow(mutator: *mut c_void, size: usize,
                                        align: usize, offset: isize, allocator: Allocator) -> *mut c_void {
    memory_manager::alloc_slow::<OpenJDK>(mutator, size, align, offset, allocator)
}

#[no_mangle]
pub unsafe extern fn post_alloc(mutator: *mut c_void, refer: ObjectReference, type_refer: ObjectReference,
                                        bytes: usize, allocator: Allocator) {
    memory_manager::post_alloc::<OpenJDK>(mutator, refer, type_refer, bytes, allocator)
}

#[no_mangle]
pub unsafe extern fn mmtk_malloc(size: usize) -> *mut c_void {
    memory_manager::mmtk_malloc::<OpenJDK>(size)
}

#[no_mangle]
pub unsafe extern fn report_delayed_root_edge(trace_local: *mut c_void, addr: *mut c_void) {
    memory_manager::report_delayed_root_edge::<OpenJDK>(trace_local, addr)
}

#[no_mangle]
pub unsafe extern fn will_not_move_in_current_collection(trace_local: *mut c_void, obj: *mut c_void) -> bool {
    memory_manager::will_not_move_in_current_collection::<OpenJDK>(trace_local, obj)
}

#[no_mangle]
pub unsafe extern fn process_interior_edge(trace_local: *mut c_void, target: *mut c_void, slot: *mut c_void, root: bool) {
    memory_manager::process_interior_edge::<OpenJDK>(trace_local, target, slot, root)
}

#[no_mangle]
pub unsafe extern fn start_worker(tls: OpaquePointer, worker: *mut c_void) {
    memory_manager::start_worker::<OpenJDK>(tls, worker)
}

#[no_mangle]
pub unsafe extern fn trace_get_forwarded_referent(trace_local: *mut c_void, object: ObjectReference) -> ObjectReference{
    memory_manager::trace_get_forwarded_referent::<OpenJDK>(trace_local, object)
}

#[no_mangle]
pub unsafe extern fn trace_get_forwarded_reference(trace_local: *mut c_void, object: ObjectReference) -> ObjectReference{
    memory_manager::trace_get_forwarded_reference::<OpenJDK>(trace_local, object)
}

#[no_mangle]
pub unsafe extern fn trace_is_live(trace_local: *mut c_void, object: ObjectReference) -> bool{
    memory_manager::trace_is_live::<OpenJDK>(trace_local, object)
}

#[no_mangle]
pub unsafe extern fn trace_retain_referent(trace_local: *mut c_void, object: ObjectReference) -> ObjectReference{
    memory_manager::trace_retain_referent::<OpenJDK>(trace_local, object)
}

#[no_mangle]
pub extern fn handle_user_collection_request(tls: OpaquePointer) {
    memory_manager::handle_user_collection_request::<OpenJDK>(tls);
}

#[no_mangle]
pub unsafe extern fn add_weak_candidate(reff: *mut c_void, referent: *mut c_void) {
    memory_manager::add_weak_candidate::<OpenJDK>(reff, referent)
}

#[no_mangle]
pub unsafe extern fn add_soft_candidate(reff: *mut c_void, referent: *mut c_void) {
    memory_manager::add_soft_candidate::<OpenJDK>(reff, referent)
}

#[no_mangle]
pub unsafe extern fn add_phantom_candidate(reff: *mut c_void, referent: *mut c_void) {
    memory_manager::add_phantom_candidate::<OpenJDK>(reff, referent)
}

use ::mmtk::SINGLETON;
use plan::Plan;
use util::constants::LOG_BYTES_IN_PAGE;

#[no_mangle]
pub extern fn openjdk_max_capacity() -> usize {
    SINGLETON.plan.get_total_pages() << LOG_BYTES_IN_PAGE
}

#[no_mangle]
pub extern fn executable() -> bool {
    true
}
