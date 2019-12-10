use std::cell::UnsafeCell;

use ::plan::TransitiveClosure;
use ::policy::space::{CommonSpace, Space};
use ::util::{Address, ObjectReference};
use ::util::constants::*;
use ::util::header_byte;
use ::util::heap::{FreeListPageResource, PageResource, VMRequest};
use ::util::treadmill::TreadMill;
use ::vm::*;
use std::sync::Mutex;
use util::conversions;
use std::collections::HashSet;

#[derive(Debug)]
pub struct RawPageSpace {
    common: UnsafeCell<CommonSpace<FreeListPageResource<RawPageSpace>>>,
    mark_state: usize,
    cells: Mutex<HashSet<Address>>,
}

impl Space for RawPageSpace {
    type PR = FreeListPageResource<RawPageSpace>;

    fn init(&mut self) {
        let me = unsafe { &*(self as *const Self) };

        let common_mut = self.common_mut();

        if common_mut.vmrequest.is_discontiguous() {
            common_mut.pr = Some(FreeListPageResource::new_discontiguous(0));
        } else {
            common_mut.pr = Some(FreeListPageResource::new_contiguous(me, common_mut.start, common_mut.extent, 0));
        }

        common_mut.pr.as_mut().unwrap().bind_space(me);
    }

    fn common(&self) -> &CommonSpace<Self::PR> {
        unsafe { &*self.common.get() }
    }

    unsafe fn unsafe_common_mut(&self) -> &mut CommonSpace<Self::PR> {
        &mut *self.common.get()
    }

    fn is_live(&self, object: ObjectReference) -> bool {
        self.is_marked(object)
    }

    fn is_movable(&self) -> bool {
        false
    }

    fn release_multiple_pages(&mut self, start: Address) {
        unreachable!();
        // self.common_mut().pr.as_mut().unwrap().release_pages(start);
    }
}

impl RawPageSpace {
    pub fn new(name: &'static str, zeroed: bool, vmrequest: VMRequest) -> Self {
        RawPageSpace {
            common: UnsafeCell::new(CommonSpace::new(name, false, false, false, vmrequest)),
            mark_state: 0,
            cells: Mutex::new(HashSet::new()),
        }
    }

    pub fn prepare(&mut self) {
        self.mark_state += 1;
        while self.mark_state == 0 {
            self.mark_state += 1;
        }
    }

    pub fn release(&mut self) {
        let mut dead_cells: Vec<Address> = vec![];
        {
            let mut cells = self.cells.lock().unwrap();
            for cell in cells.iter() {
                if unsafe { cell.load::<usize>() } != self.mark_state {
                    dead_cells.push(*cell);
                }
            }
            for cell in &dead_cells {
                cells.remove(cell);
            }
        }
        println!("Released {} objects", dead_cells.len());
        let mut freedpages = 0;
        for cell in dead_cells {
            // println!("Release {:?}", cell);
            let pages = unsafe { (cell + BYTES_IN_ADDRESS).load::<usize>() };
            debug_assert!(pages != 0);
            // VMMemory::zero(cell, pages << LOG_BYTES_IN_PAGE);
            // VMMemory::protect(cell, pages << LOG_BYTES_IN_PAGE);
            freedpages += self.common_mut().pr.as_mut().unwrap().release_pages(cell);
        }
        println!("Freed {} pages", freedpages);
    }
    
    fn get_cell(o: ObjectReference) -> Address {
        debug_assert!(!o.is_null());
        conversions::page_align(o.to_address())
    }

    pub fn cell_is_allocated(&self, o: ObjectReference) -> bool {
        let cell = Self::get_cell(o);
        let cells = self.cells.lock().unwrap();
        cells.contains(&cell)
    }

    fn is_marked(&self, o: ObjectReference) -> bool {
        let mark = unsafe { Self::get_cell(o).load::<usize>() };
        mark == self.mark_state
    }
    
    fn test_and_mark(&self, o: ObjectReference) -> bool {
        let page = Self::get_cell(o);
        if unsafe { page.load::<usize>() } != self.mark_state {
            unsafe { page.store(self.mark_state) }
            true
        } else {
            false
        }
    }

    pub fn trace_object<T: TransitiveClosure>(&self, trace: &mut T, object: ObjectReference) -> ObjectReference {
        if self.test_and_mark(object) {
            trace.process_node(object);
        }
        return object;
    }

    pub fn initialize_header(&self, object: ObjectReference) {
        let mut cells = self.cells.lock().unwrap();
        cells.insert(Self::get_cell(object));
    }
}
