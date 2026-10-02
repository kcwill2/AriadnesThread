package com.example
package legacy

import com.example.legacy.records.{AuditTrail, AuditSeal}

// Shared aggregation file of the scala-isolation fixture (D-M23-9). The
// package object below makes this file the package's aggregation candidate
// for the wired Scala planner; the heavy top-level symbols beside it are the
// extraction payload. The two stacked package clauses join to the namespace
// `com.example.legacy` exactly as the offline parser documents (consecutive
// package clauses nest per Scala semantics).
//
// TODO(M23 S4-landing): `LegacyShape`/`LegacyCircle` (sealed pair) and
// `LegacyLedger` (companion pair) exist so the sealed-constraint suite can
// assert they co-locate into one batch. Sealed pairing needs T7-S4
// (SymbolRecord.modifiers + typed extends edges) and is NOT assertable at
// the plan-witness HEAD; companion pairing (D17) already co-targets today.

package object legacy {
  val banner: String = "legacy isolation fixture"
}

/** A heavyweight report class: the justified symbol every consumer wants. */
class LegacyReport {
  def title: String = "legacy report"
  def describe(trail: AuditTrail): String = trail.note(title)
  def checksum(seed: Int): Int = seed * 3 + 1
}

/** A heavyweight sibling class no consumer uses: collateral by design. */
class LegacyImport {
  def label: String = "legacy import shim"
  def parseCount(text: String): Int = text.length
}

/** Sealed hierarchy head: co-location witness material for S4 (see TODO). */
sealed trait LegacyShape {
  def sides: Int
}

final class LegacyCircle extends LegacyShape {
  def sides: Int = 0
  def radius: Int = 2
}

/** Companion pair material: class + companion object (D17 pairing). */
class LegacyLedger {
  def balance: Int = 0
}

object LegacyLedger {
  def empty: LegacyLedger = new LegacyLedger()
  def audit(trail: AuditTrail): String = trail.note("ledger")
}
