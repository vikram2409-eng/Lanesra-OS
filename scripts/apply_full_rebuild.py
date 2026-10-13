#!/usr/bin/env python3
"""
FULL REBUILD: Ontology Graph + Object 360 UI to match the mockup.
- Rewrites graphRender() node output: pills instead of dots
- Increases layout spacing for pills
- Restructures toolbar to 3-row card layout
- Rewrites inspector panel to mockup spec
- Adds all CSS for pills, cards, chips

Idempotent: checks for markers before applying.
"""

import re
import sys

def patch_app_js(path):
    src = open(path, encoding='utf-8').read()
    orig = src
    changes = []

    # =========================================================================
    # 1. LAYOUT: Increase spacing for pills (wider than dots)
    # =========================================================================
    # LINK_LEN 125 -> 185, K 54 -> 82
    old_layout = " const TICKS=visible.length>260?60:110;\n const K=54,LINK_LEN=125;"
    new_layout = " const TICKS=visible.length>260?60:110;\n const K=82,LINK_LEN=185;"
    if old_layout in src and "const K=82,LINK_LEN=185;" not in src:
        src = src.replace(old_layout, new_layout)
        changes.append("layout spacing increased for pills")

    # =========================================================================
    # 2. NODES: Rewrite to pill-shaped output
    # =========================================================================
    # Replace the nodeDivs.push for regular nodes (dots -> pills)
    # Old: ontology-node-dot div + separate ontology-node-caption div
    # New: single og-pill div with dot + label inside

    old_node = """  const size=isRoot?ui.rootSize:ui.nodeSize;
  nodeDivs.push('<div class="ontology-node-dot'+(isRoot?' '+ui.rootClass:'')+(isInspecting?' ontology-node-inspecting':'')+(isDimmed?' ontology-node-dimmed':'')+(onPath?' ontology-node-path':'')+(inSim?' ontology-node-sim':'')+(inSim&&hl.simChange?' ontology-node-sim-'+hl.simChange:'')+'" '+ui.inspectAttr+'="'+n.type+':'+n.id+'" data-node-key="'+n.key+'" style="left:'+(p.x-size/2).toFixed(1)+'px;top:'+(p.y-size/2).toFixed(1)+'px;width:'+size+'px;height:'+size+'px;background:'+color+'" title="'+(ui.typeLabels[n.type]||n.type)+': '+n.label+'"></div>');
  nodeDivs.push('<div class="ontology-node-caption'+(isDimmed?' ontology-node-dimmed':'')+'" data-node-key="'+n.key+'" style="left:'+p.x.toFixed(1)+'px;top:'+(p.y+size/2+4).toFixed(1)+'px" title="'+n.label+'">'+short+'</div>');"""

    new_node = """  const pillCls='og-pill'+(isRoot?' og-pill-root':'')+(isInspecting?' og-pill-inspecting':'')+(isDimmed?' og-pill-dimmed':'')+(onPath?' og-pill-path':'')+(inSim?' og-pill-sim':'')+(inSim&&hl.simChange?' og-pill-sim-'+hl.simChange:'');
  const pillStyle=isRoot
   ?'left:'+p.x.toFixed(1)+'px;top:'+p.y.toFixed(1)+'px;background:'+color+';border-color:'+color
   :'left:'+p.x.toFixed(1)+'px;top:'+p.y.toFixed(1)+'px';
  const pillInner=isRoot
   ?'<span class="og-pill-label">'+short+'</span>'
   :'<span class="og-pill-dot" style="background:'+color+'"></span><span class="og-pill-label">'+short+'</span>';
  nodeDivs.push('<div class="'+pillCls+'" '+ui.inspectAttr+'="'+n.type+':'+n.id+'" data-node-key="'+n.key+'" style="'+pillStyle+'" title="'+(ui.typeLabels[n.type]||n.type)+': '+n.label+'" role="button" tabindex="0">'+pillInner+'</div>');"""

    if old_node in src:
        src = src.replace(old_node, new_node)
        changes.append("nodes rewritten as pills")
    elif "og-pill" not in src:
        print("WARNING: node pattern not found, pills not applied")
        return False

    # =========================================================================
    # 3. CLUSTERS: Pill-style for collapsed groups
    # =========================================================================
    old_cluster = """   nodeDivs.push('<div class="ontology-node-cluster" '+ui.collapseAttr+'="'+n.type+'" style="left:'+p.x.toFixed(1)+'px;top:'+p.y.toFixed(1)+'px;background:'+color+'" title="'+n.label+' — click to expand"><span>'+n.count+'</span></div>');
   nodeDivs.push('<div class="ontology-node-caption" style="left:'+p.x.toFixed(1)+'px;top:'+(p.y+26).toFixed(1)+'px">'+short+'</div>');
   return;"""

    new_cluster = """   nodeDivs.push('<div class="og-pill og-pill-cluster" '+ui.collapseAttr+'="'+n.type+'" data-node-key="'+n.key+'" style="left:'+p.x.toFixed(1)+'px;top:'+p.y.toFixed(1)+'px" title="'+n.label+' — click to expand" role="button" tabindex="0"><span class="og-pill-dot" style="background:'+color+'"></span><span class="og-pill-label">'+short+' · '+n.count+'</span></div>');
   return;"""

    if old_cluster in src:
        src = src.replace(old_cluster, new_cluster)
        changes.append("clusters rewritten as pills")

    # =========================================================================
    # 4. HOVER: Update selectors from .ontology-node-dot to .og-pill
    # =========================================================================
    # The delegation IIFE looks for .ontology-node-dot — update to .og-pill
    old_hover_dot = "const dot=e.target.closest?e.target.closest('.ontology-node-dot'):null;"
    new_hover_dot = "const dot=e.target.closest?e.target.closest('.og-pill'):null;"
    if old_hover_dot in src:
        src = src.replace(old_hover_dot, new_hover_dot)
        changes.append("hover selector updated to .og-pill")

    old_hover_qs = "canvas.querySelectorAll('.ontology-node-dot,.ontology-node-caption')"
    new_hover_qs = "canvas.querySelectorAll('.og-pill')"
    if old_hover_qs in src:
        src = src.replace(old_hover_qs, new_hover_qs)
        changes.append("hover query updated")

    old_dbl = "const dot=e.target.closest('.ontology-node-dot');if(!dot||!dot.dataset.nodeKey)return;"
    new_dbl = "const dot=e.target.closest('.og-pill');if(!dot||!dot.dataset.nodeKey)return;"
    # Only replace in wireGraphHoverFx (the dblclick handler), not the delegation
    # The delegation was already updated above, this is for the dblclick
    if src.count(old_dbl) >= 1:
        # Replace all remaining occurrences (delegation already changed)
        src = src.replace(old_dbl, new_dbl)
        changes.append("dblclick selector updated")

    # =========================================================================
    # 5. CSS: Update dimming selectors for pills
    # =========================================================================
    # This is in styles.css, handled separately

    # =========================================================================
    # 6. TOOLBAR: Restructure to 3-row card layout
    # =========================================================================
    # The current toolbar is a single div with tool-groups.
    # We'll wrap it in the card structure via CSS, but also reorder
    # to match the mockup: Row1 (Type, Component, Search, Find path),
    # Row2 (Depth, Impact/Lineage, Fields, Simulate), Row3 (Zoom)

    # For now, keep the existing toolbar HTML but add row wrapper classes
    # via a simple transform: add data-row attributes
    # Actually, the cleanest approach: CSS grid will handle the layout.
    # We'll just ensure the toolbar has the right class.
    old_toolbar_open = '<div class="graph-toolbar" role="toolbar" aria-label="Graph tools">'
    new_toolbar_open = '<div class="graph-toolbar og-toolbar-card" role="toolbar" aria-label="Graph tools">'
    if old_toolbar_open in src and "og-toolbar-card" not in src:
        src = src.replace(old_toolbar_open, new_toolbar_open)
        changes.append("toolbar card class added")

    # =========================================================================
    # 7. INSPECTOR: Rewrite to mockup spec
    # =========================================================================
    # The depDetailPanelHtml function needs restructuring.
    # We'll replace the header and actions sections.

    old_inspector_head = """ return '<div class="ontology-detail-panel card">'
  +'<div class="ontology-inspector-head">'+typeDotHtml(s.inspectType,14)+'<div><strong>'+label+'</strong><div class="ontology-inspector-subtitle">'+(SYSTEM_NODE_TYPE_LABELS[s.inspectType]||s.inspectType)+'</div></div></div>'"""

    new_inspector_head = """ return '<div class="ontology-detail-panel card og-inspector">'
  +'<div class="og-inspector-overline">'+(SYSTEM_NODE_TYPE_LABELS[s.inspectType]||s.inspectType).toUpperCase()+'</div>'
  +'<h3 class="og-inspector-title">'+label+'</h3>'
  +'<div class="ontology-inspector-head" style="display:none">'+typeDotHtml(s.inspectType,14)+'<div><strong>'+label+'</strong><div class="ontology-inspector-subtitle">'+(SYSTEM_NODE_TYPE_LABELS[s.inspectType]||s.inspectType)+'</div></div></div>'"""

    if old_inspector_head in src:
        src = src.replace(old_inspector_head, new_inspector_head)
        changes.append("inspector header rewritten with overline")

    # Update the action buttons to match mockup: full-width, specific styles
    # Button 1: primary, Button 2: outline, Button 3: text link
    old_actions = """  +'<div class="ontology-detail-actions">'
  +'<button type="button" class="btn btn-primary" id="depOpenAdmin">Open in its admin screen</button>'
  +(s.inspectType==='custom_object'?'<button type="button" class="btn btn-secondary" id="depOpen360">View records in Object 360</button>':'')
  +'<button type="button" class="btn btn-secondary" id="depExpand"'+(expandable?'':' disabled')+' title="'+(expandable?('Reveal '+expandable+' more neighbor'+(expandable===1?'':'s')+' around this node'):'Every neighbor is already shown')+'">Show what\\'s around it'+(expandable?' ('+expandable+')':'')+'</button>'
  +(isRoot?'':'<button type="button" class="btn btn-secondary" id="depRecenter">Re-center graph here</button>')
  +'</div>'"""

    new_actions = """  +'<div class="ontology-detail-actions og-actions">'
  +'<button type="button" class="btn btn-primary og-btn-full" id="depOpenAdmin">Open in its admin screen</button>'
  +'<button type="button" class="btn btn-outline og-btn-full" id="depOpen360">View records in Object 360</button>'
  +'<button type="button" class="btn btn-link og-btn-full" id="depExpand"'+(expandable?'':' disabled')+' title="'+(expandable?('Reveal '+expandable+' more neighbor'+(expandable===1?'':'s')+' around this node'):'Every neighbor is already shown')+'">Show what\\'s around it'+(expandable?' ('+expandable+')':'')+'</button>'
  +(isRoot?'':'<button type="button" class="btn btn-secondary og-btn-full" id="depRecenter">Re-center graph here</button>')
  +'</div>'"""

    if old_actions in src:
        src = src.replace(old_actions, new_actions)
        changes.append("inspector actions rewritten (3-button mockup layout)")
    else:
        print("WARNING: inspector actions pattern not found")

    # Update section labels to uppercase mockup style
    old_section_label = """'<div class="ontology-detail-section"><div class="ontology-detail-section-label">Fields · '+fields.length+'</div>'"""
    new_section_label = """'<div class="ontology-detail-section"><div class="og-section-label">FIELDS · '+fields.length+'</div>'"""
    if old_section_label in src:
        src = src.replace(old_section_label, new_section_label)
        changes.append("fields section label uppercased")

    old_rel_label = """'<div class="ontology-detail-section"><div class="ontology-detail-section-label">Relationships</div>'"""
    new_rel_label = """'<div class="ontology-detail-section"><div class="og-section-label">RELATIONSHIPS</div>'"""
    if old_rel_label in src:
        src = src.replace(old_rel_label, new_rel_label)
        changes.append("relationships section label uppercased")

    # =========================================================================
    # 8. GRAPH HEADER: Add mockup-style header classes
    # =========================================================================
    old_stage_head = """graphHtml=`<div class="graph-stage" data-graph-stage><div class="graph-stage-head"><div><div class="eyebrow">Ontology Graph</div><h3>${systemGraphNodeLabel(dependencyExplorerState.type,dependencyExplorerState.id)} neighborhood</h3></div><div class="chip-row"><span class="badge badge-outline">Depth ${String(dependencyExplorerState.depth)==='all'?'All':String(dependencyExplorerState.depth||'1')}</span><span class="badge badge-secondary">${depModel.edges.length} relationships</span>${truncatedHits?`<span class="ontology-more-chip" title="Narrow the depth or hide types to see the rest">+${truncatedHits} more</span>`:''}</div></div>"""

    new_stage_head = """graphHtml=`<div class="graph-stage" data-graph-stage><div class="graph-stage-head og-stage-head"><div><h3 class="og-stage-title">${systemGraphNodeLabel(dependencyExplorerState.type,dependencyExplorerState.id)} neighborhood</h3></div><div class="chip-row og-stage-chips"><span class="og-pill-chip">Depth ${String(dependencyExplorerState.depth)==='all'?'All':String(dependencyExplorerState.depth||'1')}</span><span class="og-pill-chip">${depModel.edges.length} relationships</span>${truncatedHits?`<span class="ontology-more-chip" title="Narrow the depth or hide types to see the rest">+${truncatedHits} more</span>`:''}</div></div>"""

    if old_stage_head in src:
        src = src.replace(old_stage_head, new_stage_head)
        changes.append("graph header restyled")

    if src != orig:
        open(path, 'w', encoding='utf-8').write(src)
        print(f"app.js: {len(changes)} changes applied")
        for c in changes:
            print(f"  - {c}")
        return True
    else:
        print("app.js: no changes (already applied?)")
        return False

def patch_styles_css(path):
    src = open(path, encoding='utf-8').read()
    if src.count("/* OG-REBUILD") >= 1:
        print("styles.css: already patched")
        return False

    css = """
/* OG-REBUILD: Full rebuild styles matching the mockup */
.og-toolbar-card{
 background:#fff;border:1px solid #e5e7eb;border-radius:12px;
 box-shadow:0 1px 3px rgba(0,0,0,0.08);padding:16px;
 display:flex;flex-direction:column;gap:12px;margin-bottom:16px;
}
.og-toolbar-card .tool-group{
 display:flex;align-items:center;gap:12px;flex-wrap:wrap;
 padding-bottom:12px;border-bottom:1px solid #f3f4f6;
}
.og-toolbar-card .tool-group:last-child{border-bottom:none;padding-bottom:0;}

/* Pill nodes */
.og-pill{
 position:absolute;transform:translate(-50%,-50%);
 display:inline-flex;align-items:center;gap:6px;
 padding:6px 12px;border-radius:9999px;
 background:#fff;border:1.5px solid #e5e7eb;
 box-shadow:0 1px 3px rgba(0,0,0,0.1);
 font-size:12px;font-weight:500;color:#1f2937;white-space:nowrap;
 cursor:pointer;transition:all 0.2s ease;z-index:2;
}
.og-pill:hover{transform:translate(-50%,-50%) scale(1.08);box-shadow:0 4px 12px rgba(0,0,0,0.15);z-index:10;}
.og-pill-root{
 background:#4f46e5!important;border-color:#4f46e5!important;
 color:#fff!important;font-weight:600;font-size:13px;
 padding:8px 16px;box-shadow:0 2px 8px rgba(79,70,229,0.3);
}
.og-pill-dot{
 width:8px;height:8px;border-radius:50%;flex-shrink:0;
}
.og-pill-label{line-height:1.2;}
.og-pill-cluster{border-style:dashed;}
.og-pill-inspecting{outline:2px solid #4f46e5;outline-offset:2px;}
.og-pill-path{background:#eef2ff!important;border-color:#4f46e5!important;}
.og-pill-dimmed{opacity:0.25;}

/* Hover spotlight for pills */
.ontology-graph-canvas.graph-hover-dim .og-pill:not(.graph-hover-on){opacity:0.18;}
.ontology-graph-canvas.graph-hover-dim .og-pill{transition:opacity 0.2s ease;}
.og-pill.graph-hover-on{opacity:1!important;}

/* Edge labels as pills */
.ontology-edge-label{
 background:rgba(255,255,255,0.9);border:1px solid #e5e7eb;
 border-radius:9999px;padding:2px 8px;font-size:10px;color:#6b7280;
 transform:translate(-50%,-50%);white-space:nowrap;
}

/* Graph header */
.og-stage-head{display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;}
.og-stage-title{font-size:20px;font-weight:600;color:#111827;margin:0;}
.og-stage-chips{display:flex;gap:8px;}
.og-pill-chip{
 background:#f3f4f6;border-radius:9999px;padding:4px 12px;
 font-size:12px;color:#4b5563;font-weight:500;
}

/* Inspector */
.og-inspector{padding:20px;}
.og-inspector-overline{
 font-size:11px;font-weight:600;letter-spacing:0.08em;
 color:#6b7280;text-transform:uppercase;margin-bottom:4px;
}
.og-inspector-title{font-size:20px;font-weight:600;color:#111827;margin:0 0 8px 0;}
.og-actions{display:flex;flex-direction:column;gap:8px;margin:16px 0;}
.og-btn-full{width:100%;justify-content:center;}
.og-section-label{
 font-size:11px;font-weight:600;letter-spacing:0.06em;
 color:#6b7280;text-transform:uppercase;margin:16px 0 8px 0;
 padding-bottom:4px;border-bottom:1px solid #f3f4f6;
}

/* Dot-grid canvas background */
.ontology-graph-canvas{
 background-image:radial-gradient(#e5e7eb 1px, transparent 1px);
 background-size:20px 20px;
}

/* Legend chips */
.ontology-legend{
 display:flex;gap:8px;overflow-x:auto;padding:12px 0;
 scrollbar-width:thin;
}
.ontology-legend-item{
 display:inline-flex;align-items:center;gap:6px;
 padding:6px 12px;border-radius:9999px;white-space:nowrap;
 background:#fff;border:1px solid #e5e7eb;font-size:12px;
 cursor:pointer;transition:all 0.15s ease;
}
.ontology-legend-item:hover{border-color:#4f46e5;}
.ontology-legend-item-off{opacity:0.4;}

/* Mobile */
@media (max-width: 640px){
 .og-toolbar-card{padding:12px;}
 .og-pill{font-size:11px;padding:4px 10px;}
 .og-stage-head{flex-direction:column;align-items:flex-start;gap:8px;}
}
"""

    src = src.rstrip() + "\n" + css + "\n"
    open(path, 'w', encoding='utf-8').write(src)
    print("styles.css: rebuild styles added")
    return True

if __name__ == '__main__':
    import os
    app_ok = patch_app_js('app.js')
    css_ok = patch_styles_css('styles.css')
    print(f"\nDone. app.js: {'patched' if app_ok else 'skipped'}, styles.css: {'patched' if css_ok else 'skipped'}")
