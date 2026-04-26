/*!
Abstract Syntax Tree (AST) for IDL files.
*/

use super::*;

mod dom;
mod lexer;
mod parser;

pub use dom::*;
use lexer::*;
pub use parser::*;

#[cfg(test)]
mod tests;
