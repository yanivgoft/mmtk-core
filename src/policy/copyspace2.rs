use ::util::heap::PageResource;
use ::util::heap::MonotonePageResource;
use ::util::heap::VMRequest;
use ::policy::space::{Space, CommonSpace};
use ::util::{Address, ObjectReference};
use ::plan::TransitiveClosure;
use ::vm::ObjectModel;
use ::vm::VMObjectModel;
use ::plan::Allocator;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

const META_DATA_PAGES_PER_REGION: usize = 0;

#[derive(Debug)]
pub struct CopySpace {
    common: UnsafeCell<CommonSpace<MonotonePageResource<CopySpace>>>,
    from_space: bool,
}

impl Space for CopySpace {
    type PR = MonotonePageResource<CopySpace>;

    fn common(&self) -> &CommonSpace<Self::PR> {
        unsafe { &*self.common.get() }
    }
    unsafe fn unsafe_common_mut(&self) -> &mut CommonSpace<Self::PR> {
        &mut *self.common.get()
    }

    fn init(&mut self) {
        // Borrow-checker fighting so that we can have a cyclic reference
        let me = unsafe { &*(self as *const Self) };

        let common_mut = self.common_mut();
        if common_mut.vmrequest.is_discontiguous() {
            common_mut.pr = Some(MonotonePageResource::new_discontiguous(META_DATA_PAGES_PER_REGION));
        } else {
            common_mut.pr = Some(MonotonePageResource::new_contiguous(common_mut.start, common_mut.extent, META_DATA_PAGES_PER_REGION));
        }
        common_mut.pr.as_mut().unwrap().bind_space(me);
    }

    fn is_live(&self, object: ObjectReference) -> bool {
        ForwardingWord::is_forwarded(object)
    }

    fn is_movable(&self) -> bool {
        true
    }

    fn release_multiple_pages(&mut self, _start: Address) {
        panic!("copyspace only releases pages enmasse")
    }
}

impl CopySpace {
    pub fn new(name: &'static str, from_space: bool, zeroed: bool, vmrequest: VMRequest) -> Self {
        CopySpace {
            common: UnsafeCell::new(CommonSpace::new(name, true, false, zeroed, vmrequest)),
            from_space,
        }
    }

    pub fn prepare(&mut self, from_space: bool) {
        self.from_space = from_space;
    }

    pub unsafe fn release(&mut self) {
        self.common().pr.as_ref().unwrap().reset();
        self.from_space = false;
    }

    pub fn trace_object<T: TransitiveClosure>(&self, trace: &mut T, object: ObjectReference, allocator: Allocator, tls: *mut ::libc::c_void) -> ObjectReference {
        if !self.from_space {
            return object; // Already copied
        }

        let status = ForwardingWord::attempt_to_forward(object);
        if ForwardingWord::state_is_forwarded_or_being_forwarded(status) {
            ForwardingWord::spin_and_get_forwarded_object(object, status)
        } else {
            let new_object = VMObjectModel::copy(object, allocator, tls);
            ForwardingWord::set_forwarding_pointer(object, new_object);
            trace.process_node(new_object);
            new_object
        }
    }

    pub fn post_copy(o: ObjectReference) {
        ForwardingWord::clear_forwarding_bits(o)
    }
}



struct ForwardingWord;

impl ForwardingWord {
    #[cfg(feature = "openjdk")]
    const BIT_SHIFT: usize = 62;
    #[cfg(feature = "jikesrvm")]
    const BIT_SHIFT: usize = 0;
    const FORWARDING_MASK: usize = 0b11 << Self::BIT_SHIFT;
    const FORWARDING_NOT_TRIGGERED_YET: usize = 0b00 << Self::BIT_SHIFT;
    const BEING_FORWARDED: usize = 0b10 << Self::BIT_SHIFT;
    const FORWARDED: usize = 0b11 << Self::BIT_SHIFT;

    fn header(o: ObjectReference) -> &'static AtomicUsize {
        unsafe { ::std::mem::transmute(o) }
    }

    fn is_forwarded(o: ObjectReference) -> bool {
        let h = Self::header(o).load(Ordering::SeqCst);
        h & Self::FORWARDING_MASK == Self::FORWARDED
    }

    fn attempt_to_forward(o: ObjectReference) -> usize {
        let header = Self::header(o);
        let mut old_value = header.load(Ordering::SeqCst);
        loop {
            if old_value & Self::FORWARDING_MASK != Self::FORWARDING_NOT_TRIGGERED_YET {
                return old_value;
            }
            if header.compare_and_swap(old_value, old_value | Self::BEING_FORWARDED, Ordering::SeqCst) == old_value {
                return old_value;
            }
            old_value = header.load(Ordering::SeqCst);
        }
    }

    fn state_is_forwarded_or_being_forwarded(status: usize) -> bool {
        status & Self::FORWARDING_MASK != 0
    }

    fn spin_and_get_forwarded_object(o: ObjectReference, mut status: usize) -> ObjectReference {
        let header = Self::header(o);
        while status & Self::FORWARDING_MASK == Self::BEING_FORWARDED {
            status = header.load(Ordering::SeqCst);
        }
        if status & Self::FORWARDING_MASK == Self::FORWARDED {
            unsafe { Address::from_usize(status & !Self::FORWARDING_MASK).to_object_reference() }
        } else {
            panic!("Invalid status word 0b{:b}", status);
        }
    }

    fn set_forwarding_pointer(old: ObjectReference, new: ObjectReference) {
        Self::header(old).store(new.to_address().as_usize() | Self::FORWARDED, Ordering::SeqCst);
    }

    fn clear_forwarding_bits(o: ObjectReference) {
        let header = Self::header(o);
        let status = header.load(Ordering::SeqCst);
        debug_assert!(status & Self::FORWARDING_MASK == Self::BEING_FORWARDED);
        header.store(status & !Self::FORWARDING_MASK, Ordering::SeqCst);
    }
}

