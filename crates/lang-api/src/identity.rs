//! Language identity re-export shim (AR-27a home move).
//!
//! M24 Part A (D-A1) introduced `LanguageId` as an OPEN-ENDED string newtype
//! so a plugin can name a language that is neither C++ nor Scala, while
//! `SourceLanguage` remained a closed enum owned by `ariadne-repo`. AR-27a
//! unified the two: the newtype is the one language type, HOMED in
//! `ariadne_repo::identity`, and `SourceLanguage` is a `pub type` alias of it
//! there. This module keeps every historical import path
//! (`ariadne_lang_api::identity::LanguageId`, and through it
//! `ariadne_lang::LanguageId`) resolving, and retains only what genuinely
//! belongs to the plugin vocabulary seam: the advisory [`language_of_file_kind`]
//! helper. The serde/Debug/Ord contracts live with the type in the spine
//! crate; see `crates/repo/src/identity.rs`.

pub use ariadne_repo::identity::{LanguageId, CPP, OTHER, SCALA};

use ariadne_repo::FileKind;

/// Maps a discovered [`FileKind`] onto the advisory [`LanguageId`] vocabulary.
///
/// ADVISORY ONLY (M24 Part A, D-A6): this is the closed `FileKind` table, not
/// the authority on a file's language. Detection asks the registered plugin's
/// detector (`LanguageRegistry::detect_language`). `FileKind` is retained
/// because `crates/repo` still needs it for ThirdParty/Generated/Frozen
/// classification; its language column no longer vetoes a plugin.
pub fn language_of_file_kind(kind: FileKind) -> LanguageId {
    LanguageId::from_file_kind(kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_of_file_kind_maps_every_file_kind_exhaustively() {
        let cases = [
            (FileKind::Header, LanguageId::Cpp),
            (FileKind::Source, LanguageId::Cpp),
            (FileKind::ScalaSource, LanguageId::Scala),
            (FileKind::ScalaScript, LanguageId::Scala),
            (FileKind::BuildDefinition, LanguageId::Scala),
            (FileKind::Other, LanguageId::Other),
            (FileKind::RustSource, LanguageId::Rust),
        ];
        for (kind, expected) in cases {
            assert_eq!(
                language_of_file_kind(kind),
                expected,
                "wrong language for {kind:?}"
            );
        }
    }
}
