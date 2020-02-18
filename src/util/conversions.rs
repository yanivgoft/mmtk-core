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

pub fn align_up(addr: Address, bits: usize) -> Address {
    addr.align_up(1 << bits)
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
    fn test_align_up() {
        let addr = unsafe { Address::from_usize(0x123456789) };
        assert_eq!(align_up(addr, 0),  unsafe { Address::from_usize(0x123456789) }); // no align
        assert_eq!(align_up(addr, 1),  unsafe { Address::from_usize(0x12345678a) }); // 2 bytes align
        assert_eq!(align_up(addr, 2),  unsafe { Address::from_usize(0x12345678c) }); // 4 bytes align
        assert_eq!(align_up(addr, 3),  unsafe { Address::from_usize(0x123456790) }); // 8
        assert_eq!(align_up(addr, 4),  unsafe { Address::from_usize(0x123456790) }); // 16
        assert_eq!(align_up(addr, 5),  unsafe { Address::from_usize(0x1234567a0) }); // 32
        assert_eq!(align_up(addr, 6),  unsafe { Address::from_usize(0x1234567c0) }); // 64
        assert_eq!(align_up(addr, 7),  unsafe { Address::from_usize(0x123456800) }); // 128
        assert_eq!(align_up(addr, 8),  unsafe { Address::from_usize(0x123456800) }); // 256
        assert_eq!(align_up(addr, 9),  unsafe { Address::from_usize(0x123456800) }); // 512
        assert_eq!(align_up(addr, 10), unsafe { Address::from_usize(0x123456800) }); // 1K
        assert_eq!(align_up(addr, 11), unsafe { Address::from_usize(0x123456800) }); // 2K
        assert_eq!(align_up(addr, 12), unsafe { Address::from_usize(0x123457000) }); // 4K
        assert_eq!(align_up(addr, 13), unsafe { Address::from_usize(0x123458000) }); // 8K
        assert_eq!(align_up(addr, 14), unsafe { Address::from_usize(0x123458000) }); // 16K
        assert_eq!(align_up(addr, 15), unsafe { Address::from_usize(0x123458000) }); // 32K
        assert_eq!(align_up(addr, 16), unsafe { Address::from_usize(0x123460000) }); // 64K
        assert_eq!(align_up(addr, 17), unsafe { Address::from_usize(0x123460000) }); // 128K
        assert_eq!(align_up(addr, 18), unsafe { Address::from_usize(0x123480000) }); // 256K
        assert_eq!(align_up(addr, 19), unsafe { Address::from_usize(0x123480000) }); // 512K
        assert_eq!(align_up(addr, 20), unsafe { Address::from_usize(0x123500000) }); // 1M
        assert_eq!(align_up(addr, 21), unsafe { Address::from_usize(0x123600000) }); // 2M
        assert_eq!(align_up(addr, 22), unsafe { Address::from_usize(0x123800000) }); // 4M

        for i in 0..=22 {
            assert!(align_up(addr, i).is_aligned_to(1 << i));
        }
    }

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