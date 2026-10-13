#!/usr/bin/env python3
"""
Definitive toolbar fix: ensure ALL controls are in the card in clean rows.
"""

def patch_css(path):
    src = open(path, encoding='utf-8').read()
    if "og-toolbar-definitive" in src:
        return []
    
    css = """
/* og-toolbar-definitive: all controls in card, clean rows */
/* Desktop: horizontal rows */
@media(min-width:641px){
 .og-toolbar-card{
  display:flex!important;flex-direction:row!important;flex-wrap:wrap!important;
  align-items:center!important;gap:8px 20px!important;
  padding:10px 16px!important;
 }
 .og-toolbar-card .tool-group{
  display:flex!important;align-items:center!important;gap:8px!important;
  border:none!important;padding:0!important;margin:0!important;
  border-right:1px solid #e5e7eb!important;padding-right:20px!important;
 }
 .og-toolbar-card .tool-group:last-child{border-right:none!important;padding-right:0!important;}
}
/* Mobile: vertical stack, all inside card */
@media(max-width:640px){
 .og-toolbar-card{display:block!important;padding:12px!important;}
 .og-toolbar-card .tool-group{
  display:flex!important;align-items:center!important;gap:8px!important;flex-wrap:wrap!important;
  padding:8px 0!important;margin:0!important;
  border-bottom:1px solid #f3f4f6!important;
 }
 .og-toolbar-card .tool-group:last-child{border-bottom:none!important;}
}
"""
    src = src.rstrip() + "\n" + css
    open(path, 'w', encoding='utf-8').write(src)
    return ["definitive toolbar CSS applied"]

if __name__ == '__main__':
    import sys
    print(patch_css(sys.argv[1] if len(sys.argv) > 1 else 'styles.css'))
