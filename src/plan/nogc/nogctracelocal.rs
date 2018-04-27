use ::plan::transitive_closure::TransitiveClosure;
use ::util::address::{Address, ObjectReference};
use ::plan::tracelocal::TraceLocal;

pub struct NoGCTraceLocal {}

impl TransitiveClosure for NoGCTraceLocal {
    fn process_edge(&mut self, _slot: Address) {
        unimplemented!();
    }

    fn process_node(&mut self, _object: ObjectReference) {
        unimplemented!()
    }
}

impl TraceLocal for NoGCTraceLocal {
    fn process_roots(&mut self) {
        unimplemented!();
    }

    fn process_root_edge(&mut self, _slot: Address, _untraced: bool) {
        unimplemented!();
    }

    fn trace_object(&mut self, _object: ObjectReference) -> ObjectReference {
        unimplemented!();
    }

    fn complete_trace(&mut self) {
        unimplemented!();
    }

    fn release(&mut self) {
        unimplemented!();
    }

    fn process_interior_edge(&mut self, _target: ObjectReference, _slot: Address, _root: bool) {
        unimplemented!()
    }
    fn report_delayed_root_edge(&mut self, _slot: Address) {
        unimplemented!()
    }

    fn will_not_move_in_current_collection(&self, _obj: ObjectReference) -> bool {
        true
    }
}

impl NoGCTraceLocal {
    pub fn new() -> Self {
        Self {}
    }
}