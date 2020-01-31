use std::sync::atomic::{AtomicUsize, Ordering};
use super::constants::*;


#[derive(Debug)]
pub struct BitMap {
    table: Vec<AtomicUsize>,
}

impl BitMap {
    pub fn new(length: usize) -> Self {
        let length = (length + (BITS_IN_WORD - 1)) >> LOG_BITS_IN_WORD;
        println!("New bitmap {:?}", length);
        let mut table = vec![];
        table.resize_with(length, Default::default);
        let map = Self { table };
        map
    }

    pub fn get(&self, index: usize) -> bool {
        let word_index = index >> LOG_BITS_IN_WORD;
        let bit_index = index & (BITS_IN_WORD - 1);
        let v = self.table[word_index].load(Ordering::Relaxed);
        v & (1 << bit_index) != 0
    }

    pub fn atomic_set(&self, index: usize, value: bool) -> bool {
        let word_index = index >> LOG_BITS_IN_WORD;
        let bit_index = index & (BITS_IN_WORD - 1);
        if value {
            let v = self.table[word_index].fetch_or(1 << bit_index, Ordering::Relaxed);
            v & (1 << bit_index) == 0
        } else {
            let v = self.table[word_index].fetch_and(!(1 << bit_index), Ordering::Relaxed);
            v & (1 << bit_index) != 0
        }
    }

    pub fn clear(&self) {
        for i in 0..self.table.len() {
            self.table[i].store(0, Ordering::Relaxed);
        }
    }
}
