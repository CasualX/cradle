use super::super::*;

fn assert_idl(input: &str, expected_errors: &[ErrorKind]) {
	let (_file, errors) = parse(input, 0);
	for err in &errors {
		let resolved = err.span.resolve("input.idl", input);
		eprintln!("Error: {:?} at {}:{}:{}\n  -> {:?}", err.kind, resolved.file_name, resolved.line_start, resolved.column_start, resolved.text);
	}
	let error_kinds: Vec<_> = errors.into_iter().map(|e| e.kind).collect();
	assert_eq!(error_kinds, expected_errors, "Expected errors {:?}, but found {:?}", expected_errors, error_kinds);
}

#[test]
fn test_smoke() {
	let input = include_str!("smoke.idl");
	assert_idl(input, &[]);
}
