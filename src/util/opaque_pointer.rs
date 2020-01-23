use libc::c_void;
use ::util::Address;

// This is mainly used to represent TLS.
// OpaquePointer does not provide any method for dereferencing, as we should not dereference it in MMTk.
// However, there are occurrences that we may need to dereference tls in the VM binding code.
// In JikesRVM's implementation of ActivePlan, we need to dereference tls to get mutator and collector context.
// This is done by transmute (unsafe).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct OpaquePointer(*mut c_void);

unsafe impl Sync for OpaquePointer {}
unsafe impl Send for OpaquePointer {}

pub const UNINITIALIZED_OPAQUE_POINTER: OpaquePointer = OpaquePointer(0 as *mut c_void);

impl OpaquePointer {
    pub fn from_address(addr: Address) -> Self {
        OpaquePointer(addr.to_ptr_mut::<c_void>())
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0 as *mut c_void
    }
}