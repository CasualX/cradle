use super::super::*;

fn assert_tokens(input: &str, expected: &[(TokenKind, &str)]) {
	let mut lexer = Lexer::new(input, 0);
	for &(kind, text) in expected {
		let token = lexer.next_token().expect("expected token");
		println!("token: {:?} {:?}", token.kind, &input[token.span.range()]);
		assert_eq!(token.kind, kind);
		let t = &input[token.span.range()];
		assert_eq!(t, text);
	}
	let token = lexer.next_token();
	assert!(token.is_none(), "expected end of input, got {:?}", token.unwrap().kind);
}

#[test]
fn test_smoke() {
	let input = r#"
		module Operator;
		/// Doc comment.
		#name = OPERATOR_Foo
		fn foo(a: i32 = 42) -> i32, error { OVERFLOW };
	"#;
	assert_tokens(input, &[
		(TokenKind::Ident, "module"),
		(TokenKind::Ident, "Operator"),
		(TokenKind::Punct(Punct::Semicolon), ";"),
		(TokenKind::Comment, "/// Doc comment."),
		(TokenKind::Punct(Punct::Hash), "#"),
		(TokenKind::Ident, "name"),
		(TokenKind::Punct(Punct::Eq), "="),
		(TokenKind::Ident, "OPERATOR_Foo"),
		(TokenKind::Ident, "fn"),
		(TokenKind::Ident, "foo"),
		(TokenKind::Punct(Punct::LParen), "("),
		(TokenKind::Ident, "a"),
		(TokenKind::Punct(Punct::Colon), ":"),
		(TokenKind::Ident, "i32"),
		(TokenKind::Punct(Punct::Eq), "="),
		(TokenKind::Literal(Literal::Integer), "42"),
		(TokenKind::Punct(Punct::RParen), ")"),
		(TokenKind::Punct(Punct::Arrow), "->"),
		(TokenKind::Ident, "i32"),
		(TokenKind::Punct(Punct::Comma), ","),
		(TokenKind::Ident, "error"),
		(TokenKind::Punct(Punct::LBrace), "{"),
		(TokenKind::Ident, "OVERFLOW"),
		(TokenKind::Punct(Punct::RBrace), "}"),
		(TokenKind::Punct(Punct::Semicolon), ";"),
	]);
}

#[test]
fn test_attribute_tokens() {
	let input = "#deprecated\n#name = OPERATOR_Foo\n";
	assert_tokens(input, &[
		(TokenKind::Punct(Punct::Hash), "#"),
		(TokenKind::Ident, "deprecated"),
		(TokenKind::Punct(Punct::Hash), "#"),
		(TokenKind::Ident, "name"),
		(TokenKind::Punct(Punct::Eq), "="),
		(TokenKind::Ident, "OPERATOR_Foo"),
	]);
}

#[test]
fn test_comments() {
	let input = r#"
		// Line comment
		/* Multiline
		   block comment */
		/// Doc comment
		/** Multiline
		    doc comment */
	"#;
	assert_tokens(input, &[
		(TokenKind::Comment, "// Line comment"),
		(TokenKind::Comment, "/* Multiline\n		   block comment */"),
		(TokenKind::Comment, "/// Doc comment"),
		(TokenKind::Comment, "/** Multiline\n		    doc comment */"),
	]);
}
