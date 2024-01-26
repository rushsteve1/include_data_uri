//! This module contains integration tests against [`include_data_uri`!]
//! These should mirror the unit tests in [lib.rs](../src/lib.rs)

use base64::{engine::general_purpose::STANDARD, Engine};
use include_data_uri::include_data_uri;

#[test]
fn text() {
	let expected = format!(
		"data:text/plain;base64,{}",
		STANDARD.encode(include_str!("cases/text/hello_world.txt"))
	);
	assert_eq!(
		expected,
		include_data_uri!("tests/cases/text/hello_world.txt")
	);
}

#[test]
fn image() {
	let expected = format!(
		"data:image/jpeg;base64,{}",
		include_str!("cases/jpeg/naw.base64")
	);
	assert_eq!(expected, include_data_uri!("tests/cases/jpeg/naw.jpeg"));
}

#[test]
fn code() {
	let expected = format!(
		"data:text/x-rust;base64,{}",
		STANDARD.encode(include_str!("macro.rs"))
	);
	assert_eq!(expected, include_data_uri!("tests/macro.rs"))
}
