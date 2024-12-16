//! This is a simple
//! [Proc Macro](https://doc.rust-lang.org/reference/procedural-macros.html)
//! library, see [`include_data_uri`] for documentation.

use std::{env, fs::read};

use anyhow::Context;
use base64::{engine::general_purpose::STANDARD, Engine};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

/// Include a file as a data URI.
///
/// This macro functions similarly to [`std::include_str`] but instead of
/// including a file literally, it encodes it with [base64],
/// gets the file type with [`mime_guess`], and then uses those to create
/// a [data URI](https://en.wikipedia.org/wiki/Data_URI_scheme)
///
/// Like `include_str` the path given must be a string literal,
/// however unlike `include_str` it is NOT relative to the current file
/// and instead relative to the current working directory when your program
/// is compiling, which *should* be the current project's root folder.
/// This behavior is due to a limitation in Proc Macros, and is subject to
/// change in a future release of this crate.
///
/// # Examples
///
/// ```
/// use include_data_uri::include_data_uri;
/// use base64::{engine::general_purpose::STANDARD, Engine};
///
/// let expected = format!(
///     "data:text/plain;base64,{}",
///     STANDARD.encode("Hello World!")
/// );
/// assert_eq!(expected, include_data_uri!("tests/cases/text/hello_world.txt"));
/// ```
///
/// # Panics
///
/// This macro will panic (preventing compilation) under the following cases
///
/// - The path does not exist
/// - The path is not a file
/// - The file cannot be read or other I/O error
///
#[proc_macro]
pub fn include_data_uri(input: TokenStream) -> TokenStream {
	let path_str = parse_macro_input!(input as LitStr).value();

	// This is the only place where unwrap/expect should
	// be allowed outside of tests.
	// A proc macro SHOULD panic on error, preventing compilation
	#[allow(clippy::expect_used)]
	let uri = inner(&path_str).expect("macro runtime error");

	TokenStream::from(quote!(#uri))
}

/// This function serves as the core implementation of the macro,
/// mostly for error handling purposes.
fn inner(path_str: &str) -> anyhow::Result<String> {
	// like include_str! the path is relative to the current file
	let path = env::current_dir()?
		.join(path_str)
		.canonicalize()
		.with_context(|| format!("canonicalize path {path_str}"))?;

	// The mimetype is determined by the path
	// If there was no mimetype found default
	let mimetype = mime_guess::from_path(&path)
		.first()
		.unwrap_or("application/octet-stream".parse()?);

	// Read the file and encode it to standard base64
	let raw = read(path).with_context(|| "reading file")?;
	let data = STANDARD.encode(raw);

	// Format the data into a data URI
	let uri = format!("data:{mimetype};base64,{data}");

	Ok(uri)
}
