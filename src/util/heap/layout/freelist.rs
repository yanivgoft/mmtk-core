
pub struct Freelist {
    table: Vec<u64>,
    head: Option<usize>,
    coalesce_boundary: Option<usize>,
}

const INVALID: usize = (1 << 30) - 1;
const NEXT_OFFSET: usize = 0;
const NEXT_MASK: u64 = ((1 << 30) - 1) << NEXT_OFFSET;
const PREV_OFFSET: usize = 30;
const PREV_MASK: u64 = ((1 << 30) - 1) << PREV_OFFSET;
const FREE_OFFSET: usize = 62;
const FREE_MASK: u64 = 1 << FREE_OFFSET;
const MULTI_OFFSET: usize = 63;
const MULTI_MASK: u64 = 1 << MULTI_OFFSET;

impl Freelist {
    pub fn new(coalesce_boundary: Option<usize>) -> Self {
        Self {
            table: vec![],
            head: None,
            coalesce_boundary,
        }
    }

    fn resize_table(&mut self, length: usize) {
        let length = length.checked_next_power_of_two().unwrap();
        let length = if length > 1024 { length } else { 1024 };
        self.table.resize(length, 0);
    }

    pub fn insert_free(&mut self, unit: usize, size: usize) {
        if unit + size > self.table.len() {
            self.resize_table(unit + size);
        }
        self.insert_free_node(unit, size);
        debug_assert!(self.get_size(unit) == size);
    }

    fn remove_free_node(&mut self, unit: usize) {
        debug_assert!(self.is_free(unit));
        self.set_free(unit, false);
        // Remove from list
        if self.head == Some(unit) {
            let next = self.get_next(unit);
            next.map(|n| self.set_prev(n, None));
            self.head = next;
        } else {
            let prev = self.get_prev(unit).unwrap();
            if let Some(next) = self.get_next(unit) {
                self.set_next(prev, Some(next));
                self.set_prev(next, Some(prev));
            } else {
                self.set_next(prev, None);
            }
        }
    }

    fn split_node_and_remove_first(&mut self, unit: usize, first_size: usize) -> usize {
        debug_assert!(first_size > 0);
        debug_assert!(self.is_free(unit));
        let size = self.get_size(unit);
        debug_assert!(size > first_size);
        self.remove_free_node(unit);
        self.set_size(unit, first_size);
        self.insert_free_node(unit + first_size, size - first_size);
        unit
    }

    pub fn alloc(&mut self, size: usize) -> Option<usize> {
        let mut unit_opt = self.head;
        while let Some(unit) = unit_opt  {
            if self.get_size(unit) >= size {
                return self.alloc_from(unit, size);
            }
            unit_opt = self.get_next(unit);
        }
        None
    }

    pub fn alloc_from(&mut self, unit: usize, size: usize) -> Option<usize> {
        let unit_size = self.get_size(unit);
        if unit_size > size {
            self.split_node_and_remove_first(unit, size);
        } else {
            self.remove_free_node(unit);
        }
        debug_assert!(self.get_size(unit) == size);
        debug_assert!(!self.is_free(unit));
        Some(unit)
    }

    fn coalesce_nodes(&mut self, left: usize, right: usize) {
        if let Some(log_boundary) = self.coalesce_boundary {
            if (left ^ right) >> log_boundary != 0 {
                return
            }
        }
        debug_assert!(left + self.get_size(left) == right);
        debug_assert!(self.is_free(left));
        debug_assert!(self.is_free(right));
        self.remove_free_node(left);
        self.remove_free_node(right);
        let size = self.get_size(left) + self.get_size(right);
        self.insert_free_node(left, size);
        debug_assert!(self.get_size(left) == size);
    }

    fn insert_free_node(&mut self, unit: usize, size: usize) {
        self.set_free(unit, true);
        self.set_size(unit, size);
        if let Some(head) = self.head {
            self.set_next(unit, Some(head));
            self.set_prev(unit, None);
            self.set_prev(head, Some(unit));
            self.head = Some(unit);
        } else {
            self.set_prev(unit, None);
            self.set_next(unit, None);
            self.head = Some(unit);
        }
    }
    
    pub fn dealloc(&mut self, unit: usize) -> (usize, usize) {
        let size = self.get_size(unit);
        // Add to freelist
        self.insert_free_node(unit, size);
        // Coalesce
        let mut coalesced_unit = unit;
        let coalescable_left = self.get_left(unit).and_then(|x| if self.is_free(x) { Some(x) } else { None });
        let coalescable_right = self.get_right(unit).and_then(|x| if self.is_free(x) { Some(x) } else { None });
        if let Some(left) = coalescable_left {
            self.coalesce_nodes(left, unit);
            coalesced_unit = left;
        }
        if let Some(right) = coalescable_right {
            self.coalesce_nodes(coalesced_unit, right);
        }
        let coalesced_size = self.get_size(coalesced_unit);
        (size, coalesced_size)
    }

    pub fn remove(&mut self, unit: usize, count: usize) {
        self.remove_free_node(unit);
    }

    fn get_left(&self, unit: usize) -> Option<usize> {
        if let Some(log_boundary) = self.coalesce_boundary {
            if unit & ((1 << log_boundary) - 1) == 0 {
                return None;
            }
        }
        if self.is_multi(unit - 1) {
            let size = self.table[unit - 1] & !MULTI_MASK;
            let left_unit = unit - size as usize;
            Some(left_unit)
        } else {
            Some(unit - 1)
        }
    }

    fn get_right(&self, unit: usize) -> Option<usize> {
        let right_unit = unit + self.get_size(unit);
        if let Some(log_boundary) = self.coalesce_boundary {
            if right_unit & ((1 << log_boundary) - 1) == 0 {
                return None;
            }
        }
        if right_unit >= self.table.len() {
            None
        } else {
            Some(right_unit)
        }
    }

    #[inline(always)]
    fn get_next(&self, unit: usize) -> Option<usize> {
        let n = ((self.table[unit] & NEXT_MASK) >> NEXT_OFFSET) as usize;
        if n == INVALID {
            None
        } else {
            Some(n)
        }
    }

    #[inline(always)]
    fn get_prev(&self, unit: usize) -> Option<usize> {
        let p = ((self.table[unit] & PREV_MASK) >> PREV_OFFSET) as usize;
        if p == INVALID {
            None
        } else {
            Some(p)
        }
    }

    fn set_next(&mut self, unit: usize, next: Option<usize>) {
        let next = next.unwrap_or(INVALID);
        self.table[unit] = (self.table[unit] & !NEXT_MASK) | ((next as u64) << NEXT_OFFSET);
    }

    fn set_prev(&mut self, unit: usize, prev: Option<usize>) {
        let prev = prev.unwrap_or(INVALID);
        self.table[unit] = (self.table[unit] & !PREV_MASK) | ((prev as u64) << PREV_OFFSET);
    }
    
    fn set_free(&mut self, unit: usize, free: bool) {
        if free {
            self.table[unit] |= 1 << FREE_OFFSET;
        } else {
            self.table[unit] &= !(1 << FREE_OFFSET);
        }
    }
    
    #[inline(always)]
    fn is_free(&self, unit: usize) -> bool {
        self.table[unit] & FREE_MASK != 0
    }

    #[inline(always)]
    fn is_multi(&self, unit: usize) -> bool {
        self.table[unit] & MULTI_MASK != 0
    }

    #[inline(always)]
    pub fn get_size(&self, unit: usize) -> usize {
        if self.is_multi(unit) {
            let x = self.table[unit + 1] & !MULTI_MASK;
            x as usize
        } else {
            1
        }
    }

    pub fn set_size(&mut self, unit: usize, size: usize) {
        if size == 1 {
            self.table[unit] &= !MULTI_MASK;
        } else {
            self.table[unit] |= MULTI_MASK;
            self.table[unit + 1] = MULTI_MASK | (size as u64);
            if size > 2 {
                self.table[unit + size - 1] = MULTI_MASK | (size as u64);
            }
        }
    }
}


impl ::std::fmt::Debug for Freelist {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Freelist {{...}}")
    }
}
