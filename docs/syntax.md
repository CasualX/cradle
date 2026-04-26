# Cradle IDL Syntax Guide

This document describes the syntax that the current `cradle-idl` implementation accepts today.

It is intentionally narrower than some of the design notes elsewhere in the repository. When `docs/concepts.md`, `docs/bindings-user-guide.md`, or older `.idl` files disagree with this document, treat this file as the parser-oriented source of truth for what is currently legal syntax.

## Scope

This guide covers:

- the lexical forms the lexer accepts
- the item shapes the parser accepts
- the extra legality rules enforced after parsing
- notable forms that appear in the repo or design docs but are not currently accepted

It does not try to be a full semantic specification for code generation or ABI behavior.

## File Structure

An IDL file is a sequence of top-level items.

The current validation pass requires:

- the first top-level item must be a `module` declaration
- there must be exactly one top-level `module` declaration per file

Legal example:

```idl
module KitchenSink;

handle File;

enum Mode : u16 {
	Read = 1,
	Write = 2,
}
```

Illegal examples:

```idl
handle File;
module KitchenSink;
```

Reason: the first item is not `module`.

```idl
module One;
module Two;
```

Reason: only one top-level `module` item is allowed.

## Whitespace And Comments

ASCII whitespace is ignored between tokens.

The lexer accepts these comment forms:

- line comments: `// comment`
- block comments: `/* comment */`
- doc line comments: `/// comment`

Only doc comments are attached as attributes to the following item, field, or variant. Regular comments are ignored by the AST.

Examples:

```idl
// ignored comment
/// Attached to the next item.
/*
ignored block comment
*/
handle File;
```

## Identifiers

Identifiers may start with an ASCII letter and may then contain ASCII letters, digits, or underscores.

Accepted examples:

- `File`
- `OpenFailure`
- `read_only`
- `Array3D`

Rejected examples:

- `_hidden`
- `3DPoint`
- `kebab-case`

The lexer currently accepts underscores after the first character, even though one lexer comment still describes a stricter rule.

## Attributes

Attributes may appear before top-level items, struct fields, enum members, and error variants.

### Documentation Attributes

Doc comments become documentation attributes.

```idl
/// Primary file handle.
handle File;
```

### Command Attributes

A command attribute is `#` followed by an identifier.

```idl
#deprecated
handle LegacyFile;
```

### Key/Value Attributes

A key/value attribute is `#name = value` where the value is either an identifier or a literal.

```idl
#link_name = MODULE_OpenFile
fn OpenFile(path: string);
```

```idl
#priority = 3
handle File;
```

Currently accepted attribute values are an identifier or a literal.

Currently not accepted: complex expressions, lists or qualified names like `Module.Name`.

## Top-Level Items

The parser recognizes these top-level item kinds:

- `module`
- `handle`
- `enum`
- `error`
- `(in|out)? struct`
- `fn`

No other top-level keyword is currently legal.

## Module Declarations

Syntax:

```idl
module ModuleName;
```

Rules:

- requires exactly one identifier
- requires a trailing semicolon
- cannot use a block body

Illegal examples:

```idl
module;
```

```idl
module Demo
```

```idl
module Demo {}
```

## Handle Declarations

Syntax:

```idl
handle File;
```

Rules:

- requires exactly one identifier
- requires a trailing semicolon
- does not accept a body

Illegal examples:

```idl
handle;
```

```idl
handle File {}
```

## Enum Declarations

Syntax:

```idl
enum Mode : u16 {
	Read = 1,
	Write = 2,
	Append = 3,
}
```

Rules:

- a top-level enum must have a name
- an inline enum may omit its name and will be named later by validation
- the representation type is required
- the representation must be one of `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, or `u64`
- enum members are identifiers
- enum member values, when present, must be integer literals
- trailing commas are allowed

Legal examples:

```idl
enum Mode : u16 {
	Read,
	Write = 2,
}
```

```idl
fn Check() -> enum : i32 {
	Ok = 0,
	Busy = 1,
};
```

Illegal examples:

```idl
enum Mode {
	Read,
}
```

Reason: enum repr is mandatory.

```idl
enum Mode : u16 {
	Read = true,
}
```

Reason: enum member values must be integer literals.

Validation notes:

- explicit enum values are parsed and later checked against the chosen repr type
- top-level anonymous enums are rejected during qualification

## Error Declarations

Syntax:

```idl
error OpenFailure {
	NotFound,
	PermissionDenied,
	Busy,
}
```

Rules:

- a top-level error must have a name
- an inline error may omit its name and will be named later by validation
- error variants are identifiers only
- error variants cannot have explicit values
- trailing commas are allowed

Legal examples:

```idl
error OpenFailure {
	NotFound,
	Busy,
}
```

```idl
fn Open() error {
	Busy,
};
```

Illegal examples:

```idl
error OpenFailure {
	NotFound = 1,
}
```

Reason: explicit discriminants are not part of error syntax.

Validation notes:

- top-level anonymous errors are rejected
- duplicate error variant names are rejected
- these reserved variant names are rejected: `CONTEXT_NULL`, `INVALID_LICENSE`, `FFI_UNWIND`, `UNIMPLEMENTED`

## Struct Declarations

Syntax:

```idl
struct Point {
	x: f64,
	y: f64,
}
```

```idl
in struct Options {
	path: string,
	create: bool = true,
}
```

```idl
out struct OpenResult {
	file: File,
	mode: Mode,
}
```

Rules:

- a top-level struct must have a name
- an inline struct may omit its name and will be named later by validation
- `in` and `out` are optional only when immediately followed by `struct`
- struct fields are comma-separated
- a trailing comma is allowed
- each field is `name: Type`
- a field may optionally have `= literal` as a default value

Legal examples:

```idl
struct Point {
	x: f64,
	y: f64,
}
```

```idl
in struct Options {
	path: string,
	create: bool = true,
	labels: string[],
}
```

```idl
fn Open() -> out struct {
	file: File,
	retryable: bool,
};
```

Illegal examples:

```idl
struct {
	x: i32,
}
```

Reason: anonymous top-level structs are rejected by validation.

```idl
struct Point {
	x i32,
}
```

Reason: fields require `:` between the name and type.

```idl
struct Point {
	x: i32 = SomeValue,
}
```

Reason: field defaults must be literals, not identifiers.

### Field Defaults

The parser accepts literal defaults only.

Accepted examples:

- `enabled: bool = true`
- `count: u32 = 4`
- `name: string = "demo"`
- `letter: char = 'x'`

Not accepted today:

- identifier defaults such as `= null`
- compound defaults such as arrays or struct literals

That means many existing WIP `.idl` files in the repo that use `= null` are ahead of the current parser.

## Function Declarations

Syntax:

```idl
fn OpenFile(
	path: string,
	options: Options,
	grid: i32[,],
) -> out struct OpenResult {
	file: File,
	mode: Mode,
	retryable: bool,
}, error OpenFailure {
	NotFound,
	PermissionDenied,
	Busy,
};
```

Rules:

- function names are required
- parameters are enclosed in `(` and `)`
- parameters use the same field syntax as struct fields
- a trailing comma is allowed in the parameter list
- the declaration must end with `;`
- return and error types are optional

The parser currently supports three function shapes:

```idl
fn Notify(message: string);
```

No return type and no error type.

```idl
fn Read(file: File) -> string;
```

Return type only.

```idl
fn Notify(message: string) error NotifyError;
```

Error type only, with no `->`.

```idl
fn Open(path: string) -> File, error OpenFailure {
	NotFound,
};
```

Return type and error type.

Important parsing detail: when `->` is present, the parser reads the return type first and then, if there is a comma, reads the error type.

### Function Attributes

The parser accepts any attribute syntax on functions, but the current AST-to-IR pass additionally requires a `#link_name = ...` attribute for every function that should lower successfully.

Legal parser example that later fails validation/lowering:

```idl
fn Open(path: string);
```

Legal parser and lowering example:

```idl
#link_name = MODULE_Open
fn Open(path: string);
```

## Types

### Primitive Types

The parser accepts these primitive type names:

- `i8`, `i16`, `i32`, `i64`, `isize`
- `u8`, `u16`, `u32`, `u64`, `usize`
- `f32`, `f64`
- `bool`
- `char`

### Special Built-In Types

The parser currently recognizes exactly one special built-in type:

- `string`

Those names parse only as ordinary type names if used, and will require matching user-defined items later.

### User-Defined Type Names

User-defined type references may be either unqualified or module-qualified.

Examples:

```idl
File
```

```idl
KitchenSink.File
```

Only one qualifier level is supported today. This is legal:

```idl
Module.Type
```

This is not:

```idl
Outer.Inner.Type
```

### Inline Type Definitions

The parser accepts inline `struct`, `in struct`, `out struct`, `enum`, and `error` definitions anywhere a type is expected.

Examples:

```idl
fn Open() -> out struct {
	file: File,
	ok: bool,
};
```

```idl
fn Open() error {
	Busy,
	Denied,
};
```

```idl
struct Container {
	mode: enum : u8 {
		A,
		B,
	},
}
```

Unnamed inline types are synthesized later from their parent path. Top-level anonymous items are not allowed.

## Type Modifiers

After the base type, the parser accepts at most one array modifier.

Legal modifiers:

- `[N]` fixed-size array, where `N` is an integer from `1` to `65535`
- `[]` one-dimensional variable-size array
- `[,]` two-dimensional variable-size array
- `[,,]` three-dimensional variable-size array

Examples:

```idl
u8[16]
string[]
i32[,]
f64[,,]
```

Illegal examples:

```idl
u8[0]
u8[70000]
```

Reason: fixed-size array length must fit in `u16` and must be greater than zero.

```idl
u8[,,,]
```

Reason: the parser rejects more than three dynamic dimensions.

```idl
u8[][4]
```

Reason: only one modifier is allowed.

Implementation note: the AST and IR have a 4D array variant, but the parser currently never produces it.

## Quick Reference

Current top-level grammar, approximately:

```text
file        := attrs? module_item item*
item        := handle_item
             | enum_item
             | error_item
             | struct_item
             | function_item

module_item := attrs? 'module' Ident ';'
handle_item := attrs? 'handle' Ident ';'
enum_item   := attrs? 'enum' Ident? ':' enum_repr enum_members
error_item  := attrs? 'error' Ident? error_members
struct_item := attrs? ('in' | 'out')? 'struct' Ident? struct_fields
function_item := attrs? 'fn' Ident fn_params
                 ( '->' type (',' type)? | type )?
                 ';'
```

And for fields and types:

```text
field       := attrs? Ident ':' type ('=' literal)?
type        := primitive
             | 'string'
             | type_name
             | struct_item
             | enum_item
             | error_item
             | type modifier?
type_name   := Ident | Ident '.' Ident
modifier    := '[' Integer ']'
             | '[]'
             | '[,]'
             | '[,,]'
```
