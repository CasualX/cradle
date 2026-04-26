use super::*;

pub fn parse_error<'a>(state: &mut Parser<'a>) -> Option<ast::ErrorItem<'a>> {
	// Attributes
	let attrs = parse_attrs(state);

	// Keyword `error`
	let keyword = state.next_or_error(ErrorKind::EErrorExpected)?;
	if state.lex.read_str(keyword.span) != "error" {
		state.errors.push(Error {
			kind: ErrorKind::EErrorExpected,
			span: keyword.span,
		});
		recover_item(state);
		return None;
	}

	// Identifier
	let mut id = None;
	let ident = state.peek_or_error(ErrorKind::EErrorIdent)?;
	if ident.kind == TokenKind::Ident {
		state.next();
		id = Some(ast::Ident {
			span: ident.span,
			name: state.lex.read_str(ident.span),
		});
	}

	// Error values `{ ... }`
	let members = parse_error_members(state)?;

	let span = keyword.span.combine(&members.span);
	Some(ast::ErrorItem { attrs, id, variants: members, span })
}

pub fn parse_error_members<'a>(state: &mut Parser<'a>) -> Option<ErrorVariants<'a>> {
	// `{` punct
	let open = state.next_or_error(ErrorKind::EErrorLBrace)?;
	if open.kind != TokenKind::Punct(Punct::LBrace) {
		state.errors.push(Error {
			kind: ErrorKind::EErrorLBrace,
			span: open.span,
		});
		recover_item(state);
		return None;
	}

	let mut members = Vec::new();
	loop {
		let token = state.peek_or_error(ErrorKind::EErrorVariantIdent)?;

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
				if let Some(value) = parse_error_member(state) {
					members.push(value);
				}
			}
		}
	}

	// `}` punct
	let close = state.next_or_error(ErrorKind::EErrorRBrace)?;
	if close.kind != TokenKind::Punct(Punct::RBrace) {
		state.errors.push(Error {
			kind: ErrorKind::EErrorRBrace,
			span: close.span,
		});
		recover_comma(state);
		return None;
	}

	let span = open.span.combine(&close.span);
	Some(ast::ErrorVariants { items: members, span })
}

pub fn parse_error_member<'a>(state: &mut Parser<'a>) -> Option<ast::ErrorVariant<'a>> {
	let attrs = parse_attrs(state);

	// Identifier
	let ident = state.next_or_error(ErrorKind::EErrorVariantIdent)?;
	if ident.kind != TokenKind::Ident {
		state.errors.push(Error {
			kind: ErrorKind::EErrorVariantIdent,
			span: ident.span,
		});
		recover_comma(state);
		return None;
	}
	let id = ast::Ident {
		span: ident.span,
		name: state.lex.read_str(ident.span),
	};

	if let Some(punct) = state.peek() {
		match punct.kind {
			TokenKind::Punct(Punct::Comma) => {
				state.next();
			}
			TokenKind::Punct(Punct::RBrace) => {
				// Don't consume the `}` as it will be handled by the caller.
			}
			_ => {
				state.errors.push(Error {
					kind: ErrorKind::EErrorVariantComma,
					span: punct.span,
				});
				recover_comma(state);
				return None;
			}
		}
	}

	let span = ident.span;
	Some(ast::ErrorVariant { attrs, id, span })
}
