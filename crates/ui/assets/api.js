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
              }
              applyLoadedData();
            }
          }
        } catch (e) { /* transient fetch failure; retry on next tick */ }
        setTimeout(loop, 2000);
      };
      setTimeout(loop, 2000);
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
