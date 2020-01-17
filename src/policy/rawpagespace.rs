use std::cell::UnsafeCell;

use ::plan::TransitiveClosure;
use ::policy::space::{CommonSpace, Space};
use ::util::{Address, ObjectReference};
use ::util::constants::*;
use ::util::heap::{FreeListPageResource, PageResource, VMRequest};
use std::sync::Mutex;
use util::conversions;
use std::collections::HashSet;
use util::bitmap::BitMap;
use util::heap::layout::vm_layout_constants::*;


const MAX_HEAP_SIZE: usize = HEAP_END.as_usize() - HEAP_START.as_usize();
const MAX_OBJECTS_IN_HEAP: usize = MAX_HEAP_SIZE / BYTES_IN_PAGE;


#[derive(Debug)]
pub struct RawPageSpace {
    common: UnsafeCell<CommonSpace<FreeListPageResource<RawPageSpace>>>,
    mark_state: usize,
    cells: Mutex<HashSet<Address>>,
    marktable: BitMap,
}

impl Space for RawPageSpace {
    type PR = FreeListPageResource<RawPageSpace>;

    fn init(&mut self) {
        let me = unsafe { &*(self as *const Self) };
        let common_mut = self.common_mut();
        assert!(common_mut.vmrequest.is_discontiguous());
        common_mut.pr = Some(FreeListPageResource::new_discontiguous(0));
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
        self.common_mut().pr.as_mut().unwrap().release_pages(start);
    }
}

impl RawPageSpace {
    pub fn new(name: &'static str) -> Self {
        RawPageSpace {
            common: UnsafeCell::new(CommonSpace::new(name, false, false, true, VMRequest::discontiguous())),
            mark_state: 0,
            cells: Mutex::new(HashSet::new()),
            marktable: BitMap::new(MAX_OBJECTS_IN_HEAP),
        }
    }

    pub fn alloc(&self, tls: *mut ::libc::c_void, pages: usize) -> Option<Address> {
        let a = self.acquire(tls, pages);
        if a.is_zero() {
            None
        } else {
            let mut cells = self.cells.lock().unwrap();
            cells.insert(a);
            Some(a)
        }
    }

    pub fn prepare(&mut self) {
        self.mark_state += 1;
        while self.mark_state == 0 {
            self.mark_state += 1;
        }
        self.marktable.clear();
    }

    pub fn release(&mut self) {
        let mut dead_cells = vec![];
        let mut cells = self.cells.lock().unwrap();
        for c in cells.iter() {
            if !self.cell_is_live(*c) {
                dead_cells.push(*c);
            }
        }
        println!("[RawPageSpace] Released {}/{} cells", dead_cells.len(), cells.len());
        for c in &dead_cells {
            cells.remove(c);
        }
        ::std::mem::drop(cells);
        for r in dead_cells {
            self.release_multiple_pages(r);
        }
    }
    
    fn get_cell_index(cell: Address) -> usize {
        (cell - HEAP_START) >> LOG_BYTES_IN_PAGE
    }

    fn cell_is_live(&self, cell: Address) -> bool {
        self.marktable.get(Self::get_cell_index(cell))
    }

    fn get_cell(o: ObjectReference) -> Address {
        debug_assert!(!o.is_null());
        conversions::page_align(o.to_address())
    }

    fn is_marked(&self, o: ObjectReference) -> bool {
        let cell = Self::get_cell(o);
        let index = Self::get_cell_index(cell);
        self.marktable.get(index)
    }
    
    fn test_and_mark(&self, o: ObjectReference) -> bool {
        let cell = Self::get_cell(o);
        let index = Self::get_cell_index(cell);
        self.marktable.atomic_set(index, true)
    }

    pub fn trace_object<T: TransitiveClosure>(&self, trace: &mut T, object: ObjectReference) -> ObjectReference {
        if self.test_and_mark(object) {
            trace.process_node(object);
        }
        return object;
    }
}
