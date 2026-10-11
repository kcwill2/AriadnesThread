//! Bounded bincode decode shared by every artifact reader in this module.
//!
//! Why (AR-03 / audit S5): the bare `bincode::deserialize*` free functions
//! run UNLIMITED - and the slice form even drops any configured limit to
//! `Infinite` internally (bincode 1.3.3 `internal::deserialize_seed`) - so a
//! corrupt or attacker-authored `artifacts/*.bin` blob could declare a
//! string or collection length far beyond its own bytes and drive the
//! allocator into a capacity-overflow abort. Routing every read through
//! `Options::deserialize_from` with `with_limit` makes bincode reject an
//! over-declared length BEFORE it allocates, surfacing the blob as the same
//! load `Err` any unreadable artifact already produces.
//!
//! Wire compatibility: the option set mirrors `bincode::deserialize`
//! exactly (fixed-int little-endian lengths, trailing bytes accepted) minus
//! the unlimited bytes, so honest artifacts decode value-identically.
//!
//! Sizing: the limit counts the bytes one decode consumes (every wire byte
//! plus every declared string buffer). Observed honest payload scales are
//! recorded in `tasks/ar-03-handoff-journal.md`: fixture-scale blobs are
//! 208 bytes (`repository/files.bin`) and 36 bytes (`load/report.bin`); the
//! pre-DP-2 stage-5 save-span transient was ~4096 MiB for BOTH report forms
//! combined, and a warm `report.bin` measured ~0.9 MiB at N=1280 files
//! (tasks/stage5-collateral-memory-ceiling-t0a-evidence.md secs. 4 and 6).
//! 16 GiB keeps at least 8x margin over the worst blob scale ever measured
//! on this project while sitting many orders of magnitude below any length
//! an attacker can encode, so hostile blobs fail fast and honest reads stay
//! untouched.

use anyhow::Result;
use bincode::Options as _;
use serde::de::DeserializeOwned;
use std::io::Read;

/// Maximum bytes a single artifact blob may consume during one decode.
pub(super) const DECODE_BYTE_LIMIT: u64 = 16 * 1024 * 1024 * 1024;

/// Bounded decode of a fully-read artifact blob (the common case).
pub(super) fn deserialize_bounded<T>(bytes: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    deserialize_bounded_reader(bytes)
}

/// Bounded decode straight from a reader, for the one large streaming blob
/// (`load/report.bin`) whose reader path must not materialize it whole.
pub(super) fn deserialize_bounded_reader<T, R>(reader: R) -> Result<T>
where
    T: DeserializeOwned,
    R: Read,
{
    Ok(bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .with_limit(DECODE_BYTE_LIMIT)
        .deserialize_from(reader)?)
}
