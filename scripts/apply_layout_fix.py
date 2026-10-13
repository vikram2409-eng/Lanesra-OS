#!/usr/bin/env python3
"""
EMERGENCY FIX: Graph layout broken after rebuild.
- Toolbar groups overlapping
- Canvas squeezed, inspector too wide
"""

def patch_css(path):
    src = open(path, encoding='utf-8').read()
    if "og-layout-emergency-fix" in src:
        return []
    
    fix = """
/* og-layout-emergency-fix: repair broken rebuild layout */
.og-toolbar-card{display:block!important;}
.og-toolbar-card .tool-group{
 display:flex!important;align-items:center;gap:10px;flex-wrap:wrap;
 margin-bottom:10px;padding-bottom:10px;border-bottom:1px solid #f3f4f6;
}
.og-toolbar-card .tool-group:last-child{margin-bottom:0;padding-bottom:0;border-bottom:none;}
/* Main graph + inspector layout: canvas flexes, inspector fixed width */
.graph-layout,.dep-layout,.ontology-layout{display:flex!important;gap:16px;align-items:flex-start;}
.graph-stage,.dep-graph-wrap,.ontology-graph-wrap{flex:1 1 auto!important;min-width:0!important;}
.ontology-detail-panel.og-inspector{flex:0 0 340px!important;max-width:340px!important;}
.ontology-graph-canvas{width:100%!important;min-height:500px;}
"""
    src = src.rstrip() + "\n" + fix
    open(path, 'w', encoding='utf-8').write(src)
    return ["emergency layout fix applied"]

if __name__ == '__main__':
    import sys
    print(patch_css(sys.argv[1] if len(sys.argv) > 1 else 'styles.css'))
