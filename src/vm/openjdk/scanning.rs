use ::vm::Scanning;
use ::plan::{TransitiveClosure, TraceLocal};
use ::util::{ObjectReference, SynchronizedCounter};
use super::UPCALLS;

use libc::c_void;

static COUNTER: SynchronizedCounter = SynchronizedCounter::new(0);

pub struct VMScanning {}

impl Scanning for VMScanning {
    fn scan_object<T: TransitiveClosure>(trace: &mut T, object: ObjectReference, tls: *mut c_void) {
        // unimplemented!()
        unsafe {
            ((*UPCALLS).scan_object)(::std::mem::transmute(trace), ::std::mem::transmute(object), tls);
        }
    }

    fn reset_thread_counter() {
        COUNTER.reset();
    }

    fn notify_initial_thread_scan_complete(partial_scan: bool, tls: *mut c_void) {
        // unimplemented!()
        // TODO
    }

    fn compute_static_roots<T: TraceLocal>(trace: &mut T, tls: *mut c_void) {
        unimplemented!()
    }

    fn compute_global_roots<T: TraceLocal>(trace: &mut T, tls: *mut c_void) {
        unimplemented!()
    }

    fn compute_thread_roots<T: TraceLocal>(trace: &mut T, tls: *mut c_void) {
        unsafe {
            ((*UPCALLS).compute_thread_roots)(::std::mem::transmute(trace), tls);
        }
    }

    fn compute_new_thread_roots<T: TraceLocal>(trace: &mut T, tls: *mut c_void) {
        unimplemented!()
    }

    fn compute_bootimage_roots<T: TraceLocal>(trace: &mut T, tls: *mut c_void) {
        unimplemented!()
    }

    fn supports_return_barrier() -> bool {
        unimplemented!()
    }
}