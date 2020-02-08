use std::collections::{LinkedList, HashMap};



pub struct Freelist {
    free_buckets: [LinkedList<usize>; Self::MAX_SIZE_CLASS],
    free_sizes: HashMap<usize, usize>,
    sizes: HashMap<usize, usize>,
}


impl Freelist {
    const MAX_SIZE_CLASS: usize = 48;

    pub fn new() -> Self {
        let f = || LinkedList::new();
        Self {
            free_buckets: [
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
                f(), f(), f(), f(), f(), f(), f(), f(),
            ],
            free_sizes: HashMap::new(),
            sizes: HashMap::new(),
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
            Some(index) => Some(index),
            _ => {
                if let Some(index) = self.alloc_cell(size_class + 1) {
                    let cell0 = index;
                    let cell1 = index + (1 << size_class);
                    self.push_to_bucket(size_class, cell1);
                    Some(cell0)
                } else {
                    None
                }
            }
        }
    }

    fn push_to_bucket(&mut self, bucket: usize, value: usize) {
        assert!(value & ((1 << bucket) - 1) == 0);
        self.free_sizes.insert(value, 1 << bucket);
        self.free_buckets[bucket].push_front(value);
    }

    fn pop_from_bucket(&mut self, bucket: usize) -> Option<usize> {
        if let Some(cell) = self.free_buckets[bucket].pop_front() {
            self.free_sizes.remove(&cell);
            Some(cell)
        } else {
            None
        }
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

    pub fn alloc(&mut self, count: usize) -> Option<usize> {
        match Self::get_size_class(count) {
            Some(size_class) => {
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
            Some((size_class, cell, _index)) => {
                // Remove this cell
                {
                    self.remove_cell(size_class, cell).unwrap();
                    // let mut tail = self.free_buckets[size_class].split_off(index);
                    // tail.pop_front();
                    // self.free_buckets[size_class].append(&mut tail);
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
    
    pub fn dealloc(&mut self, unit: usize) -> usize {
        let count = self.sizes.remove(&unit).unwrap();
        let size_class = count.trailing_zeros() as usize;
        self.push_to_bucket(size_class, unit);
        // self.__coalesce(unit, size_class);
        count
    }

    fn __coalesce(&mut self, unit: usize, size_class: usize) {
        // assume: `unit` is not in `self.free_buckets[size_class]`
        if size_class >= Self::MAX_SIZE_CLASS - 1 {
            let sc = Self::MAX_SIZE_CLASS - 1;
            for u in (unit .. (unit + (1 << size_class))).step_by(1 << sc) {
                self.push_to_bucket(size_class, u);
            }
            return
        }
        let sibling_cell = unit ^ (1 << size_class);
        if self.free_sizes.get(&sibling_cell) == Some(&(1 << size_class)) {
            // `sibling_cell` is free, we can merge these two cells
            self.remove_cell(size_class, sibling_cell).unwrap();
            let merged_unit = unit & !(1 << size_class);
            self.__coalesce(merged_unit, size_class + 1);
        } else {
            self.push_to_bucket(size_class, unit);
        }
    }

    fn remove_cell(&mut self, bucket: usize, cell: usize) -> Result<usize, ()> {
        let old_length = self.free_buckets[bucket].len();
        self.free_buckets[bucket].drain_filter(|x| *x == cell);
        debug_assert!(self.free_buckets[bucket].len() + 1 >= old_length);
        if self.free_buckets[bucket].len() + 1 == old_length {
            self.free_sizes.remove(&cell).unwrap();
            Ok(cell)
        } else {
            Err(())
        }
    }

    pub fn remove(&mut self, unit: usize, count: usize) {
        let size_class = count.trailing_zeros() as usize;
        self.remove_cell(size_class, unit).unwrap();
    }

    pub fn get_coalescable_size(&self, unit: usize) -> usize {
        let size = self.sizes[&unit];
        let size_class = size.trailing_zeros() as usize;
        self.__get_coalescable_size(unit, size_class)
    }

    fn __get_coalescable_size(&self, unit: usize, size_class: usize) -> usize {
        if size_class >= Self::MAX_SIZE_CLASS {
            return 0;
        }
        let sibling_cell = unit ^ (1 << size_class);
        if self.free_sizes.get(&sibling_cell) == Some(&(1 << size_class)) {
            // sibling_cell is free, we can merge these two cells
            let merged_unit = unit & !(1 << size_class);
            (1 << (size_class + 1)) + self.__get_coalescable_size(merged_unit, size_class + 1)
        } else {
            1 << size_class
        }
    }

    pub fn get_size(&mut self, index: usize) -> usize {
        self.sizes[&index]
    }

    pub fn reset(&mut self) {
        for x in self.free_buckets.iter_mut() {
            x.clear();
        }
        self.free_sizes.clear();
        self.sizes.clear();
    }
}


impl ::std::fmt::Debug for Freelist {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Freelist {{...}}")
    }
}
