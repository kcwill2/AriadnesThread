import sys

def modify_html(html):
    # Add Thread Bar CSS
    thread_css = """
    /* ARIADNE'S THREAD BAR */
    .thread-bar {
      height: 38px;
      background: var(--bg-card);
      border-bottom: 1px solid var(--border);
      padding: 0 16px;
      display: flex;
      align-items: center;
      gap: 8px;
      overflow-x: auto;
      z-index: 21;
    }
    .thread-item {
      padding: 4px 10px;
      background: #1e293b;
      border: 1px solid var(--border);
      border-radius: 12px;
      font-size: 11px;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 6px;
    }
    .thread-item.active {
      background: #0284c7;
      color: white;
    }
    .thread-sep {
      color: var(--text-muted);
      font-size: 10px;
    }
    .modal {
      display: none;
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      background: var(--bg-surface);
      border: 1px solid var(--border);
      padding: 20px;
      border-radius: 8px;
      z-index: 100;
      width: 400px;
      box-shadow: 0 10px 25px rgba(0,0,0,0.5);
    }
    .modal-overlay {
      display: none;
      position: fixed;
      inset: 0;
      background: rgba(0,0,0,0.5);
      z-index: 99;
    }
    .badge-expand {
      fill: #334155;
      stroke: #fff;
      cursor: pointer;
    }
    """
    
    html = html.replace('</style>', thread_css + '\n  </style>')

    # Add Modals and Thread Bar
    ui_elements = """
  <div class="thread-bar" id="ariadneThread">
    <!-- Breadcrumbs go here -->
  </div>
  
  <div class="modal-overlay" id="modalOverlay" onclick="closeModals()"></div>
  <div class="modal" id="pathfinderModal">
    <h3>Pathfinder</h3>
    <input type="text" id="pathFrom" placeholder="From Node ID">
    <input type="text" id="pathTo" placeholder="To Node ID">
    <button onclick="findPath()">Find Path</button>
    <div id="pathResult"></div>
  </div>
  <div class="modal" id="entrypointsModal">
    <h3>Curated Entrypoints</h3>
    <div id="entrypointsList"></div>
  </div>
    """
    
    html = html.replace('<!-- SECONDARY TOOLBAR -->', ui_elements + '\n  <!-- SECONDARY TOOLBAR -->')
    
    # Add Toolbar Controls
    toolbar_addition = """
      <div class="tool-group">
        <span>Direction:</span>
        <select class="select-dark" id="dirSelect">
          <option value="both">Both</option>
          <option value="out">Out</option>
          <option value="in">In</option>
        </select>
      </div>
      <button class="btn-secondary" onclick="document.getElementById('pathfinderModal').style.display='block';document.getElementById('modalOverlay').style.display='block';">Pathfinder</button>
      <button class="btn-secondary" onclick="showEntrypoints()">Entrypoints</button>
    """
    
    html = html.replace('</select>\n      </div>', '</select>\n      </div>' + toolbar_addition, 1)

    # Add JS logic stub
    js_stub = """
    function clientTraverseNeighborhood() {}
    function clientShortestPath() {}
    function clientGetEntrypoints() {}
    function closeModals() {
      document.getElementById('pathfinderModal').style.display='none';
      document.getElementById('entrypointsModal').style.display='none';
      document.getElementById('modalOverlay').style.display='none';
    }
    function findPath() {}
    function showEntrypoints() {
      document.getElementById('entrypointsModal').style.display='block';
      document.getElementById('modalOverlay').style.display='block';
    }
    """
    
    html = html.replace('</script>', js_stub + '\n  </script>')

    return html

if __name__ == "__main__":
    with open("c:\\Users\\Kurt\\Code\\AriadnesThread\\crates\\ui\\assets\\index.html", "r", encoding="utf-8") as f:
        html = f.read()
    
    new_html = modify_html(html)
    
    with open("c:\\Users\\Kurt\\Code\\AriadnesThread\\crates\\ui\\assets\\index.html", "w", encoding="utf-8") as f:
        f.write(new_html)
