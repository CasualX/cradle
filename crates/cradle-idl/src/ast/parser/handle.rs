use super::*;

pub fn parse_handle<'a>(state: &mut Parser<'a>) -> Option<ast::HandleItem<'a>> {
	// Attributes
	let attrs = parse_attrs(state);

	// `handle` keyword
	let keyword = state.next_or_error(ErrorKind::EHandleExpected)?;
	if state.lex.read_str(keyword.span) != "handle" {
		state.errors.push(Error {
			kind: ErrorKind::EHandleExpected,
			span: keyword.span,
		});
		recover_item(state);
		return None;
	}

	// Identifier
	let ident = state.next_or_error(ErrorKind::EHandleIdent)?;
	if ident.kind != TokenKind::Ident {
		state.errors.push(Error {
			kind: ErrorKind::EHandleIdent,
			span: ident.span,
		});
		recover_item(state);
		return None;
	}
	let id = ast::Ident {
		span: ident.span,
		name: state.lex.read_str(ident.span),
	};

	// `;` punct
	let punct = state.next_or_error(ErrorKind::EHandleSemicolon)?;
	if punct.kind != TokenKind::Punct(Punct::Semicolon) {
		state.errors.push(Error {
			kind: ErrorKind::EHandleSemicolon,
			span: punct.span,
		});
		recover_item(state);
		return None;
	}

	let span = keyword.span.combine(&punct.span);
	Some(ast::HandleItem { attrs, id, span })
}
