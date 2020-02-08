use std::cell::UnsafeCell;

use ::plan::TransitiveClosure;
use ::policy::space::{CommonSpace, Space};
use ::util::{Address, ObjectReference};
use ::util::constants::*;
use ::util::heap::{PageResource, FreeListPageResource, VMRequest, SpaceMemoryMeta};
use ::vm::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, AtomicU8, Ordering};
use std::collections::HashSet;
use util::heap::layout::vm_layout_constants::{MAX_CHUNKS, LOG_BYTES_IN_CHUNK, BYTES_IN_CHUNK};
use util::bitmap::BitMap;


// const MAX_HEAP_SIZE: usize = MAX_CHUNKS << LOG_BYTES_IN_CHUNK;
// const MAX_OBJECTS_IN_HEAP: usize = MAX_HEAP_SIZE / BYTES_IN_ADDRESS;
// const MAX_REGIONS_IN_HEAP: usize = MAX_HEAP_SIZE / MarkRegionSpace::BYTES_IN_REGION;

#[derive(Debug)]
pub struct MarkRegionSpace {
    common: UnsafeCell<CommonSpace<FreeListPageResource>>,
    committed_regions: Mutex<HashSet<Address>>,
    mark_state: usize,
}

impl Space for MarkRegionSpace {
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
        self.object_is_marked(object)
    }

    fn is_movable(&self) -> bool {
        false
    }

    fn release_multiple_pages(&mut self, start: Address) {
        self.common_mut().pr.release_pages(start);
    }

    fn grow_space(&self, start: Address, bytes: usize, new_chunk: bool) {
        // Clear metadata
        if new_chunk {

        }
    }
}



impl MarkRegionSpace {
    pub const LOG_BYTES_IN_REGION: usize = LOG_BYTES_IN_PAGE as usize + 3;
    // Derived
    pub const BYTES_IN_REGION: usize = 1 << Self::LOG_BYTES_IN_REGION;
    pub const REGION_MASK: usize = Self::BYTES_IN_REGION - 1;
    pub const LOG_PAGES_IN_REGION: usize = Self::LOG_BYTES_IN_REGION - LOG_BYTES_IN_PAGE as usize;
    pub const PAGES_IN_REGION: usize = 1 << Self::LOG_PAGES_IN_REGION;

    
    const MAX_OBJECTS_IN_CHUNK: usize = BYTES_IN_CHUNK / BYTES_IN_ADDRESS;
    const MAX_REGIONS_IN_CHUNK: usize = BYTES_IN_CHUNK / Self::BYTES_IN_REGION;
    const REGION_MARKTABLE_OFFSET: usize = 0;
    const OBJECT_MARKTABLE_OFFSET: usize = Self::REGION_MARKTABLE_OFFSET + Self::MAX_REGIONS_IN_CHUNK;
    const METADATA_BYTES: usize = Self::OBJECT_MARKTABLE_OFFSET + (Self::MAX_OBJECTS_IN_CHUNK >> LOG_BITS_IN_BYTE);
    const MEDATADA_PAGES: usize = (Self::METADATA_BYTES + BYTES_IN_PAGE - 1) >> LOG_BYTES_IN_PAGE;

    pub fn new(name: &'static str) -> Self {
        Self {
            common: UnsafeCell::new(CommonSpace::new(name, false, false, true, Self::MEDATADA_PAGES, VMRequest::discontiguous())),
            committed_regions: Mutex::default(),
            mark_state: 0,
        }
    }

    pub fn get_new_region(&self, tls: *mut ::libc::c_void) -> Option<Address> {
        // println!("Get new region start");
        let a = self.acquire(tls, Self::PAGES_IN_REGION);
        if a.is_zero() {
            None
        } else {
            assert!(a.as_usize() & Self::REGION_MASK == 0);
            // assert!(a < ::util::heap::layout::vm_layout_constants::HEAP_END);
            let mut regions = self.committed_regions.lock().unwrap();
            let chunk = ::util::conversions::chunk_align(a, true);
            assert!(a != chunk);
            assert!(!regions.contains(&a));
            regions.insert(a);
            // println!("Get new region {:?}", a);
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
        let mut dead_regions = vec![];
        let mut committed_regions = self.committed_regions.lock().unwrap();
        for r in committed_regions.iter() {
            if !self.region_is_marked(*r) {
                dead_regions.push(*r);
            }
        }
        println!("[MarkRegionSpace] Released {}/{} regions", dead_regions.len(), committed_regions.len());
        for r in &dead_regions {
            committed_regions.remove(r);
        }
        ::std::mem::drop(committed_regions);
        for r in dead_regions {
            self.release_multiple_pages(r);
        }
    }

    pub fn contains(&self, o: ObjectReference) -> bool {
        let region = Self::get_region_containing_object(o);
        let regions = self.committed_regions.lock().unwrap();
        regions.contains(&region)
    }
    
    fn get_object_index(o: ObjectReference) -> usize {
        VMObjectModel::object_start_ref(o).as_usize() >> LOG_BYTES_IN_WORD
    }

    fn get_region_index(a: Address) -> usize {
        a.as_usize() >> Self::LOG_BYTES_IN_REGION
    }

    fn get_object_marktable_entry(o: ObjectReference) -> (&'static AtomicUsize, usize) /*(word_slot, bit_index)*/ {
        let a = VMObjectModel::object_start_ref(o);
        let chunk = ::util::conversions::chunk_align(a, true);
        let marktable = chunk + Self::OBJECT_MARKTABLE_OFFSET;
        let object_index = (a - chunk) >> LOG_BYTES_IN_WORD; 
        let word_index = object_index >> LOG_BITS_IN_WORD;
        (unsafe { &*(marktable + (word_index << LOG_BYTES_IN_WORD)).to_ptr() }, object_index & (BITS_IN_WORD - 1))
    }

    fn get_region_marktable_entry(r: Address) -> &'static AtomicU8 {
        let chunk = ::util::conversions::chunk_align(r, true);
        let region_index = (r - chunk) >> Self::LOG_BYTES_IN_REGION;
        unsafe { &*(chunk + Self::REGION_MARKTABLE_OFFSET + region_index).to_ptr() }
    }

    pub fn object_is_marked(&self, o: ObjectReference) -> bool {
        let (entry, bit) = Self::get_object_marktable_entry(o);
        (entry.load(Ordering::Relaxed) & (1 << bit)) != 0
    }

    pub fn attempt_to_mark_object(&self, o: ObjectReference) -> bool {
        let (entry, bit) = Self::get_object_marktable_entry(o);
        let old = entry.fetch_or(1 << bit, Ordering::Relaxed);
        (old & (1 << bit)) == 0
    }

    fn region_is_marked(&self, r: Address) -> bool {
        let entry = Self::get_region_marktable_entry(r);
        entry.load(Ordering::Relaxed) != 0
    }

    fn mark_region(&self, r: Address) {
        let entry = Self::get_region_marktable_entry(r);
        entry.store(1, Ordering::Relaxed);
    }

    fn get_region_containing_object(o: ObjectReference) -> Address {
        let a = VMObjectModel::object_start_ref(o);
        unsafe { Address::from_usize(a.as_usize() & !Self::REGION_MASK) }
    }

    fn test_and_mark(&self, o: ObjectReference) -> bool {
        if self.attempt_to_mark_object(o) {
            self.mark_region(Self::get_region_containing_object(o));
            true
        } else {
            assert!(self.region_is_marked(Self::get_region_containing_object(o)));
            false
        }
    }

    pub fn trace_object<T: TransitiveClosure>(&self, trace: &mut T, object: ObjectReference) -> ObjectReference {
        if self.test_and_mark(object) {
            trace.process_node(object);
        }
        return object;
    }
}
