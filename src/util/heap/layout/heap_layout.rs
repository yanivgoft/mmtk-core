use util::heap::layout::ByteMapMmapper;
use util::heap::layout::vm_map::VMMap;

// FIXME: Use FragmentMmapper for 64-bit heaps
// FIXME: Use Map64 for 64-bit heaps
lazy_static! {
    pub static ref MMAPPER: ByteMapMmapper = ByteMapMmapper::new();
    pub static ref VM_MAP: VMMap = VMMap::new();
}