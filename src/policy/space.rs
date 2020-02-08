use ::util::Address;
use ::util::ObjectReference;
use ::util::conversions::*;

use ::vm::{ActivePlan, VMActivePlan, Collection, VMCollection, ObjectModel, VMObjectModel};
use ::util::heap::{VMRequest, PageResource};
use ::util::heap::layout::vm_layout_constants::LOG_BYTES_IN_CHUNK;
use ::plan::Plan;
use ::plan::selected_plan::PLAN;
use ::plan::{Allocator, TransitiveClosure};
use std::sync::atomic::{AtomicUsize, Ordering};
use ::util::constants::{LOG_BYTES_IN_MBYTE, BYTES_IN_PAGE, BYTES_IN_MBYTE};
use ::util::conversions;
use ::util::heap::space_descriptor;
use ::util::heap::layout::heap_layout::VM_MAP;
use std::sync::Mutex;

use std::fmt::Debug;

use libc::c_void;

pub trait Space: Sized + Debug + 'static {
    type PR: PageResource;

    fn init(&mut self);

    fn acquire(&self, tls: *mut c_void, pages: usize) -> Address {
        trace!("Space.acquire, tls={:p}", tls);
        // debug_assert!(tls != 0);
        let allow_poll = unsafe { VMActivePlan::is_mutator(tls) }
            && PLAN.is_initialized();

        trace!("Reserving pages");
        let pr = &self.common().pr;
        let pages_reserved = pr.reserve_pages(pages);
        trace!("Pages reserved");

        // FIXME: Possibly unnecessary borrow-checker fighting
        let me = unsafe { &*(self as *const Self) };

        trace!("Polling ..");

        if allow_poll && VMActivePlan::global().poll(false, me) {
            trace!("Collection required");
            pr.clear_request(pages_reserved);
            VMCollection::block_for_gc(tls);
            unsafe { Address::zero() }
        } else {
            trace!("Collection not required");
            let rtn = pr.alloc_pages(pages_reserved, pages, self.common().zeroed, self, tls);
            if rtn.is_none() {
                if !allow_poll {
                    println!("VMActivePlan::is_mutator(tls) {:?}", unsafe { VMActivePlan::is_mutator(tls) });
                    println!("PLAN.is_initialized() {:?}", PLAN.is_initialized());
                    panic!("Physical allocation failed when polling not allowed!");
                }

                let gc_performed = VMActivePlan::global().poll(true, me);
                debug_assert!(gc_performed, "GC not performed when forced.");
                pr.clear_request(pages_reserved);
                VMCollection::block_for_gc(tls);
                unsafe { Address::zero() }
            } else {
                rtn.unwrap()
            }
        }
    }

    fn in_space(&self, object: ObjectReference) -> bool {
        let start = VMObjectModel::ref_to_address(object);
        if !space_descriptor::is_contiguous(self.common().descriptor) {
            VM_MAP.get_descriptor_for_address(start) == self.common().descriptor
        } else {
            unimplemented!()
            // start.as_usize() >= self.common().start.as_usize()
            //     && start.as_usize() < self.common().start.as_usize() + self.common().extent
        }
    }

    // UNSAFE: potential data race as this mutates 'common'
    // unsafe fn grow_discontiguous_space(&self, chunks: usize) -> Address {
    //     // FIXME
    //     let new_head: Address = VM_MAP.allocate_contiguous_chunks(self.common().descriptor, chunks, self.common().head_discontiguous_region);
    //     if new_head.is_zero() {
    //         return unsafe{Address::zero()};
    //     }

    //     self.unsafe_common_mut().head_discontiguous_region = new_head;
    //     new_head
    // }

    /**
     * This hook is called by page resources each time a space grows.  The space may
     * tap into the hook to monitor heap growth.  The call is made from within the
     * page resources' critical region, immediately before yielding the lock.
     *
     * @param start The start of the newly allocated space
     * @param bytes The size of the newly allocated space
     * @param new_chunk {@code true} if the new space encroached upon or started a new chunk or chunks.
     */
    fn grow_space(&self, start: Address, bytes: usize, new_chunk: bool) {}

    fn reserved_pages(&self) -> usize {
        self.common().pr.reserved_pages()
    }

    fn get_name(&self) -> &'static str {
        self.common().name
    }

    fn common(&self) -> &CommonSpace<Self::PR>;
    fn common_mut(&mut self) -> &mut CommonSpace<Self::PR> {
        // SAFE: Reference is exclusive
        unsafe {self.unsafe_common_mut()}
    }

    // UNSAFE: This get's a mutable reference from self
    // (i.e. make sure their are no concurrent accesses through self when calling this)_
    unsafe fn unsafe_common_mut(&self) -> &mut CommonSpace<Self::PR>;

    fn is_live(&self, object: ObjectReference) -> bool;
    fn is_movable(&self) -> bool;

    // fn release_discontiguous_chunks(&mut self, chunk: Address) {
    //     debug_assert!(chunk == conversions::chunk_align(chunk, true));
    //     if chunk == self.common().head_discontiguous_region {
    //         self.common_mut().head_discontiguous_region = VM_MAP.get_next_contiguous_region(chunk);
    //     }
    //     VM_MAP.free_contiguous_chunks(chunk);
    // }

    fn release_multiple_pages(&mut self, start: Address);

    // unsafe fn release_all_chunks(&self) {
    //     VM_MAP.free_all_chunks(self.common().head_discontiguous_region);
    //     self.unsafe_common_mut().head_discontiguous_region = Address::zero();
    // }

    // fn print_vm_map(&self) {
    //     let common = self.common();
    //     print!("{:4} {:5}MB ", common.name, self.reserved_pages() * BYTES_IN_PAGE / BYTES_IN_MBYTE);
    //     if common.immortal {
    //         print!("I");
    //     } else {
    //         print!(" ");
    //     }
    //     if common.movable {
    //         print!(" ");
    //     } else {
    //         print!("N");
    //     }
    //     print!(" ");
    //     if common.contiguous {
    //         print!("{}->{}", common.start, common.start+common.extent-1);
    //         match common.vmrequest {
    //             VMRequest::RequestExtent { extent, top } => {
    //                 print!(" E {}", extent);
    //             },
    //             VMRequest::RequestFraction {frac, top } => {
    //                 print!(" F {}", frac);
    //             },
    //             _ => {}
    //         }
    //     } else {
    //         let mut a = common.head_discontiguous_region;
    //         while !a.is_zero() {
    //             print!("{}->{}", a, a + VM_MAP.get_contiguous_region_size(a) - 1);
    //             a = VM_MAP.get_next_contiguous_region(a);
    //             if !a.is_zero() {
    //                 print!(" ");
    //             }
    //         }
    //     }
    //     println!();
    // }
}

#[derive(Debug)]
pub struct CommonSpace<PR: PageResource> {
    pub name: &'static str,
    pub descriptor: usize,
    pub vmrequest: VMRequest,
    immortal: bool,
    movable: bool,
    zeroed: bool,
    pub pr: PR,
}

lazy_static! {
    static ref AVAILABLE_HEAP: Mutex<(Address, Address)> = Mutex::new(VM_MAP.heap_range);
}

impl <PR: PageResource> CommonSpace<PR> {
    pub fn new(name: &'static str, movable: bool, immortal: bool, zeroed: bool, metadata_pages_per_region: usize, vmrequest: VMRequest) -> Self {
        println!("CommonSpace: {:?}", vmrequest);

        if vmrequest.is_discontiguous() {
            let descriptor = space_descriptor::create_descriptor();
            return Self {
                name,
                descriptor,
                vmrequest,
                immortal,
                movable,
                zeroed,
                pr: PR::new_discontiguous(metadata_pages_per_region, descriptor),
            }
        }

        let mut available_heap = AVAILABLE_HEAP.lock().unwrap();
        let (extent, top) = match vmrequest {
            VMRequest::RequestFraction{frac, top: _top}                   => (get_frac_available(frac, &available_heap), _top),
            VMRequest::RequestExtent{extent: _extent, top: _top}          => (_extent, _top),
            VMRequest::RequestFixed{start: _, extent: _extent, top: _top} => (_extent, _top),
            _                                                             => unreachable!(),
        };

        if extent != raw_chunk_align(extent, false) {
            panic!("{} requested non-aligned extent: {} bytes", name, extent);
        }

        let start: Address = {
            if let VMRequest::RequestFixed { start, .. } = vmrequest {
                if start.as_usize() != chunk_align(start, false).as_usize() {
                    panic!("{} starting on non-aligned boundary: {} bytes", name, start.as_usize());
                }
                start
            } else if top {
                available_heap.1 -= extent;
                available_heap.1
            } else {
                let start = available_heap.0;
                available_heap.0 += extent;
                start
            }
        };

        if available_heap.0 > available_heap.1 {
            panic!("Out of virtual address space allocating \"{}\" at {} ({} > {})", name, available_heap.0 - extent, available_heap.0, available_heap.1);
        }
        
        let descriptor = space_descriptor::create_descriptor_from_heap_range(start, start + extent);
        VM_MAP.insert(start, extent, descriptor);

        Self {
            name,
            descriptor,
            vmrequest,
            immortal,
            movable,
            zeroed,
            pr: PR::new_contiguous(start, extent, metadata_pages_per_region, descriptor),
        }
    }
}

fn get_frac_available(frac: f32, available_heap: &(Address, Address)) -> usize {
    let total = available_heap.1 - available_heap.0;
    let bytes = (frac * total as f32) as usize;
    let mb = bytes >> LOG_BYTES_IN_MBYTE;
    let rtn = mb << LOG_BYTES_IN_MBYTE;
    let aligned_rtn = raw_chunk_align(rtn, false);
    aligned_rtn
}

pub fn required_chunks(pages: usize) -> usize {
    let extent = raw_chunk_align(pages_to_bytes(pages), false);
    extent >> LOG_BYTES_IN_CHUNK
}