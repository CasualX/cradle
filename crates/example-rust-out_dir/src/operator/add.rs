
pub fn add(input: &crate::operator::AddIn) -> Result<i32, crate::Error> {
	let input_c: crate::operator::AddIn_C = crate::operator::AddIn_C { a: input.a, b: input.b };
	let mut output_c: i32 = Default::default();
	let error = unsafe { OPERATOR_Add(std::ptr::null(), &input_c, &mut output_c) };
	if error != 0 {
		return Err(crate::Error { code: error });
	}
	Ok(output_c)
}

extern "C" {
	fn OPERATOR_Add(ctx: *const crate::Context, input: *const crate::operator::AddIn_C, output: *mut i32) -> i32;
}
