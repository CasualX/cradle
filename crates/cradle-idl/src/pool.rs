use std::cell::Cell;
use std::mem;

/// A pool for storing strings with stable references.
///
/// The pool is append-only, so once a string is stored, its reference will remain valid for the lifetime of the pool.
#[derive(Default)]
pub struct StringPool {
	strings: Cell<Vec<String>>,
}

impl StringPool {
	/// Creates a new instance.
	pub const fn new() -> StringPool {
		StringPool { strings: Cell::new(Vec::new()) }
	}

	/// Stores a string in the pool and returns a reference to it.
	pub fn store(&self, s: String) -> &str {
		// SAFETY: The pool is append-only, the inner str is stable and the returned lifetime is attached to the pool.
		let s_ref = unsafe { mem::transmute(s.as_str()) };
		let mut strings = self.strings.take();
		strings.push(s);
		let tmp = self.strings.replace(strings);
		mem::forget(tmp);
		s_ref
	}
}
