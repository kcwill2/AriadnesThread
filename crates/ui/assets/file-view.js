    // Source Code Panel (right side, collapsible & resizable)
    let sourcePanelWidth = 480;

    function startSourceResize(evt) {
      evt.preventDefault();
      const drawer = document.getElementById('sourceDrawer');
      const handle = drawer.querySelector('.resize-handle');
      drawer.classList.add('open');
      drawer.classList.add('resizing');
      if (handle) handle.classList.add('dragging');
      const startX = evt.clientX;
      const startW = drawer.getBoundingClientRect().width;
      const onMove = (ev) => {
        const maxW = Math.floor(window.innerWidth * 0.8);
        // Panel is right-anchored: dragging left grows it.
        sourcePanelWidth = Math.min(maxW, Math.max(280, startW + (startX - ev.clientX)));
        drawer.style.setProperty('--source-panel-w', sourcePanelWidth + 'px');
      };
      const onUp = () => {
        drawer.classList.remove('resizing');
        if (handle) handle.classList.remove('dragging');
        window.removeEventListener('pointermove', onMove);
        window.removeEventListener('pointerup', onUp);
      };
      window.addEventListener('pointermove', onMove);
      window.addEventListener('pointerup', onUp);
    }

    function sourceLoadError(message) {
      document.getElementById('drawerLine').textContent = 'Error';
      document.getElementById('drawerCodeContent').innerHTML =
        `<div style="color:var(--red);padding:10px;">${escapeHtml(message)}</div>`;
    }

    async function openSourceDrawer() {
      const drawer = document.getElementById('sourceDrawer');
      const node = activeNodes.find(n => n.id === selectedNodeId);
      if (!node) return;

      drawer.style.setProperty('--source-panel-w', sourcePanelWidth + 'px');
      drawer.classList.add('open');
      const restoreTab = document.getElementById('sourceRestoreTab');
      if (restoreTab) restoreTab.hidden = true;
      document.getElementById('drawerTitle').textContent = node.path || node.label;
      document.getElementById('drawerLine').textContent = 'Loading...';

      const codeContainer = document.getElementById('drawerCodeContent');
      codeContainer.innerHTML = '<div style="color:var(--text-sub);">Loading source...</div>';

      if (!node.path) {
        sourceLoadError('This node has no file path (symbol/call-graph nodes are not backed by a single file).');
        return;
      }

      try {
        const res = await fetch(`/api/source?file=${encodeURIComponent(node.path)}`);
        if (!res.ok) {
          sourceLoadError(`Could not load source for ${node.path} (server returned HTTP ${res.status}).`);
          return;
        }
        const text = await res.text();
        const lineCount = renderSourceCode(text);
        document.getElementById('drawerLine').textContent = `${lineCount} lines`;
      } catch (e) {
        sourceLoadError(`Could not load source for ${node.path}: ${e}`);
      }
    }

    function closeSourceDrawer() {
      document.getElementById('sourceDrawer').classList.remove('open');
      const restoreTab = document.getElementById('sourceRestoreTab');
      if (restoreTab) restoreTab.hidden = false;
    }

    function restoreSourceDrawer() {
      const drawer = document.getElementById('sourceDrawer');
      if (drawer) drawer.classList.add('open');
      const restoreTab = document.getElementById('sourceRestoreTab');
      if (restoreTab) restoreTab.hidden = true;
    }

    function renderSourceCode(code) {
      const container = document.getElementById('drawerCodeContent');
      const lines = code.split('\n');
      if (lines.length > 1 && lines[lines.length - 1] === '') lines.pop();
      container.innerHTML = lines.map((line, idx) => {
        const lineNum = idx + 1;
        const isHighlight = line.includes('#include') || /^\s*(class|struct|template)\b/.test(line);
        return `
          <div class="code-line ${isHighlight ? 'line-highlight' : ''}">
            <span class="line-num">${lineNum}</span>
            <span class="line-text">${escapeHtml(line)}</span>
          </div>
        `;
      }).join('');
      return lines.length;
    }

    function escapeHtml(str) {
      return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    }

    function exportData() {
      const jsonStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(graphData, null, 2));
      const downloadAnchor = document.createElement('a');
      downloadAnchor.setAttribute("href", jsonStr);
      downloadAnchor.setAttribute("download", `${graphData.repo_name || 'ariadne'}_graph.json`);
      document.body.appendChild(downloadAnchor);
      downloadAnchor.click();
      downloadAnchor.remove();
    }
