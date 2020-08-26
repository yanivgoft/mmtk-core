macro_rules! mock_vm {
    ($name: ident) => {
        {
            use crate::util::{ObjectReference, OpaquePointer, Address};
            use crate::plan::{TransitiveClosure, TraceLocal, MutatorContext, Allocator, Plan};
            use crate::plan::parallel_collector::ParallelCollector;
            use crate::SelectedPlan;
            use crate::vm::*;
            use crate::MMTK;
            use std::sync::atomic::AtomicU8;
            
            struct $name;

            impl VMBinding for $name {
                type VMObjectModel = Self;
                type VMScanning = Self;
                type VMCollection = Self;
                type VMActivePlan = Self;
                type VMReferenceGlue = Self;
            }
            
            impl ObjectModel<$name> for $name {
                const GC_BYTE_OFFSET: usize = 0;
            
                fn get_gc_byte(_object: ObjectReference) -> &'static AtomicU8 {
                    unimplemented!()
                }
            
                fn copy(_from: ObjectReference, _allocator: Allocator, _tls: OpaquePointer) -> ObjectReference {
                    unimplemented!()
                }
            
                fn copy_to(_from: ObjectReference, _to: ObjectReference, _region: Address) -> Address {
                    unimplemented!()
                }
            
                fn get_reference_when_copied_to(_from: ObjectReference, _to: Address) -> ObjectReference {
                    unimplemented!()
                }
            
                fn get_size_when_copied(_object: ObjectReference) -> usize {
                    unimplemented!()
                }
            
                fn get_align_when_copied(_object: ObjectReference) -> usize {
                    unimplemented!()
                }
            
                fn get_align_offset_when_copied(_object: ObjectReference) -> isize {
                    unimplemented!()
                }
            
                fn get_current_size(_object: ObjectReference) -> usize {
                    unimplemented!()
                }
            
                fn get_next_object(_object: ObjectReference) -> ObjectReference {
                    unimplemented!()
                }
            
                unsafe fn get_object_from_start_address(_start: Address) -> ObjectReference {
                    unimplemented!()
                }
            
                fn get_object_end_address(_object: ObjectReference) -> Address {
                    unimplemented!()
                }
            
                fn get_type_descriptor(_reference: ObjectReference) -> &'static [i8] {
                    unimplemented!()
                }
            
                fn is_array(_object: ObjectReference) -> bool {
                    unimplemented!()
                }
            
                fn is_primitive_array(_object: ObjectReference) -> bool {
                    unimplemented!()
                }
            
                fn get_array_length(_object: ObjectReference) -> usize {
                    unimplemented!()
                }
            
                fn attempt_available_bits(_object: ObjectReference, _old: usize, _new: usize) -> bool {
                    unimplemented!()
                }
            
                fn prepare_available_bits(_object: ObjectReference) -> usize {
                    unimplemented!()
                }
            
                fn write_available_byte(_object: ObjectReference, _val: u8) {
                    unimplemented!()
                }
            
                fn read_available_byte(_object: ObjectReference) -> u8 {
                    unimplemented!()
                }
            
                fn write_available_bits_word(_object: ObjectReference, _val: usize) {
                    unimplemented!()
                }
            
                fn read_available_bits_word(_object: ObjectReference) -> usize {
                    unimplemented!()
                }
            
                fn gc_header_offset() -> isize {
                    unimplemented!()
                }
            
                fn object_start_ref(_object: ObjectReference) -> Address {
                    unimplemented!()
                }
            
                fn ref_to_address(object: ObjectReference) -> Address {
                    object.to_address()
                }
            
                fn is_acyclic(_typeref: ObjectReference) -> bool {
                    unimplemented!()
                }
            
                fn dump_object(_object: ObjectReference) {
                    unimplemented!()
                }
            
                fn get_array_base_offset() -> isize {
                    unimplemented!()
                }
            
                fn array_base_offset_trapdoor<T>(_object: T) -> isize {
                    unimplemented!()
                }
            
                fn get_array_length_offset() -> isize {
                    unimplemented!()
                }
            }
            
            impl Scanning<$name> for $name {
                fn scan_object<T: TransitiveClosure>(_trace: &mut T, _object: ObjectReference, _tls: OpaquePointer) {
                    unimplemented!()
                }
            
                fn reset_thread_counter() {
                    unimplemented!()
                }
            
                fn notify_initial_thread_scan_complete(_partial_scan: bool, _tls: OpaquePointer) {
                    unimplemented!()
                }
            
                fn compute_static_roots<T: TraceLocal>(_trace: &mut T, _tls: OpaquePointer) {
                    unimplemented!()
                }
            
                fn compute_global_roots<T: TraceLocal>(_trace: &mut T, _tls: OpaquePointer) {
                    unimplemented!()
                }
            
                fn compute_thread_roots<T: TraceLocal>(_trace: &mut T, _tls: OpaquePointer) {
                    unimplemented!()
                }
            
                fn compute_new_thread_roots<T: TraceLocal>(_trace: &mut T, _tls: OpaquePointer) {
                    unimplemented!()
                }
            
                fn compute_bootimage_roots<T: TraceLocal>(_trace: &mut T, _tls: OpaquePointer) {
                    unimplemented!()
                }
            
                fn supports_return_barrier() -> bool {
                    unimplemented!()
                }
            }
            
            impl Collection<$name> for $name {
                fn stop_all_mutators(_tls: OpaquePointer) {
                    unimplemented!()
                }
            
                fn resume_mutators(_tls: OpaquePointer) {
                    unimplemented!()
                }
            
                fn block_for_gc(_tls: OpaquePointer) {
                    unimplemented!();
                }
            
                fn spawn_worker_thread<T: ParallelCollector<$name>>(_tls: OpaquePointer, _ctx: Option<&mut T>) {
                    unimplemented!();
                }
            
                fn prepare_mutator<T: MutatorContext<$name>>(_tls: OpaquePointer, _mutator: &T) {
                    unimplemented!()
                }
            }
            
            impl ActivePlan<$name> for $name {
                fn global() -> &'static SelectedPlan<$name> {
                    // This might be an issue as we cannot provide an implementation of this function, and mmtk does use it.
                    unreachable!()
                }
            
                unsafe fn collector(_tls: OpaquePointer) -> &'static mut <SelectedPlan<$name> as Plan<$name>>::CollectorT {
                    unimplemented!()
                }
            
                unsafe fn is_mutator(_tls: OpaquePointer) -> bool {
                    // FIXME
                    true
                }
            
                unsafe fn mutator(_tls: OpaquePointer) -> &'static mut <SelectedPlan<$name> as Plan<$name>>::MutatorT {
                    unimplemented!()
                }
            
                fn collector_count() -> usize {
                    unimplemented!()
                }
            
                fn reset_mutator_iterator() {
                    unimplemented!()
                }
            
                fn get_next_mutator() -> Option<&'static mut <SelectedPlan<$name> as Plan<$name>>::MutatorT> {
                    unimplemented!()
                }
            }
            
            impl ReferenceGlue<$name> for $name {
                fn set_referent(_reference: ObjectReference, _referent: ObjectReference) {
                    unimplemented!()
                }
                fn get_referent(_object: ObjectReference) -> ObjectReference {
                    unimplemented!()
                }
                fn process_reference<T: TraceLocal>(_trace: &mut T, _reference: ObjectReference, _tls: OpaquePointer) -> ObjectReference {
                    unimplemented!()
                }
            }


            MMTK::<$name>::new()
        }
    }
}
