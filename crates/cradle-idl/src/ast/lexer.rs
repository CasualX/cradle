use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Punct {
	/// `(`
	LParen,
	/// `)`
	RParen,
	/// `[`
	LBracket,
	/// `]`
	RBracket,
	/// `{`
	LBrace,
	/// `}`
	RBrace,
	/// `->`
	Arrow,
	/// `,`
	Comma,
	/// `;`
	Semicolon,
	/// `:`
	Colon,
	/// `?`
	QuestionMark,
	/// `!`
	ExclamationMark,
	/// `#`
	Hash,
	/// `.`
	Dot,
	/// `|`
	Pipe,
	/// `*`
	Star,
	/// `-`
	Dash,
	/// `=`
	Eq,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Literal {
	/// Integer literal.
	Integer,
	/// Floating-point number.
	Float,
	/// String literal, including the quotes.
	String,
	/// Char literal, including the quotes.
	Char,
	/// `true` or `false`
	Bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
	/// Identifier.
	///
	/// Identifiers are `[a-zA-Z][a-zA-Z0-9]*`.
	Ident,
	/// Literal value.
	///
	/// Examples: `123`, `3.14e-1`, `"hello"`, `'c'`, `true`, `false`.
	Literal(Literal),
	/// Punctuation.
	Punct(Punct),
	/// Comments.
	///
	/// Regular comments: `// comment` or `/* comment */`.
	///
	/// Documentation comments: `/// doc comment` or `/** doc comment */`.
	Comment,
	/// Invalid token.
	///
	/// When an invalid character is encountered, an error token is emitted with the span of the rest of the document.
	Error,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Token {
	pub kind: TokenKind,
	pub span: SourceSpan,
}

pub struct Lexer<'a> {
	input: &'a [u8],
	file_index: usize,
	position: usize,
}

impl<'a> Lexer<'a> {
	pub const fn new(input: &'a str, file_index: usize) -> Lexer<'a> {
		let input = input.as_bytes();
		Lexer { input, file_index, position: 0 }
	}

	pub fn unknown(&self) -> SourceSpan {
		SourceSpan::new(self.file_index, self.position, self.position)
	}

	pub fn read_str(&self, span: SourceSpan) -> &'a str {
		std::str::from_utf8(&self.input[span.range()]).unwrap()
	}

	/// Skips ascii whitespace and returns true if any whitespace was skipped.
	fn whitespace(&mut self) -> bool {
		let start = self.position;
		while let Some(c) = self.input.get(self.position) {
			if !c.is_ascii_whitespace() {
				break;
			}
			self.position += 1;
		}
		self.position > start
	}

	/// Skips the rest of the current line, excluding the newline character(s).
	pub fn line(&mut self) {
		while let Some(&c) = self.input.get(self.position) {
			if c == b'\n' || c == b'\r' {
				break;
			}
			self.position += 1;
		}
	}

	fn until(&mut self, s: &[u8]) {
		loop {
			if self.input[self.position..].starts_with(s) {
				self.position += s.len();
				break;
			}
			self.position += 1;
		}
	}

	pub fn next_token(&mut self) -> Option<Token> {
		self.whitespace();

		let input = &self.input[self.position..];
		if input.is_empty() {
			return None;
		}

		let c = input[0];

		let token = match c {
			b't' if input.starts_with(b"true") && input.get(4).map_or(true, |&c| !c.is_ascii_alphanumeric()) => self.bool_literal(true),
			b'f' if input.starts_with(b"false") && input.get(5).map_or(true, |&c| !c.is_ascii_alphanumeric()) => self.bool_literal(false),
			b'/' => self.comment(),
			b'"' => self.string_literal(),
			b'\'' => self.char_literal(),
			b'0'..=b'9' => self.number_literal(),
			b'-' if matches!(input.get(1), Some(b'0'..=b'9')) => self.number_literal(),
			b'+' if matches!(input.get(1), Some(b'0'..=b'9')) => self.number_literal(),
			b'a'..=b'z' | b'A'..=b'Z' => self.identifier(),
			b'(' => self.punct(Punct::LParen, 1),
			b')' => self.punct(Punct::RParen, 1),
			b'[' => self.punct(Punct::LBracket, 1),
			b']' => self.punct(Punct::RBracket, 1),
			b'{' => self.punct(Punct::LBrace, 1),
			b'}' => self.punct(Punct::RBrace, 1),
			b'-' if matches!(input.get(1), Some(b'>')) => self.punct(Punct::Arrow, 2),
			b',' => self.punct(Punct::Comma, 1),
			b';' => self.punct(Punct::Semicolon, 1),
			b':' => self.punct(Punct::Colon, 1),
			b'=' => self.punct(Punct::Eq, 1),
			b'?' => self.punct(Punct::QuestionMark, 1),
			b'!' => self.punct(Punct::ExclamationMark, 1),
			b'|' => self.punct(Punct::Pipe, 1),
			b'.' => self.punct(Punct::Dot, 1),
			b'*' => self.punct(Punct::Star, 1),
			b'-' => self.punct(Punct::Dash, 1),
			b'#' => self.punct(Punct::Hash, 1),
			_ => {
				let start = self.position;
				self.position = self.input.len();
				Token { kind: TokenKind::Error, span: SourceSpan::new(self.file_index, start, self.position) }
			}
		};
		Some(token)
	}

	fn punct(&mut self, punct: Punct, chrs: usize) -> Token {
		let start = self.position;
		self.position += chrs;
		Token { kind: TokenKind::Punct(punct), span: SourceSpan::new(self.file_index, start, self.position) }
	}

	fn comment(&mut self) -> Token {
		let start = self.position;
		let input = &self.input[self.position..];
		if input.starts_with(b"//") {
			self.line();
			return Token { kind: TokenKind::Comment, span: SourceSpan::new(self.file_index, start, self.position) };
		}
		if input.starts_with(b"/*") {
			self.until(b"*/");
			return Token { kind: TokenKind::Comment, span: SourceSpan::new(self.file_index, start, self.position) };
		}
		unreachable!()
	}

	fn string_literal(&mut self) -> Token {
		let start = self.position;
		self.position += 1; // skip opening quote
		while let Some(&c) = self.input.get(self.position) {
			self.position += 1;
			if c == b'"' {
				return Token { kind: TokenKind::Literal(Literal::String), span: SourceSpan::new(self.file_index, start, self.position) };
			}
			if c == b'\\' {
				self.position += 1; // skip escaped character
			}
		}

		Token { kind: TokenKind::Error, span: SourceSpan::new(self.file_index, start, self.position) }
	}

	fn char_literal(&mut self) -> Token {
		let start = self.position;
		self.position += 1; // skip opening quote
		while let Some(&c) = self.input.get(self.position) {
			self.position += 1;
			if c == b'\'' {
				return Token { kind: TokenKind::Literal(Literal::Char), span: SourceSpan::new(self.file_index, start, self.position) };
			}
			if c == b'\\' {
				self.position += 1; // skip escaped character
			}
		}

		Token { kind: TokenKind::Error, span: SourceSpan::new(self.file_index, start, self.position) }
	}

	fn number_literal(&mut self) -> Token {
		let start = self.position;
		if self.input[self.position] == b'-' {
			self.position += 1;
		}
		let mut lit = Literal::Integer;
		while let Some(&c) = self.input.get(self.position) {
			match c {
				b'0'..=b'9' => self.position += 1,
				b'.' | b'e' | b'E' => {
					self.position += 1;
					lit = Literal::Float;
				},
				_ => break,
			}
		}
		Token { kind: TokenKind::Literal(lit), span: SourceSpan::new(self.file_index, start, self.position) }
	}

	fn bool_literal(&mut self, value: bool) -> Token {
		let start = self.position;
		self.position += if value { 4 } else { 5 };
		Token { kind: TokenKind::Literal(Literal::Bool), span: SourceSpan::new(self.file_index, start, self.position) }
	}

	fn identifier(&mut self) -> Token {
		let start = self.position;
		while let Some(&c) = self.input.get(self.position) {
			match c {
				b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' => self.position += 1,
				_ => break,
			}
		}
		Token { kind: TokenKind::Ident, span: SourceSpan::new(self.file_index, start, self.position) }
	}
}
