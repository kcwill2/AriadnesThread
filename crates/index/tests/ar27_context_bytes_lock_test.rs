//! AR-27a byte-oracle lock (index home): the OLD enum-era
//! `CompilationContext` bincode bytes and the `context_hash` digest they
//! produce (ADAPT-8: `SHA256(bincode(ctx))` must not move).
//!
//! RED-FREE GREEN-FIRST (shape ruling section 5 step 1): the fixture came
//! from the live `bincode::serialize` of the UNTOUCHED types BEFORE the
//! alias landed (now-deleted temp printer) and this file's own load/rewrite
//! equality proved it against the current code green pre-mutation. The same
//! assertions MUST stay green after the mutation: they are the oracle for
//! the ADAPT-8 byte-freeze - the dual-format positional arm keeps the u32
//! language index, so the byte stream, its SHA256 (`context_hash`), and the
//! fingerprint chain through `TuCacheFingerprint::compute` stay identical.
//! `crates/index/src/cache.rs` itself is UNEDITED (epoch pin 2.2.0 included);
//! this file is NEW-FILE-ONLY and calls the public digest seam.

use ariadne_build::CompilationContext;
use ariadne_index::cache::{TuCacheFingerprint, CURRENT_ANALYSIS_VERSION};
use ariadne_repo::FileId;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// The context from the cache.rs test fixture shape (same fields), serialized
/// with the old enum: the trailing `[0,0,0,0]` is the pinned u32 LE Cpp
/// index. Any byte shift here would change every `context_hash` in the
/// field - the exact drift ADAPT-8 forbids.
const OLD_CTX_BYTES: &[u8] = &[
    4, 0, 0, 0, 0, 0, 0, 0, 116, 101, 115, 116, 7, 0, 0, 0, 0, 0, 0, 0, 102, 97, 107, 101, 45, 99,
    99, 1, 6, 0, 0, 0, 0, 0, 0, 0, 120, 56, 54, 95, 54, 52, 1, 5, 0, 0, 0, 0, 0, 0, 0, 99, 43, 43,
    50, 48, 1, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 68, 69, 66, 85, 71, 61, 49, 1, 0, 0, 0,
    0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 47, 117, 115, 114, 47, 105, 110, 99, 108, 117, 100, 101,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0,
    0, 45, 79, 50, 0, 0, 0, 0,
];

/// `context_hash` over `OLD_CTX_BYTES` captured pre-mutation:
/// `SHA256(bincode(ctx))` - the ADAPT-8 invariant witness.
const OLD_CTX_HASH: &str = "ee371bb91fcf2583af7a5c6c4c0e059a776337d30746447c33cc1c0643b22989";

fn context() -> CompilationContext {
    CompilationContext {
        name: "test".into(),
        compiler: "fake-cc".into(),
        target_arch: Some("x86_64".into()),
        language_standard: Some("c++20".into()),
        defines: vec!["DEBUG=1".into()],
        include_dirs: vec![PathBuf::from("/usr/include")],
        system_include_dirs: vec![],
        forced_includes: vec![],
        raw_flags: vec!["-O2".into()],
        language: Default::default(),
    }
}

#[test]
fn old_context_blob_loads_and_rewrites_byte_identically() {
    let decoded: CompilationContext = bincode::deserialize(OLD_CTX_BYTES)
        .unwrap_or_else(|err| panic!("old translation-units.bin-era context must load: {err}"));
    assert_eq!(decoded, context());
    assert_eq!(
        bincode::serialize(&decoded).unwrap(),
        OLD_CTX_BYTES,
        "the CURRENT code rewrites the old context bytes byte-identically \
         (contexts/translation-units.bin + context_hash stay frozen)"
    );
}

#[test]
fn context_hash_digest_is_byte_frozen_adapt8() {
    // The digest seam: `context_hash_of` is private; `compute` exposes the
    // digest it derives. Pinned three ways: to the captured hex, to
    // SHA256(fixture bytes), and to the value the current code derives from
    // the loaded context.
    assert_eq!(
        format!("{:x}", Sha256::digest(OLD_CTX_BYTES)),
        OLD_CTX_HASH,
        "the pinned digest is the SHA256 of the pinned bytes"
    );
    let fingerprint = TuCacheFingerprint::compute(
        CURRENT_ANALYSIS_VERSION,
        &context(),
        &FileId::from_relative_path("src/main.cpp"),
        "src_hash",
        &[],
        None,
        ariadne_lang_api::identity::LanguageId::cpp(),
        "test-plugin",
    );
    assert_eq!(
        fingerprint.context_hash, OLD_CTX_HASH,
        "ADAPT-8: compute() over the current types still derives the old context_hash"
    );
}
