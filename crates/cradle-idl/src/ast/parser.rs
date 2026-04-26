use super::*;

struct LexState<'a> {
	lexer: Lexer<'a>,
	token: Option<Token>,
}

impl<'a> LexState<'a> {
	fn new(input: &'a str, file_index: usize) -> LexState<'a> {
		LexState { lexer: Lexer::new(input, file_index), token: None }
	}
	fn peek(&mut self) -> Option<Token> {
		self.token = self.token.or_else(|| self.lexer.next_token());
		self.token
	}
	fn next(&mut self) -> Option<Token> {
		self.token.take().or_else(|| self.lexer.next_token())
	}
	fn next_if(&mut self, f: impl FnOnce(Token) -> bool) -> Option<Token> {
		let token = self.peek()?;
		if !f(token) {
			return None;
		}
		self.token = None;
		Some(token)
	}
	fn read_str(&self, span: SourceSpan) -> &'a str {
		self.lexer.read_str(span)
	}
}

struct Parser<'a> {
	lex: LexState<'a>,
	errors: Vec<Error>,
}

impl<'a> Parser<'a> {
	fn new(input: &'a str, file_index: usize) -> Parser<'a> {
		Parser { lex: LexState::new(input, file_index), errors: Vec::new() }
	}

	fn peek(&mut self) -> Option<Token> {
		self.lex.peek()
	}
	fn next(&mut self) -> Option<Token> {
		self.lex.next()
	}
	fn next_if(&mut self, f: impl FnOnce(Token) -> bool) -> Option<Token> {
		self.lex.next_if(f)
	}
	fn peek_or_error(&mut self, kind: ErrorKind) -> Option<Token> {
		self.peek().or_else(|| {
			self.errors.push(Error {
				kind,
				span: self.lex.lexer.unknown(),
			});
			None
		})
	}
	fn next_or_error(&mut self, kind: ErrorKind) -> Option<Token> {
		self.next().or_else(|| {
			self.errors.push(Error {
				kind,
				span: self.lex.lexer.unknown(),
			});
			None
		})
	}
}

pub fn parse<'a>(input: &'a str, file_index: usize) -> (IdlFile<'a>, Vec<Error>) {
	let mut parser = Parser::new(input, file_index);

	let mut items = Vec::new();
	while let Some(item) = parse_item(&mut parser) {
		items.push(item);
	}

	let errors = parser.errors;
	(IdlFile { items }, errors)
}

fn parse_hash_attr<'a>(state: &mut Parser<'a>, hash: Token) -> Option<ast::Attribute<'a>> {
	let name = state.next_if(|token| token.kind == TokenKind::Ident)?;
	let name = ast::Ident {
		span: name.span,
		name: state.lex.read_str(name.span),
	};

	let kind = if state.next_if(|token| token.kind == TokenKind::Punct(Punct::Eq)).is_some() {
		let value_token = state.next_if(|token| matches!(token.kind, TokenKind::Ident | TokenKind::Literal(_)))?;
		let value_str = state.lex.read_str(value_token.span);
		let value = ast::AttrValue {
			span: value_token.span,
			text: value_str,
		};
		let span = name.span.combine(&value_token.span);
		ast::AttrKind::Kv(ast::AttrKv { key: name, value, span })
	}
	else {
		ast::AttrKind::Cmd(ast::AttrCmd { name })
	};

	let span = hash.span.combine(&match &kind {
		ast::AttrKind::Cmd(cmd) => cmd.name.span,
		ast::AttrKind::Kv(kv) => kv.value.span,
		ast::AttrKind::Doc(doc) => doc.span,
	});

	Some(ast::Attribute { kind, span })
}

fn parse_attrs<'a>(state: &mut Parser<'a>) -> Vec<ast::Attribute<'a>> {
	let mut attrs = Vec::new();
	while let Some(token) = state.peek() {
		match token.kind {
			TokenKind::Comment => {
				let text = state.lex.read_str(token.span);
				if text.starts_with("///") || text.starts_with("/**") {
					let comment = text.trim_start_matches("///").trim_start_matches("/**").trim_end_matches("*/").trim_ascii();
					attrs.push(ast::Attribute {
						kind: ast::AttrKind::Doc(ast::AttrDoc {
							span: token.span,
							comment,
						}),
						span: token.span,
					});
				}
				state.next();
			}
			// E.g. `#foo` or `#foo = <ident>` or `#foo = <literal>`
			TokenKind::Punct(Punct::Hash) => {
				let hash = state.next().unwrap();
				if let Some(attr) = parse_hash_attr(state, hash) {
					attrs.push(attr);
				}
			}
			_ => break,
		}
	}
	return attrs;
}

fn parse_item<'a>(state: &mut Parser<'a>) -> Option<ast::Item<'a>> {
	let attrs = parse_attrs(state);

	let Some(token) = state.peek() else {
		return None;
	};

	let item = match token.kind {
		TokenKind::Ident => {
			let text = state.lex.read_str(token.span);

			match text {
				"module" => ast::Item::Module(ast::ModuleItem { attrs, ..parse_module(state)? }),
				"handle" => ast::Item::Handle(ast::HandleItem { attrs, ..parse_handle(state)? }),
				"enum" => ast::Item::Enum(ast::EnumItem { attrs, ..parse_enum(state)? }),
				"error" => ast::Item::Error(ast::ErrorItem { attrs, ..parse_error(state)? }),
				"struct" => ast::Item::Struct(ast::StructItem { attrs, ..parse_struct(state)? }),
				"in" => ast::Item::Struct(ast::StructItem { attrs, ..parse_struct(state)? }),
				"out" => ast::Item::Struct(ast::StructItem { attrs, ..parse_struct(state)? }),
				"fn" => ast::Item::Function(ast::FnItem { attrs, ..parse_fn(state)? }),
				_ => {
					state.errors.push(Error {
						kind: ErrorKind::EItemExpected,
						span: token.span,
					});
					return None;
				}
			}
		}
		_ => {
			state.errors.push(Error {
				kind: ErrorKind::EItemExpected,
				span: token.span,
			});
			return None;
		}
	};

	Some(item)
}

mod recover;
use recover::*;

mod module;
use module::*;

mod enumeration;
use enumeration::*;

mod handle;
use handle::*;

mod error;
use error::*;

mod structure;
use structure::*;

mod ty;
use ty::*;

mod function;
use function::*;

#[cfg(test)]
mod tests;
