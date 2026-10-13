#!/usr/bin/env python3
"""
Fix mobile issues:
1. Graph missing on mobile - ensure canvas visible with height
2. Dots misaligned in inspector - fix bullet button layout
"""

def patch_css(path):
    src = open(path, encoding='utf-8').read()
    if "og-mobile-bullet-fix" in src:
        return []
    
    fix = """
/* og-mobile-bullet-fix: align dots with text in inspector */
.ontology-inspector-bullet{
 display:flex!important;align-items:center;gap:8px;
 text-align:left;width:100%;padding:10px 12px;
}
.ontology-inspector-bullet span:first-child{flex-shrink:0;}
/* Ensure graph canvas visible on mobile */
@media(max-width:640px){
 .ontology-graph-canvas{display:block!important;min-height:300px!important;height:40vh!important;}
 .graph-stage{display:block!important;}
 .og-stage-head{margin-bottom:8px;}
}
"""
    src = src.rstrip() + "\n" + fix
    open(path, 'w', encoding='utf-8').write(src)
    return ["mobile bullet + canvas fix applied"]

if __name__ == '__main__':
    import sys
    print(patch_css(sys.argv[1] if len(sys.argv) > 1 else 'styles.css'))
