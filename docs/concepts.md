
Concepts
--------

### Module

An IDL file must exist within a module, which serves as a namespace for the items defined within it.
Modules cannot be nested, so every item exists as `ModuleName.ItemName`. Unqualified names are allowed within the module (e.g. `Foo` instead of `ModuleName.Foo`).

All items must have a unique name within the module. However items can be implicitly named (see later)

```idl
module Operator;
```

### Enums

Enums are a simply named constants of the underlying type.

Explicit integer values can be assigned to enum variants, but they are not required.
If not value is assigned, the variant will be assigned the value of the previous variant + 1, starting with 0 for the first variant.

```idl
enum Suit : u8 {
	Spades, // 0
	Hearts = 2,
	Diamonds, // 3
	Clubs, // 4
}
```

Stability:

The explicit integer values of enum variants are considered part of the public API and should be stable across versions of the operator.
Changing the value of an existing variant is considered a breaking change and should be avoided.
Adding new variants is a non-breaking change as long as it does not affect the existing variants.

### Errors

Errors are a special kind of enum that represent the possible error cases for an operator.
They are defined using the `error` keyword. Unlike enums, error variants cannot have explicit values assigned to them and their repr is always `i32`.

```idl
error Overflow {
	/// The value is too large to fit in the output type.
	TooLarge,
	/// The value is too small to fit in the output type.
	TooSmall,
}
```

Errors serve as the return value of an operator, always an `i32` and has three categories of values:

- The success value, which is always `0`.
- The operator-specific error values, which are defined by the operator and are always positive.
- The system error values, which are common to all operators and are always negative.

  - `-1` = `ENullContext`: The operator was called with an invalid code generator context (e.g. a null pointer, etc).
  - `-2` = `EInvalidLicense`: The operator was called without a valid license (internal license check if implemented).
  - `-3` = `EStdException`: The operator threw an exception as exceptions are not allowed to propagate across the FFI boundary. (e.g. std::exception, panic! etc). 
  - `-4` = `EUnimplemented`: A stub operator that was generated but not implemented was called.
  - `EUnknown`: An unknown error occurred (e.g. an error code that is not defined by the operator or the system).

Stability:

Error integer values are not considered stable across versions of the operator, so they should not be used for error handling in user code.
Instead code generators should return the error variant name as a string or opaque identifier that can be used for checking specific error cases.

### Types

The IDL supports a set of primitive types, as well as user-defined types.

Primitive types:

* `i8`, `i16`, `i32`, `i64`, `isize`
* `u8`, `u16`, `u32`, `u64`, `usize`
* `f32`, `f64`
* `bool`
* `char`

Then there are some special types that have specific semantics:

* `string`: represents a string as a pointer and length. The ownership semantics of the string depend on the context (input or output).
* Type name: Either a single identifier or a qualified name (e.g. `ModuleName.TypeName`) that refers to a user-defined type (enum, struct, handle).
* Inline definition: A type can be defined inline within a struct or function definition. This is only allowed for enums, errors and structs. The inline definition is implicitly named and can only be referred to by the struct or function it is defined in.

Each type can optionally be modified with an array modifier to represent an array of that type.

* `[N]`: represents a fixed-size array of `N` elements of the given type. (StrBuf is a special case of this)
* `[]`: represents a variable-size array of elements of the given type. The ownership semantics in input: pointer and length, output: owned handle.
* `[,]`: represents a variable-size 2D array of elements of the given type. The ownership semantics in input: pointer and two lengths, output: owned handle.
* `[,,]`: represents a variable-size 3D array of elements of the given type. The ownership semantics in input: pointer and three lengths, output: owned handle.

Pointer-like types (string, variable-size arrays) can be modified with the optional `?` punct to indicate that the pointer can be null.
In input context this means a null pointer can be passed to the operator, and in output context this means the operator can return a null handle.

Union types are experimental and are written by separating types with the `|` punct. Exact semantics to be determined later...

### Ownership semantics

Whether the type appears in an input or output context determines the ownership semantics of the type. Input types are borrows, output types are owned handles.

Types without ownership semantics (e.g. primitive types, enums, errors, etc) can be used in both input and output contexts without any distinction.

The pointer-like types (string, variable-size arrays) have different representations and ownership semantics in input and output contexts.

In input context, they are represented as a pointer and length(s) and the operator MUST NOT take ownership of the pointer (i.e. the operator cannot free, or return the pointer in its output. The data must be cloned if it needs to be stored or returned).

In output context, they are represented as an owned handle (the bindings are reponsible for freeing the handle when it is no longer needed).

### Structs

Structs are a collection of named fields. They are defined using the `struct` keyword.

The IDL makes a distinction between input and output structs.

Input structs are used to define the inputs of an operator, while output structs are used to define the outputs of an operator.
This allows generating different ownership semantics for the fields of the struct. Input structs are borrows, output structs are owned handles.

When a struct is neither an input nor an output struct, it is restricted to only contain fields of primitive types and enum types.

```idl
struct Point {
	x: f64,
	y: f64,
}

in struct ParametersIn {
	name: string, // pointer + length
	value: i32,
}

out struct ParametersOut {
	name: string, // Owned string handle
	value: i32,
}
```

Structs, enums, errors can be implicitly defined by using them as the type of a field in another struct.

```idl
in struct ParametersIn {
	name: string,
	value: i32,
	optional: in struct {
		metadata: string,
	},
}
```

### Functions

Functions are defined using the `fn` keyword. They represent the operators that will be implemented and called from various programming languages.

```idl
fn UpdateRecord(
	file_name: string,
	key: string,
	value: string,
	options: in struct {
		overwrite: bool,
	},
) -> out struct {
	previous_value: string,
}, error {
	FileNotFound,
	PermissionDenied,
}
```

The parameters of the function are either defined inline (which are implicitly handled as an in struct) or as a single input struct.
The return type of the function is either 

### Handles

Handles are opaque types that represent owned resources. They are defined using the `handle` keyword.

```idl
handle File;
```

Handles are context independent pointers to opaque data owned by the cradle runtime.
They can be used in both input and output contexts, they always represent owned resources.

Cradle defines some built-in handles for `string` types with special semantics.

When strings are passed in an input context, they are represented as a pointer and length, and the operator must not take ownership of the pointer (i.e. the operator cannot free, or return the pointer in its output. The data must be cloned if it needs to be stored or returned).

When strings are passed in an output context, they are represented as an owned handle, and the bindings are responsible for freeing the handle when it is no longer needed.
