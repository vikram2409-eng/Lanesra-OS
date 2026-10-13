#!/usr/bin/env python3
"""
Compact toolbar: squeeze all controls into 1-2 slim rows on desktop.
Gives more vertical real estate to the graph.
"""

def patch_css(path):
    src = open(path, encoding='utf-8').read()
    if "og-compact-toolbar" in src:
        return []
    
    css = """
/* og-compact-toolbar: slim 1-2 row toolbar on desktop */
@media(min-width:641px){
 .og-toolbar-card{padding:8px 12px!important;gap:6px!important;margin-bottom:10px!important;}
 .og-toolbar-card .tool-group{
  padding-bottom:6px!important;margin-bottom:6px!important;gap:8px!important;
 }
 .og-toolbar-card .tool-group:last-child{padding-bottom:0!important;margin-bottom:0!important;}
 .og-toolbar-card select,.og-toolbar-card input[type="search"],
 .og-toolbar-card input[type="text"]{height:30px!important;font-size:12px!important;padding:4px 8px!important;}
 .og-toolbar-card button,.og-toolbar-card .btn{height:30px!important;font-size:12px!important;padding:4px 10px!important;}
 .og-toolbar-card label{font-size:12px!important;}
 /* Try to fit groups side-by-side when space allows */
 .og-toolbar-card{display:flex!important;flex-direction:row!important;flex-wrap:wrap!important;
  align-items:center!important;gap:8px 16px!important;}
 .og-toolbar-card .tool-group{border-bottom:none!important;border-right:1px solid #f3f4f6!important;
  padding-right:16px!important;padding-bottom:0!important;margin-bottom:0!important;}
 .og-toolbar-card .tool-group:last-child{border-right:none!important;padding-right:0!important;}
}
"""
    src = src.rstrip() + "\n" + css
    open(path, 'w', encoding='utf-8').write(src)
    return ["compact toolbar CSS applied"]

if __name__ == '__main__':
    import sys
    print(patch_css(sys.argv[1] if len(sys.argv) > 1 else 'styles.css'))
