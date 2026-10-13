#!/usr/bin/env python3
"""
MOCKUP PARITY: Make the live Ontology Graph match the approved mockup artifact exactly.

Discrepancies fixed:
A. Node type colors → mockup palette (field gray, rule amber, workflow teal, agent purple, screen slate)
B. Edge color → gray #cbd5e1 default; indigo #4f46e5 + 3px on hovered neighborhood
C. Edge labels → subtle text (no pill chrome), hidden by default, shown on hover
D. Dim values → node .22, edge .14 (mockup values)
E. Inspector overline → pill badge (indigo tint, 9px/800)
F. Header badges → 6px radius, 10px/700 (not pill/12px)
G. Inspector buttons → 36px height, 11px/700
H. Dotted background → rgba(100,116,139,.18) (mockup value)

Idempotent: checks for markers.
"""

def patch_app_js(path):
    src = open(path, encoding='utf-8').read()
    orig = src
    changes = []

    # A. Node type colors → mockup palette
    old_colors = "SYSTEM_NODE_TYPE_COLORS={custom_object:'#4F46E5',custom_field:'#16A34A',relationship:'#D97706',business_rule:'#DC2626',workflow:'#2563EB',screen_layout:'#7C3AED',page_layout:'#0D9488',ai_agent:'#059669',execution_graph:'#EA580C',business_glossary_term:'#E11D48',metric_definition:'#0284C7',test_case_definition:'#9333EA',action_type_definition:'#475569'}"
    new_colors = "SYSTEM_NODE_TYPE_COLORS={custom_object:'#4F46E5',custom_field:'#64748B',relationship:'#D97706',business_rule:'#B45309',workflow:'#0F766E',screen_layout:'#475569',page_layout:'#475569',ai_agent:'#9333EA',execution_graph:'#0F766E',business_glossary_term:'#E11D48',metric_definition:'#0284C7',test_case_definition:'#9333EA',action_type_definition:'#475569'}"
    if old_colors in src:
        src = src.replace(old_colors, new_colors)
        changes.append("A: node colors → mockup palette")

    # B. Edge color: neutral gray instead of source-type color.
    _old = "stroke=" + chr(34) + chr(39) + "+color+" + chr(39) + chr(34)
    _new = "stroke=" + chr(34) + "#cbd5e1" + chr(34)
    if _old in src:
        n = src.count(_old)
        src = src.replace(_old, _new)
        changes.append('B: edges -> mockup gray #cbd5e1 (%d)' % n)
    else:
        changes.append('B: SKIPPED')

    if src != orig:
        open(path, 'w', encoding='utf-8').write(src)
    return changes


def patch_css(path):
    src = open(path, encoding='utf-8').read()
    if "og-mockup-parity" in src:
        return []
    orig = src

    css = """
/* og-mockup-parity: exact mockup matching */
/* B2. Active edges → indigo 3px on hover (mockup .edge.active) */
.ontology-graph-canvas.graph-hover-dim line.graph-hover-on{stroke:#4f46e5!important;stroke-width:3!important;stroke-opacity:1!important;}
/* C. Edge labels → mockup style: subtle text, hidden until hover */
.ontology-edge-label{
 background:transparent!important;border:none!important;border-radius:0!important;
 padding:0!important;color:#64748b!important;font-size:10px!important;font-weight:600!important;
 opacity:0;transition:opacity .18s;pointer-events:none;white-space:nowrap;
 text-shadow:0 0 4px #fff,0 0 4px #fff,0 0 4px #fff;
}
.ontology-edge-label.graph-hover-on{opacity:1!important;}
/* D. Dim values → mockup (.22 nodes, .14 edges) */
.ontology-graph-canvas.graph-hover-dim .og-pill:not(.graph-hover-on){opacity:.22!important;}
.ontology-graph-canvas.graph-hover-dim svg line:not(.graph-hover-on){opacity:.14!important;stroke-opacity:.14!important;}
/* E. Inspector overline → mockup type-pill */
.og-inspector-overline{
 display:inline-flex!important;border-radius:999px!important;
 background:#eef2ff!important;color:#3730a3!important;
 padding:5px 8px!important;font-size:9px!important;font-weight:800!important;
 letter-spacing:.09em!important;text-transform:uppercase;margin-bottom:8px;
}
/* F. Header badges → mockup .badge (6px radius, 10px/700) */
.og-pill-chip{
 border-radius:6px!important;background:#f1f5f9!important;
 color:#64748b!important;font-size:10px!important;font-weight:700!important;
 padding:0 7px!important;height:22px!important;display:inline-flex!important;align-items:center;
 border:1px solid #e2e8f0;
}
/* G. Inspector buttons → mockup .action (36px, 11px/700) */
#depOpenAdmin,#depOpen360,#depExpand{
 height:36px!important;font-size:11px!important;font-weight:700!important;
 border-radius:8px!important;width:100%!important;
}
#depOpenAdmin{background:#4f46e5!important;border-color:#4f46e5!important;color:#fff!important;}
#depOpen360{background:#fff!important;border:1px solid #e2e8f0!important;color:#111827!important;}
#depExpand{border-color:transparent!important;color:#4f46e5!important;background:transparent!important;}
/* H. Dotted background → mockup rgba(100,116,139,.18) */
.ontology-graph-canvas{
 background-image:radial-gradient(rgba(100,116,139,.18) 1px,transparent 1px)!important;
 background-size:20px 20px!important;
}
"""

    src = src.rstrip() + "\n" + css
    open(path, 'w', encoding='utf-8').write(src)
    return ["mockup parity CSS applied"]


if __name__ == '__main__':
    import sys
    js = sys.argv[1] if len(sys.argv) > 1 else 'app.js'
    cssp = sys.argv[2] if len(sys.argv) > 2 else 'styles.css'
    print("JS:", patch_app_js(js))
    print("CSS:", patch_css(cssp))
