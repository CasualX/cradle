use super::*;

// i32
// f32[] f32[,] f32[,,] ...
// char[255]
// string
// (Module.)?TypeName
// (in|out)? struct Foo { ... }
// enum Foo { ... }
// error FooError { ... }

pub fn parse_type<'a>(state: &mut Parser<'a>) -> Option<TypeDecl<'a>> {
	let token1 = state.peek_or_error(ErrorKind::ETypeExpected)?;
	if token1.kind != TokenKind::Ident {
		state.errors.push(Error {
			kind: ErrorKind::ETypeExpected,
			span: token1.span,
		});
		recover_comma(state);
		return None;
	}

	let text1 = state.lex.read_str(token1.span);
	let kind = match text1 {
		"in" | "out" | "struct" => TypeDeclKind::Struct(parse_struct(state)?),
		"enum" => TypeDeclKind::Enum(parse_enum(state)?),
		"error" => TypeDeclKind::Error(parse_error(state)?),
		"string" => { state.next(); TypeDeclKind::String }
		"i8" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::I8) }
		"i16" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::I16) }
		"i32" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::I32) }
		"i64" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::I64) }
		"isize" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::Isize) }
		"u8" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::U8) }
		"u16" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::U16) }
		"u32" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::U32) }
		"u64" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::U64) }
		"usize" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::Usize) }
		"f32" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::F32) }
		"f64" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::F64) }
		"bool" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::Bool) }
		"char" => { state.next(); TypeDeclKind::Primitive(PrimitiveType::Char) }
		_ => TypeDeclKind::TypeName(parse_type_name(state)?),
	};

	let modifier = parse_modifier(state);

	Some(TypeDecl { kind, modifier, span: token1.span })
}

pub fn parse_type_name<'a>(state: &mut Parser<'a>) -> Option<TypeName<'a>> {
	let token1 = state.next_or_error(ErrorKind::ETypeNameExpected)?;
	if token1.kind != TokenKind::Ident {
		state.errors.push(Error {
			kind: ErrorKind::ETypeNameExpected,
			span: token1.span,
		});
		recover_comma(state);
		return None;
	}

	if let Some(dot) = state.peek() {
		if dot.kind == TokenKind::Punct(Punct::Dot) {
			state.next();

			let token2 = state.next_or_error(ErrorKind::ETypeNameExpected)?;
			if token2.kind != TokenKind::Ident {
				state.errors.push(Error {
					kind: ErrorKind::ETypeNameExpected,
					span: token2.span,
				});
				recover_comma(state);
				return None;
			}

			let module = ast::Ident {
				span: token1.span,
				name: state.lex.read_str(token1.span),
			};
			let name = ast::Ident {
				span: token2.span,
				name: state.lex.read_str(token2.span),
			};
			let span = token1.span.combine(&token2.span);
			return Some(TypeName { module: Some(module), name, span });
		}
	}

	let name = ast::Ident {
		span: token1.span,
		name: state.lex.read_str(token1.span),
	};
	Some(TypeName { module: None, name, span: token1.span })
}

// [N] or [,] or [,,] ...
pub fn parse_modifier(state: &mut Parser) -> Option<TypeModifier> {
	let open = state.peek()?;
	if open.kind != TokenKind::Punct(Punct::LBracket) {
		return None; // No modifier, which is fine
	}
	state.next();

	let token = state.peek_or_error(ErrorKind::ETypeModifierExpected)?;

	if matches!(token.kind, TokenKind::Literal(_)) {
		state.next();
		if token.kind != TokenKind::Literal(Literal::Integer) {
			state.errors.push(Error {
				kind: ErrorKind::ETypeModifierIntegerExpected,
				span: token.span,
			});
			recover_grouping(state, Punct::RBracket);
			return None;
		}
		let value = state.lex.read_str(token.span);
		let ndims = match value.parse::<usize>() {
			Ok(n) => n,
			Err(_) => {
				state.errors.push(Error {
					kind: ErrorKind::ETypeModifierIntegerExpected,
					span: token.span,
				});
				recover_grouping(state, Punct::RBracket);
				return None;
			}
		};
		if ndims == 0 || ndims >= 0x10000 {
			state.errors.push(Error {
				kind: ErrorKind::ETypeModifierOutOfRange,
				span: token.span,
			});
			recover_grouping(state, Punct::RBracket);
			return None;
		};
		let ndims = ndims as u16;
		return Some(TypeModifier::ArrayLen(ndims));
	}


	// Count number of commas
	let mut ndims = 1;
	loop {
		let token = state.peek_or_error(ErrorKind::ETypeModifierExpected)?;
		match token.kind {
			TokenKind::Punct(Punct::Comma) => {
				state.next();
				ndims += 1;
			}
			TokenKind::Punct(Punct::RBracket) => break,
			_ => {
				state.errors.push(Error {
					kind: ErrorKind::ETypeModifierExpected,
					span: token.span,
				});
				recover_grouping(state, Punct::RBracket);
				return None;
			}
		}
	}

	let close = state.next_or_error(ErrorKind::ETypeModifierExpected)?;
	if close.kind != TokenKind::Punct(Punct::RBracket) {
		state.errors.push(Error {
			kind: ErrorKind::ETypeModifierExpected,
			span: close.span,
		});
		recover_grouping(state, Punct::RBracket);
		return None;
	}

	match ndims {
		1 => Some(TypeModifier::Array1D),
		2 => Some(TypeModifier::Array2D),
		3 => Some(TypeModifier::Array3D),
		_ => {
			state.errors.push(Error {
				kind: ErrorKind::ETypeModifierTooManyDimensions,
				span: open.span,
			});
			recover_grouping(state, Punct::RBracket);
			None
		}
	}
}
