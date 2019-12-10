use ::plan::{TraceLocal, TransitiveClosure};
use super::PLAN;
use ::plan::trace::Trace;
use ::policy::space::Space;
use ::util::{Address, ObjectReference};
use ::util::queue::LocalQueue;
use ::vm::*;
use libc::c_void;
use super::nogc;

pub struct NoGCTraceLocal {
    tls: *mut c_void,
    values: LocalQueue<'static, ObjectReference>,
    root_locations: LocalQueue<'static, Address>,
}

impl TransitiveClosure for NoGCTraceLocal {
    fn process_edge(&mut self, slot: Address) {
        trace!("process_edge({:?})", slot);
        let object: ObjectReference = unsafe { slot.load() };
        let new_object = self.trace_object(object);
        if self.overwrite_reference_during_trace() {
            unsafe { slot.store(new_object) };
        }
    }

    fn process_node(&mut self, object: ObjectReference) {
        trace!("process_node({:?})", object);
        self.values.enqueue(object);
    }
}

impl TraceLocal for NoGCTraceLocal {
    fn overwrite_reference_during_trace(&self) -> bool {
        false
    }
    fn process_roots(&mut self) {
        loop {
            match self.root_locations.dequeue() {
                Some(slot) => {
                    self.process_root_edge(slot, true)
                }
                None => {
                    break;
                }
            }
        }
        debug_assert!(self.root_locations.is_empty());
    }

    fn process_root_edge(&mut self, slot: Address, untraced: bool) {
        trace!("process_root_edge({:?}, {:?})", slot, untraced);
        let object: ObjectReference = unsafe { slot.load() };
        // println!("process_root_edge({:?}, {:?}) -> {:?}", slot, untraced, object);
        let new_object = self.trace_object(object);
        if self.overwrite_reference_during_trace() {
            println!("Overwrite to {:?}", new_object);
            unsafe { slot.store(new_object) };
        }
    }

    fn trace_object(&mut self, object: ObjectReference) -> ObjectReference {
        // println!("trace_object({:?})", object.to_address());
        if object.is_null() {
            trace!("trace_object: object is null");
            return object;
        }
        if object.to_address() >= ::util::heap::layout::vm_layout_constants::HEAP_END {
            println!("Object outside of heap: {:?}", object);
            VMObjectModel::dump_object(object);
            panic!("!!!");
        }
        if PLAN.space.in_space(object) {
            // VMObjectModel::dump_object(object);
            // println!("trace_object: object in copyspace0");
            return PLAN.space.trace_object(self, object);
        }
        if PLAN.vm_space.in_space(object) {
            // println!("trace_object: object in boot space");
            return PLAN.vm_space.trace_object(self, object);
        }

        panic!("No special case for space in trace_object, object = {:?}", object);
    }

    fn complete_trace(&mut self) {
        let id = self.tls;

        self.process_roots();
        debug_assert!(self.root_locations.is_empty());
        loop {
            match self.values.dequeue() {
                Some(object) => {
                    // println!("Scan {:?}", object);
                    VMScanning::scan_object(self, object, id);
                }
                None => {
                    break;
                }
            }
        }
        debug_assert!(self.root_locations.is_empty());
        debug_assert!(self.values.is_empty());
    }

    fn release(&mut self) {
        // Reset the local buffer (throwing away any local entries).
        self.root_locations.reset();
        self.values.reset();
    }

    fn process_interior_edge(&mut self, target: ObjectReference, slot: Address, root: bool) {
        let interior_ref: Address = unsafe { slot.load() };
        let offset = interior_ref - target.to_address();
        let new_target = self.trace_object(target);
        if self.overwrite_reference_during_trace() {
            unsafe { slot.store(new_target.to_address() + offset) };
        }
    }

    fn report_delayed_root_edge(&mut self, slot: Address) {
        // println!("report_delayed_root_edge {:?} -> {:?}", slot, unsafe { slot.load::<Address>() });
        self.root_locations.enqueue(slot);
    }

    fn will_not_move_in_current_collection(&self, obj: ObjectReference) -> bool {
        true
    }

    fn is_live(&self, object: ObjectReference) -> bool {
        if object.is_null() {
            return false;
        }
        if PLAN.space.in_space(object) {
            PLAN.space.is_live(object)
        } else if PLAN.vm_space.in_space(object) {
            true
        } else {
            panic!("Invalid space")
            // false
        }
    }
}

impl NoGCTraceLocal {
    pub fn new(trace: &'static Trace) -> Self {
        Self {
            tls: 0 as *mut c_void,
            values: trace.values.spawn_local(),
            root_locations: trace.root_locations.spawn_local(),
        }
    }

    pub fn init(&mut self, tls: *mut c_void) {
        self.tls = tls;
    }

    pub fn is_empty(&self) -> bool {
        self.root_locations.is_empty() && self.values.is_empty()
    }
}
