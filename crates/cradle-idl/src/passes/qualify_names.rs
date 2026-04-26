use super::*;
use std::collections::HashMap;

/// Qualifies names in the IDL file.
///
/// This includes:
///
/// - Idl file starts with and only has a single module declaration
/// - Check top level items have an identifier
/// - Auto names anonymous nested items
/// - Fully qualifies all type names
pub fn qualify_names<'a>(pool: &'a StringPool, idl: &mut ast::IdlFile<'a>, errors: &mut Vec<Error>) {

	// Check the first item is a module declaration
	let Some((first_item, items)) = idl.items.split_first_mut() else {
		return;
	};

	let module_name = if let ast::Item::Module(module) = first_item {
		module.id.clone()
	}
	else {
		errors.push(Error {
			kind: ErrorKind::EModuleFirstItem,
			span: first_item.span(),
		});
		return;
	};

	for item in items {
		qualify_item(pool, item, module_name.clone(), errors);
	}
}

fn qualify_item<'a>(pool: &'a StringPool, item: &mut ast::Item<'a>, module_name: ast::Ident<'a>, errors: &mut Vec<Error>) {
	match item {
		ast::Item::Module(module) => {
			errors.push(Error {
				kind: ErrorKind::EModuleMultiple,
				span: module.span,
			});
		},
		ast::Item::Enum(enum_) => qualify_enum(pool, enum_, module_name, &mut Vec::new(), errors),
		ast::Item::Error(error) => qualify_error(pool, error, module_name, &mut Vec::new(), errors),
		ast::Item::Struct(struct_) => qualify_struct(pool, struct_, module_name, &mut Vec::new(), errors),
		ast::Item::Function(func) => qualify_function(pool, func, module_name, &mut Vec::new(), errors),
		ast::Item::Handle(_) => (), // Handles don't need qualification
	}
}

fn create_name(parent_names: &[&str]) -> String {
	let mut name = String::new();
	for (i, part) in parent_names.iter().enumerate() {
		if i != 0 {
			name.push('_');
		}
		name.push_str(part);
	}
	name
}

fn qualify_enum<'a>(pool: &'a StringPool, enum_: &mut ast::EnumItem<'a>, _module_name: ast::Ident<'a>, parent_names: &mut Vec<&'a str>, errors: &mut Vec<Error>) {
	// If we have no parents then this is a top level item and must have a name
	if parent_names.is_empty() && enum_.id.is_none() {
		errors.push(Error {
			kind: ErrorKind::EEnumTopLevelIdent,
			span: enum_.span,
		});
		return;
	}

	// Create a name for this item if it doesn't have one
	if enum_.id.is_none() {
		let name = pool.store(create_name(parent_names));
		enum_.id = Some(ast::Ident { span: enum_.span, name });
	}
}

const RESERVED_ERROR_MEMBER_NAMES: &[&str] = &[
	"CONTEXT_NULL",
	"INVALID_LICENSE",
	"FFI_UNWIND",
	"UNIMPLEMENTED",
];

fn qualify_error<'a>(pool: &'a StringPool, error: &mut ast::ErrorItem<'a>, _module_name: ast::Ident<'a>, parent_names: &mut Vec<&'a str>, errors: &mut Vec<Error>) {
	// If we have no parents then this is a top level item and must have a name
	if parent_names.is_empty() && error.id.is_none() {
		errors.push(Error {
			kind: ErrorKind::EErrorTopLevelIdent,
			span: error.span,
		});
		return;
	}

	// Create a name for this item if it doesn't have one
	if error.id.is_none() {
		let name = pool.store(create_name(parent_names));
		error.id = Some(ast::Ident { span: error.span, name });
	}

	// Check the error members are unique
	let mut names = HashMap::new();

	for member in &error.variants.items {
		let member_name = member.id.name;
		if RESERVED_ERROR_MEMBER_NAMES.contains(&member_name) {
			errors.push(Error {
				kind: ErrorKind::EErrorVariantReservedIdent,
				span: member.id.span,
			});
		}

		if let Some(existing) = names.insert(member_name.to_string(), member.span) {
			errors.push(Error {
				kind: ErrorKind::EErrorVariantDuplicateIdent,
				span: member.id.span,
			});
			errors.push(Error {
				kind: ErrorKind::EErrorVariantDuplicateIdent,
				span: existing,
			});
		}
	}
}

fn qualify_struct<'a>(pool: &'a StringPool, stru: &mut ast::StructItem<'a>, _module_name: ast::Ident<'a>, parent_names: &mut Vec<&'a str>, errors: &mut Vec<Error>) {
	// If we have no parents then this is a top level item and must have a name
	if parent_names.is_empty() && stru.id.is_none() {
		errors.push(Error {
			kind: ErrorKind::EStructTopLevelIdent,
			span: stru.span,
		});
		return;
	}

	// Create a name for this item if it doesn't have one
	if stru.id.is_none() {
		let name = pool.store(create_name(parent_names));
		stru.id = Some(ast::Ident { span: stru.span, name });
	}

	parent_names.push(stru.id.as_ref().unwrap().name);
	for field in &mut stru.fields.items {
		qualify_field(pool, field, _module_name.clone(), parent_names, errors);
	}
	parent_names.pop();
}

fn qualify_field<'a>(pool: &'a StringPool, field: &mut ast::StructField<'a>, module_name: ast::Ident<'a>, parent_names: &mut Vec<&'a str>, errors: &mut Vec<Error>) {
	parent_names.push(field.id.name);
	match &mut field.ty.kind {
		ast::TypeDeclKind::TypeName(type_name) => {
			if type_name.module.is_none() {
				type_name.module = Some(module_name);
			}
		}
		ast::TypeDeclKind::Enum(enum_) => qualify_enum(pool, enum_, module_name.clone(), parent_names, errors),
		ast::TypeDeclKind::Error(error) => qualify_error(pool, error, module_name.clone(), parent_names, errors),
		ast::TypeDeclKind::Struct(struct_) => qualify_struct(pool, struct_, module_name.clone(), parent_names, errors),
		_ => (),
	}
	parent_names.pop();
}

fn qualify_type<'a>(pool: &'a StringPool, ty: &mut ast::TypeDecl<'a>, module_name: ast::Ident<'a>, parent_names: &mut Vec<&'a str>, errors: &mut Vec<Error>) {
	match &mut ty.kind {
		ast::TypeDeclKind::TypeName(type_name) => {
			if type_name.module.is_none() {
				type_name.module = Some(module_name);
			}
		}
		ast::TypeDeclKind::Enum(enum_) => qualify_enum(pool, enum_, module_name.clone(), parent_names, errors),
		ast::TypeDeclKind::Error(error) => qualify_error(pool, error, module_name.clone(), parent_names, errors),
		ast::TypeDeclKind::Struct(struct_) => qualify_struct(pool, struct_, module_name.clone(), parent_names, errors),
		_ => (),
	}
}

fn qualify_function<'a>(pool: &'a StringPool, func: &mut ast::FnItem<'a>, module_name: ast::Ident<'a>, parent_names: &mut Vec<&'a str>, errors: &mut Vec<Error>) {
	parent_names.push(func.id.name);
	for param in &mut func.params.items {
		qualify_field(pool, param, module_name.clone(), parent_names, errors);
	}
	if let Some(return_ty) = &mut func.return_ty {
		parent_names.push("RETURN");
		qualify_type(pool, return_ty, module_name.clone(), parent_names, errors);
		parent_names.pop();
	}
	if let Some(error_ty) = &mut func.error_ty {
		parent_names.push("ERROR");
		qualify_type(pool, error_ty, module_name.clone(), parent_names, errors);
		parent_names.pop();
	}
	parent_names.pop();
}
