use std::collections::HashMap;

pub type Lookup<'a> = HashMap<&'a str, HashMap<&'a str, &'a Item<'a>>>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EnumReprType {
	I8, I16, I32, I64,
	U8, U16, U32, U64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EnumReprValue {
	I8(i8), I16(i16), I32(i32), I64(i64),
	U8(u8), U16(u16), U32(u32), U64(u64),
}

#[derive(Clone, Debug)]
pub struct EnumItem<'a> {
	pub doc: Option<&'a str>,
	pub name: &'a str,
	pub repr: EnumReprType,
	pub members: Vec<EnumMember<'a>>,
}

#[derive(Clone, Debug)]
pub struct EnumMember<'a> {
	pub doc: Option<&'a str>,
	pub name: &'a str,
	pub value: EnumReprValue,
}

#[derive(Clone, Debug)]
pub struct ErrorItem<'a> {
	pub doc: Option<&'a str>,
	pub name: &'a str,
	pub variants: Vec<ErrorVariant<'a>>,
}

#[derive(Clone, Debug)]
pub struct ErrorVariant<'a> {
	pub doc: Option<&'a str>,
	pub name: &'a str,
}

pub use crate::ast::StructDirection;

#[derive(Clone, Debug)]
pub struct StructItem<'a> {
	pub doc: Option<&'a str>,
	pub name: &'a str,
	pub direction: Option<StructDirection>,
	pub fields: Vec<Field<'a>>,
}
impl<'a> StructItem<'a> {
	pub fn has_lifetime(&self, lookup: &'a Lookup<'a>) -> bool {
		self.fields.iter().any(|field| field.ty.has_lifetime(lookup))
	}
}

#[derive(Clone, Debug)]
pub struct Field<'a> {
	pub doc: Option<&'a str>,
	pub name: &'a str,
	pub ty: Type<'a>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PrimitiveType {
	I8, I16, I32, I64, Isize,
	U8, U16, U32, U64, Usize,
	F32, F64,
	Bool,
	Char,
}
// pub use crate::ast::PrimitiveType;

/// Named type.
///
/// Referencing a user defined type, e.g. a struct or enum.
#[derive(Clone, Debug)]
pub struct TypeName<'a> {
	pub module: &'a str,
	pub name: &'a str,
}

#[derive(Clone, Debug)]
pub enum ElementType<'a> {
	Primitive(PrimitiveType),
	TypeName(TypeName<'a>),
	String,
}
impl<'a> ElementType<'a> {
	pub fn has_lifetime(&self, lookup: &'a Lookup<'a>) -> bool {
		match self {
			ElementType::Primitive(_) => false,
			ElementType::String => true,
			ElementType::TypeName(type_name) => {
				let module = type_name.module;
				let name = type_name.name;
				lookup.get(module).and_then(|m| m.get(name)).map_or(false, |item| match item {
					Item::Struct(s) => s.has_lifetime(lookup),
					Item::Enum(_) => false,
					Item::Error(_) => false,
					Item::Function(_) => false,
					Item::Handle(_) => true, // handles are opaque pointers, so they have an implicit lifetime
				})
			}
		}
	}
}

#[derive(Clone, Debug)]
pub enum Type<'a> {
	Element(ElementType<'a>),
	ArrayN(ElementType<'a>, u16),
	Array1D(ElementType<'a>),
	Array2D(ElementType<'a>),
	Array3D(ElementType<'a>),
	Array4D(ElementType<'a>),
}
impl<'a> Type<'a> {
	pub fn has_lifetime(&self, lookup: &'a Lookup<'a>) -> bool {
		match self {
			Type::Array1D(_) => true,
			Type::Array2D(_) => true,
			Type::Array3D(_) => true,
			Type::Array4D(_) => true,
			Type::ArrayN(elem, _) => elem.has_lifetime(lookup),
			Type::Element(elem) => elem.has_lifetime(lookup),
		}
	}
}

#[derive(Clone, Debug)]
pub struct FnItem<'a> {
	pub doc: Option<&'a str>,
	pub name: &'a str,
	pub link_name: &'a str,
	pub params: StructItem<'a>,
	pub return_ty: Option<Type<'a>>,
	pub error_ty: Option<Type<'a>>,
}

#[derive(Clone, Debug)]
pub struct HandleItem<'a> {
	pub doc: Option<&'a str>,
	pub name: &'a str,
}

#[derive(Clone, Debug)]
pub enum Item<'a> {
	Enum(EnumItem<'a>),
	Error(ErrorItem<'a>),
	Struct(StructItem<'a>),
	Function(FnItem<'a>),
	Handle(HandleItem<'a>),
}
impl<'a> Item<'a> {
	pub fn name(&self) -> &'a str {
		match self {
			Self::Enum(e) => e.name,
			Self::Error(e) => e.name,
			Self::Struct(s) => s.name,
			Self::Function(f) => f.name,
			Self::Handle(h) => h.name,
		}
	}
	pub fn variant_name(&self) -> &'static str {
		match self {
			Self::Enum(_) => "Enum",
			Self::Error(_) => "Error",
			Self::Struct(_) => "Struct",
			Self::Function(_) => "Function",
			Self::Handle(_) => "Handle",
		}
	}
}

#[derive(Clone, Debug)]
pub struct Module<'a> {
	pub doc: Option<&'a str>,
	pub name: &'a str,
	pub items: Vec<Item<'a>>,
}

#[derive(Clone, Debug)]
pub struct Library<'a> {
	pub modules: Vec<Module<'a>>,
}
impl<'a> Library<'a> {
	pub fn lookup(&'a self) -> Lookup<'a> {
		let mut lookup = HashMap::new();
		for module in &self.modules {
			let mut module_lookup = HashMap::new();
			for item in &module.items {
				module_lookup.insert(item.name(), item);
			}
			lookup.insert(module.name, module_lookup);
		}
		lookup
	}
}
