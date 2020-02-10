use vm::Memory;
use libc::*;
use util::Address;
use util::constants::*;
use util::heap::layout::vm_layout_constants::*;

pub struct VMMemory;

impl Memory for VMMemory {
  fn reserve_heap() -> (Address, Address) {
    unsafe {
      let size = 0x100_0000_0000 + BYTES_IN_CHUNK;
      let ptr = mmap(0 as _, size,
          PROT_EXEC | PROT_READ | PROT_WRITE,
          MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE,
          -1, 0
      );
      assert!(ptr != MAP_FAILED);
      let start = Address::from_mut_ptr(ptr).align_up(BYTES_IN_CHUNK);
      let end = (Address::from_mut_ptr(ptr) + size).align_down(BYTES_IN_CHUNK);
      (start, end)
    }
  }

  fn dzmmap(start: Address, size: usize) -> i32 {
    let prot = libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC;
    let flags = libc::MAP_ANON | libc::MAP_PRIVATE | libc::MAP_FIXED;
    let result = unsafe { Address::from_usize(mmap(start.0 as _, size, prot, flags, -1, 0) as _) };
    if result == start {
      0
    } else {
      assert!(result.0 <= 127,
        "mmap with MAP_FIXED has unexpected behavior: demand zero mmap with MAP_FIXED on {:?} returned some other address {:?}",
        start, result
      );
      result.0 as _
    }
  }
}