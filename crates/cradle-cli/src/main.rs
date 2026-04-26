use std::fs;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Language {
	Rust,
	RustSys,
}

fn main() {
	let app = clap::Command::new("cradle-cli")
		.version("0.1.0")
		.author("Casper")
		.about("A CLI tool for working with Cradle IDL files")
		.subcommand(
			clap::Command::new("check")
				.about("Checks the syntax of an IDL file")
				.arg(clap::Arg::new("file").required(true).help("The IDL file to check")),
		)
		.subcommand(
			clap::Command::new("generate")
				.about("Generates code from an IDL file")
				.arg(clap::Arg::new("file").required(true).help("The IDL file to generate code from"))
				.arg(clap::Arg::new("out-dir").short('o').long("out-dir").value_parser(clap::value_parser!(std::path::PathBuf)).required(true).help("The output directory for generated code"))
				.arg(clap::Arg::new("language").short('l').long("language").value_parser(["rust", "rust-sys"]).required(true).help("The target programming language for code generation")),
		);

	let matches = app.get_matches();
	match matches.subcommand() {
		Some(("check", m)) => check(m),
		Some(("generate", m)) => generate(m),
		_ => println!("No subcommand was used. Use --help for more information."),
	}
}

fn check(matches: &clap::ArgMatches) {
	let file = matches.get_one::<String>("file").expect("file");
	let content = fs::read_to_string(file).expect("Failed to read file");
	let pool = cradle_idl::StringPool::new();
	let input = pool.store(content);
	let idl = cradle_idl::ast::parse(input, 0);
}

fn generate(matches: &clap::ArgMatches) {
	let file = matches.get_one::<String>("file").expect("file");
	let language = match matches.get_one::<String>("language").expect("language").as_str() {
		"rust" => Language::Rust,
		"rust-sys" => Language::RustSys,
		lang => panic!("Unsupported language: {}", lang),
	};
	let content = fs::read_to_string(file).expect("Failed to read file");
	let pool = cradle_idl::StringPool::new();
	let input = pool.store(content);
	let (mut idl, mut errors) = cradle_idl::ast::parse(input, 0);
	if errors.is_empty() {
		cradle_idl::passes::qualify_names(&pool, &mut idl, &mut errors);

	}
	let mut errors = Vec::new();
	let ir = cradle_idl::passes::ast2ir(&pool, &[idl], &mut errors);
	dbg!(&errors);
	dbg!(&ir);
	let out_dir = matches.get_one::<std::path::PathBuf>("out-dir").unwrap().clone();
	let mut project = cradle_idl::ir::FileSystemProject { out_dir };

	match language {
		Language::Rust => {
			// cradle_codegen_rust::generate(&mut project, &idl);
		}
		Language::RustSys => {
			cradle_codegen_rustsys::generate(&mut project, &ir);
		}
	}
}
