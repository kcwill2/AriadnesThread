// Traversal tab logic split from graph.js: root/dep/symbol selection,
    // API fetch, and three-subtree rendering for collateral traversal view.
    // === TRAVERSAL TAB FUNCTIONS ===

    let traversalData = null;
    let traversalRoots = [];
    let traversalDeps = [];

    // Check if a file is a header (not implementation)
    function isHeaderFile(node) {
      const path = (node.path || '').toLowerCase();
      const label = (node.label || '').toLowerCase();
      return path.endsWith('.h') || path.endsWith('.hpp') || path.endsWith('.hxx') ||
             label.endsWith('.h') || label.endsWith('.hpp') || label.endsWith('.hxx') ||
             node.kind === 'HEADER';
    }

    let traversalInitialized = false;

    function initTraversalPanel() {
      if (traversalInitialized) {
        console.log('[traversal] Already initialized, skipping');
        return;
      }
      setupSearchableDepSelect();
      populateTraversalRootSelect();
      traversalInitialized = true;
    }

function populateTraversalRootSelect() {
      const select = document.getElementById('traversalRootSelect');
      console.log('[traversal] populateTraversalRootSelect, select:', select);
      if (!select) return;
      // Get files that have outgoing includes (potential roots)
      const filesWithDeps = new Set();
      tabAllEdges.forEach(e => {
        if (!e.is_cycle) filesWithDeps.add(e.source);
      });
      console.log('[traversal] tabAllEdges:', tabAllEdges.length, 'filesWithDeps:', filesWithDeps.size);
      traversalRoots = tabAllNodes
        .filter(n => filesWithDeps.has(n.id))
        .sort((a, b) => a.label.localeCompare(b.label));
      console.log('[traversal] traversalRoots:', traversalRoots.length, traversalRoots.map(r => r.label));
      select.innerHTML = traversalRoots.map(n => `<option value="${n.id}">${escapeHtml(n.label)} (${n.path})</option>`).join('');
      
      // If there's a currently selected node that's a valid root, use it
      let rootToUse = traversalRoots[0]?.id;
      if (selectedNodeId && traversalRoots.some(r => r.id === selectedNodeId)) {
        rootToUse = selectedNodeId;
      }
      
      if (rootToUse) {
        select.value = rootToUse;
        loadTraversalForRoot();
      }
    }

    function setupSearchableDepSelect() {
      const depSelect = document.getElementById('traversalDepSelect');
      console.log('[traversal] setupSearchableDepSelect, depSelect:', depSelect);
      if (!depSelect) {
        console.log('[traversal] traversalDepSelect not found (probably already replaced), skipping');
        return;
      }
      if (!depSelect.parentNode) {
        console.error('[traversal] traversalDepSelect has no parentNode!');
        showDebugError('traversalDepSelect has no parentNode');
        return;
      }
      
      // Replace native select with searchable input + dropdown
      const wrapper = document.createElement('div');
      wrapper.className = 'searchable-select-wrapper';
      wrapper.innerHTML = `
        <input type="text" class="searchable-select-input" id="traversalDepSearch" placeholder="Type to filter dependencies..." autocomplete="off">
        <div class="searchable-select-dropdown" id="traversalDepDropdown" style="display:none;"></div>
        <select id="traversalDepSelectHidden" style="display:none;"></select>
      `;
      depSelect.parentNode.replaceChild(wrapper, depSelect);
      console.log('[traversal] Replacement done');
      
      // Set up event listeners for search input
      const searchInput = document.getElementById('traversalDepSearch');
      const dropdown = document.getElementById('traversalDepDropdown');
      
      if (searchInput && dropdown) {
        // Search filter
        searchInput.addEventListener('input', () => {
          const query = searchInput.value.toLowerCase();
          dropdown.querySelectorAll('.searchable-select-option').forEach(opt => {
            const text = opt.textContent.toLowerCase();
            opt.style.display = text.includes(query) ? 'block' : 'none';
          });
          dropdown.style.display = 'block';
        });
        
        // Show dropdown on focus
        searchInput.addEventListener('focus', () => {
          if (traversalDeps.length > 0) dropdown.style.display = 'block';
        });
        
        // Hide on blur (with delay for click to register)
        searchInput.addEventListener('blur', () => {
          setTimeout(() => dropdown.style.display = 'none', 150);
        });
      }
    }

    function showDebugError(msg) {
      const panel = document.getElementById('sidebarPanelTraversal');
      if (panel) {
        const err = document.createElement('div');
        err.style.cssText = 'color:var(--red);padding:8px;background:var(--red-wash);border:1px solid var(--red-border);border-radius:4px;margin:8px;';
        err.textContent = '[Debug] ' + msg;
        panel.appendChild(err);
      }
    }

    function updateDepDropdown(deps) {
      traversalDeps = deps;
      const dropdown = document.getElementById('traversalDepDropdown');
      const hiddenSelect = document.getElementById('traversalDepSelectHidden');
      const searchInput = document.getElementById('traversalDepSearch');
      
      if (!dropdown || !hiddenSelect) return;
      
      dropdown.innerHTML = deps.map((d, i) => 
        `<div class="searchable-select-option" data-value="${d.id}" data-index="${i}">${escapeHtml(d.label)} <span class="searchable-select-path">(${escapeHtml(d.path)})</span></div>`
      ).join('');
      
      hiddenSelect.innerHTML = deps.map(d => `<option value="${d.id}">${escapeHtml(d.label)} (${d.path})</option>`).join('');
      
      // Click handler for options
      dropdown.querySelectorAll('.searchable-select-option').forEach(opt => {
        opt.addEventListener('click', () => {
          const value = opt.dataset.value;
          hiddenSelect.value = value;
          if (searchInput) searchInput.value = opt.textContent.split(' (')[0];
          dropdown.style.display = 'none';
          loadTraversalDetail();
        });
      });
    }

    async function loadTraversalForRoot() {
      const rootSelect = document.getElementById('traversalRootSelect');
      const depSearch = document.getElementById('traversalDepSearch');
      const symbolSelect = document.getElementById('traversalSymbolSelect');
      console.log('[traversal] loadTraversalForRoot, rootSelect:', rootSelect, 'depSearch:', depSearch);
      if (!rootSelect || !depSearch) {
        console.error('[traversal] Missing elements, rootSelect:', rootSelect, 'depSearch:', depSearch);
        return;
      }
      
      const rootId = rootSelect.value;
      console.log('[traversal] rootId:', rootId);
      if (!rootId) return;

      // Get direct dependencies of this root - filter to headers only
      const deps = tabAllEdges
        .filter(e => e.source === rootId && !e.is_cycle)
        .map(e => tabAllNodes.find(n => n.id === e.target))
        .filter(Boolean)
        .filter(isHeaderFile);
      
      console.log('[traversal] Found deps:', deps.length, deps.map(d => d.label));
      
      updateDepDropdown(deps);
      
      if (deps.length > 0) {
        const hiddenSelect = document.getElementById('traversalDepSelectHidden');
        if (hiddenSelect) hiddenSelect.value = deps[0].id;
        if (depSearch) depSearch.value = deps[0].label;
        
        // Switch to Dependencies (outbound) direction for traversal view
        const dirSelect = document.getElementById('dirSelect');
        if (dirSelect) dirSelect.value = 'out';
        
        await loadTraversalDetail();
      } else {
        const dropdown = document.getElementById('traversalDepDropdown');
        if (dropdown) dropdown.innerHTML = '<div class="searchable-select-option" style="color:var(--text-sub);font-style:italic;padding:8px;">No header dependencies</div>';
        if (symbolSelect) symbolSelect.innerHTML = '';
        document.getElementById('traversalSummary').innerHTML = '';
        document.getElementById('traversalSubtrees').innerHTML = '';
      }
    }

    async function loadTraversalDetail() {
      const rootSelect = document.getElementById('traversalRootSelect');
      const hiddenDepSelect = document.getElementById('traversalDepSelectHidden');
      const symbolSelect = document.getElementById('traversalSymbolSelect');
      if (!rootSelect || !hiddenDepSelect) return;
      
      const rootId = rootSelect.value;
      const depId = hiddenDepSelect.value;
      if (!rootId || !depId) return;

      // Get available symbols for this edge from justification data
      const justificationMap = (graphData.file_justification || {})[rootId] || {};
      const depEntry = justificationMap[depId];
      
      if (depEntry && depEntry.justified_symbols && depEntry.justified_symbols.length > 0) {
        symbolSelect.innerHTML = depEntry.justified_symbols.map(s => 
          `<option value="${escapeHtml(s)}">${escapeHtml(s)}</option>`
        ).join('');
        // Select all by default
        Array.from(symbolSelect.options).forEach(opt => opt.selected = true);
      } else if (depEntry && depEntry.collateral_symbols && depEntry.collateral_symbols.length > 0) {
        // No justified symbols, but there are collateral symbols - show them as an option
        symbolSelect.innerHTML = depEntry.collateral_symbols.map(s => 
          `<option value="${escapeHtml(s)}">${escapeHtml(s)} (collateral)</option>`
        ).join('');
        Array.from(symbolSelect.options).forEach(opt => opt.selected = true);
        // Show a notice
        document.getElementById('traversalSummary').innerHTML = `
          <div style="color:var(--amber);padding:8px;background:var(--amber-wash);border:1px solid var(--amber-border);border-radius:4px;">
            <strong>No justified symbols</strong> — this include supplies nothing the root needs (all ${depEntry.collateral_symbols.length} symbols are collateral).
            <br>Selecting collateral symbols shows what <em>would be</em> necessary if they were used.
          </div>
        `;
      } else {
        symbolSelect.innerHTML = '<option value="">No symbols on this edge</option>';
        document.getElementById('traversalSummary').innerHTML = `
          <div style="color:var(--text-sub);padding:8px;">No justification data for this edge.</div>
        `;
        document.getElementById('traversalSubtrees').innerHTML = '';
        return;
      }

      // Fetch traversal data from API
      const symbols = Array.from(symbolSelect.selectedOptions).map(opt => opt.value).filter(Boolean);
      const params = new URLSearchParams({ root: rootId, dep: depId });
      if (symbols.length > 0) {
        params.append('symbols', symbols.join(','));
      }
      
      try {
        const response = await fetch(`/api/graph/traversal?${params.toString()}`);
        if (!response.ok) throw new Error('Failed to load traversal data');
        traversalData = await response.json();
        renderTraversalView();
      } catch (err) {
        console.error('Traversal load error:', err);
        document.getElementById('traversalSummary').innerHTML = `<div style="color:var(--red);padding:8px;">Error loading traversal: ${escapeHtml(err.message)}</div>`;
        document.getElementById('traversalSubtrees').innerHTML = '';
      }
    }

    function renderTraversalView() {
      if (!traversalData) return;
      
      renderTraversalSummary(traversalData);
      renderTraversalSubtrees(traversalData);
      renderTraversalExtractions(traversalData.root_file_id);
    }

    function renderTraversalSummary(data) {
      const { summary, analyzed_symbol_ids } = data;
      const summaryEl = document.getElementById('traversalSummary');
      if (!summaryEl) return;
      
      summaryEl.innerHTML = `
        <div class="traversal-summary-grid">
          <div class="traversal-summary-card necessary">
            <div class="traversal-summary-label">Necessary</div>
            <div class="traversal-summary-value">${summary.necessary_files} files</div>
            <div class="traversal-summary-sub">${summary.necessary_symbols} symbols</div>
          </div>
          <div class="traversal-summary-card actual">
            <div class="traversal-summary-label">Actual</div>
            <div class="traversal-summary-value">${summary.actual_files} files</div>
            <div class="traversal-summary-sub">${summary.actual_symbols} symbols</div>
          </div>
          <div class="traversal-summary-card collateral">
            <div class="traversal-summary-label">Collateral Delta</div>
            <div class="traversal-summary-value">${summary.collateral_files} files</div>
            <div class="traversal-summary-sub">${summary.collateral_symbols} symbols</div>
          </div>
          <div class="traversal-summary-card decoy">
            <div class="traversal-summary-label">Decoy (Still Reachable)</div>
            <div class="traversal-summary-value">${summary.decoy_files} files</div>
            <div class="traversal-summary-sub">${summary.decoy_symbols} symbols</div>
          </div>
        </div>
        <div class="traversal-analyzed-symbols">
          Analyzing ${analyzed_symbol_ids.length} symbol(s): ${analyzed_symbol_ids.map(s => `<code>${escapeHtml(s)}</code>`).join(', ')}
        </div>
      `;
    }

    function renderTraversalSubtrees(data) {
      const { necessary_subtree, actual_subtree, collateral_delta } = data;
      const container = document.getElementById('traversalSubtrees');
      if (!container) return;
      
      container.innerHTML = `
        <div class="traversal-subtree-panel necessary">
          <div class="traversal-subtree-header">
            <span class="traversal-subtree-title">Necessary Subtree</span>
            <span class="traversal-subtree-count">${necessary_subtree.file_count} files, ${necessary_subtree.total_symbols} symbols</span>
          </div>
          <div class="traversal-subtree-list">${renderSubtreeNodes(necessary_subtree.nodes, 'necessary')}</div>
        </div>
        <div class="traversal-subtree-panel actual">
          <div class="traversal-subtree-header">
            <span class="traversal-subtree-title">Actual Subtree</span>
            <span class="traversal-subtree-count">${actual_subtree.file_count} files, ${actual_subtree.total_symbols} symbols</span>
          </div>
          <div class="traversal-subtree-list">${renderSubtreeNodes(actual_subtree.nodes, 'actual')}</div>
        </div>
        <div class="traversal-subtree-panel collateral">
          <div class="traversal-subtree-header">
            <span class="traversal-subtree-title">Collateral Delta (Eliminable)</span>
            <span class="traversal-subtree-count">${collateral_delta.file_count} files, ${collateral_delta.total_symbols} symbols</span>
          </div>
          <div class="traversal-subtree-list">${renderSubtreeNodes(collateral_delta.nodes, 'collateral')}</div>
        </div>
      `;
    }

    function renderSubtreeNodes(nodes, type) {
      if (!nodes || nodes.length === 0) {
        return '<div style="color:var(--text-sub);font-style:italic;padding:8px;">No files</div>';
      }
      return nodes.map(n => `
        <div class="traversal-node-row ${type}" data-node-id="${n.id}" onclick="selectNode('${n.id}')">
          <span class="traversal-node-kind ${n.kind.toLowerCase()}">${n.kind === 'HEADER' ? '(H)' : '(C)'}</span>
          <span class="traversal-node-label">${escapeHtml(n.label)}</span>
          <span class="traversal-node-path">${escapeHtml(n.path)}</span>
          <span class="traversal-node-symbols">${n.symbol_count} sym</span>
        </div>
      `).join('');
    }

    function renderTraversalLegend() {
      const legendList = document.getElementById('legendItems');
      document.getElementById('legendViewLabel').textContent = 'Collateral Traversal';
      legendList.innerHTML = `
        <div class="legend-row">
          <div class="legend-color" style="background:var(--green);"></div>
          <span>Necessary (required headers)</span>
        </div>
        <div class="legend-row">
          <div class="legend-color" style="background:var(--cyan);"></div>
          <span>Actual (full dependency closure)</span>
        </div>
        <div class="legend-row">
          <div class="legend-color" style="background:var(--red);"></div>
          <span>Collateral Delta (eliminable)</span>
        </div>
        <div class="legend-row">
          <div class="legend-color" style="background:var(--amber);"></div>
          <span>Decoy (still reachable via other paths)</span>
        </div>
        <div style="font-size:10px;color:var(--text-muted);margin-top:6px;">
Click a file to select it in the main graph
        </div>
      `;
    }
 
    // Render extraction proposals for the current root file
    function renderTraversalExtractions(rootFileId) {
      const proposals = graphData.extraction_proposals || [];
      if (proposals.length === 0) return;
      
      // Find proposals targeting this root file (as consumer) or its dependencies
      const relevantProposals = proposals.filter(p => {
        // The proposal targets a file that this root includes, or the root itself
        return p.target_file_id === rootFileId || 
               (graphData.file_collateral?.[rootFileId]?.[p.target_file_id] > 0);
      });
      
      if (relevantProposals.length === 0) return;
      
      // Find or create extraction section
      let section = document.getElementById('traversalExtractionsSection');
      if (!section) {
        section = document.createElement('div');
        section.id = 'traversalExtractionsSection';
        section.style.marginTop = '16px';
        const subtrees = document.getElementById('traversalSubtrees');
        if (subtrees && subtrees.parentNode) {
          subtrees.parentNode.insertBefore(section, subtrees.nextSibling);
        }
      }
      
      section.innerHTML = `
        <div class="link-list-title" style="color:var(--amber); margin-bottom: 8px;">
          &#128161; Extraction Opportunities (${relevantProposals.length})
        </div>
        <div class="extraction-list">${relevantProposals.map(renderExtractionProposalCard).join('')}</div>
      `;
      
      if (window.bindPreviewControls) window.bindPreviewControls(section);
    }
 
    // Return-to-origin: the tab active when traversal was last entered.
    let traversalReturnTab = 'include';

    // Enter traversal from any tab, capturing the origin tab for Back.
    function openTraversalTab() {
      if (currentTab !== 'traversal') traversalReturnTab = currentTab;
      switchTab('traversal');
    }

    // Leave traversal back to the captured origin tab.
    function backFromTraversal() {
      switchTab(traversalReturnTab);
    }

    // Expose for graph.js
    window.renderTraversalLegend = renderTraversalLegend;
    window.initTraversalPanel = initTraversalPanel;
    window.openTraversalTab = openTraversalTab;
    window.backFromTraversal = backFromTraversal;