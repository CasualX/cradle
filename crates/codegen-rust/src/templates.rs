use std::fmt;
use cradle_idl::*;

fn doc<'a>(tabs: usize, s: &'a Option<&'a str>) -> impl 'a + fmt::Display {
	fmt::from_fn(move |f| {
		let Some(s) = s else {
			return Ok(());
		};
		for line in s.lines() {
			for _ in 0..tabs {
				f.write_str("\t")?;
			}
			f.write_str("/// ")?;
			f.write_str(line)?;
			f.write_str("\n")?;
		}
		Ok(())
	})
}

pub fn enum_item<'a>(item: &'a ir::EnumItem<'a>) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		{doc(0, &item.doc)}
		"#[derive(Copy, Clone, Eq, PartialEq)]\n"
		"#[repr(transparent)]\n"
		let item_name = case::pascal(&item.name);
		"pub struct "{item_name}"(pub "{enum_repr_type(item.repr)}");\n\n"
		"impl "{item_name}" {\n"
		for member in &item.members {
			{doc(1, &member.doc)}
			let member_name = case::pascal(&member.name);
			"\tpub const "{member_name}": Self = Self("{enum_repr_value(member.value)}");\n"
		}
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
		{doc(0, &item.doc)}
		let item_name = case::pascal(&item.name);
		"pub struct "{item_name}"(i32);\n\n"
		"impl "{item_name}" {\n"
		for (i, member) in item.variants.iter().enumerate() {
			{doc(1, &member.doc)}
			let member_name = case::pascal(&member.name);
			"\tpub const "{member_name}": Self = Self("{i}");\n"
		}
	)
}

	fn type_name<'a>(ty: &'a ir::TypeName<'a>) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		let module = case::snake(&ty.module);
		let name = case::pascal(&ty.name);
		{module}"::"{name}
	)
}

pub fn function_item<'a>(item: &'a ir::FnItem<'a>) -> impl 'a + fmt::Display {
	fmtools::fmt!(move
		{doc(0, &item.doc)}
		let func_name = case::snake(&item.name);
		let param_tyname = case::pascal(&item.params.name);
		"pub fn "{func_name}"(args: "{param_tyname}") {\n"
		"\tunimplemented!();\n"
		"}\n"
	)
}
