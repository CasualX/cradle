use super::*;

pub fn parse_enum<'a>(state: &mut Parser<'a>) -> Option<ast::EnumItem<'a>> {
	// Attributes
	let attrs = parse_attrs(state);

	// Keyword `enum`
	let keyword = state.next_or_error(ErrorKind::EEnumExpected)?;
	if state.lex.read_str(keyword.span) != "enum" {
		state.errors.push(Error {
			kind: ErrorKind::EEnumExpected,
			span: keyword.span,
		});
		recover_item(state);
		return None;
	}

	// Optional identifier
	let mut id = None;
	let ident = state.peek_or_error(ErrorKind::EEnumIdent)?;
	if ident.kind == TokenKind::Ident {
		state.next();
		id = Some(ast::Ident {
			span: ident.span,
			name: state.lex.read_str(ident.span),
		});
	}

	// Punct `:`
	let punct = state.next_or_error(ErrorKind::EEnumReprExpected)?;
	if punct.kind != TokenKind::Punct(Punct::Colon) {
		state.errors.push(Error {
			kind: ErrorKind::EEnumReprExpected,
			span: punct.span,
		});
		recover_item(state);
		return None;
	}

	// Repr identifier
	let repr = parse_enum_repr(state)?;

	// Enum values `{ ... }`
	let members = parse_enum_members(state)?;

	let span = keyword.span.combine(&members.span);
	Some(ast::EnumItem { attrs, id, repr, members, span })
}

pub fn parse_enum_repr(state: &mut Parser) -> Option<EnumReprType> {
	// Identifier
	let ident = state.next_or_error(ErrorKind::EEnumReprExpected)?;
	if ident.kind != TokenKind::Ident {
		state.errors.push(Error {
			kind: ErrorKind::EEnumReprExpected,
			span: ident.span,
		});
		recover_comma(state);
		return None;
	}

	// Must be one of the valid repr types
	let text = state.lex.read_str(ident.span);
	let repr = match text {
		"i8" => EnumReprType::I8,
		"i16" => EnumReprType::I16,
		"i32" => EnumReprType::I32,
		"i64" => EnumReprType::I64,
		"u8" => EnumReprType::U8,
		"u16" => EnumReprType::U16,
		"u32" => EnumReprType::U32,
		"u64" => EnumReprType::U64,
		_ => {
			state.errors.push(Error {
				kind: ErrorKind::EEnumReprInvalid,
				span: ident.span,
			});
			return None;
		}
	};

	Some(repr)
}

pub fn parse_enum_members<'a>(state: &mut Parser<'a>) -> Option<EnumMembers<'a>> {
	// `{` punct
	let open = state.next_or_error(ErrorKind::EEnumLBrace)?;
	if open.kind != TokenKind::Punct(Punct::LBrace) {
		state.errors.push(Error {
			kind: ErrorKind::EEnumLBrace,
			span: open.span,
		});
		recover_item(state);
		return None;
	}

	let mut members = Vec::new();
	loop {
		let token = state.peek_or_error(ErrorKind::EEnumMemberIdent)?;

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
				if let Some(value) = parse_enum_member(state) {
					members.push(value);
				}
			}
		}
	}

	// `}` punct
	let close = state.next_or_error(ErrorKind::EEnumRBrace)?;
	if close.kind != TokenKind::Punct(Punct::RBrace) {
		state.errors.push(Error {
			kind: ErrorKind::EEnumRBrace,
			span: close.span,
		});
		recover_comma(state);
		return None;
	}

	let span = open.span.combine(&close.span);
	Some(ast::EnumMembers { items: members, span })
}

pub fn parse_enum_member<'a>(state: &mut Parser<'a>) -> Option<ast::EnumMember<'a>> {
	let attrs = parse_attrs(state);

	// Identifier
	let ident = state.next_or_error(ErrorKind::EEnumMemberIdent)?;
	if ident.kind != TokenKind::Ident {
		state.errors.push(Error {
			kind: ErrorKind::EEnumMemberIdent,
			span: ident.span,
		});
		recover_comma(state);
		return None;
	}
	let id = ast::Ident {
		span: ident.span,
		name: state.lex.read_str(ident.span),
	};

	let mut value = None;

	if let Some(punct) = state.peek() {
		if punct.kind == TokenKind::Punct(Punct::Eq) {
			state.next();
			value = parse_enum_member_value(state);
		}
	}

	if let Some(comma) = state.peek() {
		match comma.kind {
			TokenKind::Punct(Punct::Comma) => {
				state.next();
			}
			TokenKind::Punct(Punct::RBrace) => {
				// Don't consume the `}` as it will be handled by the caller.
			}
			_ => {
				state.errors.push(Error {
					kind: ErrorKind::EEnumMemberComma,
					span: comma.span,
				});
				recover_comma(state);
				return None;
			}
		}
	}

	let span = if let Some(value) = &value {
		ident.span.combine(&value.span)
	}
	else {
		ident.span
	};

	Some(ast::EnumMember { attrs, id, value, span })
}

pub fn parse_enum_member_value<'a>(state: &mut Parser<'a>) -> Option<ast::EnumMemberValue<'a>> {
	let token = state.peek_or_error(ErrorKind::EEnumMemberValueExpected)?;

	if !matches!(token.kind, TokenKind::Literal(Literal::Integer)) {
		state.errors.push(Error {
			kind: ErrorKind::EEnumMemberValueExpected,
			span: token.span,
		});
		recover_comma(state);
		return None;
	}

	state.next();

	Some(ast::EnumMemberValue {
		span: token.span,
		value: state.lex.read_str(token.span),
	})
}
