#!/usr/bin/env python3
"""Fix canvas width: force graph-stage to flex properly."""

def patch_css(path):
    src = open(path, encoding='utf-8').read()
    if "og-canvas-width-fix" in src:
        return []
    fix = """
/* og-canvas-width-fix: force canvas to take available width */
.graph-stage{flex:1 1 0!important;min-width:0!important;max-width:none!important;}
.graph-stage-head{margin-bottom:12px;}
.ontology-graph-canvas{width:100%!important;}
.ontology-graph-viewport{width:100%!important;}
/* Ensure parent is flex */
div:has(> .graph-stage){display:flex!important;gap:16px;align-items:flex-start;}
"""
    src = src.rstrip() + "\n" + fix
    open(path, 'w', encoding='utf-8').write(src)
    return ["canvas width fix applied"]

if __name__ == '__main__':
    import sys
    print(patch_css(sys.argv[1] if len(sys.argv) > 1 else 'styles.css'))
