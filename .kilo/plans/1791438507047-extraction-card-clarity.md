# Plan — ui-extraction-card-clarity (architect shaping)

Artifacts (authoritative):
- tasks/ui-extraction-card-clarity-kickoff.md
- tasks/ui-extraction-card-clarity-shape.md

Milestone key verbatim: `ui-extraction-card-clarity`

Shape in one breath: contrast becomes a measurable gate (new Rust oracle
`extraction_contrast.rs` parses the four-theme token table with the
`theme_dark_grey_palette.rs` WCAG idiom; floors: muted-body >= 4.5:1 and
text-main >= 7:1 on --bg-card, --text-sub banned on card surfaces,
bg-card-vs-surface >= 1.12, border-light floors 1.35/1.50); the card restyles
onto opaque tokens (`--bg-card`/`--bg-sub`/`--border-light`/`--code-text`,
zero new hex, expected single themes.css value fix = deepen light `--green`);
the detail restructures into per-destination-file instruction blocks
(path heading / provides-line / symbol chips) with stats demoted to a footnote
strip and the honest commands block last; the summary stays the five scan
fields and the collapsed-summary interaction is untouched.

Handoffs: T-A oracle + token fixes -> T-B renderer + restyle + test amendments
(3 amended: story beats-order, story risk-base-rule, playbook styles-selector;
13 survive byte-identical, 0 killed; new `extraction_card_clarity.rs`) ->
optional T-C section-chrome split. All hands inside crates/ui; index.html and
DTO/producer untouched; no live milestone conflicts with assets.
