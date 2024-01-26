//! This is a simple
//! [Proc Macro](https://doc.rust-lang.org/reference/procedural-macros.html)
//! library, see [`include_data_uri`] for documentation

use std::{fs::read, path::PathBuf};

use anyhow::Context;
use base64::{engine::general_purpose::STANDARD, Engine};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

/// This macro functions similarly to [`std::include_str`] but instead of
/// including a file literally, it encodes it with [base64],
/// gets the file type with [`mime_guess`], and then uses those to create
/// a [data URI](https://en.wikipedia.org/wiki/Data_URI_scheme)
///
/// Like [`std::include_str`] the path given must be a string literal,
/// however unlike [`std::include_str`] it is NOT relative to the current file
/// and instead relative to the current project's `src/` folder.
/// This is due to a limitation in Proc Macros.
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
/// assert_eq!(expected, include_data_uri!("test_cases/text/hello_world.txt"));
/// ```
///
/// # Panics
///
/// This macro will panic (preventing compilation) under the following cases
///
/// - The path does not exist
/// - The path is not a file
/// - The path cannot be read or other I/O error
/// - No mime type can be guessed for the path's extension
///
#[proc_macro]
pub fn include_data_uri(input: TokenStream) -> TokenStream {
	let path_str = parse_macro_input!(input as LitStr).value();

	// This is the only place where unwrap/expect should
	// be allowed outside of tests.
	// A proc macro SHOULD panic on error, preventing compilation
	#[allow(clippy::expect_used)]
	let uri = inner(path_str).expect("macro runtime error");

	TokenStream::from(quote!(#uri))
}

/// This function serves as the core implementation of the macro,
/// mostly for error handling purposes.
fn inner(path_str: impl Into<String>) -> anyhow::Result<String> {
	// like include_str! the path is relative to the current file
	let path_str = path_str.into();
	let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
	let path = src_dir.join(path_str);
	let path = path
		.canonicalize()
		.with_context(|| format!("canonicalize path {path:#?}"))?;

	// The mimetype is determined by the path
	let mimetype = mime_guess::from_path(&path);

	// If there was no mimetype found bail
	if mimetype.is_empty() {
		anyhow::bail!("Unknown mime type")
	}

	// Read the file and encode it to standard base64
	let raw = read(path).with_context(|| "reading file")?;
	let data = STANDARD.encode(raw);

	// Format the data into a data URI
	let uri = format!(
		"data:{};base64,{}",
		mimetype.first().unwrap_or_else(|| unreachable!()),
		data
	);

	Ok(uri)
}
