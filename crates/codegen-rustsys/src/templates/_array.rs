
#[repr(C)]
pub struct StrPtr {
	pub ptr: *const u8,
	pub len: usize,
}

#[repr(C)]
pub struct Array1DPtr<T> {
	pub ptr: *mut T,
	pub len: [usize; 1],
}

#[repr(C)]
pub struct Array2DPtr<T> {
	pub ptr: *mut T,
	pub len: [usize; 2],
}

#[repr(C)]
pub struct Array3DPtr<T> {
	pub ptr: *mut T,
	pub len: [usize; 3],
}

#[repr(C)]
pub struct Array4DPtr<T> {
	pub ptr: *mut T,
	pub len: [usize; 4],
}

#[repr(C)]
pub struct Array1D<T> {
	pub ptr: *mut T,
	pub len: [usize; 1],
}

#[repr(C)]
pub struct Array2D<T> {
	pub ptr: *mut T,
	pub len: [usize; 2],
}

#[repr(C)]
pub struct Array3D<T> {
	pub ptr: *mut T,
	pub len: [usize; 3],
}

#[repr(C)]
pub struct Array4D<T> {
	pub ptr: *mut T,
	pub len: [usize; 4],
}
