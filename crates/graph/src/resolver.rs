//! Transitional `#[deprecated]` re-export shim (plan M18.1): the `#include`
//! directive parser and the include resolver were relocated verbatim to
//! `ariadne-lang-cpp` (`directive_parser` / `resolver`). This file contains
//! ONLY the re-exports so existing call sites keep compiling; it is removed
//! in M18.11.

#[deprecated(
    note = "moved to ariadne-lang-cpp (directive_parser); shim removed in M18.11"
)]
pub use ariadne_lang_cpp::directive_parser::{parse_include_directives, RawIncludeDirective};

#[deprecated(note = "moved to ariadne-lang-cpp (resolver); shim removed in M18.11")]
pub use ariadne_lang_cpp::resolver::IncludeResolver;
