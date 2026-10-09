// Shared state, legend/heat/progress helpers, boot calls, and onboarding/help.
// This file loads LAST: shared `let` state initializes here before init() runs.
    // State
    let currentTab = 'include';
      let graphData = {
        repo_name: 'Ariadne Codebase',
        include_graph: [],
        symbol_graph: [],
        call_graph: [],
        stats: { total_files: 0, total_symbols: 0, total_calls: 0, cycle_count: 0 },
        file_collateral: {},
        file_justification: {},
        extraction_proposals: []
      };
    let activeNodes = [];
    let activeEdges = [];
    let tabAllNodes = [];
    let tabAllEdges = [];
    let nodeCap = 50;
    let currentDegree = 1;
    let isAllCapped = true;
    let selectedNodeId = null;
    let threadHistory = [];
    let hopDepth = 0;
    let cyclesOnly = false;

    // Directory quick-open navigation (Include tab only)
    let dirScope = null;            // e.g. "include/" — always ends with '/'
    let qoOpen = false;
    let qoExpanded = new Set();
    let qoActiveRow = -1;
    let qoRows = [];

    // Hotspots panel (Include tab only)
    let hotspotOpen = false;
    let hotspotBurdenCache = null;      // { burden: {dep: symbols}, affected: {dep: rootCount} }
    let hotspotBurdenSource = null;     // file_collateral object the cache was built from

    // Node Inspector panel (workspace column, collapsible)
    let inspectorOpen = true;

    // Viewport transform
    let panX = 40;
    let panY = 40;
    let zoomScale = 1.0;
    let isDragging = false;
    let dragStartX = 0;
    let dragStartY = 0;

    // Legends config
    const legends = {
      include: [
        { color: 'var(--node-header-text)', label: 'Local Header (.h)' },
        { color: 'var(--node-source-text)', label: 'Source File (.cpp)' },
        { color: 'var(--red)', label: 'Include Cycle' },
      ],
      symbol: [
        { color: 'var(--node-class-text)', label: 'Class / Struct' },
        { color: 'var(--node-function-text)', label: 'Function' },
        { color: 'var(--cyan)', label: 'Complete Edge (Solid)' },
        { color: 'var(--text-muted)', label: 'Incomplete Edge (Dashed)' },
      ],
      call: [
        { color: 'var(--node-function-text)', label: 'Function' },
        { color: 'var(--cyan)', label: 'Direct Call' },
        { color: 'var(--red)', label: 'Recursive Cycle' }
      ]
    };

    // Color mapper for node cards
    function getNodeColors(kind, isCycle) {
      if (isCycle) return { bg: 'var(--node-cycle-bg)', border: 'var(--node-cycle-border)', text: 'var(--node-cycle-text)' };
      switch (kind) {
        case 'HEADER':
          return { bg: 'var(--node-header-bg)', border: 'var(--node-header-border)', text: 'var(--node-header-text)' };
        case 'SOURCE':
          return { bg: 'var(--node-source-bg)', border: 'var(--node-source-border)', text: 'var(--node-source-text)' };
        case 'CLASS':
        case 'STRUCT':
          return { bg: 'var(--node-class-bg)', border: 'var(--node-class-border)', text: 'var(--node-class-text)' };
        case 'FUNCTION':
          return { bg: 'var(--node-function-bg)', border: 'var(--node-function-border)', text: 'var(--node-function-text)' };
        default:
          return { bg: 'var(--node-other-bg)', border: 'var(--node-other-border)', text: 'var(--node-other-text)' };
      }
    }

    // 5-tier thermal palette. t = collateral / maxCollateral (0..1).
    // Temperature must be derived from graphData.file_collateral (full dataset),
    // not from the degree-limited canvas.
    function getHeatColor(collateral, maxCollateral) {
      const t = maxCollateral > 0 ? collateral / maxCollateral : 0;
      if (t <= 0.15) return { bg: 'var(--heat-0-bg)', border: 'var(--heat-0-border)', text: 'var(--heat-0-text)', tier: 0 }; // Cool Ice
      if (t <= 0.35) return { bg: 'var(--heat-1-bg)', border: 'var(--heat-1-border)', text: 'var(--heat-1-text)', tier: 1 }; // Emerald
      if (t <= 0.60) return { bg: 'var(--heat-2-bg)', border: 'var(--heat-2-border)', text: 'var(--heat-2-text)', tier: 2 }; // Amber
      if (t <= 0.85) return { bg: 'var(--heat-3-bg)', border: 'var(--heat-3-border)', text: 'var(--heat-3-text)', tier: 3 }; // Hot Orange
      return { bg: 'var(--heat-4-bg)', border: 'var(--heat-4-border)', text: 'var(--heat-4-text)', tier: 4 };                 // Blazing Crimson
    }

    const HEAT_LEGEND = [
      { color: 'var(--heat-0-border)', label: 'Cool — Low Collateral' },
      { color: 'var(--heat-1-border)', label: 'Mild Collateral' },
      { color: 'var(--heat-2-border)', label: 'Moderate Collateral' },
      { color: 'var(--heat-3-border)', label: 'High Collateral' },
      { color: 'var(--heat-4-border)', label: 'Critical Collateral' },
    ];

    // Edge verdict story: root-relative verdict helpers. Null when the payload
    // has no entry — absence of data must never render a "0 used" claim.
    function classifyEdgeVerdict(entry) {
      if (!entry) return null;
      const j = entry.justified_count || 0;
      const c = entry.collateral_count || 0;
      if (j > 0 && c === 0) return 'justified';
      if (j > 0 && c > 0) return 'partial';
      if (j === 0 && c > 0) return 'unused';
      return null;
    }

    function verdictBadgeText(tier, entry) {
      const j = (entry && entry.justified_count) || 0;
      const c = (entry && entry.collateral_count) || 0;
      if (tier === 'justified') return `${j}/${j} used`;
      if (tier === 'mixed') return `${j}/${j + c} used`;
      return `0/${c} used`;
    }

    function verdictTooltipText(rootLabel, depLabel, tier, j, c) {
      if (tier === 'justified') {
        return `${rootLabel} uses ${j} of ${j} symbols from ${depLabel} — needed `
          + '(green spine: every supplied symbol is used).';
      }
      if (tier === 'mixed' || tier === 'partial') {
        return `${rootLabel} uses ${j} of ${j + c} symbols from ${depLabel} `
          + '(amber spine: partly needed).';
      }
      if (tier === 'collateral' || tier === 'unused') {
        return `${rootLabel} uses 0 of ${c} symbols from ${depLabel}. `
          + 'Nothing here is needed (red spine) — this include is droppable or slimmable. '
          + 'AT graphs project includes only; system/stdlib headers are not analyzed or counted.';
      }
      return '';
    }

    function appendVerdictTip(el, rootLabel, depLabel, verdict, entry) {
      const tip = document.createElementNS('http://www.w3.org/2000/svg', 'title');
      tip.textContent = verdictTooltipText(rootLabel, depLabel, verdict,
        (entry && entry.justified_count) || 0, (entry && entry.collateral_count) || 0);
      el.appendChild(tip);
    }

    // Node-card legend rows: teach the spine vocabulary. Gated exactly like
    // the arrow rows so the legend never promises a spine the payload cannot show.
    function verdictCardLegendRows(hasJustifiedVerdict, hasUnusedVerdict) {
      const justifiedRow = hasJustifiedVerdict ? `
        <div class="legend-row">
          <div class="legend-card spine-justified"></div>
          <span>Card: every symbol used — needed</span>
        </div>` : '';
      const partlyRow = `
        <div class="legend-row">
          <div class="legend-card spine-mixed"></div>
          <span>Card: partly needed</span>
        </div>`;
      const unusedRow = hasUnusedVerdict ? `
        <div class="legend-row">
          <div class="legend-card spine-unused"></div>
          <span>Card: supplies nothing — this include can go</span>
        </div>` : '';
      const noSpineRow = `
        <div class="legend-row">
          <div class="legend-card spine-none"></div>
          <span>No spine — no verdict data</span>
        </div>`;
      return justifiedRow + partlyRow + unusedRow + noSpineRow;
    }

    function updateHeatLegend(targetLabel, depCount, totalCollateral, justificationMap) {
      document.getElementById('legendViewLabel').textContent = 'Heat Map';
      const legendList = document.getElementById('legendItems');
      const summary = `<div style="font-size:10px;color:var(--text-muted);margin-bottom:6px;">`
        + `${depCount} dep(s) · ${totalCollateral} symbols</div>`;
      const verdictTiers = new Set(Object.values(justificationMap || {}).map(j => classifyEdgeVerdict(j)));
      const hasJustifiedVerdict = verdictTiers.has('justified');
      const hasUnusedVerdict = verdictTiers.has('unused');
      const greenRow = hasJustifiedVerdict ? `
        <div class="legend-row">
          <div class="legend-line edge-style-justified"></div>
          <span>Needed by selected file</span>
        </div>` : '';
      const amberRow = `
        <div class="legend-row">
          <div class="legend-line edge-style-partial"></div>
          <span>Partly needed (J/T used) or no verdict data</span>
        </div>`;
      const redRow = hasUnusedVerdict ? `
        <div class="legend-row">
          <div class="legend-line edge-style-unused"></div>
          <span>Unused include (red dashed) — this arrow can go</span>
        </div>` : '';
      const contextRow = `
        <div class="legend-row">
          <div class="legend-line edge-style-context"></div>
          <span>Transitive dependency — no verdict vs selection</span>
        </div>`;
      const cardRows = verdictCardLegendRows(hasJustifiedVerdict, hasUnusedVerdict);
      const heatHeader = `<div style="font-size:10px;color:var(--text-muted);margin-top:6px;">`
        + `Card fill = collateral weight</div>`;
      const footnote = `<div style="font-size:9px;color:var(--text-muted);margin-top:6px;">`
        + `Verdicts relative to selected file · system headers not analyzed</div>`;
      legendList.innerHTML = summary + greenRow + amberRow + redRow + contextRow + cardRows
        + heatHeader + HEAT_LEGEND.map(item => `
        <div class="legend-row">
          <div class="legend-color" style="background:${item.color};"></div>
          <span>${item.label}</span>
        </div>
      `).join('') + footnote;
    }

    function restoreStandardLegend() {
      document.getElementById('legendViewLabel').textContent =
        currentTab === 'include' ? 'Include Graph' : (currentTab === 'symbol' ? 'Symbol Dependency' : 'Call Graph');
      const legendList = document.getElementById('legendItems');
      legendList.innerHTML = (legends[currentTab] || []).map(item => `
        <div class="legend-row">
          <div class="legend-color" style="background:${item.color};"></div>
          <span>${item.label}</span>
        </div>
      `).join('');
    }

    // Tri-state classification of a reachable include dependency against the
    // per-source justification entry j (undefined when the payload has no entry).
    // Returns 'justified' | 'mixed' | 'collateral' | 'neutral'.
    function classifyDependency(depId, collateralCount, j) {
      if (!j) return 'neutral';
      const justified = j.justified_count || 0;
      const collateral = j.collateral_count || 0;
      if (justified > 0 && collateral === 0) return 'justified';
      if (justified > 0 && collateral > 0) return 'mixed';
      if (justified === 0 && collateral > 0) return 'collateral';
      return 'neutral';
    }

    // Verdict spine: flat inset flag bar on the card's left edge (same idiom
    // as the extraction-card border-left); neutral deps carry no spine.
    function appendVerdictSpine(el, tier) {
      const spineClass = tier === 'justified' ? 'justified'
        : tier === 'mixed' ? 'mixed'
        : tier === 'collateral' ? 'unused' : null;
      if (!spineClass) return;
      const spine = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
      spine.setAttribute('class', `verdict-spine ${spineClass}`);
      spine.setAttribute('x', '0');
      spine.setAttribute('y', '6');
      spine.setAttribute('width', '4');
      spine.setAttribute('height', '30');
      spine.setAttribute('rx', '2');
      el.appendChild(spine);
    }

    // Heat fill/stroke applies to EVERY reachable dep incl. justified (j>0,
    // c=0 lands in the cool tier); flat tier stroke, verdict on the spine.
    function styleDependencyCard(el, tier, collateral, maxCollateral) {
      const hc = getHeatColor(collateral, maxCollateral);
      const rect = el.querySelector('rect');
      if (rect) {
        rect.setAttribute('fill', hc.bg);
        rect.setAttribute('stroke', hc.border);
      }
      const label = el.querySelector('text');
      if (label) label.setAttribute('fill', hc.text);
      el.classList.add('heat-tier-' + hc.tier);
      if (hc.tier === 4) el.classList.add('heat-critical');
      appendVerdictSpine(el, tier);
      return hc;
    }

    // Badge text mirrors the spine verdict color; the neutral +N col badge
    // keeps the tier border color.
    function verdictBadgeColor(tier, tierColor) {
      if (tier === 'justified') return 'var(--green)';
      if (tier === 'mixed') return 'var(--amber)';
      if (tier === 'collateral') return 'var(--red)';
      return tierColor;
    }

    function recordCrumb(id, label) {
      const existing = threadHistory.findIndex(c => c.id === id);
      if (existing !== -1) {
        threadHistory = threadHistory.slice(0, existing + 1);
      } else {
        threadHistory.push({ id, label: label || id });
        if (threadHistory.length > 12) threadHistory.shift();
      }
      renderThreadBar();
    }

    function renderThreadBar() {
      const bar = document.getElementById('ariadneThread');
      if (!bar) return;
      if (threadHistory.length === 0) {
        bar.innerHTML = '<span class="thread-empty">Select nodes to lay a thread &mdash; your path appears here.</span>';
        return;
      }
      bar.innerHTML = threadHistory.map((c, i) => {
        const active = (i === threadHistory.length - 1 && selectedNodeId === c.id) ? ' active' : '';
        const sep = i > 0 ? '<span class="thread-sep">&#8250;</span>' : '';
        return `${sep}<span class="thread-item${active}" onclick="jumpToCrumb(${i})" title="${escapeHtml(c.id)}">${escapeHtml(c.label)}</span>`;
      }).join('');
      bar.scrollLeft = bar.scrollWidth;
    }

    function jumpToCrumb(i) {
      const crumb = threadHistory[i];
      if (!crumb) return;
      if (i === threadHistory.length - 1 && selectedNodeId === crumb.id) return;
      selectNode(crumb.id);
    }

    function findIncludePath(fromId, toId) {
      if (fromId === toId) return [fromId];
      const queue = [[fromId]];
      const visited = new Set([fromId]);
      while (queue.length > 0) {
        const path = queue.shift();
        const curr = path[path.length - 1];
        if (curr === toId) return path;
        tabAllEdges.forEach(e => {
          if (e.source === curr && !visited.has(e.target)) {
            visited.add(e.target);
            queue.push([...path, e.target]);
          }
        });
      }
      return null;
    }

    function illuminatePath(fromId, toId) {
      const path = findIncludePath(fromId, toId);
      if (!path) return;
      const pathNodes = new Set(path);
      const pathEdgeKeys = new Set();
      for (let i = 0; i < path.length - 1; i++) {
        pathEdgeKeys.add(`${path[i]}->${path[i + 1]}`);
      }

      const nodesLayer = document.getElementById('nodesLayer');
      const edgesLayer = document.getElementById('edgesLayer');
      if (!nodesLayer || !edgesLayer) return;

      nodesLayer.querySelectorAll('.node-group').forEach(el => {
        const nid = el.dataset.nodeId;
        if (pathNodes.has(nid)) {
          el.classList.remove('faded');
        } else {
          el.classList.add('faded');
          el.style.removeProperty('--fade-opacity');
        }
      });

      edgesLayer.querySelectorAll('.edge-path').forEach(el => {
        const src = el.dataset.edgeSource;
        const dst = el.dataset.edgeTarget;
        if (pathEdgeKeys.has(`${src}->${dst}`)) {
          el.classList.add('highlighted');
          el.classList.remove('faded');
        } else {
          el.classList.remove('highlighted');
          el.classList.add('faded');
          el.style.removeProperty('--edge-fade-opacity');
        }
      });
    }

    function showProgress(stageText, percent) {
      const overlay = document.getElementById('loadingOverlay');
      const prog = document.getElementById('loadingProgress');
      const txt = document.getElementById('loadingText');
      if (overlay) overlay.style.display = 'flex';
      if (prog) prog.style.width = percent + '%';
      if (txt) txt.textContent = stageText;
    }

    function updateProgress(percent, detailText) {
      // Single trio-restore site (verdict N1/N2, Variant A): the frozen
      // error branch writes 'Error: ...' here and the ready phase calls in
      // to report progress - both must be visible again even after the
      // step-row path hid the legacy fallback trio.
      setLegacyFallbackVisible(true);
      const prog = document.getElementById('loadingProgress');
      const detail = document.getElementById('loadingDetail');
      if (prog) prog.style.width = percent + '%';
      if (detail && detailText) detail.textContent = detailText;
    }

    // The legacy single-bar fallback UI is the bar-row + loadingText +
    // loadingDetail trio (shape.md:182-184): visible on the legacy path and
    // restored when the overlay is dismissed, hidden while step rows render
    // so the overlay never tells two progress stories at once.
    function setLegacyFallbackVisible(visible) {
      const fallbackDisplay = visible ? '' : 'none';
      const legacyBar = document.getElementById('loadingProgress');
      if (legacyBar && legacyBar.parentElement) legacyBar.parentElement.style.display = fallbackDisplay;
      const legacyText = document.getElementById('loadingText');
      if (legacyText) legacyText.style.display = fallbackDisplay;
      const legacyDetail = document.getElementById('loadingDetail');
      if (legacyDetail) legacyDetail.style.display = fallbackDisplay;
    }

    function hideProgress() {
      const overlay = document.getElementById('loadingOverlay');
      const steps = document.getElementById('loadingSteps');
      if (steps) steps.textContent = '';
      if (overlay) {
        overlay.style.opacity = '0';
        setTimeout(() => {
          overlay.style.display = 'none';
          overlay.style.opacity = '1';
          // Restore the trio only after the fade-out, so the overlay shows
          // neither legacy fallback nor stale rows while invisible.
          setLegacyFallbackVisible(true);
        }, 300);
      }
    }

    // 20-cell block-glyph bar with the CLI's own integer arithmetic (the
    // format_step_bar look): clamped percent, Math.floor fill, remainder
    // always completing 20 cells, the whole bar wrapped in brackets. Both
    // runs paint the solid full-block glyph via its \u2588 escape (authored
    // content stays pure ASCII): the fill inherits the per-state text
    // color, while the remainder lives in a dedicated .step-bar-remainder
    // span so CSS paints it as a quiet solid track (no dithered dots).
    function buildStepBarNodes(percent) {
      percent = Math.min(percent, 100);
      var filled = Math.floor(20 * percent / 100);
      var remainder = 20 - filled;
      const nodes = [document.createTextNode('[')];
      nodes.push(document.createTextNode("\u2588".repeat(filled)));
      const track = document.createElement('span');
      track.className = 'step-bar-remainder';
      track.textContent = "\u2588".repeat(remainder);
      nodes.push(track);
      nodes.push(document.createTextNode(']'));
      return nodes;
    }

    // One renderer for both boot paths (ui-progress-parity T3). With step
    // rows present it draws a CLI-parity frame per step; without them it
    // delegates to the legacy single-bar width behavior (standalone exports
    // and legacy fixtures land there - the honest fallback).
    function renderProgressSnapshot(progress) {
      if (!progress.steps || progress.steps.length === 0) {
        // The fallback must stay honest against legacy fixtures: make the
        // trio visible again even if a previous snapshot rendered rows.
        setLegacyFallbackVisible(true);
        const detailText = progress.phase ? progress.phase + ': ' + progress.message : progress.message;
        updateProgress(progress.percent, detailText);
        return;
      }
      const overlay = document.getElementById('loadingOverlay');
      if (overlay) overlay.style.display = 'flex';
      setLegacyFallbackVisible(false);
      const stepsBox = document.getElementById('loadingSteps');
      if (!stepsBox) return;
      stepsBox.textContent = '';
      progress.steps.forEach(step => {
        const row = document.createElement('div');
        row.className = 'step-row step-' + step.status;
        const chip = document.createElement('span');
        chip.className = 'step-index';
        chip.textContent = '[' + step.stage + '/' + progress.total_stages + ']';
        const label = document.createElement('span');
        label.className = 'step-label';
        label.textContent = step.label;
        const bar = document.createElement('span');
        bar.className = 'step-bar';
        buildStepBarNodes(step.percent).forEach(node => bar.appendChild(node));
        const percentCell = document.createElement('span');
        percentCell.className = 'step-percent';
        percentCell.textContent = Math.min(step.percent, 100) + '%';
        const detail = document.createElement('span');
        detail.className = 'step-detail';
        detail.textContent = step.status === 'pending' ? '' : step.message;
        row.appendChild(chip);
        row.appendChild(label);
        row.appendChild(bar);
        row.appendChild(percentCell);
        row.appendChild(detail);
        stepsBox.appendChild(row);
      });
    }

    // Keyboard shortcut (Ctrl+K or /)
    window.addEventListener('keydown', e => {
      if ((e.ctrlKey && e.key === 'k') || e.key === '/') {
        e.preventDefault();
        document.getElementById('searchInput').focus();
      }
      if (e.key === 'Escape' && qoOpen) {
        closeQuickOpen();
      } else if (e.key === 'Escape' && selectedNodeId) {
        selectedNodeId = null;
        renderSvg();
        renderThreadBar();
        renderHotspots();
      } else if (e.key === 'Escape' && dirScope) {
        clearDirScope();
      }
    });

    // Start
    init();

    // === NODE INSPECTOR SHOW/HIDE (mirrors the hotspots pattern) ===
    function collapseInspector() {
      inspectorOpen = false;
      const panel = document.getElementById('inspectorPanel');
      if (panel) panel.style.display = 'none';
      const restoreTab = document.getElementById('inspectorRestoreTab');
      if (restoreTab) restoreTab.hidden = false;
    }

    function expandInspector() {
      inspectorOpen = true;
      const panel = document.getElementById('inspectorPanel');
      if (panel) panel.style.display = 'flex';
      const restoreTab = document.getElementById('inspectorRestoreTab');
      if (restoreTab) restoreTab.hidden = true;
    }

    function toggleInspector() {
      if (inspectorOpen) collapseInspector();
      else expandInspector();
    }

    // === ONBOARDING & HELP ===
    const tourSteps = [
      { id: '#tabInclude', title: 'Graph Perspectives', text: 'Switch between Include, Symbol, and Call graphs to analyze your codebase from different angles.' },
      { id: '#canvasContainer', title: 'Interactive Canvas & Selection', text: 'Pan, zoom, and click nodes to explore dependencies. Use the Clear Selection button in the toolbar or press Esc to clear fading and selection.' },
      { id: '.toolbar', title: 'Direction & Degree Navigation', text: 'Control Direction (Dependencies -> vs Consumers <-), step Degree (-/+), or use ALL-WITH-CAP to expand safely up to the node cap limit.' },
      { id: '#ariadneThread', title: 'Pathfinder & Breadcrumbs', text: 'Use Pathfinder for shortest paths and Ariadne\'s Thread to navigate your breadcrumb history.' },
      { id: '.inspector', title: 'Collateral Heat Map', text: 'Select a header to calculate collateral load. Card fill shows the heat tier (Cold to Red); the left spine states the verdict: green every symbol used, amber partly needed, red supplies nothing. Click a contributor to illuminate the include path.' },
      { id: '#hotspotPanel', title: 'Hotspots Triage', text: 'Click the Hotspots tab on the left edge to rank the worst-offender files by collateral burden, fan-in/out, closure, or cycle participation. Click a row to select the file and see its heat map live beside the list.',
        enter() {
          if (currentTab !== 'include') switchTab('include');
          if (!hotspotOpen) tourOpenedHotspots = true;
          openHotspots();
        },
        leave() {
          if (tourOpenedHotspots) { closeHotspots(); tourOpenedHotspots = false; }
        }
      },
      { id: '#btnCycles', title: 'Search & Tools', text: 'Use wildcard/regex search to find files, press ↓ to browse folders in the directory navigator and scope the graph to one, and Highlight Cycle to detect circular dependencies.' }
    ];
    let currentTourStep = -1;
    let tourOpenedHotspots = false;

    function startTour(stepIndex = 0) {
      if (document.getElementById('welcomeModal').style.display === 'block') {
        document.getElementById('welcomeModal').style.display = 'none';
      }
      currentTourStep = stepIndex;
      document.getElementById('tourOverlay').style.display = 'block';
      document.getElementById('tourCard').style.display = 'block';
      renderTourStep();
    }

    function renderTourStep() {
      if (currentTourStep < 0 || currentTourStep >= tourSteps.length) {
        endTour();
        return;
      }
      document.querySelectorAll('.tour-highlighted-element').forEach(el => el.classList.remove('tour-highlighted-element'));
      const step = tourSteps[currentTourStep];
      document.getElementById('tourTitle').textContent = step.title;
      document.getElementById('tourText').textContent = step.text;
      document.getElementById('tourStepIndicator').textContent = `Step ${currentTourStep + 1} of ${tourSteps.length}`;
      
      const btnTourPrev = document.getElementById('btnTourPrev');
      if (btnTourPrev) {
        btnTourPrev.style.display = currentTourStep === 0 ? 'none' : 'inline-block';
      }

      const btnNext = document.getElementById('btnTourNext');
      if (currentTourStep === tourSteps.length - 1) {
        btnNext.textContent = 'Finish';
      } else {
        btnNext.textContent = 'Next';
      }

      if (step.enter) step.enter();

      if (step.id === '.inspector' && !inspectorOpen) {
        expandInspector();
      }

      const targetEl = document.querySelector(step.id);
      if (targetEl) {
        targetEl.classList.add('tour-highlighted-element');
        positionTourCard(targetEl);
      }
    }

    function runCurrentTourStepLeave() {
      if (currentTourStep < 0 || currentTourStep >= tourSteps.length) return;
      const step = tourSteps[currentTourStep];
      if (step.leave) step.leave();
    }

    function positionTourCard(targetEl) {
      const rect = targetEl.getBoundingClientRect();
      const card = document.getElementById('tourCard');
      
      let top, left;
      const cardWidth = 320; 
      const cardHeight = card.offsetHeight || 180; 
      const margin = 16;
      const winW = window.innerWidth;
      const winH = window.innerHeight;
      
      if (targetEl.id === 'canvasContainer') {
        top = Math.max(margin + 120, (winH - cardHeight) / 2);
        left = Math.max(margin, (winW - 340 - cardWidth) / 2);
      } else if (targetEl.id === 'tabInclude' || targetEl.closest('.tabs')) {
        top = rect.bottom + 10;
        left = rect.left + (rect.width - cardWidth) / 2;
      } else if (targetEl.id === 'hotspotPanel') {
        top = rect.top + 40;
        left = rect.right + margin;
      } else if (targetEl.classList.contains('inspector')) {
        left = rect.left - cardWidth - margin;
        top = Math.max(margin, rect.top + 40);
      } else if (targetEl.id === 'btnCycles' || targetEl.closest('.header-actions')) {
        top = rect.bottom + 10;
        left = rect.right - cardWidth;
      } else if (targetEl.classList.contains('toolbar') || targetEl.id === 'ariadneThread') {
        top = rect.bottom + 10;
        left = rect.left + 20;
      } else {
        top = rect.bottom + 10;
        left = rect.left;
      }
      
      top = Math.max(margin, Math.min(winH - cardHeight - margin, top));
      left = Math.max(margin, Math.min(winW - cardWidth - margin, left));
      
      card.style.top = top + 'px';
      card.style.left = left + 'px';
    }

    function nextTourStep() {
      if (currentTourStep < tourSteps.length - 1) {
        runCurrentTourStepLeave();
        currentTourStep++;
        renderTourStep();
      } else {
        endTour();
      }
    }

    function prevTourStep() {
      if (currentTourStep > 0) {
        runCurrentTourStepLeave();
        currentTourStep--;
        renderTourStep();
      }
    }

    function endTour() {
      runCurrentTourStepLeave();
      currentTourStep = -1;
      document.getElementById('tourOverlay').style.display = 'none';
      document.getElementById('tourCard').style.display = 'none';
      document.querySelectorAll('.tour-highlighted-element').forEach(el => el.classList.remove('tour-highlighted-element'));
    }

    function triggerFirstVisitExperience() {
      if (!localStorage.getItem('ariadne_onboarding_seen')) {
        showWelcomeModal();
      }
    }

    function showWelcomeModal() {
      document.getElementById('modalOverlay').style.display = 'block';
      document.getElementById('welcomeModal').style.display = 'block';
    }

    function dismissWelcomeModal(dontShowAgainParam) {
      document.getElementById('modalOverlay').style.display = 'none';
      document.getElementById('welcomeModal').style.display = 'none';
      const dontShow = document.getElementById('chkDontShowAgain').checked;
      if (dontShow || dontShowAgainParam) {
        localStorage.setItem('ariadne_onboarding_seen', 'true');
      }
    }

    function resetOnboarding() {
      localStorage.removeItem('ariadne_onboarding_seen');
      closeHelpCenter();
      triggerFirstVisitExperience();
    }

    function openHelpCenter(tabName = 'onboarding') {
      document.getElementById('modalOverlay').style.display = 'block';
      document.getElementById('helpCenterModal').style.display = 'block';
      openHelpCenterTab(tabName);
    }

    function closeHelpCenter() {
      document.getElementById('helpCenterModal').style.display = 'none';
      document.getElementById('modalOverlay').style.display = 'none';
    }

    function openHelpCenterTab(tabName) {
      document.querySelectorAll('.help-tab-content').forEach(el => el.style.display = 'none');
      document.querySelectorAll('.help-tab-btn').forEach(el => el.classList.remove('active'));
      const content = document.getElementById('helpTab-' + tabName);
      if (content) content.style.display = 'block';
      
      const btns = document.querySelectorAll('.help-tab-btn');
      for(let btn of btns) {
        if (btn.textContent.toLowerCase().includes(tabName.slice(0, 4))) {
          btn.classList.add('active');
        }
      }
    }

    window.addEventListener('keydown', e => {
      if (e.key === 'Escape') {
        if (currentTourStep !== -1) endTour();
        else if (document.getElementById('welcomeModal').style.display === 'block') dismissWelcomeModal(false);
        else if (document.getElementById('helpCenterModal').style.display === 'block') closeHelpCenter();
      }
      if (currentTourStep !== -1) {
        if (e.key === 'ArrowRight') nextTourStep();
        if (e.key === 'ArrowLeft') prevTourStep();
      } else {
        if (e.key === '?' || (e.shiftKey && e.key === '/') || e.key === 'F1') {
          if (document.activeElement.tagName !== 'INPUT' && document.activeElement.tagName !== 'TEXTAREA') {
            e.preventDefault();
            openHelpCenter('shortcuts');
          }
        }
      }
    });

    triggerFirstVisitExperience();
