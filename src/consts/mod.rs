use std::fmt;
use std::time::Duration;

// pub(crate) const MAX_ALLOC_SIZE: usize = 0xFFFFFFF;
pub(crate) const MAX_MAP_SIZE: u64 = 0x0FFF_FFFF;

/// Are unaligned load/stores broken on this arch?
// pub(crate) const BROKEN_UNALIGNED: bool = false;

pub(crate) const MIN_KEYS_PER_PAGE: usize = 2;

pub(crate) type PGID = u64;

pub(crate) type TXID = u64;

pub(crate) const MAX_MMAP_STEP: u64 = 1 << 30;

/// database mime header
pub(crate) const MAGIC: u32 = 0xED0C_DAED;

/// database version
pub(crate) const VERSION: u32 = 2;

// TODO: openbsd
pub(crate) const IGNORE_NOSYNC: bool = true;

pub(crate) const DEFAULT_MAX_BATCH_SIZE: usize = 1000;

pub(crate) const DEFAULT_MAX_BATCH_DELAY: Duration = Duration::from_millis(10000);
// pub(crate) const DEFAULT_ALLOC_SIZE: u64 = 16 * 1024 * 1024;

bitflags! {
	/// Defines type of the page
	pub struct Flags: u16 {
		/// Either branch or bucket page
		const Branches = 0b00001;
		/// Leaf page
		const Leaves = 0b00010;
		/// Meta page
		const Meta = 0b00100;
		/// Freelist page
		const Freelist = 0b10000;
	}
}

impl fmt::Display for Flags {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let d = match *self {
			Flags::Branches => "branches".to_string(),
			Flags::Leaves => "leaves".to_string(),
			Flags::Meta => "meta".to_string(),
			Flags::Freelist => "freelist".to_string(),
			_ => panic!("unknown flag"),
		};
		write!(f, "{}", d)
	}
}
