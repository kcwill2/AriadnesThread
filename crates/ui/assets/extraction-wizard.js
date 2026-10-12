// Extraction wizard asset (ui-extraction-wizard W-3a + W-3b): the modal
// stepper over the read-only provider. W-3a registers the asset plus the
// stepper chrome with steps S0 (Overview) and S1 (Files to create); W-3b
// adds steps S2-S6 (per-file manifest, impact and risk, dry run, the CLI
// recipe, the after-apply description) against the provider routes, the
// C7 reset paths, and the execution-boundary statements.
//
// Load-order contract: this asset ships after hotspots.js and main.js and
// before theme.js. That position is what lets it reuse the shared globals
// instead of re-declaring them: escapeHtml/escapeAttribute ship exactly once
// (hotspots.js, the asset_injection_hardening lock), and the modal
// infrastructure is the existing #modalOverlay + .modal pair.

// === WIZARD STATE (shape C7: the one module-level state object) ===
// Singleton: the wizard is a single modal, so wizard-scoped element ids are
// safe here (unlike the two-surface cards). All three shape-C7 resets are
// live: modal open for a different plan, the 409 stale-generation refusal
// (W6), and the payload-swap re-render path.
const wizardState = {
  planId: null,
  step: 0,
  selectedFile: null,
  manifest: null,
  dryRun: null,
  error: null
};

// Shape C1 step labels in flow order S0..S6. The wizard-step-list markup in
// index.html mirrors this order; S2-S6 bodies arrive with W-3b.
const WIZARD_STEP_LABELS = ['Overview', 'Files to create', 'Per-file manifest', 'Impact and risk', 'Dry run', 'Execute', 'After apply'];

function wizardPanelOpen() {
  const modal = document.getElementById('wizardModal');
  return !!modal && modal.style.display === 'block';
}

// One reset point for the whole state (shape C7). Callers: modal open for a
// different plan, the 409 stale-generation fetch error path, and the
// payload-swap hook at the foot of this file.
function resetWizardState(planId) {
  wizardState.planId = planId;
  wizardState.step = 0;
  wizardState.selectedFile = null;
  wizardState.manifest = null;
  wizardState.dryRun = null;
  wizardState.error = null;
}

function findProposalForPlan(planId) {
  const proposals = graphData.extraction_proposals || [];
  return proposals.find(proposal => proposal.plan_id === planId) || null;
}

function renderWizardError() {
  const slot = document.getElementById('wizardError');
  if (!slot) return;
  slot.hidden = !wizardState.error;
  slot.textContent = wizardState.error || '';
}

// === STEP NAVIGATION (shape C1 chrome: hidden switching, aria-current) ===
// Steps switch by the hidden attribute only (no animation): the string locks
// and the screen-reader story both read that attribute directly.
function showWizardStep(stepIndex) {
  wizardState.step = stepIndex;
  for (let position = 0; position < WIZARD_STEP_LABELS.length; position++) {
    const section = document.getElementById('wizardStep' + position);
    if (section) section.hidden = position !== stepIndex;
    const item = document.getElementById('wizardStepItem' + position);
    if (!item) continue;
    if (position === stepIndex) item.setAttribute('aria-current', 'step');
    else item.removeAttribute('aria-current');
  }
  const back = document.getElementById('wizardBack');
  const next = document.getElementById('wizardNext');
  if (back) back.disabled = stepIndex === 0;
  if (next) next.disabled = stepIndex === WIZARD_STEP_LABELS.length - 1;
}

// W-3a shipped the step navigation as a pure showWizardStep walk; W-3b
// enters each step so its per-step fetch fires on entry (shape C6) and
// re-enters on Back so cached data re-renders without a new fetch.
function wizardStepBack() {
  if (wizardState.step <= 0) return;
  const previousIndex = wizardState.step - 1;
  showWizardStep(previousIndex);
  enterWizardStep(previousIndex);
}

function wizardStepNext() {
  if (wizardState.step >= WIZARD_STEP_LABELS.length - 1) return;
  const nextIndex = wizardState.step + 1;
  showWizardStep(nextIndex);
  enterWizardStep(nextIndex);
}

// === STEP S0: OVERVIEW (shape C1 row S0: the payload as-is) ===
// No fetch, no re-sort, no derived number beyond the shared guarded saving
// helper shipped by hotspots.js (extractionSavingText stays the one formula
// for a saving, and it is zero-guarded there).
function renderWizardStepOverview(proposal) {
  const before = proposal.collateral_load_before || 0;
  const after = proposal.collateral_load_after || 0;
  const saving = extractionSavingText(before, after);
  return '<div class="wizard-step-title">Overview</div>' +
    `<div class="wizard-row"><span class="wizard-label">Plan</span><span class="wizard-value">${escapeHtml(proposal.plan_id || '')}</span></div>` +
    `<div class="wizard-row"><span class="wizard-label">Target file</span><span class="wizard-value">${escapeHtml(proposal.target_file || '')}</span></div>` +
    `<div class="wizard-row"><span class="wizard-label">Collateral drop</span><span class="wizard-delta">${escapeHtml(before)} &rarr; ${escapeHtml(after)}</span>${saving}</div>` +
    `<div class="wizard-row"><span class="wizard-label">Risk tier</span><span class="wizard-value">${escapeHtml(proposal.risk || '')}</span></div>` +
    `<div class="wizard-row"><span class="wizard-label">Consumers</span><span class="wizard-value">${escapeHtml(proposal.consumer_tu_count || 0)} consumer TU(s) need re-validation</span></div>` +
    `<div class="wizard-row"><span class="wizard-label">Symbols</span><span class="wizard-value">${escapeHtml(proposal.total_symbols || 0)}</span></div>`;
}

// === STEP S1: FILES TO CREATE (shape C1 PRIMARY row, over the capped boot
// payload) ===
// One selectable row per destination file the boot payload carries: the real
// path, the symbol count truth, and the just-more `+K more` chip idiom for
// both the per-file name cap and the file-list cap (shape C6). The honest
// cap note renders whenever target_files_truncated is set; the row count
// stays the payload's own.
function renderWizardStepFiles(proposal) {
  const files = proposal.target_files || [];
  const fileRows = files.map(fileRow => {
    const hiddenSymbols = (fileRow.symbol_count || 0) - (fileRow.symbol_names || []).length;
    const overflow = hiddenSymbols > 0 ? `<span class="just-more">+${hiddenSymbols} more</span>` : '';
    return `<div class="wizard-file-row" data-wizard-file="${escapeAttribute(fileRow.path)}" tabindex="0">` +
      `<code class="wizard-file-path">${escapeHtml(fileRow.path)}</code>` +
      `<span class="wizard-file-count">${escapeHtml(fileRow.symbol_count || 0)} symbols</span>` +
      `${overflow}</div>`;
  }).join('');
  const capNote = proposal.target_files_truncated
    ? '<span class="just-more">more files not listed</span>'
    : '';
  const rows = fileRows.length > 0 ? fileRows : '<em>no file plan in payload</em>';
  return '<div class="wizard-step-title">Files to create</div>' +
    `<div class="wizard-file-list">${rows}${capNote}</div>`;
}

// S1 row selection feeds the C7 state; step S2 renders the manifest for
// selectedFile. Delegated (the rows re-render per open).
function selectWizardFile(filePath) {
  wizardState.selectedFile = filePath;
  document.querySelectorAll('#wizardModal .wizard-file-row').forEach(row => {
    row.classList.toggle('selected', row.dataset.wizardFile === filePath);
  });
}

function openExtractionWizard(planId) {
  if (wizardState.planId !== planId) {
    resetWizardState(planId);
  }
  const proposal = findProposalForPlan(planId);
  if (!proposal) {
    wizardState.error = 'plan ' + planId + ' is no longer in the payload';
  } else {
    document.getElementById('wizardStepContent0').innerHTML = renderWizardStepOverview(proposal);
    document.getElementById('wizardStepContent1').innerHTML = renderWizardStepFiles(proposal);
  }
  renderWizardError();
  showWizardStep(wizardState.step);
  document.getElementById('wizardModal').style.display = 'block';
  document.getElementById('modalOverlay').style.display = 'block';
}

function closeWizardPanel() {
  const modal = document.getElementById('wizardModal');
  if (modal) modal.style.display = 'none';
  const overlay = document.getElementById('modalOverlay');
  if (overlay) overlay.style.display = 'none';
}

// Close-path hook (shape C1: Esc closes via the existing closeModals path):
// #modalOverlay's onclick and every house close affordance call closeModals()
// in the frozen api.js. Wrapping the global here extends that existing path
// with the wizard panel - the house behavior runs first, byte-unchanged, and
// no frozen asset is edited.
(function installWizardCloseHook() {
  const houseCloseModals = closeModals;
  window.closeModals = function closeModalsWithWizard() {
    houseCloseModals();
    closeWizardPanel();
  };
})();

// Esc rides the same closeModals() path (shape C1 stepper chrome).
window.addEventListener('keydown', event => {
  if (event.key === 'Escape' && wizardPanelOpen()) closeModals();
});

// Card-open click on a destination row (delegated: rows are rebuilt per open).
document.addEventListener('click', event => {
  const fileRow = event.target.closest('.wizard-file-row[data-wizard-file]');
  if (fileRow) selectWizardFile(fileRow.dataset.wizardFile);
});

// === WIZARD STEPS S2-S6 (ui-extraction-wizard W-3b) ===
// Per-step fetching (shape C6: never at boot), the shape-C7 reset paths, and
// the honest S5/S6 copy. One fetcher, one cap-chip helper, one copy-only
// gate: every renderer below reads only fetched payload data through the
// shared escapers and renders no symbol body (W9).

// Renderer-side belt over the provider caps (shape C6). symbol_count and the
// truncation flags carry the truth.
const EXTRACTION_MANIFEST_MAX_FILES = 64;
const EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE = 200;
const EXTRACTION_DRY_RUN_MAX_REWRITES = 200;

// The exact W4 copy-vs-move sentence, used at every use site keyed off the
// COMPUTED move_removals path gated by includes_unavailable: the degraded
// honest-absent provider answer forces the copy_only flag false, so the flag
// itself is never read here (W-2a verdict relay).
const COPY_VS_MOVE_SENTENCE = 'The C++ backend writes the new file and leaves the source declarations in place; nothing is moved out of the source files.';

function extractionRouteUrl(routeSuffix) {
  return '/api/extraction/' + encodeURIComponent(wizardState.planId || '') + routeSuffix;
}

// The single fetch site (shape C2/C3): builds the route URL for the open
// plan and answers parsed JSON, or null with honest error copy. A 409
// stale-generation refusal resets the wizard step state from this error
// path (shape C7, invariant W6).
async function fetchWizardRoute(routeSuffix) {
  let response = null;
  try {
    response = await fetch(extractionRouteUrl(routeSuffix));
  } catch (networkError) {
    wizardState.error = 'the extraction route ' + routeSuffix + ' is unreachable';
    renderWizardError();
    return null;
  }
  if (response.status === 409) {
    resetWizardState(wizardState.planId);
    wizardState.error = 'stale analysis generation: the wizard reset to its first step. Reopen it from the card once the new analysis is ready.';
    renderWizardError();
    showWizardStep(wizardState.step);
    return null;
  }
  if (!response.ok) {
    wizardState.error = 'extraction ' + routeSuffix + ' refused (HTTP ' + response.status + ')';
    renderWizardError();
    return null;
  }
  return response.json();
}

async function ensureWizardManifest() {
  if (wizardState.manifest) return wizardState.manifest;
  const manifest = await fetchWizardRoute('/manifest');
  wizardState.manifest = manifest || null;
  return wizardState.manifest;
}

async function ensureWizardDryRun() {
  if (wizardState.dryRun) return wizardState.dryRun;
  const dryRun = await fetchWizardRoute('/dry-run');
  wizardState.dryRun = dryRun || null;
  return wizardState.dryRun;
}

// Step entry (shape C6): fetching fires on step entry only, never at boot.
// Steps 0-1 render from the boot payload when the modal opens; steps 5-6
// render copy with no route behind them.
async function enterWizardStep(stepIndex) {
  const proposal = findProposalForPlan(wizardState.planId);
  if (!proposal) return;
  if (stepIndex === 2 || stepIndex === 3) {
    await ensureWizardManifest();
    document.getElementById('wizardStepContent2').innerHTML = renderWizardStepManifest(wizardState.manifest);
  }
  if (stepIndex === 3) {
    await ensureWizardDryRun();
    document.getElementById('wizardStepContent3').innerHTML = renderWizardStepImpact(proposal, wizardState.manifest, wizardState.dryRun);
  }
  if (stepIndex === 4) {
    await ensureWizardDryRun();
    if (wizardState.dryRun) {
      document.getElementById('wizardStepContent4').innerHTML = renderWizardStepDryRun(wizardState.dryRun);
    }
  }
  if (stepIndex === 5) {
    document.getElementById('wizardStepContent5').innerHTML = renderWizardStepExecute(wizardState.planId);
  }
  if (stepIndex === 6) {
    document.getElementById('wizardStepContent6').innerHTML = renderWizardStepAfterApply();
  }
}

// The shared just-more chip (shape C6 idiom, the hotspots.js just-more
// precedent): a capped list says how many rows it hides.
function wizardCapChip(shownCount, totalCount) {
  const hiddenCount = (totalCount || 0) - shownCount;
  return hiddenCount > 0 ? `<span class="just-more">+${hiddenCount} more</span>` : '';
}

// The copy-only gate (W4 + the W-2a verdict relay): the degraded answer
// reports includes_unavailable with a forced copy_only, so the degradation
// signal gates first and the healthy answer is judged on the computed
// move_removals path itself.
function wizardCopyOnly(dryRun) {
  if (!dryRun || dryRun.includes_unavailable) return false;
  return dryRun.move_removals === 0;
}

// Step S2 (shape C1): the full symbol list for the selected file plus the
// includes that file needs (W3: engine-sourced entries or the honest
// fallback line, never a fabricated path).
function renderWizardStepManifest(manifest) {
  const stepTitle = '<div class="wizard-step-title">Per-file manifest</div>';
  if (!manifest || (manifest.files || []).length === 0) {
    return stepTitle + '<em>no manifest available for this plan</em>';
  }
  const selectedFile = (manifest.files || []).find(candidate => candidate.path === wizardState.selectedFile);
  if (!selectedFile) {
    return stepTitle + '<em>select a destination file in step 2 to see its symbols and includes</em>';
  }
  const cappedNames = (selectedFile.symbol_names || []).slice(0, EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE);
  const symbolRows = cappedNames.map(symbolName =>
    `<div class="wizard-symbol-row"><code>${escapeHtml(symbolName)}</code></div>`).join('');
  const filesCapChip = manifest.files_truncated
    ? '<span class="just-more">more files not listed</span>' : '';
  const includesBlock = selectedFile.includes_unavailable
    ? '<div class="wizard-impact-fallback">Includes unavailable: the engine could not resolve the dependencies for this new file; nothing is fabricated here.</div>'
    : `<div class="wizard-include-list">${(selectedFile.required_includes || []).map(includePath =>
        `<div class="wizard-include-row"><code>${escapeHtml(includePath)}</code></div>`).join('')}</div>`;
  return stepTitle +
    `<div class="extraction-file-block"><code class="extraction-file-path">${escapeHtml(selectedFile.path)}</code>` +
    `<span class="wizard-file-count">${escapeHtml(selectedFile.symbol_count || 0)} symbols</span>` +
    wizardCapChip(cappedNames.length, selectedFile.symbol_count || 0) + filesCapChip + '</div>' +
    `<div class="wizard-symbol-list">${symbolRows}</div>` +
    '<div class="wizard-include-heading">Includes this new file needs</div>' +
    includesBlock;
}

// Step S3 (shape C1): consumer TU count, required headers, the engine cycle
// error when any, and the copy-vs-move statement keyed off the computed
// move_removals path (W4); the includes_unavailable degradation is stated
// instead of any claim.
function renderWizardStepImpact(proposal, manifest, dryRun) {
  const headerRows = (proposal.required_headers || []).map(headerName =>
    `<div class="wizard-include-row"><code>${escapeHtml(headerName)}</code></div>`).join('');
  const cycleLine = manifest && manifest.cycle_error
    ? `<div class="wizard-error">Engine cycle error: ${escapeHtml(manifest.cycle_error)}</div>`
    : '';
  let copyMoveLine;
  if (!dryRun) {
    copyMoveLine = '<div class="wizard-impact-fallback">Copy-vs-move pending: the dry run has not been fetched yet (step 5).</div>';
  } else if (dryRun.includes_unavailable) {
    copyMoveLine = '<div class="wizard-impact-fallback">Includes unavailable: the provider could not complete the dry run for this generation, so no copy-vs-move claim is made.</div>';
  } else if (dryRun.move_removals === 0) {
    copyMoveLine = `<div class="wizard-impact-line">${escapeHtml(COPY_VS_MOVE_SENTENCE)}</div>`;
  } else {
    copyMoveLine = `<div class="wizard-impact-line">This plan moves symbols: the plugin strips ${escapeHtml(dryRun.move_removals)} moved declaration(s) from the source files.</div>`;
  }
  return '<div class="wizard-step-title">Impact and risk</div>' +
    `<div class="wizard-row"><span class="wizard-label">Consumers</span><span class="wizard-value">${escapeHtml(proposal.consumer_tu_count || 0)} consumer TU(s) need re-validation</span></div>` +
    '<div class="wizard-row"><span class="wizard-label">Required headers</span></div>' +
    (headerRows.length > 0 ? headerRows : '<em>no required headers in payload</em>') +
    cycleLine + copyMoveLine;
}

// Step S4 (shape C1/C4): the new-file paths and every consumer rewrite (old
// -> new with the preserved side-effect includes). Directive spelling
// renders through exactly as the route returns it (C4).
function renderWizardStepDryRun(dryRun) {
  const newFileRows = (dryRun.new_files || []).slice(0, EXTRACTION_MANIFEST_MAX_FILES).map(newFilePath =>
    `<div class="wizard-rewrite-path"><code>${escapeHtml(newFilePath)}</code></div>`).join('');
  const cappedRewrites = (dryRun.rewrites || []).slice(0, EXTRACTION_DRY_RUN_MAX_REWRITES);
  const rewriteRows = cappedRewrites.map(rewrite => {
    const oldBlock = (rewrite.old_includes || []).map(oldPath => escapeHtml(oldPath)).join('<br>');
    const newBlock = (rewrite.new_includes || []).map(newPath => escapeHtml(newPath)).join('<br>');
    const keptBlock = (rewrite.side_effect_preservations || []).length > 0
      ? `<div class="wizard-diff-kept">kept for side effects: ${(rewrite.side_effect_preservations || []).map(keptPath => `<code>${escapeHtml(keptPath)}</code>`).join(' ')}</div>`
      : '';
    return `<div class="wizard-rewrite-row"><code class="wizard-rewrite-file">${escapeHtml(rewrite.consumer_file)}</code>` +
      `<div class="wizard-diff-old">${oldBlock}</div><span class="wizard-diff-arrow">&rarr;</span>` +
      `<div class="wizard-diff-new">${newBlock}</div>${keptBlock}</div>`;
  }).join('');
  const truncationNote = dryRun.rewrites_truncated
    ? `<div class="wizard-truncation-note">Rewrite list truncated at ${EXTRACTION_DRY_RUN_MAX_REWRITES} rows for volume; the apply rewrites every consumer.</div>`
    : '';
  const copyMoveLine = wizardCopyOnly(dryRun)
    ? `<div class="wizard-impact-line">${escapeHtml(COPY_VS_MOVE_SENTENCE)}</div>`
    : (dryRun.includes_unavailable
        ? '<div class="wizard-impact-fallback">Includes unavailable: the provider could not complete this dry run; nothing is claimed about the writes.</div>'
        : `<div class="wizard-impact-line">This plan strips ${escapeHtml(dryRun.move_removals || 0)} moved source declaration(s) as it writes.</div>`);
  return '<div class="wizard-step-title">Dry run</div>' +
    '<div class="wizard-row"><span class="wizard-label">Files written</span></div>' +
    (newFileRows.length > 0 ? newFileRows : '<em>no new files listed</em>') +
    '<div class="wizard-row"><span class="wizard-label">Consumer rewrites</span></div>' +
    (rewriteRows.length > 0 ? rewriteRows : '<em>no consumer rewrites</em>') +
    truncationNote + copyMoveLine +
    '<div class="wizard-impact-line">The dry run changes nothing; tool apply performs the same computation before writing.</div>';
}

// Step S5 (shape C1/W5): the exact CLI recipe with the real plan id, the
// --preview-first step, and the W4 sentence. The UI never runs these.
function renderWizardStepExecute(planId) {
  const safePlanId = escapeHtml(planId || '');
  return '<div class="wizard-step-title">Execute</div>' +
    '<div class="wizard-row"><span class="wizard-label">Preview first</span><code class="extraction-command">tool migrate &lt;repo-root&gt; --plan ' + safePlanId + ' --preview</code></div>' +
    '<div class="wizard-row"><span class="wizard-label">Generate</span><code class="extraction-command">tool transform extract &lt;repo-root&gt; --plan ' + safePlanId + '</code></div>' +
    '<div class="wizard-row"><span class="wizard-label">Full apply</span><code class="extraction-command">tool apply &lt;repo-root&gt; --plan ' + safePlanId + '</code></div>' +
    `<div class="wizard-impact-line">${escapeHtml(COPY_VS_MOVE_SENTENCE)}</div>` +
    '<div class="wizard-impact-line">The UI shows this recipe and never runs it; run the preview first, then apply when you choose.</div>';
}

// Step S6 (shape C1): what the three tiers validate, what rollback restores
// and where the artifacts land - the CLI behavior described, no UI surface.
function renderWizardStepAfterApply() {
  return '<div class="wizard-step-title">After apply</div>' +
    '<div class="wizard-row"><span class="wizard-label">Tier 1</span><span class="wizard-value">Structural: the engine extraction reports no unit-graph errors or cycles.</span></div>' +
    '<div class="wizard-row"><span class="wizard-label">Tier 2</span><span class="wizard-value">Generated headers: every include a new file needs exists on disk.</span></div>' +
    '<div class="wizard-row"><span class="wizard-label">Tier 3</span><span class="wizard-value">Affected TUs: every new include of a rewritten consumer exists on disk; with no compiler toolchain this tier is skipped, and a skipped tier commits like a passed one.</span></div>' +
    '<div class="wizard-row"><span class="wizard-label">Rollback</span><span class="wizard-value">Rollback fires only on a failed tier: every snapshot file is restored and every newly created file is removed.</span></div>' +
    '<div class="wizard-row"><span class="wizard-label">Artifacts</span><span class="wizard-value">Generated units land under artifacts/generated (or in place for move languages), the migration preview under artifacts/migration/preview.json, and the outcome record under artifacts/transaction.</span></div>';
}

// Payload-swap reset (shape C7, third reset path): the live-refresh swap in
// api.js re-renders every surface through applyLoadedData (graph.js); the
// load-time wrap below extends that existing re-render path with the wizard
// reset - the same idiom the closeModals hook above uses, so no frozen asset
// is edited. The generation-derived manifest and dry-run must not survive a
// re-plan.
function resetWizardOnPayloadSwap() {
  const planId = wizardState.planId;
  resetWizardState(planId);
  if (!wizardPanelOpen()) return;
  const proposal = findProposalForPlan(planId);
  if (proposal) {
    document.getElementById('wizardStepContent0').innerHTML = renderWizardStepOverview(proposal);
    document.getElementById('wizardStepContent1').innerHTML = renderWizardStepFiles(proposal);
    renderWizardError();
    showWizardStep(0);
  } else {
    wizardState.error = 'plan ' + planId + ' is no longer in the payload';
    renderWizardError();
  }
}

(function installWizardPayloadSwapHook() {
  const houseApplyLoadedData = applyLoadedData;
  window.applyLoadedData = function applyLoadedDataWithWizardReset() {
    resetWizardOnPayloadSwap();
    houseApplyLoadedData();
  };
})();
