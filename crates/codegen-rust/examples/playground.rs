

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct MyEnum(pub i32);

impl MyEnum {
	pub const A: Self = Self(0);
	pub const B: Self = Self(1);
}


fn main() {

}
