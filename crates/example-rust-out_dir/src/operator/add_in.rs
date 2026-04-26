
pub struct AddIn {
	pub a: i32,
	pub b: i32,
}

#[repr(C)]
pub(crate) struct AddIn_C {
	pub(crate) a: i32,
	pub(crate) b: i32,
}
