/*! # Cradle Interface Definition Language (IDL)

This crate parses IDL files and contains analysis and transformation into the intermediate representation (IR) used by the code generator.
 */

pub mod case;

mod span;
pub use span::*;

mod error;
pub use error::*;

mod pool;
pub use pool::*;

pub mod ast;
pub mod ir;
pub mod passes;
