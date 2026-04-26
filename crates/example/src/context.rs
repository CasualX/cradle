
pub struct Context {
	dummy: i32,
}

#[no_mangle]
pub extern "C" fn ContextCreate() -> *mut Context {
	Box::into_raw(Box::new(Context {
		dummy: 42,
	}))
}

#[no_mangle]
pub extern "C" fn ContextDestroy(ctx: *mut Context) {
	if ctx.is_null() {
		return;
	}
	drop(unsafe { Box::from_raw(ctx) });
}
