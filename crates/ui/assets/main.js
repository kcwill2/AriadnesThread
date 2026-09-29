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
    let isAllCapped = false;
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
        { color: '#38bdf8', label: 'Local Header (.h)' },
        { color: '#818cf8', label: 'Source File (.cpp)' },
        { color: '#ef4444', label: 'Include Cycle' },
      ],
      symbol: [
        { color: '#c084fc', label: 'Class / Struct' },
        { color: '#4ade80', label: 'Function' },
        { color: '#38bdf8', label: 'Complete Edge (Solid)' },
        { color: '#94a3b8', label: 'Incomplete Edge (Dashed)' },
      ],
      call: [
        { color: '#4ade80', label: 'Function' },
        { color: '#38bdf8', label: 'Direct Call' },
        { color: '#ef4444', label: 'Recursive Cycle' },
      ]
    };

    // Color mapper for node cards
    function getNodeColors(kind, isCycle) {
      if (isCycle) return { bg: '#3b0d0d', border: '#ef4444', text: '#fca5a5' };
      switch (kind) {
        case 'HEADER':
          return { bg: '#082f49', border: '#0284c7', text: '#38bdf8' };
        case 'SOURCE':
          return { bg: '#1e1b4b', border: '#4f46e5', text: '#818cf8' };
        case 'CLASS':
        case 'STRUCT':
          return { bg: '#2e1065', border: '#9333ea', text: '#c084fc' };
        case 'FUNCTION':
          return { bg: '#052e16', border: '#16a34a', text: '#4ade80' };
        default:
          return { bg: '#1e293b', border: '#475569', text: '#94a3b8' };
      }
    }

    // 5-tier thermal palette. t = collateral / maxCollateral (0..1).
    // Temperature must be derived from graphData.file_collateral (full dataset),
    // not from the degree-limited canvas.
    function getHeatColor(collateral, maxCollateral) {
      const t = maxCollateral > 0 ? collateral / maxCollateral : 0;
      if (t <= 0.15) return { bg: '#06283d', border: '#38bdf8', text: '#7dd3fc', tier: 0 }; // Cool Ice
      if (t <= 0.35) return { bg: '#063721', border: '#22c55e', text: '#86efac', tier: 1 }; // Emerald
      if (t <= 0.60) return { bg: '#382a04', border: '#eab308', text: '#fde047', tier: 2 }; // Amber
      if (t <= 0.85) return { bg: '#431407', border: '#f97316', text: '#fdba74', tier: 3 }; // Hot Orange
      return { bg: '#450a0a', border: '#ef4444', text: '#fca5a5', tier: 4 };                // Blazing Crimson
    }

    const HEAT_LEGEND = [
      { color: '#38bdf8', label: 'Cool — Low Collateral' },
      { color: '#22c55e', label: 'Mild Collateral' },
      { color: '#eab308', label: 'Moderate Collateral' },
      { color: '#f97316', label: 'High Collateral' },
      { color: '#ef4444', label: 'Critical Collateral' },
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

    function showProgress(stageText, percent) {
      const overlay = document.getElementById('loadingOverlay');
      const prog = document.getElementById('loadingProgress');
      const txt = document.getElementById('loadingText');
      if (overlay) overlay.style.display = 'flex';
      if (prog) prog.style.width = percent + '%';
      if (txt) txt.textContent = stageText;
    }

    function updateProgress(percent, detailText) {
      const prog = document.getElementById('loadingProgress');
      const detail = document.getElementById('loadingDetail');
      if (prog) prog.style.width = percent + '%';
      if (detail && detailText) detail.textContent = detailText;
    }

    function hideProgress() {
      const overlay = document.getElementById('loadingOverlay');
      if (overlay) {
        overlay.style.opacity = '0';
        setTimeout(() => {
          overlay.style.display = 'none';
          overlay.style.opacity = '1';
        }, 300);
      }
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
      if (panel) panel.style.display = '';
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
