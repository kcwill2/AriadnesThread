package com.example.apps

import com.example.legacy._

/** Consumer in a different package: the wildcard rides in the whole shared
 *  file, but the dashboard only justifies `LegacyReport` — the rest of the
 *  package-object file is collateral the extraction is meant to shed. */
class Dashboard {
  def headline: String = new LegacyReport().title
}
