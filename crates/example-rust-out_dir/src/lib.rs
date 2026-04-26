
pub mod operator;

#[repr(C)]
pub struct Context {
	hidden: i32,
}

pub struct Error {
	pub code: i32,
}
