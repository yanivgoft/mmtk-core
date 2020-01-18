use ::plan::{TraceLocal, TransitiveClosure};
use super::PLAN;
use ::plan::trace::Trace;
use ::policy::space::Space;
use ::util::{Address, ObjectReference};
use ::util::queue::LocalQueue;
use ::vm::Scanning;
use ::vm::VMScanning;
use libc::c_void;
use super::ss;

pub struct SSTraceLocal {
    tls: *mut c_void,
    values: LocalQueue<'static, ObjectReference>,
    root_locations: LocalQueue<'static, Address>,
}

impl TransitiveClosure for SSTraceLocal {
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

impl TraceLocal for SSTraceLocal {
    fn process_roots(&mut self) {
        while let Some(slot) = self.root_locations.dequeue() {
            self.process_root_edge(slot, true);
        }
        debug_assert!(self.root_locations.is_empty());
    }

    fn process_root_edge(&mut self, slot: Address, untraced: bool) {
        trace!("process_root_edge({:?}, {:?})", slot, untraced);
        let object: ObjectReference = unsafe { slot.load() };
        // println!("process_root_edge({:?}, {:?}) -> {:?}", slot, untraced, object);
        let new_object = self.trace_object(object);
        // println!("Overwrite to {:?}", new_object);
        if self.overwrite_reference_during_trace() {
            unsafe { slot.store(new_object) };
        }
    }

    fn trace_object(&mut self, object: ObjectReference) -> ObjectReference {
        trace!("trace_object({:?})", object.to_address());
        let tls = self.tls;

        if object.is_null() {
            return object;
        }
        if PLAN.copyspace0.in_space(object) {
            return PLAN.copyspace0.trace_object(self, object, ss::ALLOC_SS, tls);
        }
        if PLAN.copyspace1.in_space(object) {
            return PLAN.copyspace1.trace_object(self, object, ss::ALLOC_SS, tls);
        }
        if PLAN.versatile_space.in_space(object) {
            return PLAN.versatile_space.trace_object(self, object);
        }
        if PLAN.vm_space.in_space(object) {
            return PLAN.vm_space.trace_object(self, object);
        }

        panic!("No special case for space in trace_object, object = {:?}", object);
    }

    fn complete_trace(&mut self) {
        let id = self.tls;
        self.process_roots();
        debug_assert!(self.root_locations.is_empty());
        while let Some(object) = self.values.dequeue() {
            VMScanning::scan_object(self, object, id);
        }
        debug_assert!(self.root_locations.is_empty());
        debug_assert!(self.values.is_empty());
    }

    fn release(&mut self) {
        // Reset the local buffer (throwing away any local entries).
        self.root_locations.reset();
        self.values.reset();
    }

    fn process_interior_edge(&mut self, target: ObjectReference, slot: Address, _root: bool) {
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
        (PLAN.hi && !PLAN.copyspace0.in_space(obj)) ||
            (!PLAN.hi && !PLAN.copyspace1.in_space(obj))
    }

    fn is_live(&self, object: ObjectReference) -> bool {
        if object.is_null() {
            return false;
        }
        if PLAN.copyspace0.in_space(object) {
            if PLAN.hi {
                return PLAN.copyspace0.is_live(object);
            } else {
                return true;
            }
        }
        if PLAN.copyspace1.in_space(object) {
            if PLAN.hi {
                return true;
            } else {
                return PLAN.copyspace1.is_live(object);
            }
        }
        // FIXME is it actually alive?
        if PLAN.versatile_space.in_space(object) {
            return true;
        }
        if PLAN.vm_space.in_space(object) {
            return true;
        }
        panic!("Invalid space")
    }
}

impl SSTraceLocal {
    pub fn new(ss_trace: &'static Trace) -> Self {
        SSTraceLocal {
            tls: 0 as *mut c_void,
            values: ss_trace.values.spawn_local(),
            root_locations: ss_trace.root_locations.spawn_local(),
        }
    }

    pub fn init(&mut self, tls: *mut c_void) {
        self.tls = tls;
    }

    pub fn is_empty(&self) -> bool {
        self.root_locations.is_empty() && self.values.is_empty()
    }
}
