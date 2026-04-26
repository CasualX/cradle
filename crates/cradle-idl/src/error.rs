use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
	/// Unexpected end of file.
	EOF,
	/// Unmatched closing brace
	EUnmatchedBraces,

	/// Expected an item
	EItemExpected,

	/// Expected a module declaration, but found something else.
	EModuleExpected,
	/// Expected an identifier after `module` keyword.
	EModuleIdent,
	/// Expected a semicolon after module declaration.
	EModuleSemicolon,
	/// Expected the first item in the IDL file to be a module declaration.
	EModuleFirstItem,
	/// Multiple module declarations in the same IDL file.
	EModuleMultiple,

	/// Expected a handle declaration, but found something else.
	EHandleExpected,
	/// Expected an identifier after `handle` keyword.
	EHandleIdent,
	/// Expected a semicolon after handle declaration.
	EHandleSemicolon,

	/// Expected an enum declaration, but found something else.
	EEnumExpected,
	/// Expected an identifier after `enum` keyword.
	EEnumIdent,
	/// Top level items must have an identifier.
	EEnumTopLevelIdent,
	/// Expected a repr after enum name
	EEnumReprExpected,
	/// Invalid enum repr identifier
	EEnumReprInvalid,
	/// Expected a left brace after enum
	EEnumLBrace,
	/// Expected a right brace after enum
	EEnumRBrace,
	/// Expected an enum member identifier
	EEnumMemberIdent,
	/// Duplicate enum member identifier
	EEnumMemberDuplicateIdent,
	/// Expected an explicit enum member value
	EEnumMemberValueExpected,
	/// Invalid enum member value for the enum repr type
	EEnumMemberValueInvalid,
	/// Missing comma between enum members
	EEnumMemberComma,

	/// Expected an error declaration, but found something else.
	EErrorExpected,
	/// Expected an identifier after `error` keyword.
	EErrorIdent,
	/// Top level items must have an identifier.
	EErrorTopLevelIdent,
	/// Expected a left brace after error
	EErrorLBrace,
	/// Expected a right brace after error
	EErrorRBrace,
	/// Expected an error variant identifier
	EErrorVariantIdent,
	/// Duplicate error variant identifier
	EErrorVariantDuplicateIdent,
	/// Reserved error variant identifier
	EErrorVariantReservedIdent,
	/// Missing comma between error variants
	EErrorVariantComma,

	/// Expected a struct declaration, but found something else.
	EStructExpected,
	/// Expected an identifier after `struct` keyword.
	EStructIdent,
	/// Top level items must have an identifier.
	EStructTopLevelIdent,
	/// Expected a left brace after struct
	EStructLBrace,
	/// Expected a right brace after struct fields
	EStructRBrace,
	/// Expected a struct field declaration
	EStructFieldExpected,
	/// Expected a struct field identifier
	EStructFieldIdent,
	/// Expected a colon after struct field identifier
	EStructFieldColon,
	/// Expected a default value after `=` in struct field
	EStructFieldDefaultValue,

	/// Expected a type declaration
	ETypeExpected,
	/// Expected a type name
	ETypeNameExpected,

	/// Expected a type modifier (e.g. `[N]` or `[]`, `[,]`, ...)
	ETypeModifierExpected,
	/// Expected an integer literal in type modifier
	ETypeModifierIntegerExpected,
	/// Type modifier integer value is out of range
	ETypeModifierOutOfRange,
	/// Too many dimensions in type modifier
	ETypeModifierTooManyDimensions,

	/// Expected a function declaration, but found something else.
	EFnExpected,
	/// Expected an identifier after `fn` keyword.
	EFnIdent,
	/// Missing trailing semicolon after function declaration
	EFnSemicolonExpected,
	/// Expected a left parenthesis after function identifier
	EFnLParen,
	/// Expected a right parenthesis after function parameters
	EFnRParen,
	/// Expected a function attribute for link_name
	EFnLinkNameRequired,
}

#[derive(Clone, Debug)]
pub struct Error {
	pub kind: ErrorKind,
	pub span: SourceSpan,
}

// impl Error {
// 	pub fn print(&self, )
// }
