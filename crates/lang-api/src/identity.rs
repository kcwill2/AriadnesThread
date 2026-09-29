//! Language identity (plan §3.2): the neutral `LanguageId` is the **existing**
//! `ariadne_repo::SourceLanguage` enum — no second language enum is invented.

use ariadne_repo::{FileKind, SourceLanguage};

pub use ariadne_repo::SourceLanguage as LanguageId;

/// Maps a discovered [`FileKind`] onto the [`LanguageId`] vocabulary.
pub fn language_of_file_kind(kind: FileKind) -> LanguageId {
    SourceLanguage::from_file_kind(kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_id_is_the_repo_source_language() {
        // Compile-time proof that no second enum exists: LanguageId *is*
        // ariadne_repo::SourceLanguage.
        fn accepts_repo_language(id: ariadne_repo::SourceLanguage) -> LanguageId {
            id
        }
        assert_eq!(
            accepts_repo_language(ariadne_repo::SourceLanguage::Cpp),
            LanguageId::Cpp
        );
    }

    #[test]
    fn maps_every_file_kind_exhaustively() {
        let cases = [
            (FileKind::Header, SourceLanguage::Cpp),
            (FileKind::Source, SourceLanguage::Cpp),
            (FileKind::ScalaSource, SourceLanguage::Scala),
            (FileKind::ScalaScript, SourceLanguage::Scala),
            (FileKind::BuildDefinition, SourceLanguage::Scala),
            (FileKind::Other, SourceLanguage::Other),
        ];
        // Exhaustive `match` guards the mapping: a new FileKind variant
        // fails to compile here until it is classified.
        for (kind, expected) in cases {
            let mapped = match kind {
                FileKind::Header | FileKind::Source => language_of_file_kind(kind),
                FileKind::ScalaSource | FileKind::ScalaScript | FileKind::BuildDefinition => {
                    language_of_file_kind(kind)
                }
                FileKind::Other => language_of_file_kind(kind),
            };
            assert_eq!(mapped, expected, "wrong language for {:?}", kind);
        }
    }

    #[test]
    fn default_language_is_cpp_for_legacy_artifacts() {
        assert_eq!(SourceLanguage::default(), SourceLanguage::Cpp);
    }
}
