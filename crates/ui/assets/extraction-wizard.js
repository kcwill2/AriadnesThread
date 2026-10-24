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
// (W6), and the payload-swap re-render path. extraction-auto-apply adds
// `applying` (the in-flight flag for the apply request) and `outcome` (the
// 200 answer of the apply route): an outcome belongs to the generation
// that produced it, so every reset path clears both through this one point.
const wizardState = {
  planId: null,
  step: 0,
  selectedFile: null,
  manifest: null,
  dryRun: null,
  error: null,
  applying: false,
  outcome: null
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
  wizardState.applying = false;
  wizardState.outcome = null;
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

// The marked-retention move sentence (extraction-symbol-move): what the C++
// apply actually does with moved declarations, plus the honest restore
// procedure (kickoff section 1 layer 3 - never a one-click claim).
const MOVE_RETENTION_SENTENCE = 'the plugin comments the moved declarations out in place under ARIADNE-EXTRACT-OUT markers that name the plan; restoring means uncommenting those regions AND reverting the consumer include rewrites, or using git - never a one-click undo.';

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
    renderWizardExecuteStepContent();
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
    copyMoveLine = `<div class="wizard-impact-line">This plan moves ${escapeHtml(dryRun.move_removals)} declaration(s): ${escapeHtml(MOVE_RETENTION_SENTENCE)}</div>`;
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
    ? `<div class="wizard-truncation-note">Rewrite list truncated at ${EXTRACTION_DRY_RUN_MAX_REWRITES} rows for volume; the apply rewrites every consumer the dry run lists.</div>`
    : '';
  const copyMoveLine = wizardCopyOnly(dryRun)
    ? `<div class="wizard-impact-line">${escapeHtml(COPY_VS_MOVE_SENTENCE)}</div>`
    : (dryRun.includes_unavailable
        ? '<div class="wizard-impact-fallback">Includes unavailable: the provider could not complete this dry run; nothing is claimed about the writes.</div>'
        : `<div class="wizard-impact-line">As it writes, this plan retires ${escapeHtml(dryRun.move_removals || 0)} source declaration(s): ${escapeHtml(MOVE_RETENTION_SENTENCE)}</div>`);
  return '<div class="wizard-step-title">Dry run</div>' +
    '<div class="wizard-row"><span class="wizard-label">Files written</span></div>' +
    (newFileRows.length > 0 ? newFileRows : '<em>no new files listed</em>') +
    '<div class="wizard-row"><span class="wizard-label">Consumer rewrites</span></div>' +
    (rewriteRows.length > 0 ? rewriteRows : '<em>no consumer rewrites</em>') +
    truncationNote + copyMoveLine +
    '<div class="wizard-impact-line">The dry run changes nothing; tool apply performs the same computation before writing.</div>';
}

// Step S5 (shape C1/W5 + extraction-auto-apply C7): the exact CLI recipe
// with the real plan id, the --preview-first step, and the apply control.
// A7: the control is enabled only with the dry run's single-use approval
// token in hand; without one the step renders the dry-run-first
// instruction and no apply request is possible. The copy claims exactly
// what the shipped path does: the transactional applier writes the new
// files, the three tiers validate and roll back automatically, and the
// outcome is reported in S6.
const DRY_RUN_FIRST_INSTRUCTION = 'Open the dry run first: the apply unlocks once the dry run returns its single-use approval token.';
const APPLY_APPROVED_INSTRUCTION = 'The dry run approved this plan: applying writes the new files, validates in three tiers with automatic rollback, and reports the outcome in the After apply step.';
const APPLY_INFLIGHT_COPY = 'Applying: the result lands in the After apply step when the transaction finishes.';

function renderWizardStepExecute(planId) {
  const safePlanId = escapeHtml(planId || '');
  const applyToken = wizardState.dryRun ? (wizardState.dryRun.apply_token || '') : '';
  const approvalMissing = applyToken === '';
  const inFlight = wizardState.applying === true;
  const applyDisabled = approvalMissing || inFlight;
  const instructionText = approvalMissing ? DRY_RUN_FIRST_INSTRUCTION : APPLY_APPROVED_INSTRUCTION;
  const applyButton = '<div class="wizard-apply-row"><button type="button" class="btn-primary wizard-apply-button" id="wizardApplyBtn"' +
    (applyDisabled ? ' disabled' : '') + '>' + (inFlight ? 'Applying' : 'Apply this plan') + '</button></div>';
  const inflightLine = inFlight
    ? '<div class="wizard-apply-inflight">' + escapeHtml(APPLY_INFLIGHT_COPY) + '</div>'
    : '';
  const moveTruthLine = !wizardState.dryRun
    ? '<div class="wizard-impact-fallback">Copy-vs-move pending: the dry run has not been fetched yet (step 5).</div>'
    : wizardCopyOnly(wizardState.dryRun)
      ? `<div class="wizard-impact-line">${escapeHtml(COPY_VS_MOVE_SENTENCE)}</div>`
      : (wizardState.dryRun.includes_unavailable
          ? '<div class="wizard-impact-fallback">Includes unavailable: the provider could not complete this dry run; nothing is claimed about the writes.</div>'
          : `<div class="wizard-impact-line">On apply this plan retires ${escapeHtml(wizardState.dryRun.move_removals)} moved declaration(s): ${escapeHtml(MOVE_RETENTION_SENTENCE)}</div>`);
  return '<div class="wizard-step-title">Execute</div>' +
    '<div class="wizard-row"><span class="wizard-label">Preview first</span><code class="extraction-command">tool migrate &lt;repo-root&gt; --plan ' + safePlanId + ' --preview</code></div>' +
    '<div class="wizard-row"><span class="wizard-label">Generate</span><code class="extraction-command">tool transform extract &lt;repo-root&gt; --plan ' + safePlanId + '</code></div>' +
    '<div class="wizard-row"><span class="wizard-label">Full apply</span><code class="extraction-command">tool apply &lt;repo-root&gt; --plan ' + safePlanId + '</code></div>' +
    moveTruthLine +
    '<div class="wizard-apply-instruction">' + escapeHtml(instructionText) + '</div>' +
    applyButton + inflightLine;
}

// Re-render S5 in place (entry, in-flight flip, refusal): the step body is
// rebuilt from state every time so the apply control always reflects the
// token, the in-flight flag and nothing stale.
function renderWizardExecuteStepContent() {
  document.getElementById('wizardStepContent5').innerHTML = renderWizardStepExecute(wizardState.planId);
}

// The apply request (extraction-auto-apply C7/C3): its own fetch site by
// mandate, because the apply's 409 has two meanings (stale_generation and
// apply_in_progress) the blanket-409 GET fetcher could not tell apart.
// Every refusal code carries its own copy; the 200 stores the outcome and
// advances to S6. A7 backstop: without a token nothing leaves the asset.
async function applyExtractionPlan() {
  const applyToken = wizardState.dryRun ? (wizardState.dryRun.apply_token || '') : '';
  if (applyToken === '' || wizardState.applying) return;
  wizardState.applying = true;
  renderWizardExecuteStepContent();
  let response = null;
  try {
    response = await fetch(extractionRouteUrl('/apply'), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ apply_token: applyToken })
    });
  } catch (networkError) {
    wizardState.applying = false;
    wizardState.error = 'the extraction route /apply is unreachable';
    renderWizardError();
    renderWizardExecuteStepContent();
    return;
  }
  if (response.status === 200) {
    const outcome = await response.json();
    wizardState.applying = false;
    wizardState.outcome = outcome;
    wizardState.error = null;
    renderWizardError();
    showWizardStep(6);
    enterWizardStep(6);
    return;
  }
  const refusal = await response.json().catch(() => ({}));
  wizardState.applying = false;
  if (response.status === 409 && refusal.error === 'stale_generation') {
    resetWizardState(wizardState.planId);
    wizardState.error = 'stale analysis generation: the wizard reset to its first step. Reopen it from the card once the new analysis is ready.';
    renderWizardError();
    showWizardStep(wizardState.step);
    return;
  }
  if (response.status === 409 && refusal.error === 'apply_in_progress') {
    wizardState.error = 'another apply is running; try again in a moment';
  } else if (response.status === 428) {
    wizardState.error = 'the approval was not accepted: re-open the dry run to refresh the approval';
  } else if (response.status === 500) {
    wizardState.error = 'apply failed and the tree was rolled back: ' + (refusal.detail || 'no detail returned');
  } else if (response.status === 503) {
    wizardState.error = 'the analysis is reloading: wait for it to land, then re-open the dry run to approve again';
  } else if (response.status === 404) {
    wizardState.error = 'this plan is no longer offered by the current analysis: re-open the card to see the current plans';
  } else {
    wizardState.error = 'extraction /apply refused (HTTP ' + response.status + ')';
  }
  renderWizardError();
  renderWizardExecuteStepContent();
}

// The S5 apply click (delegated: the step body re-renders per entry and
// per state change). The disabled control is the A7 gate; the disabled
// check here is a belt, not the lock.
document.addEventListener('click', event => {
  const applyControl = event.target.closest('#wizardApplyBtn');
  if (applyControl && !applyControl.disabled) applyExtractionPlan();
});

// S6 tier display helpers: the DTO statuses are "pass"/"fail"/"skipped";
// the step renders them as the verdict words with the shared semantic
// colors (green commits, red refuses, amber waits, same meaning per theme).
function wizardTierStatusText(status) {
  if (status === 'pass') return 'PASS';
  if (status === 'fail') return 'FAIL';
  if (status === 'skipped') return 'SKIPPED';
  return String(status || '').toUpperCase();
}

function wizardTierStatusClass(status) {
  if (status === 'pass') return 'wizard-tier-pass';
  if (status === 'fail') return 'wizard-tier-fail';
  if (status === 'skipped') return 'wizard-tier-skipped';
  return '';
}

// The honest rebuild sentence (shape C6): the front end cannot observe
// the watcher, so it states the mechanism and the fallback instead of
// claiming a rebuild it cannot know happened.
const OUTCOME_REBUILD_SENTENCE = 'The background watcher re-analyzes the repository and this wizard resets when the new analysis lands; with watching disabled the analysis refreshes at the next UI start. The outcome record also lands under artifacts/transaction.';

// The live outcome rows (shape C7/C6): the three tier results with their
// reasons, the committed-vs-rolled-back verdict, the created files and the
// verbatim server-rendered report. Every interpolated value passes through
// escapeHtml; the report stays a pre block because it is verbatim text.
function renderWizardStepOutcome(outcome) {
  const tierRows = (outcome.tiers || []).map(function outcomeTierRow(tier) {
    const statusText = wizardTierStatusText(tier.status);
    const statusClass = wizardTierStatusClass(tier.status);
    const reason = tier.detail
      ? '<div class="wizard-outcome-reason">' + escapeHtml(tier.detail) + '</div>'
      : '';
    return '<div class="wizard-outcome-tier ' + statusClass + '"><span class="wizard-label">' +
      escapeHtml(tier.tier) + '</span><span class="wizard-outcome-status">' + escapeHtml(statusText) +
      '</span>' + reason + '</div>';
  }).join('');
  const verdictClass = outcome.committed ? 'wizard-tier-pass' : 'wizard-tier-fail';
  const verdictText = outcome.committed
    ? 'Committed: the new files are on disk and the outcome record landed under artifacts/transaction.'
    : 'Rolled back: a failed tier restored the tree byte-identically; none of this apply remains on disk.';
  const createdFileRows = (outcome.new_files || []).map(function createdFileRow(filePath) {
    return '<div class="wizard-outcome-file"><code>' + escapeHtml(filePath) + '</code></div>';
  }).join('');
  return '<div class="wizard-step-title">After apply</div>' +
    tierRows +
    '<div class="wizard-outcome-verdict ' + verdictClass + '">' + escapeHtml(verdictText) + '</div>' +
    '<div class="wizard-row"><span class="wizard-label">Files created</span></div>' +
    (createdFileRows.length > 0 ? createdFileRows : '<em>no files created</em>') +
    '<div class="wizard-row"><span class="wizard-label">Report</span></div>' +
    '<div class="wizard-outcome-text"><pre>' + escapeHtml(outcome.output_text || '') + '</pre></div>' +
    '<div class="wizard-impact-line">' + escapeHtml(OUTCOME_REBUILD_SENTENCE) + '</div>';
}

// Step S6 (shape C1 + extraction-auto-apply C7): live outcome rows once an
// apply returned; until then the descriptive copy (what the three tiers
// validate, what rollback restores and where the artifacts land) keeps the
// step enterable before an apply.
function renderWizardStepAfterApply() {
  if (wizardState.outcome) return renderWizardStepOutcome(wizardState.outcome);
  return '<div class="wizard-step-title">After apply</div>' +
    '<div class="wizard-row"><span class="wizard-label">Tier 1</span><span class="wizard-value">Structural: the engine extraction reports no unit-graph errors or cycles.</span></div>' +
    '<div class="wizard-row"><span class="wizard-label">Tier 2</span><span class="wizard-value">Generated headers: every include a new file needs exists on disk.</span></div>' +
    '<div class="wizard-row"><span class="wizard-label">Tier 3</span><span class="wizard-value">Affected TUs: every new include of a rewritten consumer exists on disk; with no compiler toolchain this tier is skipped, and a skipped tier commits like a passed one.</span></div>' +
    '<div class="wizard-row"><span class="wizard-label">Rollback</span><span class="wizard-value">Rollback fires only on a failed tier: every snapshot file is restored and every newly created file is removed.</span></div>' +
    '<div class="wizard-row"><span class="wizard-label">Artifacts</span><span class="wizard-value">Generated units land under artifacts/generated (or in place for languages that stage units in place), the migration preview under artifacts/migration/preview.json, and the outcome record under artifacts/transaction.</span></div>';
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
