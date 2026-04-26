mod dom;
pub use dom::*;

pub trait Project {
	fn write_file(&mut self, path: &str, contents: &str) -> Result<(), std::io::Error>;
}

pub struct FileSystemProject {
	pub out_dir: std::path::PathBuf,
}
impl Project for FileSystemProject {
	fn write_file(&mut self, path: &str, contents: &str) -> Result<(), std::io::Error> {
		let path = self.out_dir.join(path);
		std::fs::create_dir_all(path.parent().unwrap())?;
		std::fs::write(path, contents)
	}
}
