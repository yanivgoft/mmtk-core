use std::cell::UnsafeCell;

use ::plan::TransitiveClosure;
use ::policy::space::{CommonSpace, Space};
use ::util::{Address, ObjectReference};
use ::util::constants::*;
use ::util::heap::{PageResource, FreeListPageResource, VMRequest, SpaceMemoryMeta};
use std::sync::Mutex;
use util::conversions;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use util::heap::layout::vm_layout_constants::*;
use vm::*;


// const MAX_HEAP_SIZE: usize = HEAP_END.as_usize() - HEAP_START.as_usize();
// const MAX_OBJECTS_IN_HEAP: usize = MAX_HEAP_SIZE / BYTES_IN_PAGE;


#[derive(Debug)]
pub struct RawPageSpace {
    common: UnsafeCell<CommonSpace<FreeListPageResource>>,
    mark_state: usize,
    cells: Mutex<HashSet<Address>>,
}

impl Space for RawPageSpace {
    type PR = FreeListPageResource;

    fn init(&mut self) {
        // let me = unsafe { &*(self as *const Self) };
        // let common_mut = self.common_mut();
        // assert!(common_mut.vmrequest.is_discontiguous());
        // common_mut.pr = Some(FreeListPageResource::new_discontiguous(Self::MEDATADA_PAGES, me.common().descriptor));
        // common_mut.pr.as_mut().unwrap().bind_space(me);
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
        self.common_mut().pr.release_pages(start);
    }
}

impl RawPageSpace {
    const MAX_PAGES_IN_CHUNK: usize = BYTES_IN_CHUNK / BYTES_IN_PAGE;
    const OBJECT_MARKTABLE_OFFSET: usize = 0;
    const METADATA_BYTES: usize = Self::OBJECT_MARKTABLE_OFFSET + (Self::MAX_PAGES_IN_CHUNK >> LOG_BITS_IN_BYTE);
    const METATADA_PAGES: usize = (Self::METADATA_BYTES + BYTES_IN_PAGE - 1) >> LOG_BYTES_IN_PAGE;

    pub fn new(name: &'static str) -> Self {
        RawPageSpace {
            common: UnsafeCell::new(CommonSpace::new(name, false, false, true, Self::METATADA_PAGES, VMRequest::discontiguous())),
            mark_state: 0,
            cells: Mutex::new(HashSet::new()),
        }
    }

    pub fn alloc(&self, tls: *mut ::libc::c_void, pages: usize) -> Option<Address> {
        // println!("RawPageSpace alloc {}", pages);
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
        // Clear metadata
        match self.common().pr.common().memory {
            SpaceMemoryMeta::Discontiguous { ref head } => {
                let mut chunk_start = *head.read().unwrap();
                while !chunk_start.is_zero() {
                    VMMemory::zero(chunk_start, self.common().pr.common().metadata_pages_per_region << LOG_BYTES_IN_PAGE);
                    chunk_start = ::util::heap::layout::heap_layout::VM_MAP.get_next_contiguous_region(chunk_start).unwrap_or(unsafe { Address::zero() });
                }
            }
            _ => unreachable!()
        }
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

    fn get_cell_marktable_entry(cell: Address) -> (&'static AtomicUsize, usize) {
        let chunk = ::util::conversions::chunk_align(cell, true);
        let marktable = chunk + Self::OBJECT_MARKTABLE_OFFSET;
        let page_index = (cell - chunk) >> LOG_BYTES_IN_PAGE;
        let word_index = page_index >> LOG_BITS_IN_WORD;
        (unsafe { &*(marktable + (word_index << LOG_BYTES_IN_WORD)).to_ptr() }, page_index & (BITS_IN_WORD - 1))
    }

    fn cell_is_live(&self, cell: Address) -> bool {
        let (entry, bit) = Self::get_cell_marktable_entry(cell);
        entry.load(Ordering::Relaxed) & (1 << bit) != 0
    }

    fn get_cell(o: ObjectReference) -> Address {
        debug_assert!(!o.is_null());
        conversions::page_align(o.to_address())
    }

    fn is_marked(&self, o: ObjectReference) -> bool {
        let cell = Self::get_cell(o);
        self.cell_is_live(cell)
    }
    
    fn test_and_mark(&self, o: ObjectReference) -> bool {
        let cell = Self::get_cell(o);
        let (entry, bit) = Self::get_cell_marktable_entry(cell);
        entry.fetch_or(1 << bit, Ordering::Relaxed) & (1 << bit) == 0
    }

    pub fn trace_object<T: TransitiveClosure>(&self, trace: &mut T, object: ObjectReference) -> ObjectReference {
        if self.test_and_mark(object) {
            trace.process_node(object);
        }
        return object;
    }
}
