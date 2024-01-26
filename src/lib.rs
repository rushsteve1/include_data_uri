//! This is a simple
//! [Proc Macro](https://doc.rust-lang.org/reference/procedural-macros.html)
//! library, see [include_data_uri] for documentation

use std::{fs::read, path::PathBuf};

use anyhow::Context;
use base64::{engine::general_purpose::STANDARD, Engine};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

/// This macro functions similarly to [std::include_str] but instead of
/// including a file literally, it encodes it with [base64] and,
/// gets the file type with [mime_guess], and then uses those to create
/// a [data URI](https://en.wikipedia.org/wiki/Data_URI_scheme)
///
/// Like [std::include_str] the path given must be a string literal and is
/// relative to the current file.
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
#[proc_macro]
pub fn include_data_uri(input: TokenStream) -> TokenStream {
    let path_str = parse_macro_input!(input as LitStr).value();
    let uri = inner(path_str).unwrap();
    TokenStream::from(quote!(#uri))
}

/// This function serves as the core implementation of the macro,
/// mostly for testing purposes.
fn inner(path_str: impl Into<String>) -> anyhow::Result<String> {
    // like include_str! the path is relative to the current file
    let path_str = path_str.into();
    let this_file = PathBuf::from(file!());
    let this_folder = this_file
        .parent()
        .ok_or(anyhow::anyhow!("no parent folder"))?;
    let path = this_folder
        .join(&path_str)
        .canonicalize()
        .with_context(|| format!("canonicalize path {path_str}"))?;

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

#[cfg(test)]
mod tests {
    use crate::inner;
    use base64::{engine::general_purpose::STANDARD, Engine};

    #[test]
    fn text() {
        let expected = format!(
            "data:text/plain;base64,{}",
            STANDARD.encode(include_str!("test_cases/text/hello_world.txt"))
        );
        assert_eq!(expected, inner("test_cases/text/hello_world.txt").unwrap());
    }

    #[test]
    fn image() {
        let expected = format!(
            "data:image/jpeg;base64,{}",
            include_str!("test_cases/jpeg/naw.base64")
        );
        assert_eq!(expected, inner("test_cases/jpeg/naw.jpeg").unwrap());
    }
}
