use ::util::Address;
use libc;

pub trait Memory {
  fn dzmmap(start: Address, size: usize) -> i32;
  fn zero(start: Address, len: usize) {
    unsafe {
      libc::memset(start.to_ptr_mut() as *mut libc::c_void, 0, len);
    }
  }
  fn protect(page_start: Address, bytes: usize) {
    unsafe {
      libc::mprotect(page_start.to_ptr_mut() as *mut libc::c_void, bytes, libc::PROT_NONE);
    }
  }
  fn unprotect(page_start: Address, bytes: usize) {
    unsafe {
      libc::mprotect(page_start.to_ptr_mut() as *mut libc::c_void, bytes, libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC);
    }
  }
}
