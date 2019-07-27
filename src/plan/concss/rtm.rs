pub const _XBEGIN_STARTED: u32 = 0xFFFFFFFF;
pub const _XABORT_EXPLICIT: u32 = 1 << 0;
pub const _XABORT_RETRY: u32 = 1 << 1;
pub const _XABORT_CONFLICT: u32 = 1 << 2;
pub const _XABORT_CAPACITY: u32 = 1 << 3;
pub const _XABORT_DEBUG: u32 = 1 << 4;
pub const _XABORT_NESTED: u32 = 1 << 5;

#[cfg(any(target_arch="x86_64", target_arch="x86"))]
#[inline(always)]
pub unsafe fn _xbegin() -> u32 {
	let mut result: u32;
	asm! {
		"
			mov eax, 0xFFFFFFFF
    		xbegin 1f
    		1:
			nop
		"
		: "={eax}"(result)
		::: "intel"
	}
	result
}

#[cfg(not(any(target_arch="x86_64", target_arch="x86")))]
#[inline(always)]
pub unsafe fn _xbegin() -> u32 {
	unimplemented!()
}

#[cfg(any(target_arch="x86_64", target_arch="x86"))]
#[inline(always)]
pub unsafe fn _xend() {
	asm! { "xend"::::"intel" }
}

#[cfg(not(any(target_arch="x86_64", target_arch="x86")))]
#[inline(always)]
pub unsafe fn _xend() -> u32 {
	unimplemented!()
}

#[cfg(any(target_arch="x86_64", target_arch="x86"))]
#[inline(always)]
pub unsafe fn _xabort() -> ! {
	asm! { "xabort 0"::::"intel" };
	unreachable!();
}

#[cfg(not(any(target_arch="x86_64", target_arch="x86")))]
#[inline(always)]
pub unsafe fn _xabort() -> u32 {
	unimplemented!()
}

#[cfg(any(target_arch="x86_64", target_arch="x86"))]
#[inline(always)]
pub unsafe fn _xtest() -> bool {
	let result: u8;
	asm! {
		"
			xtest
			setnz $0
		"
		: "=r" (result)
		::: "intel"
	};
	result == 1
}

#[cfg(not(any(target_arch="x86_64", target_arch="x86")))]
#[inline(always)]
pub unsafe fn _xtest() -> u32 {
	unimplemented!()
}

#[cfg(any(target_arch="x86_64", target_arch="x86"))]
#[inline(always)]
pub fn execute_transaction<R, T: FnOnce() -> R>(transaction: T) -> Result<R, u32> {
	let code = unsafe { _xbegin() };
    if code != _XBEGIN_STARTED {
        return Err(code);
    }
	let result = transaction();
    unsafe { _xend() };
	return Ok(result);
}

#[cfg(not(any(target_arch="x86_64", target_arch="x86")))]
#[inline(always)]
pub fn execute_transaction<R, T: FnOnce() -> R>(transaction: T) -> Result<R, u32> {
	unimplemented!()
}