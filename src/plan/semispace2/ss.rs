use ::policy::space::Space;
use super::SSMutator;
use super::SSTraceLocal;
use super::SSCollector;
use ::plan::plan;
use ::plan::Plan;
use ::plan::Allocator;
use ::policy::copyspace2::CopySpace;
use ::policy::rawpagespace::RawPageSpace;
use ::policy::immortalspace::ImmortalSpace;
use ::plan::Phase;
use ::plan::trace::Trace;
use ::util::ObjectReference;
use ::util::alloc::allocator::determine_collection_attempts;
use ::util::heap::layout::heap_layout::MMAPPER;
use ::util::heap::layout::Mmapper;
use ::util::Address;
use ::util::heap::VMRequest;
use libc::c_void;
use std::cell::UnsafeCell;
use std::sync::atomic::{self, Ordering};
use util::heap::PageResource;
use ::vm::{Scanning, VMScanning};
use std::thread;
use util::conversions::bytes_to_pages;
use plan::plan::create_vm_space;
use plan::plan::EMERGENCY_COLLECTION;

pub type SelectedPlan = SemiSpace;

pub const ALLOC_SS: Allocator = Allocator::Default;
pub const SCAN_BOOT_IMAGE: bool = true;

lazy_static! {
    pub static ref PLAN: SemiSpace = SemiSpace::new();
}

pub struct SemiSpace {
    pub unsync: UnsafeCell<SemiSpaceUnsync>,
    pub ss_trace: Trace,
}

pub struct SemiSpaceUnsync {
    pub hi: bool,
    pub vm_space: ImmortalSpace,
    pub copyspace0: CopySpace,
    pub copyspace1: CopySpace,
    pub versatile_space: RawPageSpace,
    total_pages: usize,
    collection_attempt: usize,
}

unsafe impl Sync for SemiSpace {}

impl Plan for SemiSpace {
    type MutatorT = SSMutator;
    type TraceLocalT = SSTraceLocal;
    type CollectorT = SSCollector;

    fn new() -> Self {
        SemiSpace {
            unsync: UnsafeCell::new(SemiSpaceUnsync {
                hi: false,
                vm_space: ImmortalSpace::new("x", true, VMRequest::discontiguous()),
                copyspace0: CopySpace::new("copyspace0", false, true, VMRequest::discontiguous()),
                copyspace1: CopySpace::new("copyspace1", true, true, VMRequest::discontiguous()),
                versatile_space: RawPageSpace::new("versatile_space"),
                total_pages: 0,
                collection_attempt: 0,
            }),
            ss_trace: Trace::new(),
        }
    }

    unsafe fn gc_init(&self, _heap_size: usize) {
        
        let heap_size = 1 * 1024 * 1024 * 1024;
        println!("GCInit: Heap Size = {}MB", heap_size / ::util::constants::BYTES_IN_MBYTE);
        // ::util::heap::layout::heap_layout::VM_MAP.finalize_static_space_map();
        let unsync = &mut *self.unsync.get();
        unsync.total_pages = bytes_to_pages(heap_size);
        unsync.vm_space.init();
        unsync.copyspace0.init();
        unsync.copyspace1.init();
        unsync.versatile_space.init();

        // These VMs require that the controller thread is started by the VM itself.
        // (Usually because it calls into VM code that accesses the TLS.)
        if !(cfg!(feature = "jikesrvm") || cfg!(feature = "openjdk")) {
            thread::spawn(|| {
                ::plan::plan::CONTROL_COLLECTOR_CONTEXT.run(0 as *mut c_void)
            });
        }
    }

    fn bind_mutator(&self, tls: *mut c_void) -> *mut c_void {
        let unsync = unsafe { &*self.unsync.get() };
        Box::into_raw(Box::new(SSMutator::new(tls, self.tospace(), &unsync.versatile_space))) as *mut c_void
    }

    fn will_never_move(&self, object: ObjectReference) -> bool {
        if self.tospace().in_space(object) || self.fromspace().in_space(object) {
            return false;
        }
        if self.versatile_space.in_space(object) {
            return true;
        }
        false // this preserves correctness over efficiency
    }

    fn is_valid_ref(&self, object: ObjectReference) -> bool {
        if self.versatile_space.in_space(object) {
            return true;
        }
        if self.vm_space.in_space(object) {
            return true;
        }
        if self.tospace().in_space(object) {
            return true;
        }
        return false;
    }

    fn collection_required(&self, space_full: bool, space: &'static impl Space) -> bool where Self: Sized {
        let heap_full = self.get_pages_reserved() > self.get_total_pages();
        if heap_full {
            println!("GC Reason: Heap Full {} {} {}", self.tospace().reserved_pages(), self.get_pages_reserved(), self.get_total_pages());
        }
        if space_full {
            println!("GC Reason: Space Full ({})", space.common().name)
        }
        space_full || heap_full
    }

    unsafe fn collection_phase(&self, tls: *mut c_void, phase: &Phase) {
        let unsync = &mut *self.unsync.get();

        match phase {
            &Phase::SetCollectionKind => {
                let unsync = &mut *self.unsync.get();
                unsync.collection_attempt = if <SelectedPlan as Plan>::is_user_triggered_collection() {
                    1 } else { determine_collection_attempts() };

                let emergency_collection = !<SelectedPlan as Plan>::is_internal_triggered_collection()
                    && self.last_collection_was_exhaustive() && unsync.collection_attempt > 1;
                EMERGENCY_COLLECTION.store(emergency_collection, Ordering::Relaxed);

                if emergency_collection {
                    self.force_full_heap_collection();
                }
            }
            &Phase::Initiate => {
                plan::set_gc_status(plan::GcStatus::GcPrepare);
                println!("GC Initiate");
                self.print_vm_map();
            }
            &Phase::PrepareStacks => {
                plan::STACKS_PREPARED.store(true, atomic::Ordering::SeqCst);
            }
            &Phase::Prepare => {
                debug_assert!(self.ss_trace.values.is_empty());
                debug_assert!(self.ss_trace.root_locations.is_empty());
                unsync.hi = !unsync.hi; // flip the semi-spaces
                // prepare each of the collected regions
                unsync.copyspace0.prepare(unsync.hi);
                unsync.copyspace1.prepare(!unsync.hi);
                unsync.versatile_space.prepare();
                unsync.vm_space.prepare();
            }
            &Phase::StackRoots => {
                VMScanning::notify_initial_thread_scan_complete(false, tls);
                plan::set_gc_status(plan::GcStatus::GcProper);
            }
            &Phase::Roots => {
                VMScanning::reset_thread_counter();
                plan::set_gc_status(plan::GcStatus::GcProper);
            }
            &Phase::Closure => {}
            &Phase::Release => {
                // release the collected region
                if unsync.hi {
                    unsync.copyspace0.release();
                } else {
                    unsync.copyspace1.release();
                }
                unsync.versatile_space.release();
                unsync.vm_space.release();
            }
            &Phase::Complete => {
                debug_assert!(self.ss_trace.values.is_empty());
                debug_assert!(self.ss_trace.root_locations.is_empty());
                plan::set_gc_status(plan::GcStatus::NotInGC);
                println!("GC Complete");
                self.print_vm_map();
            }
            _ => {
                panic!("Global phase not handled!")
            }
        }
    }

    fn get_total_pages(&self) -> usize {
        unsafe{(&*self.unsync.get()).total_pages}
    }

    fn get_collection_reserve(&self) -> usize {
        self.tospace().reserved_pages()
    }

    fn get_pages_used(&self) -> usize {
        self.tospace().reserved_pages() + self.versatile_space.reserved_pages()
    }

    fn is_bad_ref(&self, object: ObjectReference) -> bool {
        self.fromspace().in_space(object)
    }

    fn is_movable(&self, object: ObjectReference) -> bool {
        let unsync = unsafe { &*self.unsync.get() };
        if unsync.vm_space.in_space(object) {
            return unsync.vm_space.is_movable();
        }
        if unsync.copyspace0.in_space(object) {
            return unsync.copyspace0.is_movable();
        }
        if unsync.copyspace1.in_space(object) {
            return unsync.copyspace1.is_movable();
        }
        if unsync.versatile_space.in_space(object) {
            return unsync.versatile_space.is_movable();
        }
        return true;
    }

    fn is_mapped_address(&self, address: Address) -> bool {
        let unsync = unsafe { &*self.unsync.get() };
        if unsafe{
            unsync.vm_space.in_space(address.to_object_reference())  ||
            unsync.versatile_space.in_space(address.to_object_reference()) ||
            unsync.copyspace0.in_space(address.to_object_reference()) ||
            unsync.copyspace1.in_space(address.to_object_reference())
        } {
            return MMAPPER.address_is_mapped(address);
        } else {
            return false;
        }
    }
}

impl SemiSpace {
    pub fn tospace(&self) -> &'static CopySpace {
        let unsync = unsafe { &*self.unsync.get() };

        if unsync.hi {
            &unsync.copyspace1
        } else {
            &unsync.copyspace0
        }
    }

    pub fn fromspace(&self) -> &'static CopySpace {
        let unsync = unsafe { &*self.unsync.get() };

        if unsync.hi {
            &unsync.copyspace0
        } else {
            &unsync.copyspace1
        }
    }

    pub fn get_vs(&self) -> &'static RawPageSpace {
        let unsync = unsafe { &*self.unsync.get() };
        &unsync.versatile_space
    }

    pub fn get_sstrace(&self) -> &Trace {
        &self.ss_trace
    }

    pub fn print_vm_map(&self) {
        use ::util::constants::*;
        use ::util::heap::layout::vm_layout_constants::*;
        println!("Heap Size = {}MB", self.total_pages * BYTES_IN_PAGE / BYTES_IN_MBYTE);
        println!("Used Size = {}MB", (self.tospace().reserved_pages() + self.fromspace().reserved_pages()) * BYTES_IN_PAGE / BYTES_IN_MBYTE);
        println!("To space = {}MB", self.tospace().reserved_pages() * BYTES_IN_PAGE / BYTES_IN_MBYTE);
        println!("From space = {}MB", self.fromspace().reserved_pages() * BYTES_IN_PAGE / BYTES_IN_MBYTE);
        println!("Max Heap Size = {}MB", AVAILABLE_BYTES / BYTES_IN_MBYTE);
        // if super::VERBOSE {
            // self.vm_space.print_vm_map();
            // self.versatile_space.print_vm_map();
            // self.copyspace0.print_vm_map();
            // self.copyspace1.print_vm_map();
        // }
    }
}


impl ::std::ops::Deref for SemiSpace {
    type Target = SemiSpaceUnsync;
    fn deref(&self) -> &SemiSpaceUnsync {
        unsafe { &*self.unsync.get() }
    }
}

impl ::std::ops::DerefMut for SemiSpace {
    fn deref_mut(&mut self) -> &mut SemiSpaceUnsync {
        unsafe { &mut *self.unsync.get() }
    }
}