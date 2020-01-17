use util::Address;

pub fn zero(start: Address, len: usize) {
    unsafe {
        libc::memset(start.to_ptr_mut() as *mut libc::c_void, 0, len);
    }
}
