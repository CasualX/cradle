use super::*;

pub fn parse_struct<'a>(state: &mut Parser<'a>) -> Option<ast::StructItem<'a>> {
	// Attributes
	let attrs = parse_attrs(state);

	// Optional `in` or `out` keyword
	let mut direction = None;
	let keyword_inout = state.peek_or_error(ErrorKind::EStructExpected)?;
	if keyword_inout.kind == TokenKind::Ident {
		let text = state.lex.read_str(keyword_inout.span);
		if text == "in" {
			direction = Some(ast::StructDirection::In);
		}
		else if text == "out" {
			direction = Some(ast::StructDirection::Out);
		}
		if direction.is_some() {
			state.next();
		}
	}

	// `struct` keyword
	let keyword = state.next_or_error(ErrorKind::EStructExpected)?;
	if state.lex.read_str(keyword.span) != "struct" {
		state.errors.push(Error {
			kind: ErrorKind::EStructExpected,
			span: keyword.span,
		});
		recover_item(state);
		return None;
	}

	// Optional identifier
	let mut id = None;
	let ident = state.peek_or_error(ErrorKind::EStructIdent)?;
	if ident.kind == TokenKind::Ident {
		state.next();
		id = Some(ast::Ident {
			span: ident.span,
			name: state.lex.read_str(ident.span),
		});
	}

	// Struct fields `{ ... }`
	let fields = parse_struct_fields(state)?;

	let span = keyword_inout.span.combine(&fields.span);
	Some(ast::StructItem { attrs, direction, id, fields, span })
}

pub fn parse_struct_fields<'a>(state: &mut Parser<'a>) -> Option<StructFields<'a>> {
	// `{` punct
	let open = state.next_or_error(ErrorKind::EStructLBrace)?;
	if open.kind != TokenKind::Punct(Punct::LBrace) {
		state.errors.push(Error {
			kind: ErrorKind::EStructLBrace,
			span: open.span,
		});
		recover_item(state);
		return None;
	}

	let mut items = Vec::new();
	loop {
		let token = state.peek_or_error(ErrorKind::EStructFieldExpected)?;

		match token.kind {
			TokenKind::Punct(Punct::RBrace) => break,
			TokenKind::Punct(Punct::RParen) |
			TokenKind::Punct(Punct::RBracket) => {
				state.errors.push(Error {
					kind: ErrorKind::EUnmatchedBraces,
					span: open.span,
				});
				return None;
			}
			_ => {
				if let Some(value) = parse_struct_field(state) {
					items.push(value);
				}
			}
		}

		let comma = state.peek_or_error(ErrorKind::EOF)?;
		match comma.kind {
			TokenKind::Punct(Punct::Comma) => {
				state.next();
			}
			TokenKind::Punct(Punct::RBrace) => break,
			_ => {
				state.errors.push(Error {
					kind: ErrorKind::EStructFieldExpected,
					span: comma.span,
				});
				recover_comma(state);
			}
		}
	}

	// `}` punct
	let close = state.next_or_error(ErrorKind::EStructRBrace)?;
	if close.kind != TokenKind::Punct(Punct::RBrace) {
		state.errors.push(Error {
			kind: ErrorKind::EStructRBrace,
			span: close.span,
		});
		recover_comma(state);
		return None;
	}

	let span = open.span.combine(&close.span);
	Some(ast::StructFields { items, span })
}

pub fn parse_struct_field<'a>(state: &mut Parser<'a>) -> Option<ast::StructField<'a>> {
	// Attributes
	let attrs = parse_attrs(state);

	// Identifier
	let ident = state.next_or_error(ErrorKind::EStructFieldIdent)?;
	if ident.kind != TokenKind::Ident {
		state.errors.push(Error {
			kind: ErrorKind::EStructFieldIdent,
			span: ident.span,
		});
		recover_comma(state);
		return None;
	}
	let id = ast::Ident {
		span: ident.span,
		name: state.lex.read_str(ident.span),
	};

	// `:` punct
	let colon = state.next_or_error(ErrorKind::EStructFieldColon)?;
	if colon.kind != TokenKind::Punct(Punct::Colon) {
		state.errors.push(Error {
			kind: ErrorKind::EStructFieldColon,
			span: colon.span,
		});
		recover_comma(state);
		return None;
	}

	// Type
	let ty = parse_type(state)?;

	// Default value (optional)
	let mut default = None;
	let punct = state.peek()?;
	if punct.kind == TokenKind::Punct(Punct::Eq) {
		state.next();
		default = parse_default_value(state);
	}

	let span = ident.span.combine(&ty.span);
	Some(ast::StructField { attrs, id, ty, default, span })
}

pub fn parse_default_value<'a>(state: &mut Parser<'a>) -> Option<ast::DefaultValue<'a>> {
	let token = state.next_or_error(ErrorKind::EStructFieldDefaultValue)?;
	if !matches!(token.kind, TokenKind::Literal(_)) {
		state.errors.push(Error {
			kind: ErrorKind::EStructFieldDefaultValue,
			span: token.span,
		});
		return None;
	}
	Some(ast::DefaultValue {
		span: token.span,
		value: state.lex.read_str(token.span),
	})
}
