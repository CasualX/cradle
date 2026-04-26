use super::*;

#[repr(C)]
pub struct AddIn {
	pub a: i32,
	pub b: i32,
}

#[repr(i32)]
pub enum Error {
	Overflow,
}

#[no_mangle]
pub extern "C" fn OPERATOR_Add(ctx: *const Context, input: *const AddIn, output: *mut i32) -> i32 {
	if ctx.is_null() || input.is_null() || output.is_null() {
		return -1; // Invalid arguments
	}

	let ctx = unsafe { &*ctx };
	let input = unsafe { &*input };
	let output = unsafe { &mut *output };

	match add(ctx, input) {
		Ok(result) => {
			*output = result;
			0 // Success
		}
		Err(err) => err as i32, // Return error code
	}
}

fn add(_ctx: &Context, input: &AddIn) -> Result<i32, Error> {
	let result = input.a.checked_add(input.b).ok_or(Error::Overflow)?;

	Ok(result)
}
