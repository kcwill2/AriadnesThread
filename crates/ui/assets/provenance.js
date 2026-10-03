// D27 node-provenance inspector rows (M22.2c): declared-vs-defined for symbol
// nodes (plan §3.6.1) and language/unit_kind for file nodes. Reads the raw
// payload element of the current selection from graphData, so graph.js stays
// frozen; mounting on the #inspLabel mutation covers every selection/clear
// path (selectNode, clearInspector, tab switches re-render through them).
(function () {
    function provenanceRawData() {
      if (selectedNodeId === null || selectedNodeId === undefined) return null;
      const list = currentTab === 'include' ? (graphData.include_graph || [])
        : currentTab === 'symbol' ? (graphData.symbol_graph || [])
        : [];
      const el = list.find(e => String(e.data.id) === String(selectedNodeId));
      return el ? el.data : null;
    }

    function provenanceRows(data) {
      const rows = [];
      if (data.language) {
        const unit = data.unit_kind ? ' \u00b7 ' + escapeHtml(data.unit_kind) : '';
        rows.push(`<span class="prov-label">Language:</span> ${escapeHtml(data.language)}${unit}`);
      }
      if (data.defining_file) {
        rows.push(`<span class="prov-label">Defined in:</span> ${escapeHtml(String(data.defining_file))}`);
      }
      const declared = Array.isArray(data.declaring_files) ? data.declaring_files : [];
      if (declared.length > 0) {
        const names = declared.map(p => escapeHtml(String(p))).join(', ');
        rows.push(`<span class="prov-label">Declared in:</span> ${names} (${declared.length})`);
      }
      return rows;
    }

    function renderNodeProvenance() {
      const section = document.getElementById('inspProvenanceSection');
      if (!section) return;
      const data = provenanceRawData();
      const rows = data ? provenanceRows(data) : [];
      section.innerHTML = rows.map(r => `<div class="prov-row">${r}</div>`).join('');
      section.style.display = rows.length ? 'block' : 'none';
    }

    new MutationObserver(() => renderNodeProvenance()).observe(
      document.getElementById('inspLabel'),
      { childList: true, characterData: true, subtree: true }
    );
})();
