use super::*;

use std::fmt;

fn doc<'a>(doc: Option<&'a str>, indents: usize) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		if let Some(doc) = doc {
			for line in doc.lines() {
				for _ in 0..indents {
					"\t"
				}
				if line.trim_ascii().is_empty() {
					"///\n"
				} else {
					"/// "{line}"\n"
				}
			}
		}
	)
}

pub fn enum_item<'a>(item: &'a ir::EnumItem<'a>) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		{doc(item.doc, 0)}
		"#[derive(Copy, Clone, Eq, PartialEq)]\n"
		"#[repr(transparent)]\n"
		let item_name = case::pascal(&item.name);
		"pub struct "{item_name}"(pub "{enum_repr_type(item.repr)}");\n\n"
		"impl "{item_name}" {\n"
		for member in &item.members {
			let member_name = case::scream(&member.name);
			{doc(member.doc, 1)}
			"\tpub const "{member_name}": Self = Self("{enum_repr_value(member.value)}");\n"
		}
		"}\n"
	)
}

fn enum_repr_type(repr: ir::EnumReprType) -> impl fmt::Display {
	fmt::from_fn(move |f| {
		let s = match repr {
			ir::EnumReprType::I8 => "i8",
			ir::EnumReprType::I16 => "i16",
			ir::EnumReprType::I32 => "i32",
			ir::EnumReprType::I64 => "i64",
			ir::EnumReprType::U8 => "u8",
			ir::EnumReprType::U16 => "u16",
			ir::EnumReprType::U32 => "u32",
			ir::EnumReprType::U64 => "u64",
		};
		f.write_str(s)
	})
}

fn enum_repr_value(value: ir::EnumReprValue) -> impl fmt::Display {
	use fmt::Display;
	fmt::from_fn(move |f| {
		match value {
			ir::EnumReprValue::I8(v) => v.fmt(f),
			ir::EnumReprValue::I16(v) => v.fmt(f),
			ir::EnumReprValue::I32(v) => v.fmt(f),
			ir::EnumReprValue::I64(v) => v.fmt(f),
			ir::EnumReprValue::U8(v) => v.fmt(f),
			ir::EnumReprValue::U16(v) => v.fmt(f),
			ir::EnumReprValue::U32(v) => v.fmt(f),
			ir::EnumReprValue::U64(v) => v.fmt(f),
		}
	})
}

pub fn error_item<'a>(item: &'a ir::ErrorItem<'a>) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		let item_name = case::pascal(&item.name);
		{doc(item.doc, 0)}
		"#[derive(Copy, Clone, Eq, PartialEq)]\n"
		"#[repr(transparent)]\n"
		"pub struct "{item_name}"(i32);\n\n"
		"impl "{item_name}" {\n"
		"\tpub const UNIMPLEMENTED: Self = Self(-4);\n"
		"\tpub const INVALID_LICENSE: Self = Self(-3);\n"
		"\tpub const FFI_UNWIND: Self = Self(-2);\n"
			"\tpub const CONTEXT_NULL: Self = Self(-1);\n"
			"\tpub const SUCCESS: Self = Self(0);\n"
		for (i, member) in item.variants.iter().enumerate() {
			let member_name = case::scream(&member.name);
			{doc(member.doc, 1)}
			"\tpub const "{member_name}": Self = Self("{i + 1}");\n"
		}
		"}\n"
	)
}

fn write_primitive_type(f: &mut fmt::Formatter, prim: ir::PrimitiveType) -> fmt::Result {
	let s = match prim {
		ir::PrimitiveType::I8 => "i8",
		ir::PrimitiveType::I16 => "i16",
		ir::PrimitiveType::I32 => "i32",
		ir::PrimitiveType::I64 => "i64",
		ir::PrimitiveType::Isize => "isize",
		ir::PrimitiveType::U8 => "u8",
		ir::PrimitiveType::U16 => "u16",
		ir::PrimitiveType::U32 => "u32",
		ir::PrimitiveType::U64 => "u64",
		ir::PrimitiveType::Usize => "usize",
		ir::PrimitiveType::F32 => "f32",
		ir::PrimitiveType::F64 => "f64",
		ir::PrimitiveType::Bool => "bool",
		ir::PrimitiveType::Char => "char",
	};
	f.write_str(s)
}

fn write_element_type<'a>(elem: &'a ir::ElementType, dir: Option<ir::StructDirection>, lookup: &'a ir::Lookup<'a>) -> impl 'a + fmt::Display {
	fmt::from_fn(move |f| {
		match elem {
			&ir::ElementType::Primitive(prim) => write_primitive_type(f, prim),
			ir::ElementType::String => match dir {
				Some(ir::StructDirection::In) => f.write_str("crate::_array::StrPtr"),
				Some(ir::StructDirection::Out) => f.write_str("*mut crate::StringHandle"),
				None => panic!("String element type must have a struct direction"),
			},
			ir::ElementType::TypeName(type_name) => {
				write!(f, "crate::{}::{}", case::snake(type_name.module), case::pascal(type_name.name))
			}
		}
	})
}

fn ty<'a>(ty: &'a ir::Type<'a>, dir: Option<ir::StructDirection>, lookup: &'a ir::Lookup<'a>) -> impl 'a + fmt::Display {
	fmt::from_fn(move |f| {
		match ty {
			ir::Type::Element(el) => write!(f, "{}", write_element_type(el, dir, lookup)),
			ir::Type::Array1D(ir::ElementType::String) => match dir {
				Some(ir::StructDirection::Out) => write!(f, "crate::Array1D<crate::StringHandle>"),
				Some(ir::StructDirection::In) => write!(f, "crate::Array1DPtr<crate::_array::StrPtr>"),
				None => panic!("Array1D of string must have a struct direction"),
			},
			ir::Type::Array1D(el) => match dir {
				Some(ir::StructDirection::Out) => write!(f, "crate::Array1D<{}>", write_element_type(el, dir, lookup)),
				Some(ir::StructDirection::In) => write!(f, "crate::Array1DPtr<{}>", write_element_type(el, dir, lookup)),
				None => panic!("Array1D element type must have a struct direction"),
			},
			ir::Type::Array2D(el) => match dir {
				Some(ir::StructDirection::Out) => write!(f, "crate::Array2D<{}>", write_element_type(el, dir, lookup)),
				Some(ir::StructDirection::In) => write!(f, "crate::Array2DPtr<{}>", write_element_type(el, dir, lookup)),
				None => panic!("Array2D element type must have a struct direction"),
			}
			_ => unimplemented!("{:?}", ty),
		}
	})
}

pub fn function_item<'a>(item: &'a ir::FnItem<'a>, lookup: &'a ir::Lookup<'a>) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		let func_name = case::snake(&item.name);
		let c_func_name = &item.link_name;
		"extern \"C\" {\n"
		{doc(item.doc, 1)}
		"\t#[link_name = \""{c_func_name}"\"]\n"
		"\tpub fn "{func_name}"(\n"
			for p in &item.params.fields {
				let param_name = case::snake(&p.name);
				{doc(p.doc, 2)}
				"\t\t"{param_name}": "{ty(&p.ty, item.params.direction, lookup)}",\n"
			}
		"\t);\n"
		"}\n"
	)
}

pub fn cargo_toml() -> impl fmt::Display {
	fmtools::fmt!(
		"[package]\n"
		"name = \"example-sys\"\n"
		"version = \"0.1.0\"\n"
		"edition = \"2021\"\n\n"
		"[dependencies]\n"
	)
}

pub fn lib_rs<'a>(library: &'a ir::Library<'a>) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		"mod _array;\n"
		"pub use _array::*;\n\n"
		for module in &library.modules {
			let module_name = case::snake(&module.name);
			"pub mod "{module_name}";\n"
		}
	)
}

pub fn module_mod_rs<'a>(module: &'a ir::Module<'a>) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		for item in &module.items {
			let item_name = case::snake(&item.name());
			"mod "{item_name}"; // "{item.variant_name()}"\n"
		}
		for item in &module.items {
			let item_name = case::snake(&item.name());
			"pub use self::"{item_name}"::*;\n"
		}
	)
}

pub fn handle_item<'a>(item: &'a ir::HandleItem<'a>) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		let item_name = case::pascal(&item.name);
		{doc(item.doc, 0)}
		"pub struct "{item_name}"(pub *mut std::ffi::c_void);\n"
	)
}

pub fn struct_item<'a>(item: &'a ir::StructItem<'a>, lookup: &'a ir::Lookup<'a>) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		let item_name = case::pascal(&item.name);
		{doc(item.doc, 0)}
		"#[repr(C)]\n"
		"pub struct "{item_name}" {\n"
		for field in &item.fields {
			let field_name = case::snake(&field.name);
			{doc(field.doc, 1)}
			"\tpub "{field_name}": "{ty(&field.ty, item.direction, lookup)}",\n"
		}
		"}\n"
	)
}
