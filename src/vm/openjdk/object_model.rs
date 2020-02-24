use ::vm::object_model::ObjectModel;
use ::util::{Address, ObjectReference};
use ::plan::Allocator;
use ::util::OpaquePointer;
use std::sync::atomic::{AtomicUsize, Ordering};
use super::UPCALLS;
use libc::c_void;
use ::vm::*;
use plan::collector_context::CollectorContext;
use std::sync::atomic::AtomicU8;

pub struct VMObjectModel {}

impl ObjectModel for VMObjectModel {
    const GC_BYTE_OFFSET: usize = 56;
    fn get_gc_byte(o: ObjectReference) -> &'static AtomicU8 {
        unsafe {
            &*(o.to_address() + Self::GC_BYTE_OFFSET / 8).to_ptr::<AtomicU8>()
        }
    }
    fn copy(from: ObjectReference, allocator: Allocator, tls: OpaquePointer) -> ObjectReference {
        let bytes = unsafe { ((*UPCALLS).get_object_size)(from) };
        let context = unsafe { VMActivePlan::collector(tls) };
        let dst = context.alloc_copy(from, bytes, ::std::mem::size_of::<usize>(), 0, allocator);
        // Copy
        let src = from.to_address();
        for i in 0..bytes {
            unsafe { (dst + i).store((src + i).load::<u8>()) };
        }
        let to_obj = unsafe { dst.to_object_reference() };
        context.post_copy(to_obj, unsafe { Address::zero() }, bytes, allocator);
        to_obj
    }

    fn copy_to(from: ObjectReference, to: ObjectReference, region: Address) -> Address {
        unimplemented!()
    }

    fn get_reference_when_copied_to(from: ObjectReference, to: Address) -> ObjectReference {
        unimplemented!()
    }

    fn get_size_when_copied(object: ObjectReference) -> usize {
        unimplemented!()
    }

    fn get_align_when_copied(object: ObjectReference) -> usize {
        unimplemented!()
    }

    fn get_align_offset_when_copied(object: ObjectReference) -> isize {
        unimplemented!()
    }

    fn get_current_size(object: ObjectReference) -> usize {
        unimplemented!()
    }

    fn get_next_object(object: ObjectReference) -> ObjectReference {
        unimplemented!()
    }

    unsafe fn get_object_from_start_address(start: Address) -> ObjectReference {
        unimplemented!()
    }

    fn get_object_end_address(object: ObjectReference) -> Address {
        unimplemented!()
    }

    fn get_type_descriptor(reference: ObjectReference) -> &'static [i8] {
        unimplemented!()
    }

    fn is_array(object: ObjectReference) -> bool {
        unimplemented!()
    }

    fn is_primitive_array(object: ObjectReference) -> bool {
        unimplemented!()
    }

    fn get_array_length(object: ObjectReference) -> usize {
        unimplemented!()
    }

    fn attempt_available_bits(object: ObjectReference, old: usize, new: usize) -> bool {
        let mark_slot: AtomicUsize = unsafe { ::std::mem::transmute(object) };
        mark_slot.compare_and_swap(old, new, Ordering::SeqCst) == old
    }

    fn prepare_available_bits(object: ObjectReference) -> usize {
        // println!("Object = {:?}", object);
        unsafe { object.to_address().load() }
        // unimplemented!()
    }

    fn write_available_byte(object: ObjectReference, val: u8) {
        unimplemented!()
    }

    fn read_available_byte(object: ObjectReference) -> u8 {
        unimplemented!()
    }

    fn write_available_bits_word(object: ObjectReference, val: usize) {
        let loc = unsafe {
            &*(object.to_address().as_usize() as *const AtomicUsize)
        };
        loc.store(val, Ordering::SeqCst);
    }

    fn read_available_bits_word(object: ObjectReference) -> usize {
        let loc = unsafe {
            &*(object.to_address().as_usize() as *const AtomicUsize)
        };
        loc.load(Ordering::SeqCst)
        // unsafe { object.to_address().load() }
    }

    fn GC_HEADER_OFFSET() -> isize {
        unimplemented!()
    }

    fn object_start_ref(object: ObjectReference) -> Address {
        object.to_address()
    }

    fn ref_to_address(object: ObjectReference) -> Address {
        object.to_address()
    }

    fn is_acyclic(typeref: ObjectReference) -> bool {
        unimplemented!()
    }

    fn dump_object(object: ObjectReference) {
        unsafe {
            ((*UPCALLS).dump_object)(::std::mem::transmute(object));
        }
    }

    fn get_array_base_offset() -> isize {
        unimplemented!()
    }

    fn array_base_offset_trapdoor<T>(o: T) -> isize {
        unimplemented!()
    }

    fn get_array_length_offset() -> isize {
        unimplemented!()
    }
}
