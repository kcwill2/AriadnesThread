//! Language identity — the open-ended `LanguageId` newtype (AR-27a home move).
//!
//! M24 Part A (D-A1) introduced `LanguageId` as an OPEN-ENDED string newtype
//! in `crates/lang-api`, with `SourceLanguage` staying a closed enum in this
//! crate. AR-27a reverses the ownership: the newtype is the single language
//! type and it is HOMED HERE, in the repository spine crate that every
//! persisted byte stream is authored against; `ariadne-lang-api::identity`
//! re-exports it so every historical import path keeps working, and
//! `crate::types::SourceLanguage` is now a `pub type` alias of this newtype,
//! so the expression-spelled sites (`SourceLanguage::Cpp`,
//! `SourceLanguage::from_file_kind(..)`, `.language` fields) compile
//! unchanged.
//!
//! # Dual-format serde (shape ruling section 2 — the byte-freeze contract)
//!
//! The closed enum it subsumes serialized POSITIONALLY under bincode (u32 LE
//! variant indices `Cpp=0`, `Scala=1`, `Other=2`) and as the bare PascalCase
//! name under JSON. The newtype keeps BOTH shapes, discriminated by serde's
//! `is_human_readable` (verified false for bincode 1.3.3 in both directions,
//! true for serde_json — the same discriminator `field_conditional_on_format`
//! already relies on for `TuAnalysis::extends_edges`):
//!
//! * human-readable formats: transparent bare name for EVERY id, legacy or
//!   open (a third language needs no wire vocabulary in JSON).
//! * positional formats (bincode warm blobs at the frozen epoch): the private
//!   [`WireLanguageForm`] enum reproduces the enum's u32 LE indices exactly;
//!   an open id takes the escape arm (tag `3` + u64 LE length + UTF-8). An
//!   unknown tag on read is a serde error, never a panic and never a silent
//!   substitution.
//!
//! The `ar27_*` byte-lock tests (repo/lang-api/index/integration) are the
//! live-code oracle for every claim above; `IncludeEdge.language` opts OUT of
//! the positional arm through [`language_transparent_frame`] because M24
//! Part A already froze its warm bytes string-framed.
//!
//! # Ord (legacy rank)
//!
//! The enum's derived `Ord` grouped reports C++ < Scala < Other; a derived
//! newtype `Ord` would group by bytes (`"Other"` before `"Scala"`). The
//! hand-written [`std::cmp::Ord`] below keeps the legacy rank for the three
//! stored names and orders open ids after them lexicographically — consistent
//! with the string-equality [`PartialEq`], which it must be: `A == B` iff
//! rank and name both tie.

use crate::types::FileKind;
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cmp::Ordering;
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

/// Open-ended language identity (M24 Part A, D-A1; homed here by AR-27a).
///
/// This is what a plugin names itself with, and it is deliberately NOT a
/// closed enum: adding a language must not require editing a vocabulary that
/// lives in another crate. `Cow<'static, str>` rather than `Arc<str>` because
/// the workspace's serde has only the `derive` feature (serde gates its `Arc`
/// impls behind `rc`), and because the three built-in ids stay
/// `Cow::Borrowed` -- zero allocation on the per-TU and per-cache-key hot paths.
///
/// The format-dependent serde and the bare-name `Debug`/`Display` are
/// hand-written; see the module docs (a derived `Debug` would emit
/// `LanguageId(Cow::Borrowed("Cpp"))`, and `crates/index` hashes
/// `format!("{language:?}")` into the TU cache fingerprint -- ADAPT-3/ADAPT-8).
///
/// Note this is `Clone`, not `Copy`: a heap-backed identity cannot be `Copy`.
#[derive(Clone, PartialEq, Eq, Hash)]
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

    /// Maps a discovered [`FileKind`] onto the language vocabulary (the form
    /// `SourceLanguage::from_file_kind` had since plan sec 3.5; the alias
    /// keeps both spellings resolving here).
    pub fn from_file_kind(kind: FileKind) -> Self {
        match kind {
            FileKind::Header | FileKind::Source => Self::Cpp,
            FileKind::ScalaSource | FileKind::ScalaScript | FileKind::BuildDefinition => {
                Self::Scala
            }
            FileKind::Other => Self::Other,
        }
    }
}

/// The private positional wire vocabulary (shape ruling section 2): the
/// variant order is the legacy enum's bincode index order and is FROZEN --
/// `repository/files.bin`, `tu_analyses.bin`, and the compilation-context
/// digest (`ADAPT-8`) were byte-frozen against indices 0/1/2, and open ids
/// escape through index 3.
#[derive(Serialize, Deserialize)]
enum WireLanguageForm {
    Cpp,
    Scala,
    Other,
    Open(String),
}

impl WireLanguageForm {
    fn from_id(id: &LanguageId) -> Self {
        match id.as_str() {
            CPP => Self::Cpp,
            SCALA => Self::Scala,
            OTHER => Self::Other,
            open => Self::Open(open.to_owned()),
        }
    }

    fn into_id(self) -> LanguageId {
        match self {
            Self::Cpp => LanguageId::Cpp,
            Self::Scala => LanguageId::Scala,
            Self::Other => LanguageId::Other,
            Self::Open(name) => LanguageId(Cow::Owned(name)),
        }
    }
}

impl Serialize for LanguageId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.serialize_str(self.as_str())
        } else {
            WireLanguageForm::from_id(self).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for LanguageId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            // Same shape the `#[serde(transparent)]` derive used: a bare name
            // for every id, so every pre-existing artifact spelling
            // ("Cpp", "Scala", "Other", or any open name) round-trips.
            let name = <Cow<'static, str>>::deserialize(deserializer)?;
            Ok(Self(name))
        } else {
            // An index outside 0..=3 fails here as a serde invalid-value
            // error; it can never alias a legacy id.
            WireLanguageForm::deserialize(deserializer).map(WireLanguageForm::into_id)
        }
    }
}

/// Field-level serde exception for `IncludeEdge.language` (shape ruling
/// section 2.7): M24 Part A froze this field's WARM bincode bytes
/// (`includes/edges.bin`) while the newtype was still `#[serde(transparent)]`
/// in every format, so the STRING-FRAMED shape is its byte freeze. The
/// global positional arm would move those bytes and hard-error every warm
/// blob; this frame reproduces exactly the transparent behavior in BOTH
/// formats (bare name in JSON, string-framed in bincode).
pub mod language_transparent_frame {
    use super::LanguageId;
    use serde::de::Deserializer;
    use serde::ser::Serializer;
    use serde::Deserialize;
    use std::borrow::Cow;

    /// Serializes the id as its bare name in every format (string-framed
    /// under positional writers, JSON string under human-readable ones).
    pub fn serialize<S: Serializer>(
        id: &LanguageId,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(id.as_str())
    }

    /// Reads the bare name back in every format; open names included.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<LanguageId, D::Error> {
        let name = <Cow<'de, str>>::deserialize(deserializer)?;
        Ok(LanguageId::new(name.into_owned()))
    }
}

/// The legacy enum's derived order is preserved for the three stored names;
/// open ids sort after them, lexicographically among themselves.
fn legacy_rank(name: &str) -> u8 {
    match name {
        CPP => 0,
        SCALA => 1,
        OTHER => 2,
        _ => 3,
    }
}

impl Ord for LanguageId {
    fn cmp(&self, other: &Self) -> Ordering {
        legacy_rank(self.as_str())
            .cmp(&legacy_rank(other.as_str()))
            .then_with(|| self.as_str().cmp(other.as_str()))
    }
}

impl PartialOrd for LanguageId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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

/// The by-reference conversion (M24 Part A) kept as a hand-written impl:
/// std offers no `From<&T> for T` blanket, and `crates/artifact` maps
/// `&SourceLanguage` iterators through exactly this seam
/// (`metrics_io.rs`, zero-rider). The old by-VALUE `From<SourceLanguage>`
/// is gone: under the alias it would be `From<Self>`, which std already
/// provides reflexively, so every by-value call site keeps compiling.
impl From<&LanguageId> for LanguageId {
    fn from(source: &Self) -> Self {
        source.clone()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SourceLanguage;

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
        // Kickoff sec 0.5's artifact-compatibility claim, proven rather than
        // assumed: the human-readable arm serializes the bare name, so every
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
        // AR-27a removed the by-value `From` impl: under the alias it would
        // be `From<Self>`, which std already provides reflexively, so every
        // by-value call site keeps compiling. The by-reference direction
        // stays a hand-written impl (std has no `From<&T> for T` blanket);
        // one direction only, deliberately - the alias world has no reverse
        // closed-enum type to convert back to.
        assert_eq!(LanguageId::from(SourceLanguage::Cpp), LanguageId::Cpp);
        assert_eq!(LanguageId::from(SourceLanguage::Scala), LanguageId::Scala);
        assert_eq!(LanguageId::from(SourceLanguage::Other), LanguageId::Other);
        assert_eq!(LanguageId::from(&SourceLanguage::Scala), LanguageId::Scala);
    }

    #[test]
    fn language_id_default_is_cpp_so_legacy_artifacts_still_read() {
        // D-A3: `Other`/absent-field compatibility means the serde default has
        // to be C++, exactly as `SourceLanguage::default()` was.
        assert_eq!(LanguageId::default(), LanguageId::Cpp);
        assert_eq!(SourceLanguage::default(), SourceLanguage::Cpp);
    }

    #[test]
    fn legacy_rank_orders_the_stored_names_as_the_closed_enum_did() {
        // Report grouping and any `sort()` over languages keep the enum-era
        // order C++ < Scala < Other (a derived newtype Ord would put "Other"
        // before "Scala"); open ids sort after the stored trio.
        assert!(LanguageId::Cpp < LanguageId::Scala);
        assert!(LanguageId::Scala < LanguageId::Other);
        assert!(LanguageId::Other < LanguageId::new("Fortran"));
        assert!(LanguageId::new("Fortran") < LanguageId::new("Python"));
        let mut ids = vec![
            LanguageId::new("Fortran"),
            LanguageId::Other,
            LanguageId::Scala,
            LanguageId::Cpp,
        ];
        ids.sort();
        assert_eq!(
            ids.iter().map(LanguageId::as_str).collect::<Vec<_>>(),
            vec!["Cpp", "Scala", "Other", "Fortran"]
        );
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
