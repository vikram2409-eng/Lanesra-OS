#!/usr/bin/env python3
"""Ontology Graph: convert left tool rail into a horizontal toolbar below the
page header. Safe to run once per branch.
"""
import re

def patch(path):
    src = open(path, encoding='utf-8').read()
    changes = []
    old = '<aside class="graph-tools" aria-label="Graph tools">'
    new = '<div class="graph-toolbar" role="toolbar" aria-label="Graph tools">'
    assert src.count(old) == 1, "graph-tools aside not found"
    src = src.replace(old, new)
    changes.append("toolbar div")
    old_close = " </aside>`;\n let graphHtml='';"
    assert src.count(old_close) == 1, "tools close not found"
    src = src.replace(old_close, " </div>`;\n let graphHtml='';")
    changes.append("toolbar close")
    old_wb = 'body.innerHTML=`<div class="workbench">${depToolsHtml}${graphHtml||'
    assert src.count(old_wb) == 1, "workbench not found"
    new_wb = 'body.innerHTML=`<div class="graph-toolbar-wrap">${depToolsHtml}</div><div class="workbench">${graphHtml||'
    src = src.replace(old_wb, new_wb)
    changes.append("toolbar above workbench")
    old_comp = '<div class="tool-section"><div class="tool-label">Component</div><div class="path-fields"><label>Component type '
    new_comp = '<div class="tool-group"><label class="tool-inline">Type '
    assert src.count(old_comp) == 1, "component not found"
    src = src.replace(old_comp, new_comp)
    old_comp2 = '</select></label><label>Component <select'
    new_comp2 = '</select></label><label class="tool-inline">Component <select'
    assert src.count(old_comp2) == 1
    src = src.replace(old_comp2, new_comp2)
    changes.append("component compact")
    old_search = '<div class="tool-section"><div class="tool-label">Search graph</div><div class="search-wrap" style="width:100%">'
    new_search = '<div class="tool-group"><div class="search-wrap" style="min-width:200px">'
    assert src.count(old_search) == 1, "search not found"
    src = src.replace(old_search, new_search)
    old_path = '<div class="tool-section"><div class="tool-label">Find path</div><div class="path-fields">'
    new_path = '<div class="tool-group"><div class="path-fields">'
    assert src.count(old_path) == 1, "path not found"
    src = src.replace(old_path, new_path)
    old_vp = '<div class="tool-section"><div class="tool-label">Viewport</div><div class="zoom-row">'
    new_vp = '<div class="tool-group"><div class="zoom-row" role="group" aria-label="Viewport">'
    assert src.count(old_vp) == 1, "viewport not found"
    src = src.replace(old_vp, new_vp)
    old_depth = '<div class="tool-section"><div class="tool-label">Depth</div><div class="chip-row"><label>Depth '
    new_depth = '<div class="tool-group"><label class="tool-inline">Depth '
    assert src.count(old_depth) == 1, "depth not found"
    src = src.replace(old_depth, new_depth)
    old_an = '<div class="tool-section"><div class="tool-label">Analysis</div><div class="chip-row">'
    new_an = '<div class="tool-group"><div class="chip-row" role="group" aria-label="Analysis">'
    assert src.count(old_an) == 1, "analysis not found"
    src = src.replace(old_an, new_an)
    old_ct = '<div class="tool-section"><div class="tool-label">Content</div><div class="chip-row">'
    new_ct = '<div class="tool-group"><div class="chip-row">'
    assert src.count(old_ct) == 1, "content not found"
    src = src.replace(old_ct, new_ct)
    old_sim = '<div class="tool-section"><div class="tool-label">Simulate</div><div class="path-fields">'
    new_sim = '<div class="tool-group"><div class="path-fields">'
    assert src.count(old_sim) == 1, "simulate not found"
    src = src.replace(old_sim, new_sim)
    changes.append("sections compacted")
    open(path, 'w', encoding='utf-8').write(src)
    print("toolbar done:", changes)

def patch_css(path):
    src = open(path, encoding='utf-8').read()
    additions = """
/* ===== Graph toolbar: horizontal below header ===== */
.graph-toolbar-wrap{margin-bottom:14px}
.graph-toolbar{display:flex;flex-wrap:wrap;gap:10px 16px;align-items:center;padding:12px 16px;background:var(--card);border:1px solid var(--line);border-radius:var(--radius-lg)}
.tool-group{display:flex;align-items:center;gap:8px;flex-wrap:wrap}
.tool-group + .tool-group{border-left:1px solid var(--line);padding-left:16px}
.tool-inline{display:inline-flex;align-items:center;gap:6px;font-size:13px;font-weight:600;color:var(--muted);white-space:nowrap}
.tool-inline .select,.tool-group .select{width:auto;min-height:34px;padding:6px 28px 6px 10px;font-size:13px}
@media(max-width:900px){.tool-group + .tool-group{border-left:0;padding-left:0}}
"""
    clean = re.sub(r'/\*.*?\*/', '', src, flags=re.S)
    assert clean.count('{') == clean.count('}'), "braces unbalanced"
    if '.graph-toolbar{' not in src:
        src = src.rstrip('\n') + '\n' + additions
    open(path, 'w', encoding='utf-8').write(src)
    print("css done")

if __name__ == '__main__':
    patch('app.js')
    patch_css('styles.css')
    print("done")
