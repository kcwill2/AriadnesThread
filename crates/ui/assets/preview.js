    // After-extraction preview overlay (E3): renders the modelled after-state of one
    // consumer edge (root F x source S) on the include heat map. Ships as its own
    // asset because graph.js sits at the E-I6 line budget; graph.js only calls
    // clearPreview() when a re-render drops the selection.
    //
    // Data (E-I4/E-I5): live mode marks from the FULL ExtractionImpact riding the
    // cached /api/justification route response; standalone reports only carry the
    // capped payload summary, so names mark a sample while the legend keeps the
    // true counts. Preview classes COMPOSE with (never replace) the C1 tri-state
    // and heat classes renderSvg applies underneath.
    let previewActiveKey = null;
    let previewLegendShown = false;

    function previewNodesLayer() {
      return document.getElementById('nodesLayer');
    }

    function previewBasename(p) {
      const s = String(p || '').replace(/\\/g, '/');
      return s.substring(s.lastIndexOf('/') + 1);
    }

    function previewHeaderIdSet(headers) {
      return new Set((headers || []).map(h => String(h.file_id)));
    }

    // Synchronous impact presence: live checks only what the route cache already
    // holds (never fetches here), standalone reads the capped payload entry.
    function hasImpact(rootId, sourceId) {
      if (rootId === null || rootId === undefined) return false;
      const root = String(rootId);
      const source = String(sourceId);
      if (!isStandaloneReport()) {
        const detail = justificationCache[root];
        if (detail && detail.entries &&
            detail.entries.some(e => String(e.source_id) === source && e.impact)) {
          return true;
        }
      }
      const entry = ((graphData.file_justification || {})[root] || {})[source];
      return Boolean(entry && entry.impact);
    }

    async function resolvePreviewImpact(root, source) {
      if (!isStandaloneReport()) {
        let detail = justificationCache[root];
        if (!detail) detail = await fetchJustificationDetail(root);
        const row = detail && detail.entries &&
          detail.entries.find(e => String(e.source_id) === source && e.impact);
        if (row) return { full: true, impact: row.impact };
      }
      const entry = ((graphData.file_justification || {})[root] || {})[source];
      if (entry && entry.impact) return { full: false, impact: entry.impact };
      return null;
    }

    function previewMarkNodes(rootId, sourceId, full, impact) {
      const layer = previewNodesLayer();
      if (!layer) return;
      let eliminated = null, still = null, kept = null, samples = null;
      if (full) {
        eliminated = previewHeaderIdSet(impact.dropped_from_root);
        still = previewHeaderIdSet(impact.still_reachable);
        kept = previewHeaderIdSet(impact.required_headers);
        kept.add(String(sourceId));
      } else {
        samples = new Set(impact.dropped_sample || []);
      }
      layer.querySelectorAll('.node-group').forEach(el => {
        const nid = el.dataset.nodeId;
        if (nid === String(rootId)) return;
        let cls = null;
        if (full) {
          if (eliminated.has(nid)) cls = 'preview-eliminated';
          else if (still.has(nid)) cls = 'preview-still-reachable';
          else if (kept.has(nid)) cls = 'preview-kept';
        } else {
          const node = tabAllNodes.find(n => n.id === nid);
          // Honest ambiguity: the capped sample exposes basenames only, so every
          // node with a matching basename is marked — same-named headers all light.
          if (node && (samples.has(node.label) || samples.has(previewBasename(node.path || node.label)))) {
            cls = 'preview-eliminated';
          }
        }
        if (cls) el.classList.add(cls);
      });
    }

    function previewPaintLegend(full, impact) {
      const dropped = full ? (impact.dropped_from_root || []).length : (impact.dropped_from_root_count || 0);
      const still = full ? (impact.still_reachable || []).length : (impact.still_reachable_count || 0);
      const hidden = full ? 0 : dropped - (impact.dropped_sample || []).length;
      const moreNote = hidden > 0 ? ` <span class="just-more">+${hidden} more eliminated</span>` : '';
      const sampleNote = full ? '' : ' <span class="just-more">(names capped to a sample)</span>';
      document.getElementById('legendViewLabel').textContent = '\u26A0 After extraction';
      const summary = `<div style="font-size:10px;color:var(--text-muted);margin-bottom:6px;">`
        + `After extraction: closure ${impact.root_closure_before || 0} &rarr; ${impact.root_closure_after || 0}`
        + ` &middot; ${dropped} headers dropped${moreNote}${sampleNote}`
        + ` &middot; ${still} still reach via other paths`
        + ` &middot; ${impact.symbols_no_longer_parsed || 0} symbols no longer parsed</div>`;
      const rows = [
        ['#ef4444', 'Eliminated — never included again'],
        ['#f59e0b', 'Still reachable via other paths'],
        ['#4ade80', 'Kept — needed by the slim edge'],
      ].map(item => `
        <div class="legend-row">
          <div class="legend-color" style="background:${item[0]};"></div>
          <span>${item[1]}</span>
        </div>`).join('');
      document.getElementById('legendItems').innerHTML = summary + rows;
      previewLegendShown = true;
    }

    // Restores the legend that WOULD be showing without a preview: the heat legend
    // while a node is selected on the include tab, the standard legend otherwise.
    function previewRestoreGraphLegend() {
      if (currentTab === 'include' && selectedNodeId) {
        const deps = computeFullDependencies(selectedNodeId);
        const collateralMap = (graphData.file_collateral || {})[selectedNodeId] || {};
        const justificationMap = (graphData.file_justification || {})[selectedNodeId] || {};
        const total = [...deps].reduce((s, id) => s + (collateralMap[id] || 0), 0);
        const rootNode = tabAllNodes.find(n => n.id === selectedNodeId);
        updateHeatLegend(rootNode ? rootNode.label : selectedNodeId, deps.size, total, justificationMap);
      } else {
        restoreStandardLegend();
      }
    }

    // The after-state is a hypothesis, not the current graph: frame the whole
    // canvas in amber dashed and raise the banner on top of it.
    function previewShowHypothetical() {
      const banner = document.getElementById('previewBanner');
      if (banner) banner.hidden = false;
      const container = document.getElementById('canvasContainer');
      if (container) container.classList.add('preview-hypothetical');
    }

    // Buttons arrive through innerHTML re-renders, so bookkeeping is a sweep of
    // the live DOM, not a stored element: null deactivates every button.
    function previewActivateButton(btn) {
      document.querySelectorAll('.just-preview-btn').forEach(other => {
        other.classList.remove('just-preview-btn-active');
      });
      if (btn) btn.classList.add('just-preview-btn-active');
    }

    function clearPreview() {
      previewActiveKey = null;
      const layer = previewNodesLayer();
      if (layer) {
        layer.querySelectorAll('.node-group').forEach(el => {
          el.classList.remove('preview-eliminated', 'preview-kept', 'preview-still-reachable');
        });
      }
      const banner = document.getElementById('previewBanner');
      if (banner) banner.hidden = true;
      const container = document.getElementById('canvasContainer');
      if (container) container.classList.remove('preview-hypothetical');
      previewActivateButton(null);
      if (previewLegendShown) {
        previewLegendShown = false;
        previewRestoreGraphLegend();
      }
    }

    // rootId must equal the current selection: the preview frames the edge FROM the
    // consumer being inspected. Re-clicking the same pair toggles the overlay off.
    async function applyPreview(rootId, sourceId, triggerBtn) {
      const root = String(rootId);
      const source = String(sourceId);
      const key = root + '|' + source;
      if (previewActiveKey === key) { clearPreview(); return; }
      if (selectedNodeId === null || String(selectedNodeId) !== root) return;
      const resolved = await resolvePreviewImpact(root, source);
      if (!resolved) return;
      if (String(selectedNodeId) !== root || currentTab !== 'include') return;
      clearPreview();
      previewActiveKey = key;
      previewMarkNodes(root, source, resolved.full, resolved.impact);
      previewPaintLegend(resolved.full, resolved.impact);
      previewShowHypothetical();
      previewActivateButton(triggerBtn || null);
    }

    // Buttons arrive through innerHTML re-renders, so listeners bind after each
    // render (no inline onclick strings). stopPropagation keeps the proposal-card
    // click-to-select handler from hijacking the preview toggle.
    function bindPreviewControls(container) {
      if (!container) return;
      container.querySelectorAll('.just-preview-btn').forEach(btn => {
        btn.addEventListener('click', e => {
          e.stopPropagation();
          window.ariadnePreview.applyPreview(btn.dataset.previewRoot, btn.dataset.previewSource, btn);
        });
      });
    }

    window.ariadnePreview = {
      applyPreview: applyPreview,
      clearPreview: clearPreview,
      hasImpact: hasImpact,
    };
