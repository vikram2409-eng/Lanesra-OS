#!/usr/bin/env python3
"""
Fix mobile graph: auto-fit on narrow screens.
The graph uses fixed coordinates; on mobile the initial view is wrong.
This triggers a fit after render when the viewport is narrow.
"""

def patch_js(path):
    src = open(path, encoding='utf-8').read()
    if "__mobileAutoFit" in src:
        return []
    
    # Find depWirePanZoom call and add auto-fit after it
    old = "depWirePanZoom();"
    new = """depWirePanZoom();
 // __mobileAutoFit: on narrow screens, auto-fit graph after render
 try{
  const vp=document.getElementById('depGraphViewport');
  if(vp&&vp.clientWidth>0&&vp.clientWidth<500&&!window.__mobileFitDone){
   window.__mobileFitDone=true;
   setTimeout(()=>{
    const fit=document.getElementById('depFit');
    if(fit)fit.click();
    window.__mobileFitDone=false;
   },300);
  }
 }catch(e){}"""
    
    if src.count(old) >= 1:
        # Only patch the first occurrence (in depWireToolbar)
        src = src.replace(old, new, 1)
        open(path, 'w', encoding='utf-8').write(src)
        return ["mobile auto-fit added"]
    return ["depWirePanZoom not found"]

def patch_css(path):
    src = open(path, encoding='utf-8').read()
    if "og-bullet-order-fix" in src:
        return []
    
    fix = """
/* og-bullet-order-fix: dot + label left, arrow right */
.ontology-inspector-bullet{display:flex!important;align-items:center;gap:8px;justify-content:flex-start!important;}
.ontology-inspector-bullet > *:first-child{order:3;margin-left:auto;}
"""
    src = src.rstrip() + "\n" + fix
    open(path, 'w', encoding='utf-8').write(src)
    return ["bullet order CSS applied"]

if __name__ == '__main__':
    import sys
    js = sys.argv[1] if len(sys.argv) > 1 else 'app.js'
    css = sys.argv[2] if len(sys.argv) > 2 else 'styles.css'
    print("JS:", patch_js(js))
    print("CSS:", patch_css(css))
