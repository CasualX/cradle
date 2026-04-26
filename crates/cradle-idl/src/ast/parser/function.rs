use super::*;

// fn Foo(a: i32, b: string) -> i32, error { ... };
// fn Foo(a: i32, b: string) -> i32, NoReturnError;
// fn Foo(a: i32, b: string) -> i32;
// fn NoReturn(a: i32, b: string) error { ... };
// fn NoReturn(a: i32, b: string) NoReturnError;
// fn NoReturnNoError(a: i32, b: string);

pub fn parse_fn<'a>(state: &mut Parser<'a>) -> Option<FnItem<'a>> {
	// Attributes
	let attrs = parse_attrs(state);

	// keyword `fn`
	let keyword = state.next_or_error(ErrorKind::EFnExpected)?;
	if state.lex.read_str(keyword.span) != "fn" {
		state.errors.push(Error {
			kind: ErrorKind::EFnExpected,
			span: keyword.span,
		});
		recover_item(state);
		return None;
	}

	let ident = state.next_or_error(ErrorKind::EFnIdent)?;
	if ident.kind != TokenKind::Ident {
		state.errors.push(Error {
			kind: ErrorKind::EFnIdent,
			span: ident.span,
		});
		recover_item(state);
		return None;
	}
	let id = ast::Ident {
		span: ident.span,
		name: state.lex.read_str(ident.span),
	};

	let params = parse_fn_params(state)?;

	// Optional -> return type
	let mut return_ty = None;
	let mut error_ty = None;

	let arrow_token = state.peek_or_error(ErrorKind::EOF)?;
	if arrow_token.kind == TokenKind::Punct(Punct::Arrow) {
		state.next();
		return_ty = parse_type(state);

		// Optional error type
		let token2 = state.peek_or_error(ErrorKind::EOF)?;
		match token2.kind {
			TokenKind::Punct(Punct::Comma) => {
				state.next();
				error_ty = parse_type(state);
			}
			_ => {}
		}
	}
	else if arrow_token.kind != TokenKind::Punct(Punct::Semicolon) {
		error_ty = parse_type(state);
	}

	// semicolon `;`
	let semicolon = state.next_or_error(ErrorKind::EOF)?;
	if semicolon.kind != TokenKind::Punct(Punct::Semicolon) {
		state.errors.push(Error {
			kind: ErrorKind::EFnSemicolonExpected,
			span: semicolon.span,
		});
		recover_item(state);
		return None;
	}

	let span = keyword.span.combine(&semicolon.span);
	Some(ast::FnItem { attrs, id, params, return_ty, error_ty, span })
}

fn parse_fn_params<'a>(state: &mut Parser<'a>) -> Option<FnParams<'a>> {
	// `(` punct
	let open = state.next_or_error(ErrorKind::EFnLParen)?;
	if open.kind != TokenKind::Punct(Punct::LParen) {
		state.errors.push(Error {
			kind: ErrorKind::EFnLParen,
			span: open.span,
		});
		recover_item(state);
		return None;
	}

	let mut items = Vec::new();
	loop {
		let token = state.peek_or_error(ErrorKind::EOF)?;

		match token.kind {
			TokenKind::Punct(Punct::RParen) => break,
			TokenKind::Punct(Punct::RBrace) |
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
			TokenKind::Punct(Punct::RParen) => break,
			_ => {
				state.errors.push(Error {
					kind: ErrorKind::EStructFieldExpected,
					span: comma.span,
				});
				recover_comma(state);
			}
		}
	}

	// `)` punct
	let close = state.next_or_error(ErrorKind::EFnRParen)?;
	if close.kind != TokenKind::Punct(Punct::RParen) {
		state.errors.push(Error {
			kind: ErrorKind::EFnRParen,
			span: close.span,
		});
		recover_comma(state);
		return None;
	}

	let span = open.span.combine(&close.span);
	Some(ast::FnParams { items, span })
}
