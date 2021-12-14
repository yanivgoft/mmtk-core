//! Memory policies that can be used for spaces.

/// This class defines and manages spaces.  Each policy is an instance
/// of a space.  A space is a region of virtual memory (contiguous or
/// discontigous) which is subject to the same memory management
/// regime.  Multiple spaces (instances of this class or its
/// descendants) may have the same policy (eg there could be numerous
/// instances of CopySpace, each with different roles). Spaces are
/// defined in terms of a unique region of virtual memory, so no two
/// space instances ever share any virtual memory.<p>
/// In addition to tracking virtual memory use and the mapping to
/// policy, spaces also manage memory consumption (<i>used</i> virtual
/// memory).
pub mod space;

/// Space funciton table captures functions that reflect _space-specific per-object
/// semantics_.   These functions are implemented for each object via a special
/// space-based dynamic/static dispatch mechanism where the semantics are _not_
/// determined by the object's _type_, but rather, are determined by the _space_
/// that the object is in.
pub mod sft;

pub mod copyspace;
pub mod immix;
pub mod immortalspace;
pub mod largeobjectspace;
pub mod lockfreeimmortalspace;
pub mod mallocspace;
pub mod markcompactspace;
