use super::*;

fn skip_grouping(state: &mut Parser) -> Option<()> {
	let left = state.peek()?;
	let right = match left.kind {
		TokenKind::Punct(Punct::LParen) => Punct::RParen,
		TokenKind::Punct(Punct::LBracket) => Punct::RBracket,
		TokenKind::Punct(Punct::LBrace) => Punct::RBrace,
		_ => return None,
	};
	state.next();
	while let Some(token) = state.peek() {
		if let TokenKind::Punct(punct) = token.kind {
			if punct == right {
				state.next();
				break;
			}
			match token.kind {
				TokenKind::Punct(Punct::LParen) |
				TokenKind::Punct(Punct::LBracket) |
				TokenKind::Punct(Punct::LBrace) => {
					skip_grouping(state);
				}
				TokenKind::Punct(Punct::RParen) |
				TokenKind::Punct(Punct::RBracket) |
				TokenKind::Punct(Punct::RBrace) => {
					state.errors.push(Error {
						kind: ErrorKind::EUnmatchedBraces,
						span: left.span,
					});
					return None;
				}
				_ => {
					state.next();
				}
			}
		}
		else {
			state.next();
		}
	}
	Some(())
}

/// Recover until the next semicolon, or the start of a new item.
pub fn recover_item(state: &mut Parser) {
	while let Some(token) = state.peek() {
		match token.kind {
			TokenKind::Punct(Punct::Semicolon) => {
				state.next();
				return;
			}
			TokenKind::Punct(Punct::LParen) |
			TokenKind::Punct(Punct::LBracket) |
			TokenKind::Punct(Punct::LBrace) => {
				skip_grouping(state);
			}
			TokenKind::Ident => {
				let text = state.lex.read_str(token.span);
				if matches!(text, "module" | "enum" | "error" | "struct" | "in" | "out" | "fn" | "handle") {
					return;
				}
			}
			_ => {}
		}
		state.next();
	}
}

/// Recover until after the next comma.
///
/// Intended to recover from errors in lists of items, such as struct fields or enum members.
///
/// Does not consume the closing delimiter, as it will be handled by the caller.
pub fn recover_comma(state: &mut Parser) {
	while let Some(token) = state.peek() {
		match token.kind {
			TokenKind::Punct(Punct::Comma) => {
				state.next();
				break;
			}
			TokenKind::Punct(Punct::LParen) |
			TokenKind::Punct(Punct::LBracket) |
			TokenKind::Punct(Punct::LBrace) => {
				skip_grouping(state);
			}
			TokenKind::Punct(Punct::RParen) |
			TokenKind::Punct(Punct::RBracket) |
			TokenKind::Punct(Punct::RBrace) => {
				// Don't consume the closing delimiter as it will be handled by the caller.
				break;
			}
			_ => {
				state.next();
			}
		}
	}
}

// Skip all tokens until the expected closing delimiter, handling nested groupings.
pub fn recover_grouping(state: &mut Parser, expected_closing: Punct) {
	while let Some(token) = state.peek() {
		match token.kind {
			TokenKind::Punct(punct) => {
				if punct == expected_closing {
					state.next();
					break;
				}
				match token.kind {
					TokenKind::Punct(Punct::LParen) |
					TokenKind::Punct(Punct::LBracket) |
					TokenKind::Punct(Punct::LBrace) => {
						skip_grouping(state);
					}
					TokenKind::Punct(Punct::RParen) |
					TokenKind::Punct(Punct::RBracket) |
					TokenKind::Punct(Punct::RBrace) => {
						state.errors.push(Error {
							kind: ErrorKind::EUnmatchedBraces,
							span: token.span,
						});
						return;
					}
					_ => {
						state.next();
					}
				}
			}
			_ => {
				state.next();
			}
		}
	}
}
