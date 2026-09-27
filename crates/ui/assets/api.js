// Data & boot layer split from index.html: live/offline data loading and client API helpers.
    // Initialize application
    async function init() {
      setupCanvasPanZoom();

      if (window.__ARIADNE_DATA__ && window.__ARIADNE_DATA__.include_graph) {
        showProgress('[1/3] Loading graph datasets... 25%', 25);
        // Use standalone exported data
        graphData = window.__ARIADNE_DATA__;
        setTimeout(() => {
          updateProgress(65, '[2/3] Processing graph models... 65%');
          setTimeout(() => {
             updateProgress(100, '[3/3] Rendering visualization... 100%');
             applyLoadedData();
             hideProgress();
          }, 50);
        }, 50);
      } else {
        showProgress('Connecting to backend...', 0);
        const overlay = document.getElementById('loadingOverlay');
        if (overlay) overlay.style.display = 'flex';

        let lastSeenRefresh = 0;

        const pollStatus = async () => {
          try {
            const statusRes = await fetch('/api/status');
            if (statusRes.ok) {
              const status = await statusRes.json();
              
              if (status.progress && status.progress.state === 'loading') {
                updateProgress(status.progress.percent, status.progress.phase + ': ' + status.progress.message);
                setTimeout(pollStatus, 150);
                return;
              }
              
              if (status.progress && status.progress.state === 'ready') {
                graphData.repo_name = status.repo_name || 'Repository';
                if (status.stats) graphData.stats = status.stats;
                lastSeenRefresh = status.last_refresh || 0;
                
                updateProgress(90, 'Fetching graph endpoints...');
                const [incRes, symRes, callRes, exportRes] = await Promise.all([
                  fetch('/api/graph/include').catch(() => null),
                  fetch('/api/graph/symbol').catch(() => null),
                  fetch('/api/graph/call').catch(() => null),
                  fetch('/api/export').catch(() => null),
                ]);

                if (incRes && incRes.ok) graphData.include_graph = await incRes.json();
                if (symRes && symRes.ok) graphData.symbol_graph = await symRes.json();
                if (callRes && callRes.ok) graphData.call_graph = await callRes.json();
                if (exportRes && exportRes.ok) {
                  const exportPayload = await exportRes.json();
                  graphData.file_collateral = exportPayload.file_collateral || {};
                  graphData.file_justification = exportPayload.file_justification || {};
                  graphData.extraction_proposals = exportPayload.extraction_proposals || [];
                }

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
              const [incRes, symRes, callRes, exportRes] = await Promise.all([
                fetch('/api/graph/include').catch(() => null),
                fetch('/api/graph/symbol').catch(() => null),
                fetch('/api/graph/call').catch(() => null),
                fetch('/api/export').catch(() => null),
              ]);
              if (incRes && incRes.ok) graphData.include_graph = await incRes.json();
              if (symRes && symRes.ok) graphData.symbol_graph = await symRes.json();
              if (callRes && callRes.ok) graphData.call_graph = await callRes.json();
              if (exportRes && exportRes.ok) {
                const p = await exportRes.json();
                graphData.file_collateral = p.file_collateral || {};
                graphData.file_justification = p.file_justification || {};
                graphData.extraction_proposals = p.extraction_proposals || [];
              }
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

    function justificationCombined(row) {
      return (row.entry.justified_count || 0) + (row.entry.collateral_count || 0);
    }

    function justificationBasename(id) {
      const node = tabAllNodes.find(n => n.id === id);
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
        ? `<button type="button" class="just-preview-btn" data-preview-root="${escapeHtml(String(nodeId))}" data-preview-source="${escapeHtml(String(row.id))}">Preview after extraction</button>`
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
      section.style.display = 'block';
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
          resultDiv.innerHTML = path.join(' &rarr; ');
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
          listDiv.innerHTML = eps.map(ep => `<div style="cursor:pointer; color:var(--cyan); margin: 5px 0;" onclick="selectNode('${ep}'); closeModals();">${ep}</div>`).join('');
        } else {
          listDiv.innerHTML = 'No entrypoints found.';
        }
      } catch(e) {
        listDiv.innerHTML = 'Error loading entrypoints.';
      }
    }
