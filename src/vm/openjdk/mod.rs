use std::ptr::null_mut;

use vm::VMBinding;
use util::OpaquePointer;
use libc::c_void;
use util::ObjectReference;
pub mod scanning;
pub mod collection;
pub mod object_model;
pub mod active_plan;
pub mod reference_glue;
pub mod api;

#[repr(C)]
pub struct OpenJDK_Upcalls {
    pub stop_all_mutators: extern "C" fn(tls: OpaquePointer),
    pub resume_mutators: extern "C" fn(tls: OpaquePointer),
    pub spawn_collector_thread: extern "C" fn(tls: OpaquePointer, ctx: *mut c_void),
    pub block_for_gc: extern "C" fn(),
    pub active_collector: extern "C" fn(tls: OpaquePointer) -> *mut c_void,
    pub get_next_mutator: extern "C" fn() -> *mut c_void,
    pub reset_mutator_iterator: extern "C" fn(),
    pub compute_static_roots: extern "C" fn(trace: *mut c_void, tls: OpaquePointer),
    pub compute_global_roots: extern "C" fn(trace: *mut c_void, tls: OpaquePointer),
    pub compute_thread_roots: extern "C" fn(trace: *mut c_void, tls: OpaquePointer),
    pub scan_object: extern "C" fn(trace: *mut c_void, object: *mut c_void, tls: OpaquePointer),
    pub dump_object: extern "C" fn(object: *mut c_void),
    pub get_object_size: extern "C" fn(object: ObjectReference) -> usize,
    pub get_mmtk_mutator: extern "C" fn(tls: OpaquePointer) -> *mut c_void,
    pub is_mutator: extern "C" fn(tls: OpaquePointer) -> bool,
}

pub static mut UPCALLS: *const OpenJDK_Upcalls = null_mut();

pub struct OpenJDK;

impl VMBinding for OpenJDK {
    type VMObjectModel = object_model::VMObjectModel;
    type VMScanning = scanning::VMScanning;
    type VMCollection = collection::VMCollection;
    type VMActivePlan = active_plan::VMActivePlan;
    type VMReferenceGlue = reference_glue::VMReferenceGlue;
}
