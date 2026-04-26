use super::*;

fn doc<'a>(pool: &'a StringPool, attrs: &[ast::Attribute<'a>]) -> Option<&'a str> {
	let mut text = String::new();
	for attr in attrs {
		if let ast::AttrKind::Doc(doc) = &attr.kind {
			text.push_str(doc.comment);
			text.push('\n');
		}
	}
	if text.is_empty() {
		None
	}
	else {
		Some(pool.store(text))
	}
}

fn link_name<'a>(_pool: &'a StringPool, attrs: &[ast::Attribute<'a>]) -> Option<&'a str> {
	for attr in attrs {
		if let ast::AttrKind::Kv(kv) = &attr.kind {
			if kv.key.name == "link_name" {
				return Some(kv.value.text);
			}
		}
	}
	None
}

pub fn ast2ir<'a>(pool: &'a StringPool, idl: &[ast::IdlFile<'a>], errors: &mut Vec<Error>) -> ir::Library<'a> {
	let mut modules = BTreeMap::new();

	for file in idl {
		let Some(module) = file.get_module_item() else {
			continue;
		};
		let module_name = module.id.name;
		let ir_module = modules.entry(module_name).or_insert_with(|| ir::Module {
			doc: None,
			name: module_name,
			items: Vec::new(),
		});

		for ast_item in &file.items {
			match ast_item {
				ast::Item::Module(_) => (), // We've already processed the module declaration
				ast::Item::Enum(ast_item) => enum_item(pool, ir_module, ast_item, errors),
				ast::Item::Error(ast_item) => error_item(pool, ir_module, ast_item, errors),
				ast::Item::Struct(ast_item) => struct_item(pool, ir_module, ast_item, errors),
				ast::Item::Function(ast_item) => fn_item(pool, ir_module, ast_item, errors),
				ast::Item::Handle(ast_item) => handle_item(pool, ir_module, ast_item, errors),
			}
		}
	}

	let modules = modules.into_values().collect();
	ir::Library { modules }
}

fn enum_item<'a>(pool: &'a StringPool, ir_module: &mut ir::Module<'a>, ast_item: &ast::EnumItem<'a>, errors: &mut Vec<Error>) {
	let doc = doc(pool, &ast_item.attrs);
	let name = ast_item.id.as_ref().unwrap().name;
	let (repr, mut value) = match ast_item.repr {
		ast::EnumReprType::U8 => (ir::EnumReprType::U8, ir::EnumReprValue::U8(0)),
		ast::EnumReprType::U16 => (ir::EnumReprType::U16, ir::EnumReprValue::U16(0)),
		ast::EnumReprType::U32 => (ir::EnumReprType::U32, ir::EnumReprValue::U32(0)),
		ast::EnumReprType::U64 => (ir::EnumReprType::U64, ir::EnumReprValue::U64(0)),
		ast::EnumReprType::I8 => (ir::EnumReprType::I8, ir::EnumReprValue::I8(0)),
		ast::EnumReprType::I16 => (ir::EnumReprType::I16, ir::EnumReprValue::I16(0)),
		ast::EnumReprType::I32 => (ir::EnumReprType::I32, ir::EnumReprValue::I32(0)),
		ast::EnumReprType::I64 => (ir::EnumReprType::I64, ir::EnumReprValue::I64(0)),
	};
	let members = ast_item.members.items.iter().map(|ast_member| enum_member(pool, ast_member, repr, &mut value, errors)).collect();
	let item = ir::EnumItem { doc, name, repr, members };
	ir_module.items.push(ir::Item::Enum(item));
}

fn enum_member<'a>(pool: &'a StringPool, ast_member: &ast::EnumMember<'a>, repr: ir::EnumReprType, value: &mut ir::EnumReprValue, errors: &mut Vec<Error>) -> ir::EnumMember<'a> {
	let doc = doc(pool, &ast_member.attrs);
	let name = ast_member.id.name;
	if let Some(ast_value) = &ast_member.value {
		*value = parse_enum_value(pool, ast_value, repr, errors);
	}
	ir::EnumMember { doc, name, value: *value }
}

fn parse_enum_value<'a>(_pool: &'a StringPool, ast_value: &ast::EnumMemberValue<'a>, repr: ir::EnumReprType, errors: &mut Vec<Error>) -> ir::EnumReprValue {
	fn or_else<T: Default>(errors: &mut Vec<Error>, ast_value: &ast::EnumMemberValue) -> impl FnOnce(num::ParseIntError) -> T {
		move |_| {
			errors.push(Error { kind: ErrorKind::EEnumMemberValueInvalid, span: ast_value.span });
			T::default()
		}
	}
	let text: &str = ast_value.value;
	match repr {
		ir::EnumReprType::U8 => ir::EnumReprValue::U8(text.parse().unwrap_or_else(or_else(errors, ast_value))),
		ir::EnumReprType::U16 => ir::EnumReprValue::U16(text.parse().unwrap_or_else(or_else(errors, ast_value))),
		ir::EnumReprType::U32 => ir::EnumReprValue::U32(text.parse().unwrap_or_else(or_else(errors, ast_value))),
		ir::EnumReprType::U64 => ir::EnumReprValue::U64(text.parse().unwrap_or_else(or_else(errors, ast_value))),
		ir::EnumReprType::I8 => ir::EnumReprValue::I8(text.parse().unwrap_or_else(or_else(errors, ast_value))),
		ir::EnumReprType::I16 => ir::EnumReprValue::I16(text.parse().unwrap_or_else(or_else(errors, ast_value))),
		ir::EnumReprType::I32 => ir::EnumReprValue::I32(text.parse().unwrap_or_else(or_else(errors, ast_value))),
		ir::EnumReprType::I64 => ir::EnumReprValue::I64(text.parse().unwrap_or_else(or_else(errors, ast_value))),
	}
}

fn error_item<'a>(pool: &'a StringPool, ir_module: &mut ir::Module<'a>, ast_item: &ast::ErrorItem<'a>, _errors: &mut Vec<Error>) {
	let doc = doc(pool, &ast_item.attrs);
	let name = ast_item.id.as_ref().unwrap().name;
	let variants = ast_item.variants.items.iter().map(|ast_variant| error_variant(pool, ast_variant)).collect();
	let item = ir::ErrorItem { doc, name, variants };
	ir_module.items.push(ir::Item::Error(item));
}

fn error_variant<'a>(pool: &'a StringPool, ast_variant: &ast::ErrorVariant<'a>) -> ir::ErrorVariant<'a> {
	let doc = doc(pool, &ast_variant.attrs);
	let name = ast_variant.id.name;
	ir::ErrorVariant { doc, name }
}

fn type_name<'a>(ast_type_name: &ast::TypeName<'a>) -> ir::TypeName<'a> {
	let module = ast_type_name.module.as_ref().expect("TypeName should have been resolved to a TypeName with module and name").name;
	let name = ast_type_name.name.name;
	ir::TypeName { module, name }
}

fn primitive_type(ast_prim: ast::PrimitiveType) -> ir::PrimitiveType {
	match ast_prim {
		ast::PrimitiveType::I8 => ir::PrimitiveType::I8,
		ast::PrimitiveType::I16 => ir::PrimitiveType::I16,
		ast::PrimitiveType::I32 => ir::PrimitiveType::I32,
		ast::PrimitiveType::I64 => ir::PrimitiveType::I64,
		ast::PrimitiveType::Isize => ir::PrimitiveType::Isize,
		ast::PrimitiveType::U8 => ir::PrimitiveType::U8,
		ast::PrimitiveType::U16 => ir::PrimitiveType::U16,
		ast::PrimitiveType::U32 => ir::PrimitiveType::U32,
		ast::PrimitiveType::U64 => ir::PrimitiveType::U64,
		ast::PrimitiveType::Usize => ir::PrimitiveType::Usize,
		ast::PrimitiveType::F32 => ir::PrimitiveType::F32,
		ast::PrimitiveType::F64 => ir::PrimitiveType::F64,
		ast::PrimitiveType::Bool => ir::PrimitiveType::Bool,
		ast::PrimitiveType::Char => ir::PrimitiveType::Char,
	}
}

fn parse_type<'a>(pool: &'a StringPool, ir_module: &mut ir::Module<'a>, ast_ty: &ast::TypeDecl<'a>, errors: &mut Vec<Error>) -> ir::Type<'a> {
	// Handle inline type declarations
	let kind = match &ast_ty.kind {
		ast::TypeDeclKind::Enum(ast_enum) => {
			enum_item(pool, ir_module, ast_enum, errors);
			let name = ast_enum.id.clone().unwrap();
			let span = name.span;
			let module = Some(ast::Ident { name: ir_module.name, span });
			ast::TypeDeclKind::TypeName(ast::TypeName { module, name, span })
		}
		ast::TypeDeclKind::Error(ast_error) => {
			error_item(pool, ir_module, ast_error, errors);
			let name = ast_error.id.clone().unwrap();
			let span = name.span;
			let module = Some(ast::Ident { name: ir_module.name, span });
			ast::TypeDeclKind::TypeName(ast::TypeName { module, name, span })
		}
		ast::TypeDeclKind::Struct(ast_struct) => {
			struct_item(pool, ir_module, ast_struct, errors);
			let name = ast_struct.id.clone().unwrap();
			let span = name.span;
			let module = Some(ast::Ident { name: ir_module.name, span });
			ast::TypeDeclKind::TypeName(ast::TypeName { module, name, span })
		}
		kind => kind.clone(),
	};
	let element_ty = match kind {
		ast::TypeDeclKind::Primitive(prim) => ir::ElementType::Primitive(primitive_type(prim)),
		ast::TypeDeclKind::String => ir::ElementType::String,
		ast::TypeDeclKind::TypeName(ast_type_name) => ir::ElementType::TypeName(type_name(&ast_type_name)),
		_ => unreachable!("Only Primitive, String and TypeName should remain after handling inline declarations"),
	};
	match ast_ty.modifier {
		None => ir::Type::Element(element_ty),
		Some(ast::TypeModifier::ArrayLen(len)) => ir::Type::ArrayN(element_ty, len),
		Some(ast::TypeModifier::Array1D) => ir::Type::Array1D(element_ty),
		Some(ast::TypeModifier::Array2D) => ir::Type::Array2D(element_ty),
		Some(ast::TypeModifier::Array3D) => ir::Type::Array3D(element_ty),
		Some(ast::TypeModifier::Array4D) => ir::Type::Array4D(element_ty),
	}
}

fn struct_field<'a>(pool: &'a StringPool, ir_module: &mut ir::Module<'a>, ast_field: &ast::StructField<'a>, errors: &mut Vec<Error>) -> ir::Field<'a> {
	let doc = doc(pool, &ast_field.attrs);
	let name = ast_field.id.name;
	let ty = parse_type(pool, ir_module, &ast_field.ty, errors);
	ir::Field { doc, name, ty }
}

fn struct_item<'a>(pool: &'a StringPool, ir_module: &mut ir::Module<'a>, ast_item: &ast::StructItem<'a>, errors: &mut Vec<Error>) {
	let doc = doc(pool, &ast_item.attrs);
	let name = ast_item.id.as_ref().unwrap().name;
	let direction = ast_item.direction;
	let fields = ast_item.fields.items.iter().map(|ast_field| struct_field(pool, ir_module, ast_field, errors)).collect();
	let item = ir::StructItem { doc, name, direction, fields };
	ir_module.items.push(ir::Item::Struct(item));
}

fn fn_item<'a>(pool: &'a StringPool, ir_module: &mut ir::Module<'a>, ast_item: &ast::FnItem<'a>, _errors: &mut Vec<Error>) {
	let doc = doc(pool, &ast_item.attrs);
	let name = ast_item.id.name;
	let Some(link_name) = link_name(pool, &ast_item.attrs) else {
		_errors.push(Error { kind: ErrorKind::EFnLinkNameRequired, span: ast_item.id.span });
		return;
	};
	let params_name = pool.store(format!("{}In", name));
	let params_doc = None; // We could consider supporting doc comments on parameters in the future, but for now we'll just ignore them
	let params_fields = ast_item.params.items.iter().map(|ast_param| struct_field(pool, ir_module, ast_param, _errors)).collect();
	let param_ty = ir::StructItem { doc: params_doc, name: params_name, direction: Some(ir::StructDirection::In), fields: params_fields };
	let return_ty = ast_item.return_ty.as_ref().map(|ast_return_ty| parse_type(pool, ir_module, ast_return_ty, _errors));
	let error_ty = ast_item.error_ty.as_ref().map(|ast_error_ty| parse_type(pool, ir_module, ast_error_ty, _errors));
	let item = ir::FnItem { doc, name, link_name, params: param_ty, return_ty, error_ty };
	ir_module.items.push(ir::Item::Function(item));
}

fn handle_item<'a>(pool: &'a StringPool, ir_module: &mut ir::Module<'a>, ast_item: &ast::HandleItem<'a>, _errors: &mut Vec<Error>) {
	let doc = doc(pool, &ast_item.attrs);
	let name = ast_item.id.name;
	let item = ir::HandleItem { doc, name };
	ir_module.items.push(ir::Item::Handle(item));
}
