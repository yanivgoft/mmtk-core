use ::util::Address;
use ::util::heap::layout::vm_layout_constants::*;
use ::util::constants::*;

pub fn is_address_aligned(addr: Address) -> bool {
    addr.is_aligned_to(BYTES_IN_ADDRESS)
}

pub fn page_align(address: Address) -> Address {
    address.align_down(BYTES_IN_PAGE)
}

pub fn is_page_aligned(address: Address) -> bool {
    address.is_aligned_to(BYTES_IN_PAGE)
}

// const function cannot have conditional expression
pub const fn chunk_align_up(addr: Address) -> Address {
    addr.align_up(BYTES_IN_CHUNK)
}

// const function cannot have conditional expression
pub const fn chunk_align_down(addr: Address) -> Address {
    addr.align_down(BYTES_IN_CHUNK)
}

pub fn chunk_align(addr: Address, down: bool) -> Address {
    if down {
        chunk_align_down(addr)
    } else {
        chunk_align_up(addr)
    }
}

pub fn raw_chunk_align(immut_addr: usize, down: bool) -> usize {
    let addr = if !down { immut_addr + BYTES_IN_CHUNK - 1 } else { immut_addr };
    (addr >> LOG_BYTES_IN_CHUNK) << LOG_BYTES_IN_CHUNK
}

pub const fn raw_align_up(val: usize, align: usize) -> usize {
    (val + align - 1) & !(align - 1)
}

pub const fn raw_align_down(val: usize, align: usize) -> usize {
    val & !(align - 1)
}

pub fn pages_to_bytes(pages: usize) -> usize {
    pages << LOG_BYTES_IN_PAGE
}

pub fn bytes_to_pages_up(bytes: usize) -> usize {
    (bytes + BYTES_IN_PAGE - 1) >> LOG_BYTES_IN_PAGE
}

pub fn bytes_to_pages(bytes: usize) -> usize {
    let pages = bytes_to_pages_up(bytes);

    if cfg!(debug = "true") {
        let computed_extent = pages << LOG_BYTES_IN_PAGE;
        let bytes_match_pages = computed_extent == bytes;
        assert!(bytes_match_pages, "ERROR: number of bytes computed from pages must match original byte amount!\
                                           bytes = {}\
                                           pages = {}\
                                           bytes computed from pages = {}", bytes, pages, computed_extent);
    }

    pages
}

#[cfg(test)]
mod tests {
    use util::Address;
    use util::conversions::*;

    #[test]
    fn test_page_align() {
        let addr = unsafe { Address::from_usize(0x123456789) };
        assert_eq!(page_align(addr), unsafe { Address::from_usize(0x123456000) });
        assert!(!is_page_aligned(addr));
        assert!(is_page_aligned(page_align(addr)));
    }

    #[test]
    fn test_chunk_align() {
        let addr = unsafe { Address::from_usize(0x123456789) };
        assert_eq!(chunk_align(addr, true),  unsafe { Address::from_usize(0x123400000) });
        assert_eq!(chunk_align(addr, false), unsafe { Address::from_usize(0x123800000) });
    }
}