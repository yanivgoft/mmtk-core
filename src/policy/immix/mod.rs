pub mod block;
pub mod chunk;
pub mod defrag;
pub mod immixspace;
pub mod line;

pub use immixspace::*;

use crate::policy::immix::block::Block;

/// Mark/sweep memory for block-level only
pub const BLOCK_ONLY: bool = true;

/// Opportunistic copying
pub const DEFRAG: bool = false;

/// Mark lines when scanning objects.
/// Otherwise, do it at mark time.
pub const MARK_LINE_AT_SCAN_TIME: bool = false;

macro_rules! validate {
    ($x: expr) => { assert!($x, stringify!($x)) };
    ($x: expr => $y: expr) => { if $x { assert!($y, stringify!($x implies $y)) } };
}

fn validate_features() {
    // Block-only immix cannot do defragmentation
    validate!(DEFRAG => !BLOCK_ONLY);
    // Number of lines in a block should not exceed BlockState::MARK_MARKED
    assert!(Block::LINES / 2 <= u8::MAX as usize - 2);
}
