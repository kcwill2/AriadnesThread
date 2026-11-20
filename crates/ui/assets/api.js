// Data & boot layer split from index.html: live/offline data loading and client API helpers.

    // Live-refresh state. AR-15 scope fix: `lastSeenRefresh` used to be
    // let-bound inside init()'s else-block, but the top-level
    // startLiveRefresh loop reads it on every tick - a block-scoped
    // binding made that read a silently-swallowed ReferenceError, so the
    // delta branch never ran once the server reported a refresh. Top
    // level is the only scope both sites share.
    let lastSeenRefresh = 0;
    // Live collateral loads on demand: graphData.file_collateral caches whole
    // rows (root -> {header: count}) as they arrive, and this caches columns
    // (header -> {root: count}) for the payer-side heat view. A row lands in
    // the cache only complete, so every existing lookup stays exact.
    let collateralColumnCache = {};

    // Synthetic boot snapshot for the standalone export path: same wire shape
    // as /api/status progress, three steps at the historical 25/65/100
    // percents, statuses advancing pending -> active -> done as `activeIndex`
    // moves. Routed through renderProgressSnapshot - one renderer, no second.
    function standaloneBootSnapshot(activeIndex) {
      const stepLabels = ['Loading graph datasets', 'Processing graph models', 'Rendering visualization'];
      const stepPercents = [25, 65, 100];
      const steps = stepLabels.map((label, stepIndex) => ({
        stage: stepIndex + 1,
        label: label,
        percent: stepIndex <= activeIndex ? stepPercents[stepIndex] : 0,
        message: '',
        status: stepIndex < activeIndex ? 'done' : (stepIndex === activeIndex ? 'active' : 'pending'),
      }));
      return {
        state: 'loading',
        phase: '',
        percent: stepPercents[activeIndex],
        message: '',
        total_stages: 3,
        steps: steps,
      };
    }

    // Initialize application
    async function init() {
      setupCanvasPanZoom();

      if (window.__ARIADNE_DATA__ && window.__ARIADNE_DATA__.include_graph) {
        // Use standalone exported data
        graphData = window.__ARIADNE_DATA__;
        renderProgressSnapshot(standaloneBootSnapshot(0));
        setTimeout(() => {
          renderProgressSnapshot(standaloneBootSnapshot(1));
          setTimeout(() => {
             renderProgressSnapshot(standaloneBootSnapshot(2));
             applyLoadedData();
             hideProgress();
          }, 50);
        }, 50);
      } else {
        showProgress('Connecting to backend...', 0);
        const overlay = document.getElementById('loadingOverlay');
        if (overlay) overlay.style.display = 'flex';

        // lastSeenRefresh is declared at api.js top level (AR-15 scope fix).

        const pollStatus = async () => {
          try {
            const statusRes = await fetch('/api/status');
            if (statusRes.ok) {
              const status = await statusRes.json();
              
              if (status.progress && status.progress.state === 'loading') {
                renderProgressSnapshot(status.progress);
                setTimeout(pollStatus, 150);
                return;
              }
              
              if (status.progress && status.progress.state === 'ready') {
                graphData.repo_name = status.repo_name || 'Repository';
                if (status.stats) graphData.stats = status.stats;
                lastSeenRefresh = status.last_refresh || 0;
                
                updateProgress(90, 'Fetching graph endpoints...');
                await loadLiveDatasets();

                updateProgress(100, 'Rendering visualization... 100%');
                applyLoadedData();
                hideProgress();
                startLiveRefresh();
                return;
              }
              
              if (status.progress && status.progress.state === 'error') {
                 updateProgress(100, 'Error: ' + status.progress.message);
                 return;
              }
            }
          } catch (e) {
            console.warn('Live fetch failed, using fallback empty state', e);
            applyLoadedData();
            hideProgress();
            return;
          }
          setTimeout(pollStatus, 150);
        };
        pollStatus();
      }
    }

    // Each live dataset downloads and parses on its own, so one that fails
    // (on large repos, a text too big to parse) cannot skip the others. The
    // page reads justification and proposals from the lean /api/analysis
    // (the export repeats all three graphs); collateral comes on demand; the
    // symbol and call graphs, often the largest, load when their tab opens.
    async function fetchLiveJson(path) {
      try {
        const res = await fetch(path);
        if (!res || !res.ok) return null;
        return await res.json();
      } catch (e) {
        console.warn('Live fetch failed: ' + path, e);
        return null;
      }
    }

    let lazyGraphsLoaded = { symbol: false, call: false };

    async function loadLiveDatasets() {
      lazyGraphsLoaded = { symbol: false, call: false };
      const [include, analysis] = await Promise.all([
        fetchLiveJson('/api/graph/include'),
        fetchLiveJson('/api/analysis'),
        resetLiveCollateral(),
      ]);
      if (include) graphData.include_graph = include;
      if (analysis) {
        graphData.file_justification = analysis.file_justification || {};
        graphData.extraction_proposals = analysis.extraction_proposals || [];
      }
      if (!lazyGraphReady(currentTab)) await ensureLazyGraph(currentTab);
    }

    function lazyGraphReady(tab) {
      return isStandaloneReport() || !(tab in lazyGraphsLoaded) || lazyGraphsLoaded[tab];
    }

    // One attempt per generation: a failed load renders the tab empty
    // instead of retrying in a loop; the next live refresh tries again.
    async function ensureLazyGraph(tab) {
      if (lazyGraphReady(tab)) return;
      const elements = await fetchLiveJson('/api/graph/' + tab);
      if (elements) graphData[tab + '_graph'] = elements;
      lazyGraphsLoaded[tab] = true;
    }

    function startLiveRefresh() {
      const chip = document.getElementById('refreshChip');
      const loop = async () => {
        try {
          const r = await fetch('/api/status');
          if (r.ok) {
            const s = await r.json();
            if (chip) {
              chip.style.display = s.rebuilding ? 'inline-block' : 'none';
              if (s.stages && s.stages.length) {
                chip.title = s.stages.map(st => st.stage + ': ' + st.reuse).join('\n');
              }
            }
            if (s.last_refresh && s.last_refresh !== lastSeenRefresh) {
              lastSeenRefresh = s.last_refresh;
              if (s.stats) graphData.stats = s.stats;
              await loadLiveDatasets();
              // Payload swap: drop per-node /api/justification detail cached against the previous build.
              justificationCache = {};
              applyLoadedData();
            }
          }
        } catch (e) { /* transient fetch failure; retry on next tick */ }
        setTimeout(loop, 2000);
      };
      setTimeout(loop, 2000);
    }

    // Inspector include-justification breakdown: live mode hydrates the unbounded
    // /api/justification detail (cached per node so reselecting never refetches);
    // standalone reports render from the capped export payload only (I5).
    let justificationCache = {};

    function isStandaloneReport() {
      return Boolean(window.__ARIADNE_DATA__ && window.__ARIADNE_DATA__.include_graph);
    }

    function fetchJustificationDetail(nodeId) {
      if (isStandaloneReport()) return Promise.resolve(null);
      if (justificationCache[nodeId]) return Promise.resolve(justificationCache[nodeId]);
      return fetch('/api/justification?file=' + encodeURIComponent(nodeId))
        .then(res => (res && res.ok) ? res.json() : null)
        .catch(() => null)
        .then(detail => {
          if (detail) justificationCache[nodeId] = detail;
          return detail;
        });
    }

    // null on failure, so a transient error is retried, never cached.
    function fetchCollateralJson(path) {
      return fetch(path)
        .then(res => (res && res.ok) ? res.json() : null)
        .catch(() => null);
    }

    // New generation: drop cached rows and columns, reload the per-header
    // burden totals, and refetch what the current selection reads.
    async function resetLiveCollateral() {
      if (isStandaloneReport()) return;
      graphData.file_collateral = {};
      collateralColumnCache = {};
      graphData.collateral_burden = (await fetchCollateralJson('/api/collateral/burden')) || {};
      if (selectedNodeId) await ensureCollateralFor(selectedNodeId, hotspotHeatLens === 'in');
    }

    function collateralReadyFor(id, needsColumn) {
      if (isStandaloneReport()) return true;
      const rowReady = Boolean((graphData.file_collateral || {})[id]);
      return rowReady && (!needsColumn || Boolean(collateralColumnCache[id]));
    }

    // Fetch the row (and, for the payer view, the column) of `id` if missing.
    function ensureCollateralFor(id, needsColumn) {
      if (collateralReadyFor(id, needsColumn)) return Promise.resolve();
      const fileParam = encodeURIComponent(id);
      const rowLoad = (graphData.file_collateral || {})[id]
        ? null
        : fetchCollateralJson('/api/collateral/row?file=' + fileParam).then(row => {
            if (!row) return;
            if (!graphData.file_collateral) graphData.file_collateral = {};
            graphData.file_collateral[id] = row;
          });
      const columnLoad = (!needsColumn || collateralColumnCache[id])
        ? null
        : fetchCollateralJson('/api/collateral/column?file=' + fileParam).then(column => {
            if (column) collateralColumnCache[id] = column;
          });
      return Promise.all([rowLoad, columnLoad]);
    }

    function justificationCombined(row) {
      return (row.entry.justified_count || 0) + (row.entry.collateral_count || 0);
    }

    function justificationBasename(id) {
      const node = tabNodeById(id);
      const path = String(node ? (node.path || node.label) : id).replace(/\\/g, '/');
      return path.substring(path.lastIndexOf('/') + 1) || id;
    }

    // Payload path: capped export entries, re-sorted per I6 (combined desc, id asc).
    function payloadJustificationRows(nodeId) {
      const map = (graphData.file_justification || {})[nodeId] || {};
      return Object.keys(map)
        .map(id => ({ id, label: justificationBasename(id), entry: map[id], impact: map[id].impact || null }))
        .sort((a, b) => justificationCombined(b) - justificationCombined(a) || Number(a.id) - Number(b.id))
        .slice(0, 16);
    }

    function justificationSidePart(tag, tagClass, names, truncated, count) {
      if (!count && names.length === 0) return '';
      const hidden = count - names.length;
      const more = truncated && hidden > 0 ? `<span class="just-more">+${hidden} more</span>` : '';
      const text = names.length ? escapeHtml(names.join(', ')) + more : '<em>none</em>';
      return `<div class="just-part"><span class="just-tag ${tagClass}">${tag}</span><span>${text}</span></div>`;
    }

    function justificationNeedsPart(headers) {
      if (!headers || headers.length === 0) return '';
      return `<div class="just-part"><span class="just-tag just-needs">needs</span><span>${escapeHtml(headers.join(', '))}</span></div>`;
    }

    function justificationRowHtml(row, expanded, nodeId) {
      const e = row.entry;
      const body = [
        justificationSidePart('for', 'just-justified', e.justified_symbols || [], e.justified_truncated, e.justified_count || 0),
        justificationNeedsPart(e.required_headers),
        justificationSidePart('collateral', 'just-collateral', e.collateral_symbols || [], e.collateral_truncated, e.collateral_count || 0),
      ].join('');
      const previewBtn = row.impact
        ? `<button type="button" class="just-preview-btn" data-preview-root="${escapeAttribute(String(nodeId))}" data-preview-source="${escapeAttribute(String(row.id))}">Preview after extraction</button>`
        : '';
      const summary = `<summary><span style="color:var(--cyan);">${escapeHtml(row.label || row.id)}</span>`
        + `<span style="color:var(--text-sub);">${e.justified_count || 0} sym · ${e.collateral_count || 0} col</span></summary>`;
      return `<details class="link-item just-row"${expanded ? ' open' : ''}>${summary}<div class="just-body">${body || '<div class="just-part">No symbol names recorded.</div>'}${previewBtn}</div></details>`;
    }

    function justificationTotals(rows, detail) {
      if (detail) return `${detail.total_justified || 0} justified · ${detail.total_collateral || 0} collateral`;
      let justified = 0;
      let collateral = 0;
      rows.forEach(r => {
        justified += r.entry.justified_count || 0;
        collateral += r.entry.collateral_count || 0;
      });
      return `${justified} justified · ${collateral} collateral`;
    }

    function renderJustificationBreakdown(nodeId, detail) {
      const section = document.getElementById('inspJustificationSection');
      if (!section) return false;
      const rows = (detail && Array.isArray(detail.entries))
        ? detail.entries.slice(0, 16).map(e => ({ id: String(e.source_id), label: e.source_label, entry: e.entry, impact: e.impact || null }))
        : payloadJustificationRows(nodeId);
      if (rows.length === 0) {
        section.style.display = 'none';
        return false;
      }
      const list = document.getElementById('inspJustificationList');
      list.innerHTML = rows.map((row, i) => justificationRowHtml(row, i === 0, nodeId)).join('');
      if (window.bindPreviewControls) window.bindPreviewControls(list);
      document.getElementById('inspJustificationCount').textContent = justificationTotals(rows, detail);
      section.style.display = 'flex';
      const collateralSection = document.getElementById('inspCollateralSection');
      if (collateralSection) collateralSection.style.display = 'none';
      return true;
    }

    // Inspector tie-in (D2): when the selected node is a proposal target, show the
    // matching card built by hotspots.js' shared renderExtractionProposalCard (I5).
    // No match — or empty/absent extraction_proposals — hides the section entirely (I4).
    function renderInspectorProposalCard(nodeId) {
      const section = document.getElementById('inspProposalSection');
      if (!section) return;
      const proposals = graphData.extraction_proposals || [];
      const match = nodeId !== null && nodeId !== undefined
        ? proposals.find(p => String(p.target_file_id) === String(nodeId))
        : null;
      if (!match) {
        section.style.display = 'none';
        return;
      }
      const cardSlot = document.getElementById('inspProposalCard');
      cardSlot.innerHTML = renderExtractionProposalCard(match);
      if (window.bindPreviewControls) window.bindPreviewControls(cardSlot);
      section.style.display = 'block';
    }

// Client-side traversal, shortest path and entrypoints modals.
    function clientTraverseNeighborhood(id) { setHops(1); }
    function clientShortestPath(from, to) {
      document.getElementById('pathFrom').value = from;
      document.getElementById('pathTo').value = to;
      document.getElementById('pathfinderModal').style.display='block';
      document.getElementById('modalOverlay').style.display='block';
      findPath();
    }
    function clientGetEntrypoints() { showEntrypoints(); }

    // Traversal API
    async function fetchTraversalData(rootId, depId, symbolIds = []) {
      const params = new URLSearchParams({ root: rootId, dep: depId });
      if (symbolIds.length > 0) {
        params.append('symbols', symbolIds.join(','));
      }
      const res = await fetch(`/api/graph/traversal?${params.toString()}`);
      if (!res.ok) throw new Error(`Traversal fetch failed: ${res.status}`);
      return res.json();
    }
    function closeModals() {
      document.getElementById('pathfinderModal').style.display='none';
      document.getElementById('entrypointsModal').style.display='none';
      document.getElementById('modalOverlay').style.display='none';
    }
    async function findPath() {
      const from = document.getElementById('pathFrom').value;
      const to = document.getElementById('pathTo').value;
      const resultDiv = document.getElementById('pathResult');
      resultDiv.innerHTML = 'Searching...';
      try {
        const res = await fetch(`/api/graph/path?graph_type=${currentTab}&from=${encodeURIComponent(from)}&to=${encodeURIComponent(to)}`);
        if (res.ok) {
          const path = await res.json();
          resultDiv.innerHTML = path.map(n => `<span class="path-node">${escapeHtml(n)}</span>`).join('<span class="path-sep">&rarr;</span>');
          if (path.length > 0) {
            selectNode(path[0]);
          }
        } else {
          resultDiv.innerHTML = 'Path not found.';
        }
      } catch(e) {
        resultDiv.innerHTML = 'Error finding path.';
      }
    }
    async function showEntrypoints() {
      document.getElementById('entrypointsModal').style.display='block';
      document.getElementById('modalOverlay').style.display='block';
      const listDiv = document.getElementById('entrypointsList');
      listDiv.innerHTML = 'Loading...';
      try {
        const res = await fetch(`/api/graph/entrypoints?graph_type=${currentTab}&limit=10`);
        if (res.ok) {
          const eps = await res.json();
          listDiv.innerHTML = eps.map(ep => `<div style="cursor:pointer; color:var(--cyan); margin: 5px 0;" data-select-id="${escapeAttribute(ep)}">${escapeHtml(ep)}</div>`).join('');
        } else {
          listDiv.innerHTML = 'No entrypoints found.';
        }
      } catch(e) {
        listDiv.innerHTML = 'Error loading entrypoints.';
      }
    }
