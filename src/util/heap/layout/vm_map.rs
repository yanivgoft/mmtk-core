use std::sync::{Mutex, MutexGuard, RwLock};
use super::freelist::*;
use util::Address;
use util::constants::*;
use super::vm_layout_constants::{LOG_BYTES_IN_CHUNK, BYTES_IN_CHUNK};
use std::sync::atomic::{AtomicUsize, Ordering};

pub const LOG_MAX_CHUNKS: usize = 48 - LOG_BYTES_IN_CHUNK;
pub const MAX_CHUNKS: usize = 1 << LOG_MAX_CHUNKS;

pub struct Region;

impl BlockDescriptor for Region {
    const LOG_SIZE: usize = 22;
}

pub struct VMMap {
    // shared_fl_map: Vec<Option<&'static CommonFreeListPageResource>>,
    // total_available_discontiguous_chunks: usize,
    descriptor_map: Vec<AtomicUsize>,
    heap_range: Mutex<(Address, Address)>,
    freelist: Mutex<Freelist>,
}


impl VMMap {
    pub fn new() -> Self {
        let mut map = vec![];
        println!("Resize start {}mb", MAX_CHUNKS * 8 / 1024 / 1024);
        map.resize_with(MAX_CHUNKS, Default::default);
        println!("Resize end");
        Self {
            heap_range: Mutex::new(unsafe { (Address::zero(), Address::zero()) }),
            freelist: Mutex::new(Freelist::new()),
            descriptor_map: map,//(0..MAX_CHUNKS).map(|_| AtomicUsize::new(0)).collect(),
        }
    }

    pub fn allocate_contiguous_chunks(&self, chunks: usize, space_desc: usize) -> Option<Address> {
        let mut freelist = self.freelist.lock().unwrap();
        match freelist.alloc(chunks) {
            Some(chunk_index) => {
                let chunk = unsafe { Address::from_usize(chunk_index << LOG_BYTES_IN_CHUNK) };
                for i in 0..chunks {
                    self.map_chunk(chunk + (i << LOG_BYTES_IN_CHUNK), space_desc);
                }
                Some(chunk)
            },
            _ => {
                if self.grow_heap(chunks, freelist) {
                    self.allocate_contiguous_chunks(chunks, space_desc)
                } else {
                    None
                }
            }
        }
    }

    pub fn release_contiguous_chunks(&self, start: Address) {
        let mut freelist = self.freelist.lock().unwrap();
        let index = start.as_usize() >> LOG_BYTES_IN_CHUNK;
        let count = freelist.get_size(index);
        for i in 0..count {
            self.unmap_chunk(start + (i << LOG_BYTES_IN_CHUNK));
        }
        freelist.dealloc(index);
    }

    fn grow_heap(&self, chunks: usize, mut freelist: MutexGuard<Freelist>) -> bool {
        // Grow virtual heap by 1G
        unsafe {
            use ::libc::*;

            let a = 0x7000_0000_0000usize;
            let size = BYTES_IN_MBYTE * 1024 * 2;
            let ptr = mmap(a as _, size,
                PROT_EXEC | PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE | MAP_FIXED,
                -1, 0
            );
            println!("VMMAP::grow_heap mmap -> {:?}", ptr);
            assert!(ptr != MAP_FAILED);
            if ptr == MAP_FAILED {
                return false;
            }
            let raw_start = Address::from_mut_ptr(ptr);
            let raw_limit = raw_start + size;
            let start = raw_start.align_up(BYTES_IN_CHUNK);
            let limit = raw_limit.align_down(BYTES_IN_CHUNK);
            let index = start.as_usize() >> LOG_BYTES_IN_CHUNK;
            let chunks = (limit - start) >> LOG_BYTES_IN_CHUNK;
            freelist.insert_free(index, chunks);
            true
        }
    }

    pub fn get_descriptor_for_address(&self, address: Address) -> usize {
        let index = address.as_usize() >> LOG_BYTES_IN_CHUNK;
        // println!("{:?} {} -> {}", address, index, self.descriptor_map[index].load(Ordering::Relaxed));
        self.descriptor_map[index].load(Ordering::Relaxed)
    }

    fn map_chunk(&self, chunk: Address, space: usize) {
        let index = chunk.as_usize() >> LOG_BYTES_IN_CHUNK;
        debug_assert!(index < MAX_CHUNKS, "{:?} {} {}", chunk, index, MAX_CHUNKS);
        // println!("{:?} {} = {}", chunk, index, space);
        self.descriptor_map[index].store(space, Ordering::Relaxed);
    }

    fn unmap_chunk(&self, chunk: Address) {
        let index = chunk.as_usize() >> LOG_BYTES_IN_CHUNK;
        self.descriptor_map[index].store(0, Ordering::Relaxed);
    }
}
