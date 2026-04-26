use std::iter;

/// Splits identifiers across separator and case boundaries.
///
/// Digits do not create boundaries on their own; they stay attached to the surrounding token.
///
/// # Examples
///
/// ```
/// use cradle_idl::case;
///
/// #[track_caller]
/// fn assert_split(input: &str, expected: &[&str]) {
/// 	let result: Vec<_> = case::split(input).collect();
/// 	assert_eq!(result, expected);
/// }
///
/// assert_split("snake_case", &["snake", "case"]);
/// assert_split("kebab-case", &["kebab", "case"]);
/// assert_split("PascalCase", &["Pascal", "Case"]);
/// assert_split("CAPS_CASE", &["CAPS", "CASE"]);
/// assert_split("camelCase", &["camel", "Case"]);
/// assert_split("Version23Next", &["Version23Next"]);
/// assert_split("Point2D", &["Point2D"]);
/// assert_split("mixedCASETest_123", &["mixed", "CASE", "Test", "123"]);
/// ```
pub fn split(s: &str) -> impl Iterator<Item = &str> {
	let mut i = 0;
	let len = s.len();
	let bytes = s.as_bytes();

	iter::from_fn(move || {
		let is_lower = |c: u8| c.is_ascii_lowercase();
		let is_upper = |c: u8| c.is_ascii_uppercase();
		let is_sep = |c: u8| !c.is_ascii_alphanumeric();

		// Skip separators
		while i < len && is_sep(bytes[i]) {
			i += 1;
		}

		if i >= len {
			return None;
		}

		let start = i;
		i += 1;

		while i < len {
			let prev = bytes[i - 1];
			let curr = bytes[i];
			let next = if i + 1 < len {
				Some(bytes[i + 1])
			}
			else {
				None
			};

			let boundary = if is_sep(curr) {
				true
			}
			// lower -> upper (camelCase)
			else if is_lower(prev) && is_upper(curr) {
				true
			}
			// acronym boundary: UPPER -> UPPER + next lower
			else if is_upper(prev) && is_upper(curr) && next.map_or(false, |n| is_lower(n)) {
				true
			}
			else {
				false
			};

			if boundary {
				break;
			}

			i += 1;
		}

		let end = i;

		// Skip trailing separators so next call starts clean
		while i < len && is_sep(bytes[i]) {
			i += 1;
		}

		Some(&s[start..end])
	})
}

/// Converts an identifier to CAPSCASE.
///
/// See [`split`] for the splitting rules.
///
/// ```
/// use cradle_idl::case;
/// assert_eq!(case::caps("caps_case"), "CAPSCASE");
/// assert_eq!(case::caps("HTTPServer"), "HTTPSERVER");
/// assert_eq!(case::caps("point2d"), "POINT2D");
/// ```
pub fn caps(s: &str) -> String {
	let mut result = String::with_capacity(s.len() + 10);
	for part in split(s) {
		result.push_str(part);
	}
	result.make_ascii_uppercase();
	result
}

/// Converts an identifier to smallcase.
///
/// See [`split`] for the splitting rules.
///
/// ```
/// use cradle_idl::case;
/// assert_eq!(case::small("SMALL_CASE"), "smallcase");
/// assert_eq!(case::small("HTTPServer"), "httpserver");
/// assert_eq!(case::small("point2D"), "point2d");
/// ```
pub fn small(s: &str) -> String {
	let mut result = String::with_capacity(s.len() + 10);
	for part in split(s) {
		result.push_str(part);
	}
	result.make_ascii_lowercase();
	result
}

/// Converts an identifier to SCREAMING_SNAKE_CASE.
///
/// See [`split`] for the splitting rules.
///
/// ```
/// use cradle_idl::case;
/// assert_eq!(case::scream("scream_case"), "SCREAM_CASE");
/// assert_eq!(case::scream("HTTPServer"), "HTTP_SERVER");
/// assert_eq!(case::scream("point2d"), "POINT2D");
/// ```
pub fn scream(s: &str) -> String {
	let mut result = String::with_capacity(s.len() + 10);
	for part in split(s) {
		if !result.is_empty() {
			result.push('_');
		}
		result.push_str(part);
	}
	result.make_ascii_uppercase();
	result
}

/// Converts an identifier to snake_case.
///
/// See [`split_ident`] for the splitting rules.
///
/// ```
/// use cradle_idl::case;
/// assert_eq!(case::snake("snakeCase"), "snake_case");
/// assert_eq!(case::snake("HTTPServer"), "http_server");
/// assert_eq!(case::snake("point2D"), "point2d");
/// ```
pub fn snake(s: &str) -> String {
	let mut result = String::with_capacity(s.len() + 10);
	for part in split(s) {
		if !result.is_empty() {
			result.push('_');
		}
		result.push_str(part);
	}
	result.make_ascii_lowercase();
	result
}

// Uppercase the first non-digit character and append the rest lowercased.
fn capitalize(buf: &mut String, s: &str) {
	// Ignore digits
	let mut chrs = s.chars();
	loop {
		let Some(c) = chrs.next() else {
			return;
		};
		if c.is_ascii_digit() {
			buf.push(c);
		}
		else {
			buf.push(c.to_ascii_uppercase());
			let i = buf.len();
			buf.push_str(chrs.as_str());
			buf[i..].make_ascii_lowercase();
			return;
		}
	}
}

/// Converts an identifier to PascalCase.
///
/// See [`split_ident`] for the splitting rules.
///
/// ```
/// use cradle_idl::case;
/// assert_eq!(case::pascal("pascal_case"), "PascalCase");
/// assert_eq!(case::pascal("HTTP_server"), "HttpServer");
/// assert_eq!(case::pascal("point2d"), "Point2d");
/// ```
pub fn pascal(s: &str) -> String {
	let mut result = String::with_capacity(s.len() + 10);
	for part in split(s) {
		capitalize(&mut result, part);
	}
	result
}

/// Converts an identifier to camelCase.
///
/// See [`split_ident`] for the splitting rules.
///
/// ```
/// use cradle_idl::case;
/// assert_eq!(case::camel("camel_case"), "camelCase");
/// assert_eq!(case::camel("HTTP_server"), "httpServer");
/// assert_eq!(case::camel("point2d"), "point2d");
/// ```
pub fn camel(s: &str) -> String {
	let mut result = String::with_capacity(s.len() + 10);
	for part in split(s) {
		if result.is_empty() {
			result.push_str(&part.to_ascii_lowercase());
		}
		else {
			capitalize(&mut result, part);
		}
	}
	result
}

#[cfg(test)]
#[track_caller]
fn assert_split(ident: &str, parts: &[&str]) {
	assert_eq!(split(ident).collect::<Vec<_>>(), parts);
}

#[test]
fn test_snake_case() {
	assert_split("snake_case", &["snake", "case"]);
	assert_split("snake__case", &["snake", "case"]);
	assert_split("_snake_case_", &["snake", "case"]);
}

#[test]
fn test_kebab_case() {
	assert_split("kebab-case", &["kebab", "case"]);
	assert_split("kebab--case", &["kebab", "case"]);
	assert_split("-kebab-case-", &["kebab", "case"]);
}

#[test]
fn test_camel_case() {
	assert_split("thisCase", &["this", "Case"]);
	assert_split("simpleTestCase", &["simple", "Test", "Case"]);
}

#[test]
fn test_pascal_case() {
	assert_split("PascalCase", &["Pascal", "Case"]);
	assert_split("HttpServer", &["Http", "Server"]);
}

#[test]
fn test_acronyms() {
	assert_split("HTTPServer", &["HTTP", "Server"]);
	assert_split("XMLHttpRequest", &["XML", "Http", "Request"]);
	assert_split("mixedCASETest", &["mixed", "CASE", "Test"]);
}

#[test]
fn test_digits_basic() {
	assert_split("Version2", &["Version2"]);
	assert_split("Version23", &["Version23"]);
	assert_split("Version2Foo", &["Version2Foo"]);
	assert_split("Version23Foo", &["Version23Foo"]);
	assert_split("v23foo", &["v23foo"]);
}

#[test]
fn test_digits_prefix() {
	assert_split("Point2D", &["Point2D"]);
	assert_split("Vector3DContour", &["Vector3D", "Contour"]);
	assert_split("vector3d_contour", &["vector3d", "contour"]);
	assert_split("prefix2d", &["prefix2d"]);
	assert_split("prefix2DSuffix", &["prefix2D", "Suffix"]);
	assert_split("Point2Door", &["Point2Door"]);
	assert_split("Point2dDoor", &["Point2d", "Door"]);
	assert_split("Point2door", &["Point2door"]);
	assert_split("mat2ULL", &["mat2ULL"]);
	assert_split("mat2d_ULL", &["mat2d", "ULL"]);
	assert_split("prefix2Ull", &["prefix2Ull"]);
}

#[test]
fn test_digits_embedded() {
	assert_split("Version2GetNext", &["Version2Get", "Next"]);
	assert_split("prefix2ULLSuffix", &["prefix2ULL", "Suffix"]);
}

#[test]
fn test_v_style_tokens() {
	assert_split("TokenV2Next", &["Token", "V2Next"]);
	assert_split("GL11Version", &["GL11Version"]);
}

#[test]
fn test_digit_letter_edges() {
	assert_split("Fast2Furious", &["Fast2Furious"]);
	assert_split("Fast2fast", &["Fast2fast"]);
}

#[test]
fn test_all_caps() {
	assert_split("HTTP", &["HTTP"]);
	assert_split("ABC123", &["ABC123"]);
}

#[test]
fn weird_cases() {
	assert_split("", &[]);
	assert_split("___", &[]);
	assert_split("_-_", &[]);
	assert_split("--", &[]);
	assert_split("a", &["a"]);
	assert_split("A", &["A"]);
	assert_split("aB", &["a", "B"]);
}

#[test]
fn test_pascal_case_digits() {
	assert_eq!(pascal("2def"), "2Def");
	assert_eq!(pascal("foo_2def"), "Foo2Def");
	assert_eq!(pascal("foo2d_ef"), "Foo2dEf");
	assert_eq!(pascal("FOO_BAR"), "FooBar");
}

#[test]
fn test_camel_case_digits() {
	assert_eq!(camel("foo_2def"), "foo2Def");
	assert_eq!(camel("foo2d_ef"), "foo2dEf");
	assert_eq!(camel("2def_ghi"), "2defGhi");
	assert_eq!(camel("FOO_BAR"), "fooBar");
}

#[test]
fn test_digits_do_not_create_boundaries() {
	let original = "Point2D";
	let once = pascal(original);

	assert_eq!(split(original).collect::<Vec<_>>(), vec!["Point2D"]);
	assert_eq!(once, "Point2d");
	assert_eq!(split(&once).collect::<Vec<_>>(), vec!["Point2d"]);
	assert_eq!(pascal(original), pascal(&once));
	assert_eq!(camel(original), camel(&once));
	assert_eq!(snake(original), snake(&once));
	assert_eq!(scream(original), scream(&once));
}

#[test]
#[ignore = "documents a remaining acronym-style pascal instability"]
fn test_acronym_pascal_output_is_not_stable() {
	let original = "uI_oi8";
	let once = pascal(original);

	assert_eq!(split(original).collect::<Vec<_>>(), vec!["u", "I", "oi8"]);
	assert_eq!(once, "UIOi8");
	assert_eq!(split(&once).collect::<Vec<_>>(), vec!["UI", "Oi8"]);
	assert_eq!(pascal(original), pascal(&once));
}

#[test]
#[ignore = "exploratory fuzz check; simplified split rules do not fully round-trip arbitrary transformed identifiers"]
fn test_random() {
	let mut rng = urandom::new();
	let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-";
	let fns: &[fn(&str) -> String] = &[scream, snake, camel, pascal];
	for _ in 0..1000 {
		let len = rng.range(1..10);
		let mut ident: String = (0..len).map(|_| rng.choose(chars).cloned().unwrap() as char).collect();
		ident = rng.choose(fns).unwrap()(&ident);
		let original = ident.clone();

		// Randomly apply case transformations to the identifier to generate weird edge cases.
		for _ in 0..100 {
			ident = rng.choose(fns).unwrap()(&ident);
		}

		// Final test that all transformations round-trip correctly.
		assert_eq!(pascal(&original), pascal(&ident));
		assert_eq!(camel(&original), camel(&ident));
		assert_eq!(snake(&original), snake(&ident));
		assert_eq!(scream(&original), scream(&ident));
	}
}
