/// https://github.com/JikesRVM/JikesRVM/blob/master/MMTk/src/org/mmtk/utility/ForwardingWord.java
use ::util::{Address, ObjectReference};
use ::vm::ObjectModel;
use ::vm::VMObjectModel;
use ::util::OpaquePointer;
use std::sync::atomic::Ordering;
use libc::c_void;

use ::plan::Allocator;

// ...00
const FORWARDING_NOT_TRIGGERED_YET: u8 = 0;
// ...10
const BEING_FORWARDED: u8 = 2;
// ...11
const FORWARDED: u8 = 3;
// ...11
const FORWARDING_MASK: u8 = 3;
const FORWARDING_BITS: usize = 2;

pub fn attempt_to_forward(object: ObjectReference) -> u8 {
    let gc_byte = VMObjectModel::get_gc_byte(object);
    let mut old_value = gc_byte.load(Ordering::SeqCst);
    if old_value & FORWARDING_MASK != FORWARDING_NOT_TRIGGERED_YET {
        return old_value;
    }
    while old_value != gc_byte.compare_and_swap(old_value, old_value | BEING_FORWARDED, Ordering::SeqCst) {
        old_value = gc_byte.load(Ordering::SeqCst);
        if old_value & FORWARDING_MASK != FORWARDING_NOT_TRIGGERED_YET {
            return old_value;
        }
    }
    return old_value;
}

pub fn spin_and_get_forwarded_object(object: ObjectReference, gc_byte: u8) -> ObjectReference {
    let mut gc_byte = gc_byte;
    let gc_byte_slot = VMObjectModel::get_gc_byte(object);
    while gc_byte & FORWARDING_MASK == BEING_FORWARDED {
        gc_byte = gc_byte_slot.load(Ordering::SeqCst);
    }
    if gc_byte & FORWARDING_MASK == FORWARDED {
        let status_word = VMObjectModel::read_available_bits_word(object);
        let a = status_word & !((FORWARDING_MASK as usize) << VMObjectModel::GC_BYTE_OFFSET);
        unsafe { Address::from_usize(a).to_object_reference() }
    } else {
        panic!("Invalid header value 0x{:x} 0x{:x}", gc_byte, VMObjectModel::read_available_bits_word(object));
        object
    }
}

pub fn forward_object(object: ObjectReference, allocator: Allocator, tls: OpaquePointer) -> ObjectReference {
    let new_object = VMObjectModel::copy(object, allocator, tls);
    let forwarded = (FORWARDED as usize) << VMObjectModel::GC_BYTE_OFFSET;
    VMObjectModel::write_available_bits_word(object, new_object.to_address().as_usize() | forwarded);
    // let gc_byte = VMObjectModel::get_gc_byte(object);
    // gc_byte.store(gc_byte.load(Ordering::SeqCst) | FORWARDED, Ordering::SeqCst);
    new_object
}

// pub fn set_forwarding_pointer(object: ObjectReference, ptr: ObjectReference) {
//     VMObjectModel::write_available_bits_word(object, ptr.to_address().as_usize() | FORWARDED as usize);
// }

pub fn is_forwarded(object: ObjectReference) -> bool {
    VMObjectModel::get_gc_byte(object).load(Ordering::Relaxed) & FORWARDING_MASK == FORWARDED
}

pub fn is_forwarded_or_being_forwarded(object: ObjectReference) -> bool {
    VMObjectModel::get_gc_byte(object).load(Ordering::Relaxed) & FORWARDING_MASK != 0
}

pub fn state_is_forwarded_or_being_forwarded(gc_byte: u8) -> bool {
    gc_byte & FORWARDING_MASK != 0
}

pub fn state_is_being_forwarded(gc_byte: u8) -> bool {
    gc_byte & FORWARDING_MASK == BEING_FORWARDED
}

pub fn clear_forwarding_bits(object: ObjectReference) {
    let gc_byte = VMObjectModel::get_gc_byte(object);
    gc_byte.store(gc_byte.load(Ordering::SeqCst) & !FORWARDING_MASK, Ordering::SeqCst);
}

// pub fn extract_forwarding_pointer(forwarding_word: usize) -> ObjectReference {
//     unsafe { Address::from_usize(forwarding_word & (!(FORWARDING_MASK as usize))).to_object_reference() }
// }