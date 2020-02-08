use std::marker::PhantomData;
use std::collections::{LinkedList, HashMap};


pub trait BlockDescriptor {
    const LOG_SIZE: usize;
    const SIZE: usize = 1 << Self::LOG_SIZE;
    const MASK: usize = Self::SIZE - 1;
}

pub struct Freelist {
    committed_buckets: [LinkedList<usize>; 48],
    free_buckets: [LinkedList<usize>; 48],
    sizes: HashMap<usize, usize>,
}


impl Freelist {
    pub fn new() -> Self {
        let f = || LinkedList::new();
        Self {
            committed_buckets: [
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
            ],
            free_buckets: [
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
            ],
            sizes: HashMap::new(),
            // phantom: PhantomData,
        }
    }

    fn get_size_class(count: usize) -> Option<usize> {
        count.checked_next_power_of_two().map(|v| v.trailing_zeros() as usize)
    }

    fn alloc_cell(&mut self, size_class: usize) -> Option<usize> {
        if size_class >= self.free_buckets.len() {
            return None;
        }
        match self.pop_from_bucket(size_class) {
            Some(index) => {
                // println!("Pop from size class {} index={}", size_class, index);
                Some(index)
            },
            _ => {
                if let Some(index) = self.alloc_cell(size_class + 1) {
                    let cell0 = index;
                    let cell1 = index + (1 << size_class);
                    self.push_to_bucket(size_class, cell1);
                    // self.committed_buckets[size_class].push_front(cell0);
                    Some(cell0)
                } else {
                    None
                }
            }
        }
    }

    fn push_to_bucket(&mut self, bucket: usize, value: usize) {
        assert!(value & ((1 << bucket) - 1) == 0);
        self.free_buckets[bucket].push_front(value);
    }

    fn pop_from_bucket(&mut self, bucket: usize) -> Option<usize> {
        self.free_buckets[bucket].pop_front()
    }

    pub fn insert_free(&mut self, index: usize, count: usize) {
        let index = index;
        let mut limit = index + count;
        for size_class in (0..48).rev() {
            let i = (index + (1 << size_class) - 1) >> size_class << size_class;
            let j = i + (1 << size_class);
            if j <= limit {
                self.push_to_bucket(size_class, i);
                limit = i;
            }
        }
    }

    pub fn insert_committed(&mut self, index: usize, count: usize) {
        unimplemented!()
    }

    pub fn alloc(&mut self, count: usize) -> Option<usize> {
        match Self::get_size_class(count) {
            Some(size_class) => {
                // println!("size class = {}", size_class);
                match self.alloc_cell(size_class) {
                    Some(index) => {
                        self.sizes.insert(index, count);
                        Some(index)
                    }
                    _ => None
                }
            },
            _ => None,
        }
    }

    fn get_cell_containing(&self, index: usize, count: usize) -> Option<(usize, usize, usize)> {
        let min_size_class = Self::get_size_class(count).unwrap();
        for size_class in (min_size_class..48).rev() {
            let size = 1 << size_class;
            let mut i = 0;
            for cell in &self.free_buckets[size_class] {
                if *cell <= index && (cell + size) >= (index + count) {
                    return Some((size_class, *cell, i));
                }
                i += 1;
            }
        }
        None
    }

    pub fn alloc_from(&mut self, start: usize, count: usize) -> Option<usize> {
        match self.get_cell_containing(start, count) {
            Some((size_class, cell, index)) => {
                // Remove this cell
                {
                    let mut tail = self.free_buckets[size_class].split_off(index);
                    tail.pop_front();
                    self.free_buckets[size_class].append(&mut tail);
                }
                let size = 1 << size_class;
                let pieces = [(cell, start - cell), (start, count), (start + count, cell + size - start - count)];
                if pieces[0].1 > 0 {
                    self.insert_free(pieces[0].0, pieces[0].1);
                }
                if pieces[2].1 > 0 {
                    self.insert_free(pieces[2].0, pieces[2].1);
                }
                self.sizes.insert(start, count);
                Some(start)
            },
            _ => None,
        }
    }
    
    pub fn dealloc(&mut self, index: usize) -> usize {
        let count = self.sizes.remove(&index).unwrap();
        self.insert_free(index, count);
        count
    }

    pub fn get_size(&mut self, index: usize) -> usize {
        self.sizes[&index]
    }

    pub fn reset(&mut self) {
        for x in self.committed_buckets.iter_mut() {
            x.clear();
        }
        for x in self.free_buckets.iter_mut() {
            x.clear();
        }
        self.sizes.clear();
    }
}


impl ::std::fmt::Debug for Freelist {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Freelist {{...}}")
    }
}
