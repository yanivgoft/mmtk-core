use std::cell::UnsafeCell;

use ::plan::TransitiveClosure;
use ::policy::space::{CommonSpace, Space};
use ::util::{Address, ObjectReference};
use ::util::constants::*;
use ::util::heap::{FreeListPageResource, PageResource, VMRequest};
use ::vm::*;
use std::sync::Mutex;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use util::heap::layout::vm_layout_constants::*;

const MAX_HEAP_SIZE: usize = HEAP_END.as_usize() - HEAP_START.as_usize(); // 4G
const MAX_OBJECTS_IN_HEAP: usize = MAX_HEAP_SIZE / BYTES_IN_ADDRESS;
const MAX_REGIONS_IN_HEAP: usize = MAX_HEAP_SIZE / MarkRegionSpace::BYTES_IN_REGION;

#[derive(Debug)]
pub struct MarkRegionSpace {
    common: UnsafeCell<CommonSpace<FreeListPageResource<MarkRegionSpace>>>,
    committed_regions: Mutex<HashSet<Address>>,
    mark_state: usize,
    object_mark_table: BitMap,
    region_mark_table: BitMap,
}

impl Space for MarkRegionSpace {
    type PR = FreeListPageResource<Self>;

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
        self.object_is_marked(object)
    }

    fn is_movable(&self) -> bool {
        false
    }

    fn release_multiple_pages(&mut self, start: Address) {
        unreachable!();
    }
}

impl MarkRegionSpace {
    pub const LOG_BYTES_IN_REGION: usize = LOG_BYTES_IN_PAGE as usize + 3;
    // Derived
    pub const BYTES_IN_REGION: usize = 1 << Self::LOG_BYTES_IN_REGION;
    pub const REGION_MASK: usize = Self::BYTES_IN_REGION - 1;
    pub const LOG_PAGES_IN_REGION: usize = Self::LOG_BYTES_IN_REGION - LOG_BYTES_IN_PAGE as usize;
    pub const PAGES_IN_REGION: usize = 1 << Self::LOG_PAGES_IN_REGION;


    pub fn new(name: &'static str, zeroed: bool, vmrequest: VMRequest) -> Self {
        println!("New MRS");
        let mrs = Self {
            common: UnsafeCell::new(CommonSpace::new(name, false, false, true, vmrequest)),
            committed_regions: Mutex::default(),
            mark_state: 0,
            object_mark_table: BitMap::new(MAX_OBJECTS_IN_HEAP),
            region_mark_table: BitMap::new(MAX_REGIONS_IN_HEAP),
        };
        println!("New MRS END");
        mrs
    }

    pub fn get_new_region(&self, tls: *mut ::libc::c_void) -> Option<Address> {
        let a = self.acquire(tls, Self::PAGES_IN_REGION);
        if a.is_zero() {
            None
        } else {
            assert!(a.as_usize() & Self::REGION_MASK == 0);
            assert!(a < ::util::heap::layout::vm_layout_constants::HEAP_END);
            let mut regions = self.committed_regions.lock().unwrap();
            assert!(!regions.contains(&a));
            regions.insert(a);
            Some(a)
        }
    }

    pub fn prepare(&mut self) {
        self.mark_state += 1;
        while self.mark_state == 0 {
            self.mark_state += 1;
        }
        self.object_mark_table.clear();
        self.region_mark_table.clear();
    }

    pub fn release(&mut self) {
        let mut dead_regions = vec![];
        let mut committed_regions = self.committed_regions.lock().unwrap();
        for r in committed_regions.iter() {
            if !self.region_is_marked(*r) {
                dead_regions.push(*r);
            }
        }
        println!("Released {}/{} regions", dead_regions.len(), committed_regions.len());
        for r in &dead_regions {
            committed_regions.remove(r);
        }
        ::std::mem::drop(committed_regions);
        for r in dead_regions {
            self.common_mut().pr.as_mut().unwrap().release_pages(r);
            assert!(self.reserved_pages() < 1000000);
        }
    }

    pub fn contains(&self, o: ObjectReference) -> bool {
        let region = Self::get_region_containing_object(o);
        let regions = self.committed_regions.lock().unwrap();
        regions.contains(&region)
    }
    
    fn get_object_index(o: ObjectReference) -> usize {
        (VMObjectModel::object_start_ref(o) - HEAP_START) >> LOG_BYTES_IN_WORD
    }

    fn get_region_index(a: Address) -> usize {
        (a - HEAP_START) >> Self::LOG_BYTES_IN_REGION
    }

    pub fn object_is_marked(&self, o: ObjectReference) -> bool {
        self.object_mark_table.get(Self::get_object_index(o))
    }

    fn region_is_marked(&self, r: Address) -> bool {
        self.region_mark_table.get(Self::get_region_index(r))
    }

    fn get_region_containing_object(o: ObjectReference) -> Address {
        let a = VMObjectModel::object_start_ref(o);
        unsafe { Address::from_usize(a.as_usize() & !Self::REGION_MASK) }
    }

    fn test_and_mark(&self, o: ObjectReference) -> bool {
        if self.object_mark_table.atomic_set(Self::get_object_index(o), true) {
            let region = Self::get_region_containing_object(o);
            self.region_mark_table.atomic_set(Self::get_region_index(region), true);
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

#[derive(Debug)]
struct BitMap {
    table: Vec<AtomicUsize>,
}

impl BitMap {
    pub fn new(length: usize) -> Self {
        let length = (length + (BITS_IN_WORD - 1)) >> LOG_BITS_IN_WORD;
        println!("New bitmap {:?}", length);
        let mut table = vec![];
        table.resize_with(length, Default::default);
        let map = Self { table };
        println!("New bitmap {:?} ebd", length);
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
