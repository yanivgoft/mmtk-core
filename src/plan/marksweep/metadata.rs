use crate::util::conversions;
use crate::util::heap::layout::vm_layout_constants::BYTES_IN_CHUNK;
use crate::util::side_metadata::SideMetadata;
use crate::util::side_metadata::SideMetadataID;
use crate::util::Address;
use crate::util::ObjectReference;
use conversions::chunk_align_down;
use std::collections::HashSet;
use std::sync::RwLock;

lazy_static! {
    pub static ref MAPPED_CHUNKS: RwLock<HashSet<Address>> = RwLock::default();
}

pub static mut ALLOCATION_METADATA_ID: SideMetadataID = SideMetadataID::new();
pub static mut MARKING_METADATA_ID: SideMetadataID = SideMetadataID::new();

pub fn meta_space_mapped(address: Address) -> bool {
    let chunk_start = chunk_align_down(address);
    MAPPED_CHUNKS.read().unwrap().contains(&chunk_start)
}

pub unsafe fn map_meta_space_for_chunk(chunk_start: Address) {
    SideMetadata::map_meta_space(chunk_start, BYTES_IN_CHUNK, ALLOCATION_METADATA_ID);
    SideMetadata::map_meta_space(chunk_start, BYTES_IN_CHUNK, MARKING_METADATA_ID);
    MAPPED_CHUNKS.write().unwrap().insert(chunk_start);
}

// Check if a given object was allocated by malloc
pub fn is_malloced(object: ObjectReference) -> bool {
    let address = object.to_address();
    unsafe {
        meta_space_mapped(address)
            && SideMetadata::load_atomic(ALLOCATION_METADATA_ID, address) == 1
    }
}

// check the corresponding bit in the metadata table
pub fn is_marked(object: ObjectReference) -> bool {
    let address = object.to_address();
    debug_assert!(meta_space_mapped(address));
    unsafe { SideMetadata::load_atomic(MARKING_METADATA_ID, address) == 1 }
}

pub fn set_alloc_bit(address: Address) {
    debug_assert!(meta_space_mapped(address));
    unsafe {
        SideMetadata::store_atomic(ALLOCATION_METADATA_ID, address, 1);
    }
}

pub fn set_mark_bit(address: Address) {
    debug_assert!(meta_space_mapped(address));
    unsafe {
        SideMetadata::store_atomic(MARKING_METADATA_ID, address, 1);
    }
}

pub fn unset_alloc_bit(address: Address) {
    debug_assert!(meta_space_mapped(address));
    unsafe {
        SideMetadata::store_atomic(ALLOCATION_METADATA_ID, address, 0);
    }
}

pub fn unset_mark_bit(address: Address) {
    debug_assert!(meta_space_mapped(address));
    unsafe {
        SideMetadata::store_atomic(MARKING_METADATA_ID, address, 0);
    }
}
