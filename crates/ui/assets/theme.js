// Theme switching (plan 1790656207440 T6): theme registry, persistence,
// toolbar-select wiring, and the re-render a theme swap requires. This
// file loads LAST (after main.js): applyTheme() re-renders through the
// global renderSvg() entry point declared in graph.js, so the graph
// layer must already be evaluated before this file runs.
//
// The inline <head> bootstrap in index.html DUPLICATES the
// read-validate-apply pair below on purpose: that snippet must run
// synchronously before the first paint (every deferred/end-of-body asset
// loads AFTER the themed CSS, so deferred-only bootstrapping flashes the
// wrong theme), while this file owns the deferred half — the select, the
// storage rewrite of stale values, and the re-render. Change the accepted
// ids or the fallback here and you MUST mirror the bootstrap; the parity
// test locks the themes.css [data-theme] blocks to THEMES below.
const THEMES = ['light', 'dark', 'dark-grey', 'dark-steampunk'];
const DEFAULT_THEME = 'dark';
const THEME_STORAGE_KEY = 'ariadne_theme';

// One validated theme id: an unknown or stale stored value (e.g. a theme
// removed in a later release) degrades to the default, so <html> never
// carries a data-theme that matches no themes.css block — that would
// silently yield undefined tokens instead of the default palette.
function resolveTheme(name) {
  return THEMES.includes(name) ? name : DEFAULT_THEME;
}

// Apply a theme: pin <html data-theme>, persist the choice, sync the
// toolbar select, then re-render. The re-render is REQUIRED: node
// fills/strokes are baked as SVG attributes and the heat palette is
// recomputed per render, so without it every card keeps stale colors
// until the next interaction (the most likely shipped bug, per plan
// Risks). The typeof guard keeps applyTheme safe on pages where the
// graph assets have not loaded (or failed to load).
function applyTheme(name) {
  const theme = resolveTheme(name);
  document.documentElement.dataset.theme = theme;
  try {
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch (err) {
    // Private-mode localStorage: the attribute still themes the page.
  }
  const select = document.getElementById('themeSelect');
  if (select) select.value = theme;
  if (typeof renderSvg === 'function') renderSvg();
}

function readStoredTheme() {
  try {
    return localStorage.getItem(THEME_STORAGE_KEY);
  } catch (err) {
    return null;
  }
}

// Boot: wire the select, then apply the stored theme — applyTheme's
// validation rewrites a stale/unknown key to the default on the way in.
(function initThemeControl() {
  const select = document.getElementById('themeSelect');
  if (select) select.addEventListener('change', () => applyTheme(select.value));
  applyTheme(readStoredTheme());
})();
