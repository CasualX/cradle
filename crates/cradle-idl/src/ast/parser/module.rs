use super::*;

pub fn parse_module<'a>(state: &mut Parser<'a>) -> Option<ast::ModuleItem<'a>> {
	// Attributes
	let attrs = parse_attrs(state);

	// `module` keyword
	let keyword = state.next_or_error(ErrorKind::EModuleExpected)?;
	if state.lex.read_str(keyword.span) != "module" {
		state.errors.push(Error {
			kind: ErrorKind::EModuleExpected,
			span: keyword.span,
		});
		recover_item(state);
		return None;
	}

	// Identifier
	let ident = state.next_or_error(ErrorKind::EModuleIdent)?;
	if ident.kind != TokenKind::Ident {
		state.errors.push(Error {
			kind: ErrorKind::EModuleIdent,
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
	let punct = state.next_or_error(ErrorKind::EModuleSemicolon)?;
	if punct.kind != TokenKind::Punct(Punct::Semicolon) {
		state.errors.push(Error {
			kind: ErrorKind::EModuleSemicolon,
			span: punct.span,
		});
		recover_item(state);
		return None;
	}

	let span = keyword.span.combine(&punct.span);
	Some(ast::ModuleItem { attrs, id, span })
}
