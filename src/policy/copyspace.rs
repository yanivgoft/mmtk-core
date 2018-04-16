use std::sync::Mutex;

use ::util::heap::PageResource;
use ::util::heap::MonotonePageResource;
use ::util::heap::VMRequest;
use ::util::constants::CARD_META_PAGES_PER_REGION;

use ::policy::space::*;
use ::util::{Address, ObjectReference};
use ::plan::TransitiveClosure;
use ::util::forwarding_word as ForwardingWord;
use ::vm::ObjectModel;
use ::vm::VMObjectModel;
use ::plan::Allocator;
use ::util::class::*;

use std::cell::UnsafeCell;

const META_DATA_PAGES_PER_REGION: usize = CARD_META_PAGES_PER_REGION;

type Common = CommonSpace<MonotonePageResource<CopySpace>>;
#[derive(Debug)]
pub struct CopySpace {
    common: UnsafeCell<Common>,
    from_space: bool,
}

impl AbstractSpace for CopySpace {
    type PR = MonotonePageResource<CopySpace>;

    fn init(this: &mut Self) {
        // Borrow-checker fighting so that we can have a cyclic reference
        let me = unsafe { &*(this as *const Self) };

        let common_mut = this.common_mut();
        if common_mut.vmrequest.is_discontiguous() {
            common_mut.pr = Some(MonotonePageResource::new_discontiguous(
                META_DATA_PAGES_PER_REGION));
        } else {
            common_mut.pr = Some(MonotonePageResource::new_contiguous(common_mut.start,
                                                                      common_mut.extent,
                                                                      META_DATA_PAGES_PER_REGION));
        }
        common_mut.pr.as_mut().unwrap().bind_space(me);
    }
}
impl AbstractClass<Common> for CopySpace {
    type This = Self;
    fn common(this: &Self::This) -> &Common { unsafe { &*this.common.get() } }
    fn common_mut(this: &mut Self::This) -> &mut Common  { unsafe { &mut *this.common.get() } }
}
impl AbstractMutableClass<Common> for CopySpace {
    unsafe fn unsafe_common_mut(this: &Self::This) -> &mut Common  { &mut *this.common.get() }
}

impl CopySpace {
    pub fn new(name: &'static str, from_space: bool, zeroed: bool, vmrequest: VMRequest) -> Self {
        CopySpace {
            common: UnsafeCell::new(Common::new(name, true, false, zeroed, vmrequest)),
            from_space,
        }
    }

    pub fn prepare(&mut self, from_space: bool) {
        self.from_space = from_space;
    }

    pub fn release(&mut self) {
        self.common().pr.as_ref().unwrap().reset();
        self.from_space = false;
    }

    pub fn trace_object<T: TransitiveClosure>(
        &self,
        trace: &mut T,
        object: ObjectReference,
        allocator: Allocator,
        thread_id: usize,
    ) -> ObjectReference
    {
        trace!("copyspace.trace_object(, {:?}, {:?}, {:?})", object, allocator, thread_id);
        if !self.from_space {
            return object;
        }
        trace!("attempting to forward");
        let mut forwarding_word = ForwardingWord::attempt_to_forward(object);
        trace!("checking if object is being forwarded");
        if ForwardingWord::state_is_forwarded_or_being_forwarded(forwarding_word) {
            trace!("... yes it is");
            while ForwardingWord::state_is_being_forwarded(forwarding_word) {
                forwarding_word = VMObjectModel::read_available_bits_word(object);
            }
            trace!("Returning");
            return ForwardingWord::extract_forwarding_pointer(forwarding_word);
        } else {
            trace!("... no it isn't. Copying");
            let new_object = VMObjectModel::copy(object, allocator, thread_id);
            trace!("Setting forwarding pointer");
            ForwardingWord::set_forwarding_pointer(object, new_object);
            trace!("Forwarding pointer");
            trace.process_node(new_object);
            trace!("Copying [{:?} -> {:?}]", object, new_object);
            return new_object;
        }
    }
}