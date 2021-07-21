pub mod block;
pub mod chunk;
pub mod defrag;
pub mod immixspace;
pub mod line;

pub use immixspace::*;

/// Mark/sweep memory for block-level only
pub const BLOCK_ONLY: bool = false;

/// Opportunistic copying
pub const DEFRAG: bool = true;

/// Mark lines when scanning objects.
/// Otherwise, do it at mark time.
pub const MARK_LINE_AT_SCAN_TIME: bool = true;

macro_rules! validate {
    ($x: expr) => { assert!($x, stringify!($x)) };
    ($x: expr => $y: expr) => { if $x { assert!($y, stringify!($x implies $y)) } };
}

fn validate_features() {
    // Block-only immix cannot do defragmentation
    validate!(DEFRAG => !BLOCK_ONLY);
}
