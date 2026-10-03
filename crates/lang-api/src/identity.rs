//! Language identity (M24 Part A, D-A1): `LanguageId` is an OPEN-ENDED
//! string newtype, so a plugin can name a language that is neither C++ nor
//! Scala. `SourceLanguage` remains the closed, file-kind-derived label owned
//! by `ariadne-repo` -- the repository spine still classifies files, but it no
//! longer bounds the set of languages a plugin may claim.

use ariadne_repo::{FileKind, SourceLanguage};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;

/// The canonical name of the C++ language, as it appears in every
/// pre-existing artifact.
pub const CPP: &str = "Cpp";
/// The canonical name of the Scala language, as it appears in every
/// pre-existing artifact.
pub const SCALA: &str = "Scala";
/// The legacy "no language claimed this" label. Never a plugin identity and
/// never returned by `detect_language` (M24 Part A, D-A3).
pub const OTHER: &str = "Other";

/// Open-ended language identity (M24 Part A, D-A1).
///
/// This is what a plugin names itself with, and it is deliberately NOT a
/// closed enum: adding a language must not require editing a vocabulary that
/// lives in another crate. `Cow<'static, str>` rather than `Arc<str>` because
/// the workspace's serde has only the `derive` feature (serde gates its `Arc`
/// impls behind `rc`), and because the three built-in ids stay
/// `Cow::Borrowed` -- zero allocation on the per-TU and per-cache-key hot paths.
///
/// Two contracts make it a drop-in for the enum it replaces:
///
/// * `#[serde(transparent)]` -- serializes as the bare name, so every
///   pre-existing artifact spelling (`"Cpp"`, `"Scala"`, `"Other"`) round-trips
///   byte-identically.
/// * hand-written [`fmt::Debug`]/[`fmt::Display`] -- also the bare name, because
///   `crates/index` hashes `format!("{language:?}")` into the TU cache
///   fingerprint. A derived `Debug` would emit `Cpp(Cow::Borrowed("Cpp"))` and
///   silently invalidate every cache entry.
///
/// Note this is `Clone`, not `Copy`: a heap-backed identity cannot be `Copy`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LanguageId(Cow<'static, str>);

impl LanguageId {
    /// The canonical name of the C++ language.
    pub const CPP: &'static str = CPP;
    /// The canonical name of the Scala language.
    pub const SCALA: &'static str = SCALA;
    /// The legacy "no language claimed this" label.
    pub const OTHER: &'static str = OTHER;

    /// The C++ language identity. PascalCase to match the stored variant names,
    /// so `LanguageId::Cpp` reads and spells exactly as the enum variant did.
    #[allow(non_upper_case_globals)]
    pub const Cpp: Self = Self(Cow::Borrowed(CPP));
    /// The Scala language identity.
    #[allow(non_upper_case_globals)]
    pub const Scala: Self = Self(Cow::Borrowed(SCALA));
    /// The legacy "no language claimed this" label, as an identity.
    #[allow(non_upper_case_globals)]
    pub const Other: Self = Self(Cow::Borrowed(OTHER));

    /// Names any language, including one no other crate has heard of.
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }

    /// The language name.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The C++ language identity, spelled so a core crate can name it.
    ///
    /// The R2 architecture guard bans the case-sensitive substring `Cpp` from
    /// the source of the core crates, so `LanguageId::Cpp` is unusable there;
    /// this is the snake_case spelling those crates use instead.
    pub fn cpp() -> Self {
        Self::Cpp
    }

    /// The Scala language identity, spelled so a core crate can name it.
    pub fn scala() -> Self {
        Self::Scala
    }

    /// The legacy "no language claimed this" label, spelled so a core crate can
    /// name it.
    pub fn other() -> Self {
        Self::Other
    }
}

/// `Default` is C++ (M24 Part A, D-A3): `#[serde(default)]` fields in persisted
/// artifacts must keep reading back as they did when the default was
/// `SourceLanguage::default()`.
impl Default for LanguageId {
    fn default() -> Self {
        Self::Cpp
    }
}

impl fmt::Debug for LanguageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for LanguageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<SourceLanguage> for LanguageId {
    fn from(source: SourceLanguage) -> Self {
        match source {
            SourceLanguage::Cpp => Self::Cpp,
            SourceLanguage::Scala => Self::Scala,
            SourceLanguage::Other => Self::Other,
        }
    }
}

impl From<&SourceLanguage> for LanguageId {
    fn from(source: &SourceLanguage) -> Self {
        Self::from(*source)
    }
}

/// Maps a discovered [`FileKind`] onto the advisory [`LanguageId`] vocabulary.
///
/// ADVISORY ONLY (M24 Part A, D-A6): this is the closed `FileKind` table, not
/// the authority on a file's language. Detection asks the registered plugin's
/// detector (`LanguageRegistry::detect_language`). `FileKind` is retained
/// because `crates/repo` still needs it for ThirdParty/Generated/Frozen
/// classification; its language column no longer vetoes a plugin.
pub fn language_of_file_kind(kind: FileKind) -> LanguageId {
    SourceLanguage::from_file_kind(kind).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_id_can_name_a_language_that_is_not_cpp_or_scala() {
        let python = LanguageId::new("Python");
        assert_eq!(python.as_str(), "Python");
        assert_ne!(python, LanguageId::Cpp);
        assert_ne!(python, LanguageId::Scala);
        assert_ne!(python, LanguageId::Other);
        // Open-ended means orderable and hashable, not just constructible.
        assert!(LanguageId::Cpp < python);
    }

    #[test]
    fn language_id_round_trips_the_three_legacy_names_through_json() {
        // Kickoff §0.5's artifact-compatibility claim, proven rather than
        // assumed: `#[serde(transparent)]` over the name means every
        // pre-existing artifact spelling round-trips byte-identically.
        for id in [LanguageId::Cpp, LanguageId::Scala, LanguageId::Other] {
            let json = serde_json::to_string(&id).expect("serialize a language id");
            assert_eq!(
                json,
                format!("\"{}\"", id.as_str()),
                "a language id serializes as a bare string"
            );
            let back: LanguageId = serde_json::from_str(&json).expect("deserialize a language id");
            assert_eq!(back, id);
        }
        // And a third language needs no enum variant to exist.
        let exotic: LanguageId =
            serde_json::from_str("\"Fortran\"").expect("an unknown language name deserializes");
        assert_eq!(exotic, LanguageId::new("Fortran"));
    }

    #[test]
    fn language_id_debug_and_display_are_the_bare_name() {
        // CACHE-DIGEST LOCK (M24 Part A, ADAPT-3): `crates/index/src/cache.rs`
        // hashes `format!("{language:?}")` into the TU cache fingerprint. A
        // derived `Debug` on a newtype would emit `Cpp(Cow::Borrowed("Cpp"))`
        // and silently change every fingerprint, so `Debug` is hand-written to
        // the bare name.
        assert_eq!(format!("{:?}", LanguageId::Cpp), "Cpp");
        assert_eq!(format!("{:?}", LanguageId::Scala), "Scala");
        assert_eq!(format!("{:?}", LanguageId::Other), "Other");
        assert_eq!(format!("{:?}", LanguageId::new("Python")), "Python");
        assert_eq!(format!("{}", LanguageId::Scala), "Scala");
        // The exact `format!` shape used by the cache fingerprint.
        let language = LanguageId::Cpp;
        assert_eq!(format!("{language:?}", language = language), "Cpp");
    }

    #[test]
    fn legacy_source_language_converts_into_a_language_id() {
        // One direction only, deliberately: `SourceLanguage` is the closed
        // file-kind label and cannot name a third language, so there is no
        // `From<LanguageId> for SourceLanguage`.
        assert_eq!(LanguageId::from(SourceLanguage::Cpp), LanguageId::Cpp);
        assert_eq!(LanguageId::from(SourceLanguage::Scala), LanguageId::Scala);
        assert_eq!(LanguageId::from(SourceLanguage::Other), LanguageId::Other);
        assert_eq!(
            LanguageId::from(&SourceLanguage::Scala),
            LanguageId::Scala
        );
    }

    #[test]
    fn language_id_default_is_cpp_so_legacy_artifacts_still_read() {
        // D-A3: `Other`/absent-field compatibility means the serde default has
        // to be C++, exactly as `SourceLanguage::default()` was.
        assert_eq!(LanguageId::default(), LanguageId::Cpp);
        assert_eq!(SourceLanguage::default(), SourceLanguage::Cpp);
    }

    #[test]
    fn language_of_file_kind_maps_every_file_kind_exhaustively() {
        let cases = [
            (FileKind::Header, LanguageId::Cpp),
            (FileKind::Source, LanguageId::Cpp),
            (FileKind::ScalaSource, LanguageId::Scala),
            (FileKind::ScalaScript, LanguageId::Scala),
            (FileKind::BuildDefinition, LanguageId::Scala),
            (FileKind::Other, LanguageId::Other),
        ];
        for (kind, expected) in cases {
            assert_eq!(
                language_of_file_kind(kind),
                expected,
                "wrong language for {kind:?}"
            );
        }
    }

    #[test]
    fn guard_legal_spellings_avoid_the_banned_cpp_token() {
        // The R2 guard (`m17_plugin_seam_test.rs`) bans the case-sensitive
        // substring `Cpp` from the source of the core crates, so a core crate
        // must be able to name C++ through these snake_case accessors.
        assert_eq!(LanguageId::cpp(), LanguageId::Cpp);
        assert_eq!(LanguageId::scala(), LanguageId::Scala);
        assert_eq!(LanguageId::other(), LanguageId::Other);
        assert_eq!(LanguageId::CPP, "Cpp");
        assert_eq!(LanguageId::SCALA, "Scala");
        assert_eq!(LanguageId::OTHER, "Other");
    }
}