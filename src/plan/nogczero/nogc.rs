use ::policy::space::Space;
use ::policy::immortalspace::ImmortalSpace;
use ::policy::rawpagespace::RawPageSpace;
use ::plan::controller_collector_context::ControllerCollectorContext;
use ::plan::{Plan, Phase};
use ::util::ObjectReference;
use ::util::heap::VMRequest;
use ::util::heap::layout::heap_layout::MMAPPER;
use ::util::heap::layout::Mmapper;
use ::util::Address;
use ::util::alloc::allocator::determine_collection_attempts;
use plan::plan::EMERGENCY_COLLECTION;
use ::vm::*;
use ::plan::trace::Trace;
use util::constants::*;
use util::heap::layout::vm_layout_constants::AVAILABLE_BYTES;
use std::cell::UnsafeCell;
use std::thread;
use libc::c_void;
use std::sync::atomic::{self, Ordering};
use ::plan::plan;
use util::heap::PageResource;
use std::mem::uninitialized;

lazy_static! {
    pub static ref PLAN: NoGC = NoGC::new();
}

use super::NoGCTraceLocal;
use super::NoGCMutator;
use super::NoGCCollector;
use util::conversions::bytes_to_pages;
use plan::plan::create_vm_space;

pub type SelectedPlan = NoGC;

pub struct NoGC {
    unsync: UnsafeCell<NoGCUnsync>,
    pub trace: Trace,
}

unsafe impl Sync for NoGC {}

pub struct NoGCUnsync {
    pub vm_space: ImmortalSpace,
    pub space: RawPageSpace,
    pub versatile_space: ImmortalSpace,
    pub total_pages: usize,
    pub collection_attempt: usize,
}

static mut INSTANT: Option<::std::time::Instant> = None;

impl Plan for NoGC {
    type MutatorT = NoGCMutator;
    type TraceLocalT = NoGCTraceLocal;
    type CollectorT = NoGCCollector;

    fn new() -> Self {
        NoGC {
            unsync: UnsafeCell::new(NoGCUnsync {
                vm_space: create_vm_space(),
                space: RawPageSpace::new("rps"),
                versatile_space: ImmortalSpace::new("vs", true, VMRequest::discontiguous()),
                total_pages: 0,
                collection_attempt: 0,
            }),
            trace: Trace::new(),
        }
    }

    unsafe fn gc_init(&self, heap_size: usize) {
        // let heap_size = 10000 * BYTES_IN_MBYTE;
        println!("GCInit: Heap Size = {}MB", heap_size / BYTES_IN_MBYTE);
        ::util::heap::layout::heap_layout::VM_MAP.finalize_static_space_map();
        let unsync = &mut *self.unsync.get();
        unsync.total_pages = bytes_to_pages(heap_size);
        // FIXME correctly initialize spaces based on options
        unsync.vm_space.init();
        unsync.space.init();
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
        let ptr = Box::into_raw(Box::new(NoGCMutator::new(tls, &unsync.space))) as *mut c_void;
        // {
        //     let mut mutators = super::MUTATORS.lock().unwrap();
        //     mutators.push(ptr as usize);
        // }
        // println!("bind_mutator T {:?}, M {:?}", tls, ptr);
        ptr
    }
    
    fn will_never_move(&self, object: ObjectReference) -> bool {
        true
    }

    unsafe fn collection_phase(&self, tls: *mut c_void, phase: &Phase) {
        println!("Global {:?}", phase);
        let unsync = &mut *self.unsync.get();

        match phase {
            &Phase::SetCollectionKind => {
                {
                    INSTANT = Some(::std::time::Instant::now());
                }
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
            }
            &Phase::PrepareStacks => {
                plan::STACKS_PREPARED.store(true, atomic::Ordering::SeqCst);
            }
            &Phase::Prepare => {
                debug_assert!(self.trace.values.is_empty());
                debug_assert!(self.trace.root_locations.is_empty());
                unsync.vm_space.prepare();
                unsync.space.prepare();
                unsync.versatile_space.prepare();
                // self.print_vm_map();
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
                unsync.vm_space.release();
                unsync.space.release();
                unsync.versatile_space.release();
            }
            &Phase::Complete => {
                debug_assert!(self.trace.values.is_empty());
                debug_assert!(self.trace.root_locations.is_empty());                
                plan::set_gc_status(plan::GcStatus::NotInGC);
                // self.print_vm_map();
                {
                    println!("GC TIME: {}", INSTANT.as_ref().unwrap().elapsed().as_secs());
                    INSTANT = None;
                }
            }
            _ => {
                panic!("Global phase not handled!")
            }
        }
    }

    fn collection_required<PR: PageResource>(&self, space_full: bool, space: &'static PR::Space) -> bool where Self: Sized {
        let heap_full = self.get_pages_reserved() > self.get_total_pages();
        if heap_full {
            println!("GC Reason: Heap Full")
        }
        if space_full {
            println!("GC Reason: Space Full ({})", space.common().name)
        }
        space_full || heap_full
    }

    fn get_total_pages(&self) -> usize {
        let unsync = unsafe { &*self.unsync.get() };
        unsync.total_pages
    }

    fn get_pages_used(&self) -> usize {
        let unsync = unsafe { &*self.unsync.get() };
        unsync.space.reserved_pages() + unsync.versatile_space.reserved_pages()
    }

    fn is_valid_ref(&self, object: ObjectReference) -> bool {
        let unsync = unsafe { &*self.unsync.get() };
        if unsync.space.in_space(object) {
            return true;
        }
        if unsync.vm_space.in_space(object) {
            return true;
        }
        if unsync.versatile_space.in_space(object) {
            return true;
        }
        return false;
    }

    fn is_bad_ref(&self, object: ObjectReference) -> bool {
        false
    }

    fn is_mapped_address(&self, address: Address) -> bool {
        let unsync = unsafe { &*self.unsync.get() };
        if unsafe {
            unsync.space.in_space(address.to_object_reference()) ||
            unsync.vm_space.in_space(address.to_object_reference()) ||
            unsync.versatile_space.in_space(address.to_object_reference())
        } {
            return MMAPPER.address_is_mapped(address);
        } else {
            return false;
        }
    }

    fn is_movable(&self, object: ObjectReference) -> bool {
        let unsync = unsafe { &*self.unsync.get() };
        if unsync.space.in_space(object) {
            return unsync.space.is_movable();
        }
        if unsync.vm_space.in_space(object) {
            return unsync.vm_space.is_movable();
        }
        if unsync.versatile_space.in_space(object) {
            return unsync.versatile_space.is_movable();
        }
        return true;
    }
}

impl NoGC {
    fn print_vm_map(&self) {
        println!("Heap Size = {}MB", self.total_pages * BYTES_IN_PAGE / BYTES_IN_MBYTE);
        println!("Max Heap Size = {}MB", AVAILABLE_BYTES / BYTES_IN_MBYTE);
        // if super::VERBOSE {
            self.vm_space.print_vm_map();
            self.versatile_space.print_vm_map();
            self.space.print_vm_map();
        // }
    }
}

impl ::std::ops::Deref for NoGC {
    type Target = NoGCUnsync;
    fn deref(&self) -> &NoGCUnsync {
        unsafe { &*self.unsync.get() }
    }
}

impl ::std::ops::DerefMut for NoGC {
    fn deref_mut(&mut self) -> &mut NoGCUnsync {
        unsafe { &mut *self.unsync.get() }
    }
}