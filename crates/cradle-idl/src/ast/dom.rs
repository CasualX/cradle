use super::*;

/// Identifier.
#[derive(Clone, Debug)]
pub struct Ident<'a> {
	pub span: SourceSpan,
	pub name: &'a str,
}

/// Attribute.
///
/// E.g. `#foo` or `#foo = <ident or literal>` or doc comment
#[derive(Clone, Debug)]
pub struct Attribute<'a> {
	pub kind: AttrKind<'a>,
	pub span: SourceSpan,
}

#[derive(Clone, Debug)]
pub struct AttrDoc<'a> {
	pub span: SourceSpan,
	pub comment: &'a str,
}

#[derive(Clone, Debug)]
pub struct AttrCmd<'a> {
	pub name: Ident<'a>,
}

#[derive(Clone, Debug)]
pub struct AttrKv<'a> {
	pub key: Ident<'a>,
	pub value: AttrValue<'a>,
	pub span: SourceSpan,
}

#[derive(Clone, Debug)]
pub struct AttrValue<'a> {
	pub span: SourceSpan,
	pub text: &'a str,
}

#[derive(Clone, Debug)]
pub enum AttrKind<'a> {
	Doc(AttrDoc<'a>),
	Cmd(AttrCmd<'a>),
	Kv(AttrKv<'a>),
}

/// Module item.
///
/// E.g. `module Foo;`
#[derive(Clone, Debug)]
pub struct ModuleItem<'a> {
	pub attrs: Vec<Attribute<'a>>,
	pub id: Ident<'a>,
	pub span: SourceSpan,
}


/// Enum member value.
///
/// E.g. `1`
#[derive(Clone, Debug)]
pub struct EnumMemberValue<'a> {
	pub span: SourceSpan,
	pub value: &'a str,
}

/// Enum member.
///
/// E.g. `Bar,` or `Baz = 1,`
#[derive(Clone, Debug)]
pub struct EnumMember<'a> {
	pub attrs: Vec<Attribute<'a>>,
	pub id: Ident<'a>,
	pub value: Option<EnumMemberValue<'a>>,
	pub span: SourceSpan,
}

/// Enum members block.
///
/// Including outer `{ ... }` braces.
#[derive(Clone, Debug)]
pub struct EnumMembers<'a> {
	pub items: Vec<EnumMember<'a>>,
	pub span: SourceSpan,
}

/// Enum representation type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EnumReprType {
	I8, I16, I32, I64,
	U8, U16, U32, U64,
}

/// Enum item.
///
/// E.g. `enum Foo : u32 { Bar, Baz = 1 }`
#[derive(Clone, Debug)]
pub struct EnumItem<'a> {
	pub attrs: Vec<Attribute<'a>>,
	pub id: Option<Ident<'a>>,
	pub repr: EnumReprType,
	pub members: EnumMembers<'a>,
	pub span: SourceSpan,
}


/// Error member.
///
/// E.g. `Overflow,`
#[derive(Clone, Debug)]
pub struct ErrorVariant<'a> {
	pub attrs: Vec<Attribute<'a>>,
	pub id: Ident<'a>,
	pub span: SourceSpan,
}

/// Error members.
///
/// Including outer `{ ... }` braces.
#[derive(Clone, Debug)]
pub struct ErrorVariants<'a> {
	pub items: Vec<ErrorVariant<'a>>,
	pub span: SourceSpan,
}

/// Error item.
///
/// E.g. `error Foo { Overflow, Underflow }`
#[derive(Clone, Debug)]
pub struct ErrorItem<'a> {
	pub attrs: Vec<Attribute<'a>>,
	pub id: Option<Ident<'a>>,
	pub variants: ErrorVariants<'a>,
	pub span: SourceSpan,
}


#[derive(Clone, Debug)]
pub struct TypeName<'a> {
	pub module: Option<Ident<'a>>,
	pub name: Ident<'a>,
	pub span: SourceSpan,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PrimitiveType {
	I8, I16, I32, I64, Isize,
	U8, U16, U32, U64, Usize,
	F32, F64,
	Bool,
	Char,
}

#[derive(Clone, Debug)]
pub enum TypeDeclKind<'a> {
	Primitive(PrimitiveType),
	String,
	TypeName(TypeName<'a>),
	Enum(EnumItem<'a>),
	Error(ErrorItem<'a>),
	Struct(StructItem<'a>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TypeModifier {
	ArrayLen(u16),
	Array1D,
	Array2D,
	Array3D,
	Array4D,
}

#[derive(Clone, Debug)]
pub struct TypeDecl<'a> {
	pub kind: TypeDeclKind<'a>,
	pub modifier: Option<TypeModifier>,
	pub span: SourceSpan,
}

#[derive(Clone, Debug)]
pub struct TypesDecl<'a> {
	pub types: Vec<TypeDecl<'a>>,
	pub optional: bool,
	pub span: SourceSpan,
}


#[derive(Clone, Debug)]
pub struct DefaultValue<'a> {
	pub span: SourceSpan,
	pub value: &'a str,
}

#[derive(Clone, Debug)]
pub struct StructField<'a> {
	pub attrs: Vec<Attribute<'a>>,
	pub id: Ident<'a>,
	pub ty: TypeDecl<'a>,
	pub default: Option<DefaultValue<'a>>,
	pub span: SourceSpan,
}

#[derive(Clone, Debug)]
pub struct StructFields<'a> {
	pub items: Vec<StructField<'a>>,
	pub span: SourceSpan,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StructDirection {
	/// Input parameter.
	In,
	/// Output parameter.
	Out,
}

#[derive(Clone, Debug)]
pub struct StructItem<'a> {
	pub attrs: Vec<Attribute<'a>>,
	pub direction: Option<StructDirection>,
	pub id: Option<Ident<'a>>,
	pub fields: StructFields<'a>,
	pub span: SourceSpan,
}


#[derive(Clone, Debug)]
pub struct FnParams<'a> {
	pub items: Vec<StructField<'a>>,
	pub span: SourceSpan,
}

#[derive(Clone, Debug)]
pub struct FnItem<'a> {
	pub attrs: Vec<Attribute<'a>>,
	pub id: Ident<'a>,
	pub params: FnParams<'a>,
	pub return_ty: Option<TypeDecl<'a>>,
	pub error_ty: Option<TypeDecl<'a>>,
	pub span: SourceSpan,
}


#[derive(Clone, Debug)]
pub struct HandleItem<'a> {
	pub attrs: Vec<Attribute<'a>>,
	pub id: Ident<'a>,
	pub span: SourceSpan,
}


#[derive(Clone, Debug)]
pub enum Item<'a> {
	Module(ModuleItem<'a>),
	Enum(EnumItem<'a>),
	Error(ErrorItem<'a>),
	Struct(StructItem<'a>),
	Function(FnItem<'a>),
	Handle(HandleItem<'a>),
}
impl<'a> Item<'a> {
	pub fn span(&self) -> SourceSpan {
		match self {
			Item::Module(item) => item.span,
			Item::Enum(item) => item.span,
			Item::Error(item) => item.span,
			Item::Struct(item) => item.span,
			Item::Function(item) => item.span,
			Item::Handle(item) => item.span,
		}
	}
}


#[derive(Clone, Debug)]
pub struct IdlFile<'a> {
	pub items: Vec<Item<'a>>,
}

impl<'a> IdlFile<'a> {
	pub fn get_module_item(&self) -> Option<&ModuleItem<'a>> {
		for item in &self.items {
			if let Item::Module(module) = item {
				return Some(module);
			}
		}
		return None;
	}
}
