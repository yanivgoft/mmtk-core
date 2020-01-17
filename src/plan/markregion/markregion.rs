use policy::space::Space;
use policy::immortalspace::ImmortalSpace;
use policy::rawpagespace::RawPageSpace;
use policy::markregionspace::MarkRegionSpace;
use plan::{plan, Plan, Phase};
use util::{ObjectReference, Address};
use util::heap::{VMRequest, PageResource};
use util::heap::layout::heap_layout::MMAPPER;
use util::heap::layout::Mmapper;
use util::alloc::allocator::determine_collection_attempts;
use plan::plan::EMERGENCY_COLLECTION;
use vm::*;
use plan::trace::Trace;
use util::constants::*;
use util::heap::layout::vm_layout_constants::AVAILABLE_BYTES;
use std::cell::UnsafeCell;
use std::thread;
use libc::c_void;
use std::sync::atomic::{self, Ordering};
use super::*;
use util::conversions::bytes_to_pages;
use plan::plan::create_vm_space;



lazy_static! {
    pub static ref PLAN: MarkRegion = MarkRegion::new();
}

pub type SelectedPlan = MarkRegion;



pub struct MarkRegion {
    unsync: UnsafeCell<MarkRegionUnsync>,
    pub trace: Trace,
}

unsafe impl Sync for MarkRegion {}

pub struct MarkRegionUnsync {
    pub vm_space: ImmortalSpace,
    pub space: MarkRegionSpace,
    pub versatile_space: RawPageSpace,
    pub total_pages: usize,
    pub collection_attempt: usize,
}

static mut INSTANT: Option<::std::time::Instant> = None;

impl Plan for MarkRegion {
    type MutatorT = MarkRegionMutator;
    type TraceLocalT = MarkRegionTraceLocal;
    type CollectorT = MarkRegionCollector;

    fn new() -> Self {
        Self {
            unsync: UnsafeCell::new(MarkRegionUnsync {
                vm_space: create_vm_space(),
                space: MarkRegionSpace::new("mr"),
                versatile_space: RawPageSpace::new("vs"),
                total_pages: 0,
                collection_attempt: 0,
            }),
            trace: Trace::new(),
        }
    }

    unsafe fn gc_init(&self, heap_size: usize) {
        // let heap_size = 10000 * BYTES_IN_MBYTE;
        let heap_size = 1 * 1024 * 1024 * 1024;
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
        let ptr = Box::into_raw(Box::new(MarkRegionMutator::new(tls, &unsync.versatile_space, &unsync.space))) as *mut c_void;
        ptr
    }

    fn will_never_move(&self, _object: ObjectReference) -> bool {
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
        self.total_pages
    }

    fn get_pages_used(&self) -> usize {
        // println!("{} {}", self.space.reserved_pages(), self.versatile_space.reserved_pages());
        // println!("{} + {} = {}", self.space.reserved_pages(), self.versatile_space.reserved_pages(), self.space.reserved_pages() + self.versatile_space.reserved_pages());
        self.space.reserved_pages() + self.versatile_space.reserved_pages()
    }

    fn is_valid_ref(&self, object: ObjectReference) -> bool {
        if self.space.in_space(object) {
            return true;
        }
        if self.vm_space.in_space(object) {
            return true;
        }
        if self.versatile_space.in_space(object) {
            return true;
        }
        return false;
    }

    fn is_bad_ref(&self, object: ObjectReference) -> bool {
        !self.is_valid_ref(object)
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

impl MarkRegion {
    pub fn print_vm_map(&self) {
        println!("Heap Size = {}MB", self.total_pages * BYTES_IN_PAGE / BYTES_IN_MBYTE);
        println!("Max Heap Size = {}MB", AVAILABLE_BYTES / BYTES_IN_MBYTE);
        // if super::VERBOSE {
            self.vm_space.print_vm_map();
            self.versatile_space.print_vm_map();
            self.space.print_vm_map();
        // }
    }
}

impl ::std::ops::Deref for MarkRegion {
    type Target = MarkRegionUnsync;
    fn deref(&self) -> &MarkRegionUnsync {
        unsafe { &*self.unsync.get() }
    }
}

impl ::std::ops::DerefMut for MarkRegion {
    fn deref_mut(&mut self) -> &mut MarkRegionUnsync {
        unsafe { &mut *self.unsync.get() }
    }
}