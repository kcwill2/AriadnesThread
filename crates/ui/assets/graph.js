// Graph rendering & interaction split from index.html: legends, layouts, SVG rendering,
// selection, inspector, breadcrumbs, path illumination, pan/zoom, and the subgraph filter engine.
    function applyLoadedData() {
      document.getElementById('repoNameDisplay').textContent = graphData.repo_name || 'AriadnesThread';
      document.getElementById('cycleCountText').textContent = `${graphData.stats.cycle_count || 0} Cycles`;
      switchTab('include');
    }

    function switchTab(tab) {
      currentTab = tab;
      ['include', 'symbol', 'call'].forEach(t => {
        const btn = document.getElementById('tab' + t.charAt(0).toUpperCase() + t.slice(1));
        if (btn) {
          if (t === tab) btn.classList.add('active');
          else btn.classList.remove('active');
        }
      });

      // Hotspots are include-tab only: hide button + panel elsewhere, restore open state here.
      const hsBtn = document.getElementById('btnHotspots');
      if (hsBtn) hsBtn.style.display = tab === 'include' ? 'inline-block' : 'none';
      const hsPanel = document.getElementById('hotspotPanel');
      if (hsPanel) hsPanel.style.display = (tab === 'include' && hotspotOpen) ? 'flex' : 'none';

      // Update legend
      document.getElementById('legendViewLabel').textContent = 
        tab === 'include' ? 'Include Graph' : (tab === 'symbol' ? 'Symbol Dependency' : 'Call Graph');
      const legendList = document.getElementById('legendItems');
      legendList.innerHTML = (legends[tab] || []).map(item => `
        <div class="legend-row">
          <div class="legend-color" style="background:${item.color};"></div>
          <span>${item.label}</span>
        </div>
      `).join('');

      // Load elements for active tab
      showProgress('Switching tab...', 20);
      setTimeout(() => {
        updateProgress(50, 'Parsing elements...');
        let rawElements = [];
        if (tab === 'include') rawElements = graphData.include_graph || [];
        else if (tab === 'symbol') rawElements = graphData.symbol_graph || [];
        else if (tab === 'call') rawElements = graphData.call_graph || [];

        parseAndRenderElements(rawElements);
        updateProgress(100, 'Done');
        hideProgress();
      }, 10);
    }

    function parseAndRenderElements(rawElements) {
      tabAllNodes = [];
      tabAllEdges = [];
      selectedNodeId = null;
      threadHistory = [];
      renderThreadBar();
      dirScope = null;
      updateDirChip();
      closeQuickOpen();

      rawElements.forEach(el => {
        if (el.data.source && el.data.target) {
          tabAllEdges.push({
            id: el.data.id || (el.data.source + '-' + el.data.target),
            source: String(el.data.source),
            target: String(el.data.target),
            kind: el.data.kind || '',
            is_cycle: el.data.is_cycle || el.classes === 'cycle',
            incomplete: el.data.incomplete || el.classes === 'incomplete',
          });
        } else if (el.data.id) {
          tabAllNodes.push({
            id: String(el.data.id),
            label: el.data.label || el.data.id,
            path: el.data.path || el.data.label || el.data.id,
            kind: el.data.kind || 'NODE',
            closure: el.data.closure || 0,
            in_degree: el.data.in_degree || 0,
            out_degree: el.data.out_degree || 0,
            is_cycle: el.data.is_cycle || el.classes === 'cycle',
            x: 0,
            y: 0,
          });
        }
      });

      filterAndExpandSubgraph();
      renderHotspots();
    }

    // DAG Hierarchical / Topological Layout
    function applyLayout() {
      const mode = document.getElementById('layoutSelect').value;
      if (activeNodes.length === 0) {
        renderSvg();
        return;
      }

      if (mode === 'dag') {
        computeDagLayout();
      } else if (mode === 'concentric') {
        computeConcentricLayout();
      } else {
        computeGridLayout();
      }

      renderSvg();
    }

    function computeDagLayout() {
      // In-degree & adjacency
      const inDegree = {};
      const adj = {};
      activeNodes.forEach(n => { inDegree[n.id] = 0; adj[n.id] = []; });
      activeEdges.forEach(e => {
        if (inDegree[e.target] !== undefined) inDegree[e.target]++;
        if (adj[e.source]) adj[e.source].push(e.target);
      });

      // Layer assignment via BFS from roots
      const layers = {};
      let queue = activeNodes.filter(n => inDegree[n.id] === 0).map(n => n.id);
      if (queue.length === 0 && activeNodes.length > 0) {
        // Has cycles or mutual edges, pick first node
        queue = [activeNodes[0].id];
      }

      queue.forEach(id => { layers[id] = 0; });
      const visited = new Set(queue);

      while (queue.length > 0) {
        const curr = queue.shift();
        const currLayer = layers[curr] || 0;
        (adj[curr] || []).forEach(next => {
          if (!layers[next] || layers[next] < currLayer + 1) {
            layers[next] = currLayer + 1;
          }
          if (!visited.has(next)) {
            visited.add(next);
            queue.push(next);
          }
        });
      }

      // Group by layer
      const layerGroups = {};
      activeNodes.forEach(n => {
        const l = layers[n.id] || 0;
        if (!layerGroups[l]) layerGroups[l] = [];
        layerGroups[l].push(n);
      });

      // Assign coordinates
      const colWidth = 240;
      const rowHeight = 72;
      const maxRowsPerCol = 10;
      let currentX = 60;
      const sortedLayers = Object.keys(layerGroups).map(Number).sort((a, b) => a - b);
      sortedLayers.forEach(l => {
        const nodesInLayer = layerGroups[l];
        const numSubCols = Math.max(1, Math.ceil(nodesInLayer.length / maxRowsPerCol));
        nodesInLayer.forEach((node, idx) => {
          const subCol = Math.floor(idx / maxRowsPerCol);
          const row = idx % maxRowsPerCol;
          node.x = currentX + subCol * colWidth;
          node.y = 60 + row * rowHeight;
        });
        currentX += numSubCols * colWidth + 60;
      });
    }

    function computeConcentricLayout() {
      const centerX = 380;
      const centerY = 260;
      const radius = Math.min(220, 40 + activeNodes.length * 20);
      const angleStep = (2 * Math.PI) / Math.max(1, activeNodes.length);

      activeNodes.forEach((node, i) => {
        node.x = centerX + radius * Math.cos(i * angleStep) - 80;
        node.y = centerY + radius * Math.sin(i * angleStep) - 20;
      });
    }

    function computeGridLayout() {
      const cols = Math.ceil(Math.sqrt(activeNodes.length));
      const colWidth = 220;
      const rowHeight = 80;
      activeNodes.forEach((node, i) => {
        node.x = 60 + (i % cols) * colWidth;
        node.y = 60 + Math.floor(i / cols) * rowHeight;
      });
    }

    function renderSvg() {
      const nodesLayer = document.getElementById('nodesLayer');
      const edgesLayer = document.getElementById('edgesLayer');
      nodesLayer.innerHTML = '';
      edgesLayer.innerHTML = '';
      // E3: every re-render (new selection, Esc, filters) drops preview state; the overlay returns only by explicit click.
      if (window.ariadnePreview) window.ariadnePreview.clearPreview();

      if (selectedNodeId) {
        edgesLayer.classList.add('edges-has-selection');
      } else {
        edgesLayer.classList.remove('edges-has-selection');
      }

      const nodeMap = {};
      activeNodes.forEach(n => { nodeMap[n.id] = n; });

      const normalEdges = [];
      const highlightedEdges = [];

      activeEdges.forEach(e => {
        if (cyclesOnly && !e.is_cycle) return;
        
        const isEntering = Boolean(selectedNodeId && e.target === selectedNodeId);
        const isLeaving = Boolean(selectedNodeId && e.source === selectedNodeId);
        const isHighlighted = isEntering || isLeaving;
        
        if (isHighlighted) {
          highlightedEdges.push({e, isEntering, isLeaving, isHighlighted});
        } else {
          normalEdges.push({e, isEntering, isLeaving, isHighlighted});
        }
      });

      const renderEdge = ({e, isEntering, isLeaving, isHighlighted}) => {
        const src = nodeMap[e.source];
        const dst = nodeMap[e.target];
        if (!src || !dst) return;

        const isCycle = e.is_cycle;
        const isIncomplete = e.incomplete;
        const isSelf = e.source === e.target;

        let pathD = '';
        if (isSelf) {
          pathD = `M ${src.x + 80} ${src.y} C ${src.x + 50} ${src.y - 45}, ${src.x + 110} ${src.y - 45}, ${src.x + 95} ${src.y}`;
        } else {
          const startX = src.x + 160;
          const startY = src.y + 20;
          const endX = dst.x;
          const endY = dst.y + 20;
          const dx = Math.max(40, (endX - startX) * 0.5);
          pathD = `M ${startX} ${startY} C ${startX + dx} ${startY}, ${endX - dx} ${endY}, ${endX} ${endY}`;
        }

        const edgeEl = document.createElementNS('http://www.w3.org/2000/svg', 'path');
        edgeEl.setAttribute('d', pathD);
        
        let classes = `edge-path ${isCycle ? 'cycle-edge' : ''} ${isIncomplete ? 'incomplete' : ''}`;
        if (isHighlighted) classes += ' highlighted';
        if (isEntering) classes += ' edge-entering';
        if (isLeaving) classes += ' edge-leaving';
        
        edgeEl.setAttribute('class', classes.trim());
        let markerUrl = 'url(#arrow)';
        if (isCycle) markerUrl = 'url(#arrow-cycle)';
        else if (isEntering) markerUrl = 'url(#arrow-entering)';
        else if (isLeaving) markerUrl = 'url(#arrow-leaving)';
        else if (isHighlighted) markerUrl = 'url(#arrow-selected)';
        
        edgeEl.setAttribute('marker-end', markerUrl);
        edgeEl.dataset.edgeId = e.id;
        edgeEl.dataset.edgeSource = e.source;
        edgeEl.dataset.edgeTarget = e.target;
        edgesLayer.appendChild(edgeEl);
      };

      normalEdges.forEach(renderEdge);
      highlightedEdges.forEach(renderEdge);

      // Render Nodes
      activeNodes.forEach(n => {
        if (cyclesOnly && !n.is_cycle) return;

        const colors = getNodeColors(n.kind, n.is_cycle);
        const isSelected = n.id === selectedNodeId;

        const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        g.setAttribute('class', `node-group ${isSelected ? 'selected' : ''}`);
        g.setAttribute('transform', `translate(${n.x}, ${n.y})`);
        g.dataset.nodeId = n.id;
        g.onclick = (evt) => {
          evt.stopPropagation();
          selectNode(n.id);
        };

        // Node card rect
        const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
        rect.setAttribute('width', '170');
        rect.setAttribute('height', '42');
        rect.setAttribute('fill', colors.bg);
        rect.setAttribute('stroke', colors.border);

        // Kind pill circle/badge
        const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
        circle.setAttribute('cx', '18');
        circle.setAttribute('cy', '21');
        circle.setAttribute('r', '5');
        circle.setAttribute('fill', colors.border);

        // Node Label
        const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        text.setAttribute('x', '32');
        text.setAttribute('y', '25');
        text.setAttribute('fill', '#f8fafc');
        text.setAttribute('font-size', '11');
        text.setAttribute('font-family', 'monospace');
        text.setAttribute('font-weight', '600');
        
        let labelText = n.label;
        if (labelText.length > 17) labelText = labelText.substring(0, 15) + '..';
        text.textContent = labelText;

        g.appendChild(rect);
        g.appendChild(circle);
        g.appendChild(text);
        nodesLayer.appendChild(g);
      });

      // Apply include-graph fading, heat colors, and legend when a node is selected
      if (currentTab === 'include' && selectedNodeId) {
        const reachableDependencies = computeFullDependencies(selectedNodeId);
        const collateralMap = (graphData.file_collateral || {})[selectedNodeId] || {};
        const justificationMap = (graphData.file_justification || {})[selectedNodeId] || {};
        const depCollaterals = [...reachableDependencies].map(id => collateralMap[id] || 0);
        const maxCollateral = depCollaterals.length > 0 ? Math.max(...depCollaterals) : 0;
        let totalCollateral = depCollaterals.reduce((s, c) => s + c, 0);

        nodesLayer.querySelectorAll('.node-group').forEach(el => {
          const nid = el.dataset.nodeId;
          if (nid === selectedNodeId) {
            el.classList.add('heat-root');
            const badge = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            badge.setAttribute('class', 'heat-badge');
            badge.setAttribute('x', '32');
            badge.setAttribute('y', '36');
            badge.setAttribute('fill', 'var(--cyan)');
            badge.textContent = '[ROOT]';
            el.appendChild(badge);
          } else if (reachableDependencies.has(nid)) {
            const c = collateralMap[nid] || 0;
            const j = justificationMap[nid];
            const tier = classifyDependency(nid, c, j);
            const badge = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            badge.setAttribute('class', 'heat-badge');
            badge.setAttribute('x', '32');
            badge.setAttribute('y', '36');
            if (tier === 'justified') {
              el.classList.add('dep-justified');
              badge.setAttribute('fill', '#4ade80');
              badge.textContent = `${j.justified_count || 0} sym`;
            } else {
              const hc = getHeatColor(c, maxCollateral);
              const rect = el.querySelector('rect');
              if (rect) {
                rect.setAttribute('fill', hc.bg);
                rect.setAttribute('stroke', hc.border);
              }
              const txt = el.querySelector('text');
              if (txt) txt.setAttribute('fill', hc.text);
              if (hc.tier === 4) el.classList.add('heat-critical');
              if (tier === 'mixed') el.classList.add('dep-mixed');
              badge.setAttribute('fill', hc.border);
              badge.textContent = `+${c} col`;
            }
            el.appendChild(badge);
          } else {
            el.classList.add('faded');
          }
        });

        edgesLayer.querySelectorAll('.edge-path').forEach(el => {
          const src = el.dataset.edgeSource;
          if (!reachableDependencies.has(src) && src !== selectedNodeId) {
            el.classList.add('faded');
          }
        });

        const rootNode = tabAllNodes.find(n => n.id === selectedNodeId);
        updateHeatLegend(rootNode ? rootNode.label : selectedNodeId, reachableDependencies.size, totalCollateral, justificationMap);
      } else {
        restoreStandardLegend();
      }
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

    // BFS over the full (unfiltered) edge dataset to find all outgoing include deps.
    // Temperature/heat must use this set, not the degree-limited activeEdges.
    function computeFullDependencies(nodeId) {
      const reachableDependencies = new Set();
      const queue = [nodeId];
      while (queue.length > 0) {
        const curr = queue.shift();
        tabAllEdges.forEach(e => {
          if (e.source === curr && !reachableDependencies.has(e.target)) {
            reachableDependencies.add(e.target);
            queue.push(e.target);
          }
        });
      }
      return reachableDependencies;
    }

    function selectNode(id, skipFilter = false, skipCrumb = false) {
      selectedNodeId = id;
      const node = activeNodes.find(n => n.id === id) || tabAllNodes.find(n => n.id === id);
      if (!node) { renderThreadBar(); return; }
      if (!skipCrumb) recordCrumb(id, node.label);

      // Update Inspector
      document.getElementById('inspLabel').textContent = node.label;
      document.getElementById('inspSub').textContent = node.path || node.id;
      document.getElementById('inspKind').textContent = node.kind;

      // Calculate connections from activeEdges (or tabAllEdges if you prefer, but UI usually shows visible)
      // Actually let's show true global connections in inspector
      const upstream = tabAllEdges.filter(e => e.target === id);
      const downstream = tabAllEdges.filter(e => e.source === id);

      document.getElementById('inspClosure').textContent = node.closure ? `${node.closure} files` : `${downstream.length}`;
      document.getElementById('inspClosureSub').textContent = node.closure ? 'Transitive include closure' : 'Outgoing dependencies';

      document.getElementById('inspDegree').textContent = String(upstream.length + downstream.length);

      // Include justification breakdown replaces the generic advice when the
      // payload has data for this node; live mode hydrates the unbounded detail.
      let justificationShown = false;
      if (currentTab === 'include') {
        justificationShown = renderJustificationBreakdown(id, null);
        if (!isStandaloneReport()) {
          fetchJustificationDetail(id).then(detail => {
            if (detail && selectedNodeId === id && currentTab === 'include') {
              renderJustificationBreakdown(id, detail);
            }
          });
        }
      } else {
        const justSection = document.getElementById('inspJustificationSection');
        if (justSection) justSection.style.display = 'none';
      }

      // Recommendations — with justification data the breakdown below supersedes
      // the high-fan-out platitude; cycle/stable messages stay unchanged.
      const insight = document.getElementById('inspInsight');
      const insightText = document.getElementById('inspInsightText');
      if (node.is_cycle) {
        insight.style.borderColor = 'rgba(239, 68, 68, 0.4)';
        insight.style.background = 'rgba(239, 68, 68, 0.1)';
        insightText.textContent = `Circular dependency detected involving ${node.label}. Breaking this cycle eliminates compile-time ordering hazards.`;
      } else if (downstream.length > 4 && !justificationShown) {
        insight.style.borderColor = 'rgba(251, 146, 60, 0.3)';
        insight.style.background = 'rgba(251, 146, 60, 0.1)';
        insightText.textContent = `High fan-out (${downstream.length} dependencies). Consider header extraction or forward declaration to reduce collateral load.`;
      } else if (downstream.length > 4) {
        insight.style.borderColor = 'rgba(251, 146, 60, 0.3)';
        insight.style.background = 'rgba(251, 146, 60, 0.1)';
        insightText.textContent = `High fan-out (${downstream.length} dependencies). See the Include Justification breakdown below for what each dependency is needed for.`;
      } else {
        insight.style.borderColor = 'rgba(56, 189, 248, 0.3)';
        insight.style.background = 'rgba(56, 189, 248, 0.08)';
        insightText.textContent = `Stable node with ${upstream.length} consumer(s) and ${downstream.length} direct dependency(s).`;
      }

      // Populate Upstream
      document.getElementById('inspUpstreamCount').textContent = String(upstream.length);
      const upList = document.getElementById('inspUpstreamList');
      upList.innerHTML = upstream.map(e => {
        const srcNode = tabAllNodes.find(n => n.id === e.source);
        const name = srcNode ? srcNode.label : e.source;
        return `
          <div class="link-item" onclick="selectNode('${e.source}')">
            <span>${name}</span>
            <span style="color:var(--text-sub);">${e.kind || 'Direct'}</span>
          </div>
        `;
      }).join('') || '<div style="color:var(--text-sub);font-style:italic;padding:4px;">None (Root)</div>';

      // Populate Downstream
      document.getElementById('inspDownstreamCount').textContent = String(downstream.length);
      const downList = document.getElementById('inspDownstreamList');
      downList.innerHTML = downstream.map(e => {
        const dstNode = tabAllNodes.find(n => n.id === e.target);
        const name = dstNode ? dstNode.label : e.target;
        return `
          <div class="link-item" onclick="selectNode('${e.target}')">
            <span style="color:var(--cyan);">${name}</span>
            <span style="color:${e.is_cycle ? '#ef4444' : 'var(--text-sub)'};">${e.is_cycle ? 'Cycle' : (e.kind || 'Direct')}</span>
          </div>
        `;
      }).join('') || '<div style="color:var(--text-sub);font-style:italic;padding:4px;">None (Leaf)</div>';

      // Populate Collateral Breakdown
      const collateralSection = document.getElementById('inspCollateralSection');
      const collateralList = document.getElementById('inspCollateralList');
      if (currentTab === 'include' && !justificationShown) {
        const reachableDeps = computeFullDependencies(id);
        const collateralMap = (graphData.file_collateral || {})[id] || {};
        const entries = [...reachableDeps].map(depId => {
          const depNode = tabAllNodes.find(n => n.id === depId);
          const col = collateralMap[depId] || 0;
          return { id: depId, label: depNode ? depNode.label : depId, col };
        }).sort((a, b) => b.col - a.col);

        const totalCol = entries.reduce((acc, e) => acc + e.col, 0);
        const maxCol = entries.length > 0 ? entries[0].col : 0;

        if (entries.length > 0 && collateralSection) {
          collateralSection.style.display = 'block';
          document.getElementById('inspCollateralCount').textContent = `${totalCol} symbols`;
          collateralList.innerHTML = entries.map(entry => {
            const hc = getHeatColor(entry.col, maxCol);
            return `
              <div class="link-item" onclick="illuminatePath('${id}', '${entry.id}')" title="Click to illuminate path from ${node.label} to ${entry.label}">
                <div style="display:flex;align-items:center;gap:6px;">
                  <span style="width:8px;height:8px;border-radius:2px;background:${hc.border};display:inline-block;"></span>
                  <span>${entry.label}</span>
                </div>
                <span style="font-family:monospace;font-weight:700;color:${hc.text};">+${entry.col}</span>
              </div>
            `;
          }).join('');

          const top = entries[0];
          if (top && top.col > 0) {
            insightText.textContent = `Total Collateral: ${totalCol} symbols. Top contributor: ${top.label} (+${top.col} symbols).`;
          }
        } else if (collateralSection) {
          collateralSection.style.display = 'none';
        }
      } else if (collateralSection) {
        collateralSection.style.display = 'none';
      }

      document.querySelectorAll('.node-group').forEach(el => el.classList.remove('selected'));
      renderSvg();

      if (!skipFilter) {
        filterAndExpandSubgraph();
      }
      renderHotspots();
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
        }
      });
    }

    function clearInspector() {
      document.getElementById('inspLabel').textContent = 'No selection';
      document.getElementById('inspSub').textContent = 'Graph is empty';
      document.getElementById('inspClosure').textContent = '0';
      document.getElementById('inspDegree').textContent = '0';
      document.getElementById('inspUpstreamList').innerHTML = '';
      document.getElementById('inspDownstreamList').innerHTML = '';
      const collateralSection = document.getElementById('inspCollateralSection');
      if (collateralSection) collateralSection.style.display = 'none';
      const justSection = document.getElementById('inspJustificationSection');
      if (justSection) justSection.style.display = 'none';
    }

    // Canvas panning and zooming
    function setupCanvasPanZoom() {
      const container = document.getElementById('canvasContainer');
      const svg = document.getElementById('graphSvg');
      const vp = document.getElementById('viewportGroup');

      function updateTransform() {
        vp.setAttribute('transform', `translate(${panX}, ${panY}) scale(${zoomScale})`);
      }

      container.addEventListener('mousedown', e => {
        if (e.target.closest('.node-group')) return;
        isDragging = true;
        dragStartX = e.clientX - panX;
        dragStartY = e.clientY - panY;
      });

      container.addEventListener('click', e => {
        if (!e.target.closest('.node-group') && dragStartX === e.clientX - panX && dragStartY === e.clientY - panY) {
          selectedNodeId = null;
          renderThreadBar();
          filterAndExpandSubgraph();
        }
      });

      window.addEventListener('mousemove', e => {
        if (!isDragging) return;
        panX = e.clientX - dragStartX;
        panY = e.clientY - dragStartY;
        updateTransform();
      });

      window.addEventListener('mouseup', () => { isDragging = false; });

      container.addEventListener('wheel', e => {
        e.preventDefault();
        const zoomDelta = e.deltaY < 0 ? 1.1 : 0.9;
        const newScale = Math.min(2.5, Math.max(0.2, zoomScale * zoomDelta));
        const rect = container.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;

        panX = mouseX - (mouseX - panX) * (newScale / zoomScale);
        panY = mouseY - (mouseY - panY) * (newScale / zoomScale);
        zoomScale = newScale;
        updateTransform();
      }, { passive: false });

      updateTransform();
    }

    function resetZoom() {
      panX = 40;
      panY = 40;
      zoomScale = 1.0;
      document.getElementById('viewportGroup').setAttribute('transform', `translate(${panX}, ${panY}) scale(${zoomScale})`);
    }

    function setHops(h) {
      // Legacy wrapper just in case
      filterAndExpandSubgraph();
    }

    function focusSelectedNeighborhood() {
      // Button hook
      filterAndExpandSubgraph();
    }

    function filterCyclesOnly() {
      cyclesOnly = !cyclesOnly;
      renderSvg();
    }

    function handleSearch(q) {
      selectedNodeId = null;
      renderThreadBar();
      filterAndExpandSubgraph();
      if (currentTab === 'include') {
        if (qoOpen) renderQuickOpen();
        else openQuickOpen();
      }
    }

// Subgraph filter & expansion engine (search matcher, degree stepper, node cap).
    let isFiltering = false;
    
    function compileMatcher(query) {
      if (!query) return null;
      let field = null;
      let val = query.trim();
      
      const parts = val.split(':');
      if (parts.length > 1 && ['path', 'location', 'kind'].includes(parts[0].toLowerCase())) {
        field = parts[0].toLowerCase();
        if (field === 'location') field = 'path';
        val = parts.slice(1).join(':').trim();
      }
      
      let isRegex = val.startsWith('/') && val.endsWith('/') && val.length > 2;
      let regex;
      
      if (isRegex) {
        let pattern = val.substring(1, val.length - 1);
        try { regex = new RegExp(pattern, 'i'); } catch (e) { regex = new RegExp(val.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i'); }
      } else if (val.includes('*') || val.includes('?')) {
        let escaped = val.replace(/[.+^${}()|[\]\\]/g, '\\$&').replace(/\*/g, '.*').replace(/\?/g, '.');
        regex = new RegExp(`^${escaped}$`, 'i');
      } else {
        let escaped = val.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        regex = new RegExp(escaped, 'i');
      }
      return { field, regex };
    }

    function matchesFilter(node, matcher) {
      if (dirScope && !((node.path || '').startsWith(dirScope))) return false;
      if (!matcher) return true;
      if (matcher.field) {
        const val = node[matcher.field] || '';
        return matcher.regex.test(val);
      }
      return matcher.regex.test(node.label) || matcher.regex.test(node.path);
    }

    function filterAndExpandSubgraph() {
      if (isFiltering) return;
      isFiltering = true;
      
      const q = document.getElementById('searchInput').value.trim();
      const matcher = compileMatcher(q);
      const dir = document.getElementById('dirSelect').value;
      
      let seeds = [];
      let matchCount = 0;
      if (q) {
        let matches = tabAllNodes.filter(n => matchesFilter(n, matcher));
        matchCount = matches.length;
      }
      
      if (selectedNodeId) {
        const n = tabAllNodes.find(n => n.id === selectedNodeId);
        if (n) seeds = [n];
      } else if (q) {
        seeds = tabAllNodes.filter(n => matchesFilter(n, matcher));
      } else if (dirScope) {
        seeds = tabAllNodes.filter(n => (n.path || '').startsWith(dirScope));
      } else {
        const inDeg = {};
        const outDeg = {};
        tabAllNodes.forEach(n => { inDeg[n.id] = 0; outDeg[n.id] = 0; });
        tabAllEdges.forEach(e => { 
          if (inDeg[e.target] !== undefined) inDeg[e.target]++; 
          if (outDeg[e.source] !== undefined) outDeg[e.source]++; 
        });
        let sorted = tabAllNodes.slice().sort((a,b) => {
          const aHasOut = outDeg[a.id] > 0 ? 0 : 1;
          const bHasOut = outDeg[b.id] > 0 ? 0 : 1;
          if (aHasOut !== bHasOut) return aHasOut - bHasOut;
          return inDeg[a.id] - inDeg[b.id];
        });
        seeds = sorted.slice(0, Math.min(sorted.length, nodeCap));
      }

      const matchCountBadge = document.getElementById('searchMatchCount');
      if (q) {
        matchCountBadge.style.display = 'inline-block';
        matchCountBadge.textContent = matchCount + ' matches';
        document.getElementById('searchClearBtn').style.display = 'inline-flex';
        document.getElementById('btnShowAllMatches').style.display = selectedNodeId ? 'inline-block' : 'none';
      } else {
        matchCountBadge.style.display = 'none';
        document.getElementById('searchClearBtn').style.display = 'none';
        document.getElementById('btnShowAllMatches').style.display = 'none';
      }

      const adjOut = {};
      const adjIn = {};
      tabAllNodes.forEach(n => { adjOut[n.id] = []; adjIn[n.id] = []; });
      tabAllEdges.forEach(e => {
        if(adjOut[e.source]) adjOut[e.source].push(e.target);
        if(adjIn[e.target]) adjIn[e.target].push(e.source);
      });

      let includedNodes = new Set();
      let queue = [];
      
      seeds.forEach(s => {
        includedNodes.add(s.id);
        let branchDir = 'both';
        if (dir === 'out') branchDir = 'out';
        if (dir === 'in') branchDir = 'in';
        queue.push({ id: s.id, dist: 0, branchDir });
      });

      let isCapped = false;
      const capReached = () => includedNodes.size >= nodeCap;

      if (includedNodes.size > nodeCap) {
        const toKeep = new Set(Array.from(includedNodes).slice(0, nodeCap));
        includedNodes = toKeep;
        queue = queue.filter(q => includedNodes.has(q.id));
        isCapped = true;
      }

      while (queue.length > 0 && !capReached()) {
        const { id, dist, branchDir } = queue.shift();
        
        if (!isAllCapped && dist >= currentDegree) continue;

        let neighbors = [];
        if (branchDir === 'both' || branchDir === 'out') {
          (adjOut[id] || []).forEach(nxt => neighbors.push({ nxt, nextDir: 'out' }));
        }
        if (branchDir === 'both' || branchDir === 'in') {
          (adjIn[id] || []).forEach(nxt => neighbors.push({ nxt, nextDir: 'in' }));
        }

        for (const { nxt, nextDir } of neighbors) {
          if (!includedNodes.has(nxt)) {
            includedNodes.add(nxt);
            queue.push({ id: nxt, dist: dist + 1, branchDir: nextDir });
            if (capReached()) {
              isCapped = true;
              break;
            }
          }
        }
      }

      activeNodes = tabAllNodes.filter(n => includedNodes.has(n.id));
      activeEdges = tabAllEdges.filter(e => includedNodes.has(e.source) && includedNodes.has(e.target));

      document.getElementById('badgeNodes').textContent = `${activeNodes.length} / ${tabAllNodes.length} Nodes`;
      document.getElementById('badgeEdges').textContent = `${activeEdges.length} Edges`;
      document.getElementById('capWarningBadge').style.display = isCapped ? 'inline-block' : 'none';

      applyLayout();

      if (selectedNodeId && activeNodes.find(n => n.id === selectedNodeId)) {
      } else if (activeNodes.length > 0) {
        selectNode(activeNodes[0].id, true, true);
      } else {
        clearInspector();
      }
      
      isFiltering = false;
      renderHotspots();
    }

    function stepDegree(delta) {
      currentDegree = Math.max(0, currentDegree + delta);
      isAllCapped = false;
      document.getElementById('degValue').textContent = currentDegree + ' deg';
      document.getElementById('hopAllCap').classList.remove('active');
      filterAndExpandSubgraph();
    }

    function toggleAllCapped() {
      isAllCapped = !isAllCapped;
      const btn = document.getElementById('hopAllCap');
      if (isAllCapped) btn.classList.add('active');
      else btn.classList.remove('active');
      filterAndExpandSubgraph();
    }

    function setNodeCap(cap) {
      nodeCap = cap;
      filterAndExpandSubgraph();
    }

    function clearFilter() {
      document.getElementById('searchInput').value = '';
      currentDegree = 1;
      isAllCapped = false;
      document.getElementById('degValue').textContent = '1 deg';
      document.getElementById('hopAllCap').classList.remove('active');
      filterAndExpandSubgraph();
    }

    function clearSearch() {
      document.getElementById('searchInput').value = '';
      filterAndExpandSubgraph();
    }
