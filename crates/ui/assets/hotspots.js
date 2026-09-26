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

    function openHotspots() {
      if (currentTab !== 'include') return;
      hotspotOpen = true;
      document.getElementById('hotspotPanel').style.display = 'flex';
      document.getElementById('btnHotspots').classList.add('active');
      renderHotspots();
    }

    function closeHotspots() {
      hotspotOpen = false;
      const panel = document.getElementById('hotspotPanel');
      if (panel) panel.style.display = 'none';
      const btn = document.getElementById('btnHotspots');
      if (btn) btn.classList.remove('active');
    }

    function toggleHotspots() {
      if (currentTab !== 'include') return;
      if (hotspotOpen) closeHotspots();
      else openHotspots();
    }

    document.addEventListener('click', e => {
      if (qoOpen && !e.target.closest('.search-wrap')) closeQuickOpen();
    });
