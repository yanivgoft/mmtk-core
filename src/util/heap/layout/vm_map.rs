use std::sync::{Mutex, MutexGuard};
use util::freelist::*;
use util::Address;
use super::vm_layout_constants::{LOG_BYTES_IN_CHUNK, BYTES_IN_CHUNK};
use std::sync::atomic::{AtomicUsize, Ordering};
use vm::*;



pub struct VMMap {
    prev_link: Vec<AtomicUsize>,
    next_link: Vec<AtomicUsize>,
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
        map.resize_with(chunks, Default::default);
        let mut freelist = Freelist::new(None);
        {
            let start = heap_range.0.align_up(BYTES_IN_CHUNK);
            let limit = heap_range.1.align_down(BYTES_IN_CHUNK);
            let chunks = (limit - start) >> LOG_BYTES_IN_CHUNK;
            freelist.insert_free(0, chunks);
        }
        Self {
            prev_link: (0..chunks).map(|_| AtomicUsize::new(0)).collect(),
            next_link: (0..chunks).map(|_| AtomicUsize::new(0)).collect(),
            heap_range: heap_range,
            freelist: Mutex::new(freelist),
            descriptor_map: map,
        }
    }

    pub fn insert(&self, start: Address, extent: usize, descriptor: usize) {
        debug_assert!(::util::conversions::chunk_align(start, true) == start);
        let mut freelist = self.freelist.lock().unwrap();
        let chunk_index = self.get_chunk_index(start);
        let count = extent >> LOG_BYTES_IN_CHUNK;
        freelist.alloc_from(chunk_index, count).unwrap();
        for i in 0..count {
            self.map_chunk(start + (i << LOG_BYTES_IN_CHUNK), descriptor);
        }
    }

    pub fn allocate_contiguous_chunks(&self, chunks: usize, space_desc: usize, head: Address) -> Option<Address> {
        // println!("vm allocate_contiguous_chunks {:?}", chunks);
        let mut freelist = self.freelist.lock().unwrap();
        match freelist.alloc(chunks) {
            Some(chunk_index) => {
                // Add chunk to list
                if head.is_zero() {
                    self.next_link[chunk_index].store(0, Ordering::Relaxed);
                    self.prev_link[chunk_index].store(0, Ordering::Relaxed);
                } else {
                    let head_index = self.get_chunk_index(head);
                    self.next_link[chunk_index].store(head_index, Ordering::Relaxed);
                    self.prev_link[chunk_index].store(0, Ordering::Relaxed);
                    self.prev_link[head_index].store(chunk_index, Ordering::Relaxed);
                }
                // Map chunks
                let chunk = self.get_chunk_address(chunk_index);
                for i in 0..chunks {
                    self.map_chunk(chunk + (i << LOG_BYTES_IN_CHUNK), space_desc);
                }
                // println!("vm allocate_contiguous_chunks -> {:?}", chunk);
                Some(chunk)
            },
            _ => {
                // println!("vm allocate_contiguous_chunks -> None");
                None

            },
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
        // Remove chunk from list
        let next = self.next_link[index].load(Ordering::Relaxed);
        let prev = self.prev_link[index].load(Ordering::Relaxed);
        if next != 0 { self.prev_link[next].store(prev, Ordering::Relaxed) };
        if prev != 0 { self.next_link[prev].store(next, Ordering::Relaxed) };
        self.prev_link[index].store(0, Ordering::Relaxed);
        self.next_link[index].store(0, Ordering::Relaxed);
    }

    pub fn get_next_contiguous_region(&self, start: Address) -> Option<Address> {
        debug_assert!(start == ::util::conversions::chunk_align(start, true));
        let chunk = self.get_chunk_index(start);
        let next = self.next_link[chunk].load(Ordering::Relaxed);
        if next == 0 {
            None
        } else {
            Some(self.get_chunk_address(next))
        }
    }

    pub fn free_all_chunks(&self, any_chunk: Address) {
        if any_chunk.is_zero() { return }
        let mut freelist = self.freelist.lock().unwrap();
        let chunk = self.get_chunk_index(any_chunk);
        while self.next_link[chunk].load(Ordering::Relaxed) != 0 {
            let x = self.next_link[chunk].load(Ordering::Relaxed);
            self.free_contiguous_chunks_no_lock(x, &mut freelist);
        }
        while self.prev_link[chunk].load(Ordering::Relaxed) != 0 {
            let x = self.prev_link[chunk].load(Ordering::Relaxed);
            self.free_contiguous_chunks_no_lock(x, &mut freelist);
        }
        self.free_contiguous_chunks_no_lock(chunk as _, &mut freelist);
    }

    fn free_contiguous_chunks_no_lock(&self, chunk: usize, freelist: &mut MutexGuard<Freelist>) {
        let count = freelist.get_size(chunk);
        for i in 0..count {
            self.unmap_chunk(self.get_chunk_address(chunk) + (i << LOG_BYTES_IN_CHUNK));
        }
        freelist.dealloc(chunk);
        // Remove chunk from list
        let next = self.next_link[chunk].load(Ordering::Relaxed);
        let prev = self.prev_link[chunk].load(Ordering::Relaxed);
        if next != 0 { self.prev_link[next].store(prev, Ordering::Relaxed) };
        if prev != 0 { self.next_link[prev].store(next, Ordering::Relaxed) };
        self.prev_link[chunk].store(0, Ordering::Relaxed);
        self.next_link[chunk].store(0, Ordering::Relaxed);
    }

    pub fn get_contiguous_region_chunks(&self, start: Address) -> usize {
        debug_assert!(start == ::util::conversions::chunk_align(start, true));
        let chunk = self.get_chunk_index(start);
        self.freelist.lock().unwrap().get_size(chunk)
    }

    pub fn get_descriptor_for_address(&self, address: Address) -> usize {
        let index = self.get_chunk_index(address);
        self.descriptor_map[index].load(Ordering::Relaxed)
    }

    fn map_chunk(&self, chunk: Address, space: usize) {
        let index = self.get_chunk_index(chunk);
        self.descriptor_map[index].store(space, Ordering::Relaxed);
    }

    fn unmap_chunk(&self, chunk: Address) {
        let index = self.get_chunk_index(chunk);
        self.descriptor_map[index].store(0, Ordering::Relaxed);
    }
}
