use super::*;

#[track_caller]
fn assert_errors(actual: &[Error], expected: &[ErrorKind]) {
	for (i, (actual, expected)) in actual.iter().zip(expected).enumerate() {
		assert_eq!(&actual.kind, expected, "Error {}: Expected error kind {:?}, got {:?}", i, expected, actual.kind);
	}
	assert_eq!(actual.len(), expected.len(), "Expected {} errors, got {}: {:?}", expected.len(), actual.len(), actual);
}

fn next_text(state: &mut Parser) -> Option<String> {
	let token = state.next()?;
	Some(state.lex.read_str(token.span).to_string())
}

fn doc_text(_state: &Parser, attrs: &[Attribute]) -> Option<String> {
	let mut text = String::new();
	for attr in attrs {
		if let AttrKind::Doc(doc) = &attr.kind {
			text.push_str(doc.comment);
			text.push('\n');
		}
	}
	if text.is_empty() { None } else { Some(text) }
}

fn cmd_name<'a>(_state: &'a Parser, attrs: &'a [Attribute]) -> Option<&'a str> {
	attrs.iter().find_map(|attr| {
		if let AttrKind::Cmd(cmd) = &attr.kind {
			Some(cmd.name.name)
		}
		else {
			None
		}
	})
}

fn kv_attr<'a>(_state: &'a Parser, attrs: &'a [Attribute], key_name: &str) -> Option<&'a str> {
	attrs.iter().find_map(|attr| {
		if let AttrKind::Kv(kv) = &attr.kind {
			if kv.key.name == key_name {
				return Some(kv.value.text);
			}
		}
		None
	})
}

#[test]
fn test_parse_doc_line_comments() {
	let mut state = Parser::new("/// First\n/// Second\nmodule Demo;", 0);
	let attrs = parse_attrs(&mut state);
	assert_eq!(doc_text(&state, &attrs).as_deref(), Some("First\nSecond\n"));
	assert_eq!(next_text(&mut state).as_deref(), Some("module"));
	assert!(state.errors.is_empty());
}

#[test]
fn test_parse_doc_block_comment() {
	let mut state = Parser::new("/**\n * Hello\n */\nhandle File;", 0);
	let attrs = parse_attrs(&mut state);
	assert_eq!(doc_text(&state, &attrs).as_deref(), Some("* Hello\n"));
	assert_eq!(next_text(&mut state).as_deref(), Some("handle"));
	assert!(state.errors.is_empty());
}

#[test]
fn test_parse_command_attribute() {
	let mut state = Parser::new("#deprecated\nhandle File;", 0);
	let attrs = parse_attrs(&mut state);
	assert_eq!(cmd_name(&state, &attrs), Some("deprecated"));
	assert_eq!(next_text(&mut state).as_deref(), Some("handle"));
	assert!(state.errors.is_empty());
}

#[test]
fn test_parse_key_value_attribute() {
	let mut state = Parser::new("#name = OPERATOR_Foo\nfn Demo();", 0);
	let attrs = parse_attrs(&mut state);
	assert_eq!(kv_attr(&state, &attrs, "name"), Some("OPERATOR_Foo"));
	assert_eq!(next_text(&mut state).as_deref(), Some("fn"));
	assert!(state.errors.is_empty());
}

#[test]
fn test_module_item() {
	let mut state = Parser::new("/// API module.\nmodule MyModule;", 0);
	let item = parse_module(&mut state).unwrap();
	assert_eq!(doc_text(&state, &item.attrs).as_deref(), Some("API module.\n"));
	assert_eq!(item.id.name, "MyModule");
	assert!(state.errors.is_empty());
}

#[test]
fn test_handle_item() {
	let mut state = Parser::new("handle File;", 0);
	let item = parse_handle(&mut state).unwrap();
	assert_eq!(item.id.name, "File");
	assert!(state.errors.is_empty());
}

#[test]
fn test_type_name() {
	let mut state = Parser::new("Foo", 0);
	let type_name = parse_type_name(&mut state).unwrap();
	assert_eq!(type_name.name.name, "Foo");
	assert!(type_name.module.is_none());
	assert!(state.errors.is_empty());
}

#[test]
fn test_type_name_qualified() {
	let mut state = Parser::new("Module.Foo", 0);
	let type_name = parse_type_name(&mut state).unwrap();
	assert_eq!(type_name.module.as_ref().unwrap().name, "Module");
	assert_eq!(type_name.name.name, "Foo");
	assert!(state.errors.is_empty());
}

#[test]
fn test_type_primitive() {
	let mut state = Parser::new("i32", 0);
	let ty = parse_type(&mut state).unwrap();
	assert!(matches!(ty.kind, TypeDeclKind::Primitive(PrimitiveType::I32)));
	assert_eq!(ty.modifier, None);
	assert!(state.errors.is_empty());
}

#[test]
fn test_type_qualified_name_with_modifier() {
	let mut state = Parser::new("Storage.File[,]", 0);
	let ty = parse_type(&mut state).unwrap();
	assert!(matches!(ty.kind, TypeDeclKind::TypeName(_)));
	assert_eq!(ty.modifier, Some(TypeModifier::Array2D));
	assert!(state.errors.is_empty());
}

#[test]
fn test_type_inline_struct_outer_bits() {
	let mut state = Parser::new("out struct Reply { value: u32 }", 0);
	let ty = parse_type(&mut state).unwrap();
	match ty.kind {
		TypeDeclKind::Struct(item) => {
			assert_eq!(item.direction, Some(StructDirection::Out));
			assert_eq!(item.id.as_ref().map(|id| id.name), Some("Reply"));
			assert_eq!(item.fields.items.len(), 1);
		}
		other => panic!("expected struct type, got {:?}", other),
	}
	assert!(state.errors.is_empty());
}

#[test]
fn test_modifier_array_shapes() {
	let mut state = Parser::new("[] [,] [,,]", 0);
	assert_eq!(parse_modifier(&mut state), Some(TypeModifier::Array1D));
	assert_eq!(parse_modifier(&mut state), Some(TypeModifier::Array2D));
	assert_eq!(parse_modifier(&mut state), Some(TypeModifier::Array3D));
	assert!(state.errors.is_empty());
}

#[test]
fn test_modifier_fixed_length() {
	let mut state = Parser::new("[8]", 0);
	assert_eq!(parse_modifier(&mut state), Some(TypeModifier::ArrayLen(8)));
	assert!(state.errors.is_empty());
}

#[test]
fn test_modifier_too_many_dimensions() {
	let mut state = Parser::new("[,,,]", 0);
	assert_eq!(parse_modifier(&mut state), None);
	assert_errors(&state.errors, &[ErrorKind::ETypeModifierTooManyDimensions]);
}

#[test]
fn test_enum_item() {
	let mut state = Parser::new("/// Modes.\nenum MyEnum : i32 { A, B = 5, C }", 0);
	let item = parse_enum(&mut state).unwrap();
	assert_eq!(doc_text(&state, &item.attrs).as_deref(), Some("Modes.\n"));
	assert_eq!(item.id.as_ref().unwrap().name, "MyEnum");
	assert_eq!(item.repr, EnumReprType::I32);
	assert_eq!(item.members.items.len(), 3);
	assert!(state.errors.is_empty());
}

#[test]
fn test_enum_members() {
	let mut state = Parser::new("{ A, B, C }", 0);
	let members = parse_enum_members(&mut state).unwrap();
	assert_eq!(members.items.len(), 3);
	assert!(state.errors.is_empty());
}

#[test]
fn test_enum_member() {
	let mut state = Parser::new("/// Alpha\nA,", 0);
	let member = parse_enum_member(&mut state).unwrap();
	assert_eq!(doc_text(&state, &member.attrs).as_deref(), Some("Alpha\n"));
	assert_eq!(member.id.name, "A");
	assert!(member.value.is_none());
	assert!(state.errors.is_empty());
}

#[test]
fn test_enum_member_with_value() {
	let mut state = Parser::new("A = 1,", 0);
	let member = parse_enum_member(&mut state).unwrap();
	assert_eq!(member.id.name, "A");
	assert_eq!(member.value.as_ref().unwrap().value, "1");
	assert!(state.errors.is_empty());
}

#[test]
fn test_enum_missing_repr() {
	let mut state = Parser::new("enum MyEnum { A, B, C }", 0);
	parse_enum(&mut state);
	assert_errors(&state.errors, &[ErrorKind::EEnumReprExpected]);
}

#[test]
fn test_enum_invalid_repr() {
	let mut state = Parser::new("enum MyEnum : unknown { A, B, C }", 0);
	parse_enum(&mut state);
	assert_errors(&state.errors, &[ErrorKind::EEnumReprInvalid]);
}

#[test]
fn test_error_item() {
	let mut state = Parser::new("error OpenError { NotFound, Denied, }", 0);
	let item = parse_error(&mut state).unwrap();
	assert_eq!(item.id.as_ref().unwrap().name, "OpenError");
	assert_eq!(item.variants.items.len(), 2);
	assert!(state.errors.is_empty());
}

#[test]
fn test_error_members() {
	let mut state = Parser::new("{ First, Second }", 0);
	let members = parse_error_members(&mut state).unwrap();
	assert_eq!(members.items.len(), 2);
	assert!(state.errors.is_empty());
}

#[test]
fn test_error_member() {
	let mut state = Parser::new("/// Overflow\nOverflow", 0);
	let member = parse_error_member(&mut state).unwrap();
	assert_eq!(doc_text(&state, &member.attrs).as_deref(), Some("Overflow\n"));
	assert_eq!(member.id.name, "Overflow");
	assert!(state.errors.is_empty());
}

#[test]
fn test_struct_item_outer_bits() {
	let mut state = Parser::new("in struct Request { name: string, count: u32 }", 0);
	let item = parse_struct(&mut state).unwrap();
	assert_eq!(item.direction, Some(StructDirection::In));
	assert_eq!(item.id.as_ref().unwrap().name, "Request");
	assert_eq!(item.fields.items.len(), 2);
	assert!(state.errors.is_empty());
}

#[test]
fn test_struct_fields() {
	let mut state = Parser::new("{ first: i32, second: string }", 0);
	let fields = parse_struct_fields(&mut state).unwrap();
	assert_eq!(fields.items.len(), 2);
	assert!(state.errors.is_empty());
}

#[test]
fn test_struct_field() {
	let mut state = Parser::new("/// Count\ncount: u32 = 4", 0);
	let field = parse_struct_field(&mut state).unwrap();
	assert_eq!(doc_text(&state, &field.attrs).as_deref(), Some("Count\n"));
	assert_eq!(state.lex.read_str(field.id.span), "count");
	assert!(matches!(field.ty.kind, TypeDeclKind::Primitive(PrimitiveType::U32)));
	assert_eq!(field.default.as_ref().map(|value| state.lex.read_str(value.span)), Some("4"));
	assert!(state.errors.is_empty());
}

#[test]
fn test_default_value_bool_literal() {
	let mut state = Parser::new("false", 0);
	let value = parse_default_value(&mut state).unwrap();
	assert_eq!(state.lex.read_str(value.span), "false");
	assert!(state.errors.is_empty());
}

#[test]
fn test_function_outer_bits() {
	let mut state = Parser::new("/// Reads data.\nfn Read(file: File, offset: u64) -> string, error ReadError { EOF };", 0);
	let item = parse_fn(&mut state).unwrap();
	assert_eq!(doc_text(&state, &item.attrs).as_deref(), Some("Reads data.\n"));
	assert_eq!(state.lex.read_str(item.id.span), "Read");
	assert_eq!(item.params.items.len(), 2);
	assert!(matches!(item.return_ty.as_ref().map(|ty| &ty.kind), Some(TypeDeclKind::String)));
	assert!(matches!(item.error_ty.as_ref().map(|ty| &ty.kind), Some(TypeDeclKind::Error(_))));
	assert!(state.errors.is_empty());
}

#[test]
fn test_function_without_return_type() {
	let mut state = Parser::new("fn Notify(message: string) error { Failed };", 0);
	let item = parse_fn(&mut state).unwrap();
	assert_eq!(state.lex.read_str(item.id.span), "Notify");
	assert_eq!(item.params.items.len(), 1);
	assert!(item.return_ty.is_none());
	assert!(item.error_ty.is_some());
	assert!(state.errors.is_empty());
}

#[test]
fn test_recover_item_stops_at_semicolon() {
	let mut state = Parser::new("oops(1, 2); module Next;", 0);
	recover_item(&mut state);
	assert_eq!(next_text(&mut state).as_deref(), Some("module"));
	assert!(state.errors.is_empty());
}

#[test]
fn test_recover_item_stops_at_next_item_keyword() {
	let mut state = Parser::new("garbage tokens fn Next();", 0);
	recover_item(&mut state);
	assert_eq!(next_text(&mut state).as_deref(), Some("fn"));
	assert!(state.errors.is_empty());
}

#[test]
fn test_recover_comma_stops_after_nested_group() {
	let mut state = Parser::new("broken { nested: [1, 2] }, next", 0);
	recover_comma(&mut state);
	assert_eq!(next_text(&mut state).as_deref(), Some("next"));
	assert!(state.errors.is_empty());
}

#[test]
fn test_recover_comma_stops_before_closing_delimiter() {
	let mut state = Parser::new("broken } trailing", 0);
	recover_comma(&mut state);
	assert_eq!(next_text(&mut state).as_deref(), Some("}"));
	assert!(state.errors.is_empty());
}

#[test]
fn test_recover_grouping_skips_nested_tokens() {
	let mut state = Parser::new("one [two] { three } } next", 0);
	recover_grouping(&mut state, Punct::RBrace);
	assert_eq!(next_text(&mut state).as_deref(), Some("next"));
	assert!(state.errors.is_empty());
}
