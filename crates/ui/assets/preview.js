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
      document.getElementById('legendViewLabel').textContent =
        '\u26A0 After extraction (hypothetical)';
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
      const proposalRow = `
        <div class="legend-row">
          <div class="legend-color" style="background:transparent;border:1px dashed #a78bfa;"></div>
          <span>Proposed header (placeholder name)</span>
        </div>`;
      document.getElementById('legendItems').innerHTML = summary + rows + proposalRow;
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
        layer.querySelectorAll('[data-preview-synthetic="1"]').forEach(el => el.remove());
      }
      const edgesLayer = document.getElementById('edgesLayer');
      if (edgesLayer) {
        edgesLayer.querySelectorAll('[data-preview-synthetic="1"]').forEach(el => el.remove());
        edgesLayer.querySelectorAll('.preview-slim-edge').forEach(el => {
          el.classList.remove('preview-slim-edge');
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
      const proposal = previewInjectProposedNode(root, source);
      previewInjectProposedEdges(proposal, resolved.full, resolved.impact);
      previewPaintLegend(resolved.full, resolved.impact);
      previewShowHypothetical();
      previewActivateButton(triggerBtn || null);
    }

    function previewFindNode(id) {
      return tabAllNodes.find(node => String(node.id) === String(id));
    }

    // Mirrors renderSvg's card label rule: 17 chars, then ".." on 15.
    function previewTruncatedCardLabel(label) {
      return label.length > 17 ? label.substring(0, 15) + '..' : label;
    }

    // Placeholder naming follows the planner convention {}_extracted.h
    // (crates/cluster/src/planner.rs): split the stem at the LAST dot, and a
    // dotless basename keeps its whole name as the stem.
    function previewProposedFilename(sourceNode) {
      const base = previewBasename((sourceNode && (sourceNode.path || sourceNode.label)) || '');
      const dot = base.lastIndexOf('.');
      const stem = dot > 0 ? base.substring(0, dot) : base;
      return `${stem}_extracted.h`;
    }

    // Same bezier family as renderEdge: horizontal control offsets that bow
    // the curve toward the destination anchor.
    function previewProposedEdgePath(startX, startY, endX, endY) {
      const bend = Math.max(40, (endX - startX) * 0.5);
      return `M ${startX} ${startY} C ${startX + bend} ${startY}, ${endX - bend} ${endY}, ${endX} ${endY}`;
    }

    // Layered and grid gutters (~70px) can never fit a 170x42 card between
    // two cards, so the proposal parks below the source column: one row pitch
    // down per step until its AABB clears every RENDERED card (activeNodes —
    // tabAllNodes keeps stale coordinates for filtered-out nodes), then, if
    // the step budget runs out, below the whole layout.
    function previewProposedRectCollides(candidateX, candidateY, renderedNodes) {
      return renderedNodes.some(node =>
        candidateX < node.x + 170 && candidateX + 170 > node.x &&
        candidateY < node.y + 42 && candidateY + 42 > node.y);
    }

    function previewResolveProposedPosition(source, renderedNodes) {
      const x = source.x;
      let y = source.y + 72;
      let steps = 0;
      while (steps < 8 && previewProposedRectCollides(x, y, renderedNodes)) {
        y += 72;
        steps += 1;
      }
      if (previewProposedRectCollides(x, y, renderedNodes)) {
        return { x: x, y: Math.max(...renderedNodes.map(node => node.y)) + 72 };
      }
      return { x: x, y: y };
    }

    // The extraction result gets a node: a violet dashed card mirroring the
    // real card geometry, parked below the source column. I3: it is synthetic
    // chrome (no data-node-id, pointer-events: none via CSS), never selectable.
    // Returns the card origin plus endpoint nodes for edge anchoring, or null.
    function previewInjectProposedNode(rootId, sourceId) {
      const layer = previewNodesLayer();
      const root = tabAllNodes.find(node => String(node.id) === String(rootId));
      const source = tabAllNodes.find(node => String(node.id) === String(sourceId));
      if (!layer || !source) return null;
      const origin = previewResolveProposedPosition(source, activeNodes);

      const card = document.createElementNS('http://www.w3.org/2000/svg', 'g');
      card.setAttribute('class', 'node-group preview-proposed-node');
      card.setAttribute('transform', `translate(${origin.x}, ${origin.y})`);
      card.setAttribute('data-preview-synthetic', '1');
      const hint = document.createElementNS('http://www.w3.org/2000/svg', 'title');
      hint.textContent = 'Proposed header (hypothetical placeholder name)';
      card.appendChild(hint);

      const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
      rect.setAttribute('width', '170');
      rect.setAttribute('height', '42');
      card.appendChild(rect);

      const pill = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
      pill.setAttribute('cx', '18');
      pill.setAttribute('cy', '21');
      pill.setAttribute('r', '5');
      card.appendChild(pill);

      const label = previewTruncatedCardLabel(previewProposedFilename(source));
      const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      text.setAttribute('x', '32');
      text.setAttribute('y', '25');
      text.setAttribute('font-size', '11');
      text.setAttribute('font-family', 'monospace');
      text.setAttribute('font-weight', '600');
      text.textContent = label;
      card.appendChild(text);

      // Badge and label live on separate rows (y=12 vs baseline y=25), so it
      // always renders; x keeps its ~34px width 8px inside the 170px card.
      const badge = document.createElementNS('http://www.w3.org/2000/svg', 'text');
      badge.setAttribute('class', 'preview-proposed-badge');
      badge.setAttribute('x', '128');
      badge.setAttribute('y', '12');
      badge.setAttribute('font-size', '7');
      badge.setAttribute('fill', '#a78bfa');
      badge.textContent = 'PROPOSED';
      card.appendChild(badge);
      layer.appendChild(card);
      return { x: origin.x, y: origin.y, root: root, source: source };
    }

    // Proposed edges: root -> proposed header always; full payloads also fan
    // the proposed header out to each required header (capped, unknown ids
    // skipped). The real root->source edge gets the amber slim-edge treatment.
    function previewInjectProposedEdges(proposal, full, impact) {
      const edgesLayer = document.getElementById('edgesLayer');
      if (!edgesLayer || !proposal) return;
      const root = proposal.root;
      const source = proposal.source;
      const addEdge = (startX, startY, endX, endY) => {
        const edge = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        edge.setAttribute('d', previewProposedEdgePath(startX, startY, endX, endY));
        edge.setAttribute('class', 'edge-path preview-proposed-edge');
        edge.setAttribute('marker-end', 'url(#arrow-proposed)');
        edge.setAttribute('data-preview-synthetic', '1');
        edgesLayer.appendChild(edge);
      };

      if (root) addEdge(root.x + 160, root.y + 20, proposal.x, proposal.y + 20);
      if (root && full) {
        (impact.required_headers || []).slice(0, 16).forEach(header => {
          const needed = previewFindNode(header.file_id);
          if (needed) addEdge(proposal.x + 160, proposal.y + 20, needed.x, needed.y + 20);
        });
      }
      if (root) {
        const slimEdge = edgesLayer.querySelector(
          `.edge-path[data-edge-source="${root.id}"][data-edge-target="${source.id}"]`);
        if (slimEdge) slimEdge.classList.add('preview-slim-edge');
      }
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
