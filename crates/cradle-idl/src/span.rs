
/// Source spans for error reporting and diagnostics.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct SourceSpan {
	pub index: usize,
	pub start: u32,
	pub end: u32,
}

impl SourceSpan {
	/// Creates an empty `SourceSpan` for the given file index.
	#[inline]
	pub const fn empty(index: usize) -> SourceSpan {
		SourceSpan { index, start: !0, end: !0 }
	}

	/// Creates a new `SourceSpan` from the given start and end offsets.
	#[inline]
	pub const fn new(index: usize, start: usize, end: usize) -> SourceSpan {
		SourceSpan { index, start: start as u32, end: end as u32 }
	}

	/// Combines two spans into one that covers both (and everything in between).
	#[inline]
	pub fn combine(&self, other: &SourceSpan) -> SourceSpan {
		assert_eq!(self.index, other.index, "Cannot combine spans from different files");
		SourceSpan {
			index: self.index,
			start: self.start.min(other.start),
			end: self.end.max(other.end),
		}
	}

	/// Returns the range of the span as a `std::ops::Range<usize>`.
	#[inline]
	pub const fn range(&self) -> std::ops::Range<usize> {
		self.start as usize..self.end as usize
	}

	pub fn resolve<'a>(&self, file_name: &'a str, text: &'a str) -> ResolvedSpan<'a> {
		let (line_start, column_start) = line_col(text, self.start as usize);
		let (line_end, column_end) = line_col(text, self.end as usize);
		ResolvedSpan {
			file_name,
			text: &text[self.range()],
			line_start,
			line_end,
			column_start,
			column_end,
		}
	}
}

pub struct ResolvedSpan<'a> {
	pub file_name: &'a str,
	pub text: &'a str,

	pub line_start: usize,
	pub line_end: usize,
	pub column_start: usize,
	pub column_end: usize,
}

fn line_col(input: &str, offset: usize) -> (usize, usize) {
	let mut line = 1;
	let mut column = 1;
	for (i, c) in input.char_indices() {
		if i >= offset {
			break;
		}
		if c == '\n' {
			line += 1;
			column = 1;
		} else {
			column += 1;
		}
	}
	(line, column)
}
