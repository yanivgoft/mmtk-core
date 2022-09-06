use crate::plan::global::BasePlan;
use crate::plan::global::CommonPlan;
use crate::plan::global::GcStatus;
use crate::plan::marksweep::gc_work::{MSGCWorkContext, MSSweepChunks};
use crate::plan::marksweep::mutator::ALLOCATOR_MAPPING;
use crate::plan::AllocationSemantics;
use crate::plan::Plan;
use crate::plan::PlanConstraints;
use crate::policy::mallocspace::metadata::ACTIVE_CHUNK_METADATA_SPEC;
use crate::policy::mallocspace::MallocSpace;
use crate::policy::space::Space;
use crate::scheduler::*;
use crate::util::alloc::allocators::AllocatorSelector;
#[cfg(not(feature = "global_alloc_bit"))]
use crate::util::alloc_bit::ALLOC_SIDE_METADATA_SPEC;
use crate::util::heap::layout::heap_layout::Mmapper;
use crate::util::heap::layout::heap_layout::VMMap;
use crate::util::heap::HeapMeta;
use crate::util::metadata::side_metadata::{SideMetadataContext, SideMetadataSanity};
use crate::util::options::Options;
use crate::util::VMWorkerThread;
use crate::vm::VMBinding;
use std::sync::Arc;
//use crate::util::Address;

use enum_map::EnumMap;

use mmtk_macros::PlanTraceObject;

use std::collections::HashMap;
use std::fs::File;
use crate::util::Address;
use std::io::Write;
use std::sync::Mutex;

/*
struct Maps{
    count_map : HashMap<Address, u32>,
    max_count_map : HashMap<Address, u32>,
    gc_count : u32,
}
impl Maps {
    fn new(&mut self){
        self.count_map = HashMap::new();
        self.max_count_map = HashMap::new();
        self.gc_count = 0;
    }
}*/

/*
static mut count_map :HashMap<Address, u32> = HashMap::with_capacity(10000);
static mut max_count_map :HashMap<Address, u32> = HashMap::with_capacity(10000);
*/
lazy_static!{
    static ref gc_count : Mutex<u32> = Mutex::new(0);
}
//static mut my_maps: Maps = Maps{count_map: HashMap::new(), max_count_map: /*HashMap::new()*/,gc_count: 0};

lazy_static!{
    static ref address_vec : Mutex<Vec<Address>> = Mutex::new(vec![]/*vec![Address::default(); 30000]*/);
}
lazy_static!{
    static ref count_vec : Mutex<Vec<u8>> = Mutex::new(vec![]/*vec![0; 30000]*/);
}
lazy_static!{
    static ref max_count_vec : Mutex<Vec<u8>> = Mutex::new(vec![]/*vec![0;30000]*/);
}


// static mut address_arr : Address[(0),10000];
// static mut count_arr : int[10000];
// static mut max_count_arr : int[10000];


/*
lazy_static! {
    static ref count_map: HashMap<Address,u32> = HashMap::new(1000);
}
lazy_static! {
    static ref max_count_map: HashMap<Address,u32> = HashMap::new(1000);
}
lazy_static! {
    static ref gc_count: u32 = 0;
}*/
#[derive(PlanTraceObject)]
pub struct MarkSweep<VM: VMBinding> {
    #[fallback_trace]
    common: CommonPlan<VM>,
    #[trace]
    ms: MallocSpace<VM>,
}

pub fn add_to_count_map(add: Address) {
    let mut flag : bool = true;
    for i in 0..count_vec.lock().unwrap().len() {
        /*if address_vec.lock().unwrap()[i] ==Address::default(){
            address_vec.lock().unwrap()[i]=add;
            count_vec.lock().unwrap()[i] = 1;
            break;
        }*/
        if address_vec.lock().unwrap()[i]==add{
            count_vec.lock().unwrap()[i]+=1;
            flag = false;
        }
    }
    if flag{
        address_vec.lock().unwrap().push(add);
        count_vec.lock().unwrap().push(1);
    }
    /*
    let count = count_map.get(&add);
    match count{
        Some(val) => {
            count_map.insert(add, val+1);
        }
        None => {
            count_map.insert(add, 1);
        }
    }
    //let count = count_map.entry(add).get().unwrap_or_else(|v| v.insert(0));
    //count +=1;*/
}


pub const MS_CONSTRAINTS: PlanConstraints = PlanConstraints {
    moves_objects: false,
    gc_header_bits: 2,
    gc_header_words: 0,
    num_specialized_scans: 1,
    may_trace_duplicate_edges: true,
    ..PlanConstraints::default()
};

impl<VM: VMBinding> Plan for MarkSweep<VM> {
    type VM = VM;

    fn get_spaces(&self) -> Vec<&dyn Space<Self::VM>> {
        let mut ret = self.common.get_spaces();
        ret.push(&self.ms);
        ret
    }

    fn schedule_collection(&'static self, scheduler: &GCWorkScheduler<VM>) {
        self.base().set_collection_kind::<Self>(self);
        self.base().set_gc_status(GcStatus::GcPrepare);
        scheduler.schedule_common_work::<MSGCWorkContext<VM>>(self);
        scheduler.work_buckets[WorkBucketStage::Prepare].add(MSSweepChunks::<VM>::new(self));
    }

    fn get_allocator_mapping(&self) -> &'static EnumMap<AllocationSemantics, AllocatorSelector> {
        &*ALLOCATOR_MAPPING
    }

    fn prepare(&mut self, tls: VMWorkerThread) {
        //print!("AHAHAHAHAHAHAH");
        
        self.common.prepare(tls, true);
        count_vec.lock().unwrap().clear();
        //count_map.drain();

        // Dont need to prepare for MallocSpace
    }

    fn release(&mut self, tls: VMWorkerThread) {
        trace!("Marksweep: Release");
        
        *gc_count.lock().unwrap() += 1;
        let mut output = File::create("/home/yaniv/mmtk-core/collectionStats.txt").unwrap();
        for i in 0..address_vec.lock().unwrap().len(){
            /*if address_vec.lock().unwrap()[i] ==Address::default(){
                break;
            }*/
            if i < max_count_vec.lock().unwrap().len(){
                if max_count_vec.lock().unwrap()[i]<count_vec.lock().unwrap()[i]{
                    max_count_vec.lock().unwrap()[i]=count_vec.lock().unwrap()[i];
                }
            }
            else{
                max_count_vec.lock().unwrap().push(count_vec.lock().unwrap()[i]);
            }
            if count_vec.lock().unwrap()[i]!=0{
                let buffer = format!("{} {} {}\n",address_vec.lock().unwrap()[i], count_vec.lock().unwrap()[i], max_count_vec.lock().unwrap()[i]);
                output.write_all(buffer.as_bytes());
            }
        }
        // for (k,v) in count_map.iter(){
        //     let result = max_count_map.get(&k);
        //     match result{
        //         Some(val) => {
        //             max_count_map.insert(k.clone(), std::cmp::max(val.clone(),v.clone()));
        //             let buffer = format!("{} {}",&v, std::cmp::max(val.clone(),v.clone()));
        //             output.write_all(buffer.as_bytes());
        //             //write!(&mut output, "{} {}",&v, std::cmp::max(val.clone(),v)).unwrap();
        //         }
        //         None => {
        //             max_count_map.insert(k.clone(), v.clone());
        //             let buffer = format!("{} {}",v.clone(), v.clone());
        //             output.write_all(buffer.as_bytes());
        //         }
        //     }
            
        // }
        //TODO Drop info to file here
        self.common.release(tls, true);
    }

    fn collection_required(&self, space_full: bool, _space: Option<&dyn Space<Self::VM>>) -> bool {
        self.base().collection_required(self, space_full)
    }

    fn get_used_pages(&self) -> usize {
        self.common.get_used_pages() + self.ms.reserved_pages()
    }

    fn base(&self) -> &BasePlan<VM> {
        &self.common.base
    }

    fn common(&self) -> &CommonPlan<VM> {
        &self.common
    }

    fn constraints(&self) -> &'static PlanConstraints {
        &MS_CONSTRAINTS
    }
}

impl<VM: VMBinding> MarkSweep<VM> {
    pub fn new(vm_map: &'static VMMap, mmapper: &'static Mmapper, options: Arc<Options>) -> Self {
        let heap = HeapMeta::new(&options);
        // if global_alloc_bit is enabled, ALLOC_SIDE_METADATA_SPEC will be added to
        // SideMetadataContext by default, so we don't need to add it here.
        #[cfg(feature = "global_alloc_bit")]
        let global_metadata_specs =
            SideMetadataContext::new_global_specs(&[ACTIVE_CHUNK_METADATA_SPEC]);
        // if global_alloc_bit is NOT enabled,
        // we need to add ALLOC_SIDE_METADATA_SPEC to SideMetadataContext here.
        #[cfg(not(feature = "global_alloc_bit"))]
        let global_metadata_specs = SideMetadataContext::new_global_specs(&[
            ALLOC_SIDE_METADATA_SPEC,
            ACTIVE_CHUNK_METADATA_SPEC,
        ]);

        let res = MarkSweep {
            ms: MallocSpace::new(global_metadata_specs.clone()),
            common: CommonPlan::new(
                vm_map,
                mmapper,
                options,
                heap,
                &MS_CONSTRAINTS,
                global_metadata_specs,
            ),
        };

        // Use SideMetadataSanity to check if each spec is valid. This is also needed for check
        // side metadata in extreme_assertions.
        {
            let mut side_metadata_sanity_checker = SideMetadataSanity::new();
            res.common
                .verify_side_metadata_sanity(&mut side_metadata_sanity_checker);
            res.ms
                .verify_side_metadata_sanity(&mut side_metadata_sanity_checker);
        }

        res
    }

    pub fn ms_space(&self) -> &MallocSpace<VM> {
        &self.ms
    }
}
