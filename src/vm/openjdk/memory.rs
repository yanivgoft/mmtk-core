use ::vm::Memory;
use libc;
use ::util::Address;
use util::constants::*;

pub struct VMMemory;

impl Memory for VMMemory {
  fn reserve_heap() -> (Address, Address) {
    unsafe {
      use ::libc::*;

      let start = Address::from_usize(0x7000_0000_0000usize);
      let end = Address::from_usize(0x7100_0000_0000usize);;
      let size = end - start;
      let ptr = mmap(start.to_ptr_mut(), size,
          PROT_EXEC | PROT_READ | PROT_WRITE,
          MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE | MAP_FIXED,
          -1, 0
      );
      println!("VMMAP::grow_heap mmap -> {:?}", ptr);
      assert!(ptr != MAP_FAILED);
      (start, end)
    }
  }

  fn dzmmap(start: Address, size: usize) -> i32 {
    let prot = libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC;
    let flags = libc::MAP_ANON | libc::MAP_PRIVATE | libc::MAP_FIXED;
    let result = unsafe { Address::from_usize(libc::mmap(start.0 as _, size, prot, flags, -1, 0) as _) };
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