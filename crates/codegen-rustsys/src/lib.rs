use cradle_idl::*;

mod templates;

pub struct Version {
	pub major: u32,
	pub minor: u32,
	pub patch: u32,
}

pub struct Metadata {
	pub name: String,
	pub version: Version,
}

pub fn generate(
	project: &mut dyn ir::Project,
	library: &ir::Library,
) {
	{
		let contents = templates::cargo_toml().to_string();
		project.write_file("Cargo.toml", &contents).unwrap();
	}

	{
		let contents = templates::lib_rs(library).to_string();
		project.write_file("src/lib.rs", &contents).unwrap();
	}

	{
		project.write_file("src/_array.rs", include_str!("templates/_array.rs")).unwrap();
	}

	let lookup = library.lookup();

	for module in &library.modules {
		{
			let contents = templates::module_mod_rs(module).to_string();
			let path = format!("src/{}/mod.rs", case::snake(&module.name));
			project.write_file(&path, &contents).unwrap();
		}

		for item in &module.items {
			match item {
				ir::Item::Enum(enumeration) => {
					let contents = templates::enum_item(enumeration).to_string();
					let path = format!("src/{}/{}.rs", case::snake(&module.name), case::snake(&enumeration.name));
					project.write_file(&path, &contents).unwrap();
				}
				ir::Item::Error(error) => {
					let contents = templates::error_item(error).to_string();
					let path = format!("src/{}/{}.rs", case::snake(&module.name), case::snake(&error.name));
					project.write_file(&path, &contents).unwrap();
				}
				ir::Item::Function(func) => {
					let contents = templates::function_item(func, &lookup).to_string();
					let path = format!("src/{}/{}.rs", case::snake(&module.name), case::snake(&func.name));
					project.write_file(&path, &contents).unwrap();
				}
				ir::Item::Handle(handle) => {
					let contents = templates::handle_item(handle).to_string();
					let path = format!("src/{}/{}.rs", case::snake(&module.name), case::snake(&handle.name));
					project.write_file(&path, &contents).unwrap();
				}
				ir::Item::Struct(structure) => {
					let contents = templates::struct_item(structure, &lookup).to_string();
					let path = format!("src/{}/{}.rs", case::snake(&module.name), case::snake(&structure.name));
					project.write_file(&path, &contents).unwrap();
				}
			}
		}
	}
}
