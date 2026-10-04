// Include-tab side panels split from index.html: directory quick-open navigation and hotspots triage.
    document.getElementById('dirSelect').onchange = () => filterAndExpandSubgraph();

    // === DIRECTORY QUICK-OPEN NAVIGATION (Include tab) ===
    function buildDirTree(nodes) {
      const root = { dirs: new Map(), files: [], prefix: '' };
      nodes.forEach(n => {
        const p = n.path || '';
        if (!p || p === n.label || !p.includes('/')) { root.files.push(n); return; }
        const segs = p.split('/');
        let cur = root;
        for (let i = 0; i < segs.length - 1; i++) {
          const seg = segs[i];
          if (!cur.dirs.has(seg)) cur.dirs.set(seg, { dirs: new Map(), files: [], prefix: segs.slice(0, i + 1).join('/') + '/' });
          cur = cur.dirs.get(seg);
        }
        cur.files.push(n);
      });
      return root;
    }

    function countMatchesIn(sub, matcher) {
      let c = sub.files.filter(n => matchesFilter(n, matcher)).length;
      sub.dirs.forEach(d => { c += countMatchesIn(d, matcher); });
      return c;
    }

    function renderQuickOpen() {
      const panel = document.getElementById('quickOpenPanel');
      if (!panel) return;
      if (!qoOpen || currentTab !== 'include') { panel.style.display = 'none'; return; }

      const q = document.getElementById('searchInput').value.trim();
      const matcher = q ? compileMatcher(q) : null;
      const tree = buildDirTree(tabAllNodes);
      qoRows = [];

      function collect(sub, depth) {
        const dirs = Array.from(sub.dirs.entries()).sort((a, b) => a[0] < b[0] ? -1 : 1);
        dirs.forEach(([name, child]) => {
          let count = 0;
          if (matcher) {
            count = countMatchesIn(child, matcher);
            if (count === 0) return;
          }
          const expanded = matcher ? true : (qoExpanded.has(child.prefix) || depth === 0);
          qoRows.push({ type: 'dir', name: name + '/', prefix: child.prefix, depth, expanded, count });
          if (expanded) collect(child, depth + 1);
        });
        sub.files.slice().sort((a, b) => a.label < b.label ? -1 : 1).forEach(n => {
          if (matcher && !matchesFilter(n, matcher)) return;
          qoRows.push({ type: 'file', node: n, depth });
        });
      }
      collect(tree, 0);

      const capped = qoRows.length > 200;
      if (capped) {
        const dropped = qoRows.length - 200;
        qoRows = qoRows.slice(0, 200);
        qoRows.push({ type: 'more', text: `… ${dropped} more files` });
      }

      panel.innerHTML = qoRows.map((r, i) => {
        const pad = r.depth * 14;
        if (r.type === 'dir') {
          return `<div class="qo-row qo-dir${i === qoActiveRow ? ' active' : ''}" data-idx="${i}" style="padding-left:${8 + pad}px">` +
            `<span class="qo-caret">${r.expanded ? '▾' : '▸'}</span><span>${escapeHtml(r.name)}</span>` +
            (r.count ? `<span class="qo-count">${r.count}</span>` : '') + `</div>`;
        }
        if (r.type === 'file') {
          return `<div class="qo-row qo-file${i === qoActiveRow ? ' active' : ''}" data-idx="${i}" style="padding-left:${8 + pad + 12}px">` +
            `<span>${escapeHtml(r.node.label)}</span><span class="qo-count">${escapeHtml(r.node.kind || '')}</span></div>`;
        }
        return `<div class="qo-more" style="padding-left:${8 + pad}px">${escapeHtml(r.text)}</div>`;
      }).join('');
      panel.style.display = 'block';
    }

    function openQuickOpen() {
      if (currentTab !== 'include') return;
      qoOpen = true;
      qoActiveRow = -1;
      renderQuickOpen();
    }

    function closeQuickOpen() {
      qoOpen = false;
      qoActiveRow = -1;
      const panel = document.getElementById('quickOpenPanel');
      if (panel) panel.style.display = 'none';
    }

    function moveQoRow(delta) {
      if (qoRows.length === 0) return;
      let idx = qoActiveRow;
      for (let step = 0; step < qoRows.length; step++) {
        idx = (idx + delta + qoRows.length) % qoRows.length;
        if (qoRows[idx].type !== 'more') break;
      }
      if (qoRows[idx].type === 'more') return;
      qoActiveRow = idx;
      renderQuickOpen();
      const active = document.getElementById('quickOpenPanel').querySelector('.qo-row.active');
      if (active) active.scrollIntoView({ block: 'nearest' });
    }

    function activateQoRow(row) {
      if (!row) return;
      if (row.type === 'dir') setDirScope(row.prefix);
      else if (row.type === 'file') { closeQuickOpen(); selectNode(row.node.id); }
    }

    function qoPanelClick(e) {
      const rowEl = e.target.closest('.qo-row');
      if (!rowEl) return;
      const row = qoRows[Number(rowEl.dataset.idx)];
      if (!row) return;
      if (e.target.closest('.qo-caret') && row.type === 'dir') {
        if (qoExpanded.has(row.prefix)) qoExpanded.delete(row.prefix);
        else qoExpanded.add(row.prefix);
        renderQuickOpen();
        return;
      }
      activateQoRow(row);
    }

    function qoKeyDown(e) {
      if (currentTab !== 'include') return;
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (!qoOpen) openQuickOpen();
        else moveQoRow(1);
      } else if (e.key === 'ArrowUp' && qoOpen) {
        e.preventDefault();
        moveQoRow(-1);
      } else if (e.key === 'Enter' && qoOpen && qoActiveRow >= 0) {
        e.preventDefault();
        activateQoRow(qoRows[qoActiveRow]);
      } else if ((e.key === 'ArrowLeft' || e.key === 'ArrowRight') && qoOpen) {
        const r = qoRows[qoActiveRow];
        if (r && r.type === 'dir') {
          e.preventDefault();
          if (e.key === 'ArrowLeft') qoExpanded.delete(r.prefix);
          else qoExpanded.add(r.prefix);
          renderQuickOpen();
        }
      } else if (e.key === 'Escape') {
        if (qoOpen) { e.stopPropagation(); closeQuickOpen(); }
        else if (dirScope) { e.stopPropagation(); clearDirScope(); }
      }
    }

    function setDirScope(prefix) {
      dirScope = prefix;
      closeQuickOpen();
      updateDirChip();
      filterAndExpandSubgraph();
      renderHotspots();
    }

    function clearDirScope() {
      dirScope = null;
      updateDirChip();
      filterAndExpandSubgraph();
      renderHotspots();
    }

    function updateDirChip() {
      const chip = document.getElementById('dirScopeChip');
      if (!chip) return;
      if (dirScope && currentTab === 'include') {
        chip.style.display = 'inline-flex';
        document.getElementById('dirScopeLabel').textContent = '📁 ' + dirScope;
      } else {
        chip.style.display = 'none';
      }
    }

    // === HOTSPOTS PANEL (Include tab) ===
    // Aggregate collateral burden over the full file_collateral map (rootFileId -> {depId: symbols}).
    // Cached by object identity; a payload reload swaps graphData.file_collateral and invalidates it.
    function getBurdenMaps() {
      const collateral = graphData.file_collateral || {};
      if (hotspotBurdenSource !== collateral) {
        const burden = {};
        const affected = {};
        Object.keys(collateral).forEach(root => {
          const deps = collateral[root] || {};
          Object.keys(deps).forEach(dep => {
            burden[dep] = (burden[dep] || 0) + deps[dep];
            affected[dep] = (affected[dep] || 0) + 1;
          });
        });
        hotspotBurdenCache = { burden, affected };
        hotspotBurdenSource = collateral;
      }
      return hotspotBurdenCache;
    }

    function escapeHtml(s) {
      return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
    }

    // Rank ALL files (not just the capped active view), optionally filtered by dirScope.
    function computeHotspots(metric) {
      let nodes = metric === 'cycle' ? tabAllNodes.filter(n => n.is_cycle) : tabAllNodes.slice();
      if (dirScope) nodes = nodes.filter(n => (n.path || '').startsWith(dirScope));

      let entries;
      if (metric === 'burden') {
        const maps = getBurdenMaps();
        entries = nodes.map(n => ({ node: n, value: maps.burden[n.id] || 0, affected: maps.affected[n.id] || 0 }));
      } else if (metric === 'fanin') {
        entries = nodes.map(n => ({ node: n, value: n.in_degree || 0 }));
      } else if (metric === 'fanout') {
        entries = nodes.map(n => ({ node: n, value: n.out_degree || 0 }));
      } else if (metric === 'closure') {
        entries = nodes.map(n => ({ node: n, value: n.closure || 0 }));
      } else {
        entries = nodes.map(n => ({ node: n, value: (n.in_degree || 0) + (n.out_degree || 0) }));
      }
      entries.sort((a, b) => b.value - a.value);
      return entries.filter(e => e.value > 0);
    }

    function renderHotspots() {
      syncExtractionProposalSurfaces();
      const panel = document.getElementById('hotspotPanel');
      if (!hotspotOpen || !panel || currentTab !== 'include') return;

      let metric = document.getElementById('hotspotMetric').value;
      const hint = document.getElementById('hotspotHint');
      if (metric === 'burden' && Object.keys(graphData.file_collateral || {}).length === 0) {
        metric = 'fanin';
        document.getElementById('hotspotMetric').value = 'fanin';
        hint.textContent = 'No collateral data in this payload — showing Fan-in instead.';
      } else {
        hint.textContent = dirScope ? 'Scoped to 📁 ' + dirScope : 'Top offenders across all files';
      }

      const entries = computeHotspots(metric).slice(0, 25);
      const top = entries.length > 0 ? entries[0].value : 0;
      const list = document.getElementById('hotspotList');
      if (entries.length === 0) {
        list.innerHTML = '<div style="color:var(--text-sub);font-style:italic;padding:4px;">No hotspots for this metric.</div>';
        return;
      }
      list.innerHTML = entries.map((e, i) => {
        const pct = top > 0 ? Math.max(4, Math.round((e.value / top) * 100)) : 0;
        let tip = String(e.value);
        if (metric === 'burden') tip = e.affected + ' file(s) pay ' + e.value + ' symbols';
        else if (metric === 'cycle') tip = e.node.in_degree + ' in / ' + e.node.out_degree + ' out';
        const active = e.node.id === selectedNodeId ? ' active' : '';
        return `
          <div class="hotspot-row${active}" onclick="selectHotspotRow('${e.node.id}')" title="${escapeHtml(tip)}">
            <span class="hotspot-rank">${i + 1}</span>
            <span class="hotspot-name">${escapeHtml(e.node.label)}</span>
            <span class="hotspot-val">${e.value}</span>
            <span class="hotspot-path">${escapeHtml(e.node.path || '')}</span>
            <span class="hotspot-bar"><span class="hotspot-bar-fill" style="width:${pct}%;"></span></span>
          </div>
        `;
      }).join('');
    }

    function selectHotspotRow(id) {
      selectNode(id);
    }

    // === PROPOSED EXTRACTIONS (payload D1 data; one builder for both surfaces) ===
    // Card renderer shared by the hotspot panel list and the inspector tie-in
    // (api.js renderInspectorProposalCard). Payload order is already sorted
    // (delta desc, plan_id asc) per D1, so render it as-is; never re-sort here.
    // The preview control (E3) runs against the CURRENTLY selected consumer root,
    // never the planner's global target — the label says so — and only appears
    // when that root's justification data carries impact for this card's file.
    // Chips replace the old comma run-on blobs: every item passes through
    // escapeHtml here once, so both chip rows (moved symbols, required
    // headers) share one escape path.
    function extractionChips(items) {
      const escaped = (items || []).map(escapeHtml);
      return escaped.map(safeLabel => `<span class="extraction-chip">${safeLabel}</span>`).join('');
    }

    // Derived saving text for the cost beat: '-D (-P%)' shown only when the
    // before load is a real positive baseline AND the after load is genuinely
    // smaller - no division by zero, no negative percent otherwise.
    function extractionSavingText(before, after) {
      if (!(before > 0) || !(after < before)) return '';
      const drop = before - after;
      const percent = Math.round(drop * 100 / before);
      return ` <span class="extraction-saving">-${drop} (-${percent}%)</span>`;
    }

    // Tiered risk badge: the class suffix is lowercased and stripped to
    // [a-z], so unknown values degrade to the neutral base rule instead of
    // injecting fragments into the class attribute.
    function renderExtractionRiskTag(risk) {
      const tier = (risk || '').toLowerCase().replace(/[^a-z]/g, '');
      return `<span class="extraction-risk extraction-risk-${tier}">Risk: ${escapeHtml(risk || '')}</span>`;
    }

    // Five-beat story card (why -> what moves -> what it costs -> who is
    // affected -> preview): identity head with plan_id chip, the problem
    // sentence visible in the body, a labeled collateral drop with derived
    // saving, symbol/header chips, and the reader-worded consumer-TU count.
    // The preview button markup stays byte-faithful (class, label, attrs).
    function renderExtractionProposalCard(p) {
      const targetLabel = escapeHtml(p.target_file || String(p.target_file_id));
      const before = p.collateral_load_before || 0;
      const after = p.collateral_load_after || 0;
      const saving = extractionSavingText(before, after);
      const hiddenCount = (p.total_symbols || 0) - (p.symbol_names || []).length;
      const overflow = hiddenCount > 0 ? `<span class="just-more">+${hiddenCount} more</span>` : '';
      const symbolChips = (p.symbol_names || []).length
        ? `${extractionChips(p.symbol_names)}${overflow}`
        : '<em>no symbol names</em>';
      const headerChips = (p.required_headers || []).length
        ? extractionChips(p.required_headers)
        : '<em>no additional headers</em>';
      const previewRoot = selectedNodeId === null || selectedNodeId === undefined ? null : String(selectedNodeId);
      const previewable = previewRoot !== null && window.ariadnePreview
        && window.ariadnePreview.hasImpact(previewRoot, String(p.target_file_id));
      const previewBtn = previewable
        ? `<div class="extraction-row"><button type="button" class="just-preview-btn" data-preview-root="${escapeHtml(previewRoot)}" data-preview-source="${escapeHtml(String(p.target_file_id))}">Preview drop from selected file</button></div>`
        : '';
      return `
        <div class="extraction-card" onclick="selectHotspotRow('${escapeHtml(String(p.target_file_id))}')" title="${targetLabel}">
          <div class="extraction-head">
            <span class="extraction-plan-id">${escapeHtml(p.plan_id || '')}</span>
            <span class="extraction-title">${targetLabel}</span>
          </div>
          <div class="extraction-problem"><span class="extraction-label">Why</span>${escapeHtml(p.problem || '')}</div>
          <div class="extraction-cost">
            <span class="extraction-label">Collateral drop</span>
            <span class="extraction-delta">${before} &rarr; ${after}</span>${saving}
          </div>
          <div class="extraction-row"><span class="extraction-label">Extract (${p.total_symbols || 0})</span><span class="extraction-chips">${symbolChips}</span></div>
          <div class="extraction-row"><span class="extraction-label">extracted symbols need</span><span class="extraction-chips">${headerChips}</span></div>
          <div class="extraction-row"><span class="extraction-label">Impact</span><span class="extraction-tu">${p.consumer_tu_count || 0} consumer TU(s) need re-validation</span>${renderExtractionRiskTag(p.risk)}</div>
          ${previewBtn}
        </div>`;
    }

    // Section lives and dies with the payload: absent/empty extraction_proposals
    // hides the whole section (I4), byte-identical to pre-D2 output. The E3
    // preview control is selection-dependent, so the source cache also keys on
    // the selected root to refresh card buttons when the selection changes.
    let extractionProposalsSource = null;
    let extractionProposalsRoot = null;
    function renderProposalsSection() {
      const section = document.getElementById('proposedExtractionsSection');
      if (!section) return;
      const proposals = graphData.extraction_proposals || [];
      if (proposals.length === 0) {
        section.style.display = 'none';
        extractionProposalsSource = null;
        extractionProposalsRoot = null;
        return;
      }
      if (extractionProposalsSource === proposals && extractionProposalsRoot === selectedNodeId) return;
      document.getElementById('proposedExtractionsList').innerHTML =
        proposals.map(renderExtractionProposalCard).join('');
      if (window.bindPreviewControls) window.bindPreviewControls(document.getElementById('proposedExtractionsList'));
      extractionProposalsSource = proposals;
      extractionProposalsRoot = selectedNodeId;
      section.style.display = 'block';
    }

    function syncExtractionProposalSurfaces() {
      renderProposalsSection();
      renderInspectorProposalCard(selectedNodeId);
    }

    function openHotspots() {
      if (currentTab !== 'include') return;
      hotspotOpen = true;
      document.getElementById('hotspotPanel').style.display = 'flex';
      const restoreTab = document.getElementById('hotspotRestoreTab');
      if (restoreTab) restoreTab.hidden = true;
      renderHotspots();
    }

    function closeHotspots() {
      hotspotOpen = false;
      const panel = document.getElementById('hotspotPanel');
      if (panel) panel.style.display = 'none';
      const restoreTab = document.getElementById('hotspotRestoreTab');
      if (restoreTab) restoreTab.hidden = currentTab !== 'include';
    }

    document.addEventListener('click', e => {
      if (qoOpen && !e.target.closest('.search-wrap')) closeQuickOpen();
    });
