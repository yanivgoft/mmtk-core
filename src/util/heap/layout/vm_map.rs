use std::sync::{Mutex, MutexGuard, RwLock};
use super::freelist::*;
use util::Address;
use util::constants::*;
use super::vm_layout_constants::{LOG_BYTES_IN_CHUNK, BYTES_IN_CHUNK};
use std::sync::atomic::{AtomicUsize, Ordering};
use vm::*;


// pub const LOG_MAX_CHUNKS: usize = 48 - LOG_BYTES_IN_CHUNK;
// pub const MAX_CHUNKS: usize = 1 << LOG_MAX_CHUNKS;

pub struct Region;

impl BlockDescriptor for Region {
    const LOG_SIZE: usize = 22;
}

pub struct VMMap {
    // shared_fl_map: Vec<Option<&'static CommonFreeListPageResource>>,
    // total_available_discontiguous_chunks: usize,
    descriptor_map: Vec<AtomicUsize>,
    pub heap_range: (Address, Address),
    freelist: Mutex<Freelist>,
}


impl VMMap {
    fn get_chunk_index(&self, chunk: Address) -> usize {
        (chunk - self.heap_range.0) >> LOG_BYTES_IN_CHUNK
    }
    fn get_chunk_address(&self, index: usize) -> Address {
        self.heap_range.0 + (index << LOG_BYTES_IN_CHUNK)
    }

    pub fn new() -> Self {
        let heap_range = VMMemory::reserve_heap();
        let chunks = (heap_range.1 - heap_range.0) >> LOG_BYTES_IN_CHUNK;
        let mut map = vec![];
        println!("Resize start {}mb", chunks * 8 / 1024 / 1024);
        map.resize_with(chunks, Default::default);
        println!("Resize end");
        let mut freelist = Freelist::new();
        {
            let start = heap_range.0.align_up(BYTES_IN_CHUNK);
            let limit = heap_range.1.align_down(BYTES_IN_CHUNK);
            let chunks = (limit - start) >> LOG_BYTES_IN_CHUNK;
            freelist.insert_free(0, chunks);
        }
        Self {
            heap_range: heap_range,
            freelist: Mutex::new(freelist),
            descriptor_map: map,//(0..MAX_CHUNKS).map(|_| AtomicUsize::new(0)).collect(),
        }
    }

    pub fn allocate_contiguous_chunks(&self, chunks: usize, space_desc: usize) -> Option<Address> {
        let mut freelist = self.freelist.lock().unwrap();
        match freelist.alloc(chunks) {
            Some(chunk_index) => {
                let chunk = self.get_chunk_address(chunk_index);
                for i in 0..chunks {
                    self.map_chunk(chunk + (i << LOG_BYTES_IN_CHUNK), space_desc);
                }
                Some(chunk)
            },
            _ => None,
        }
    }

    pub fn release_contiguous_chunks(&self, start: Address) {
        let mut freelist = self.freelist.lock().unwrap();
        let index = self.get_chunk_index(start);
        let count = freelist.get_size(index);
        for i in 0..count {
            self.unmap_chunk(start + (i << LOG_BYTES_IN_CHUNK));
        }
        freelist.dealloc(index);
    }

    pub fn get_descriptor_for_address(&self, address: Address) -> usize {
        let index = self.get_chunk_index(address);
        // println!("{:?} {} -> {}", address, index, self.descriptor_map[index].load(Ordering::Relaxed));
        self.descriptor_map[index].load(Ordering::Relaxed)
    }

    fn map_chunk(&self, chunk: Address, space: usize) {
        let index = self.get_chunk_index(chunk);
        // debug_assert!(index < MAX_CHUNKS, "{:?} {} {}", chunk, index, MAX_CHUNKS);
        // println!("{:?} {} = {}", chunk, index, space);
        self.descriptor_map[index].store(space, Ordering::Relaxed);
    }

    fn unmap_chunk(&self, chunk: Address) {
        let index = self.get_chunk_index(chunk);
        self.descriptor_map[index].store(0, Ordering::Relaxed);
    }
}
