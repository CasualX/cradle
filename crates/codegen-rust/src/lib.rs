use cradle_idl::*;

mod templates;

pub fn generate(
	project: &mut dyn ir::Project,
	library: &ir::Library,
) {
	for module in &library.modules {
		for item in &module.items {
			match item {
				ir::Item::Enum(enumeration) => {
					let contents = templates::enum_item(enumeration).to_string();
					let path = format!("{}/{}.rs", case::snake(&module.name), case::snake(&enumeration.name));
					project.write_file(&path, &contents).unwrap();
				}
				ir::Item::Error(error) => {
					let contents = templates::error_item(error).to_string();
					let path = format!("{}/{}.rs", case::snake(&module.name), case::snake(&error.name));
					project.write_file(&path, &contents).unwrap();
				}
				ir::Item::Function(func) => {
					let contents = templates::function_item(func).to_string();
					let path = format!("{}/{}.rs", case::snake(&module.name), case::snake(&func.name));
					project.write_file(&path, &contents).unwrap();
				}
				_ => unimplemented!(),
			}
		}
	}
}
