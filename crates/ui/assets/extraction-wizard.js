// Extraction wizard asset (ui-extraction-wizard W-3a): the modal stepper shell
// over the boot proposal payload. W-3a registers the asset plus the stepper
// chrome with steps S0 (Overview) and S1 (Files to create); steps S2-S6 are
// W-3b content against the read-only provider routes.
//
// Load-order contract: this asset ships after hotspots.js and main.js and
// before theme.js. That position is what lets it reuse the shared globals
// instead of re-declaring them: escapeHtml/escapeAttribute ship exactly once
// (hotspots.js, the asset_injection_hardening lock), and the modal
// infrastructure is the existing #modalOverlay + .modal pair.

// === WIZARD STATE (shape C7: the one module-level state object) ===
// Singleton: the wizard is a single modal, so wizard-scoped element ids are
// safe here (unlike the two-surface cards). Reset happens on modal open for
// a different plan; W-3b adds the 409 stale-generation and payload-swap
// resets named by shape C7.
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
// different plan now; 409 stale generation and payload swap join in W-3b.
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

function wizardStepBack() {
  if (wizardState.step > 0) showWizardStep(wizardState.step - 1);
}

function wizardStepNext() {
  if (wizardState.step < WIZARD_STEP_LABELS.length - 1) showWizardStep(wizardState.step + 1);
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

// S1 row selection feeds the C7 state; W-3b consumes selectedFile for the
// per-file manifest fetch. Delegated (the rows re-render per open).
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
