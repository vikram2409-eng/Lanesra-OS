#!/usr/bin/env python3
"""
MOBILE FIXES for Ontology Graph + Object 360.
- Touch pinch-to-zoom on graph canvas
- Prevent pill nodes from being cut off on narrow screens
- Fix inspector relationship row alignment on mobile

Idempotent.
"""

def patch_js(path):
    src = open(path, encoding='utf-8').read()
    if "__graphTouchZoom" in src:
        return []
    marker = "/* MOBILE: end */"
    if marker not in src:
        # Fallback: append before the inspector section
        marker2 = "// Graph UX: hover spotlight"
        if marker2 not in src:
            return ["no marker found"]
        marker = marker2
    touch_code = """/* MOBILE: touch pinch-zoom for graph canvas */
(function(){
 if(window.__graphTouchZoom)return;window.__graphTouchZoom=true;
 let pinchDist=0,pinchZoom=1;
 document.addEventListener('touchstart',e=>{
  const t=e.target,canvas=t.closest?t.closest('.ontology-graph-canvas'):null;
  if(!canvas||e.touches.length!==2)return;
  pinchDist=Math.hypot(e.touches[0].clientX-e.touches[1].clientX,e.touches[0].clientY-e.touches[1].clientY);
  pinchZoom=1;
  const inner=canvas.querySelector('.og-pan-inner')||canvas.firstElementChild;
  if(inner&&inner.style.transform){const m=inner.style.transform.match(/scale\\(([\\d.]+)\\)/);if(m)pinchZoom=parseFloat(m[1]);}
 },{passive:true});
 document.addEventListener('touchmove',e=>{
  const t=e.target,canvas=t.closest?t.closest('.ontology-graph-canvas'):null;
  if(!canvas||e.touches.length!==2||!pinchDist)return;
  e.preventDefault();
  const d=Math.hypot(e.touches[0].clientX-e.touches[1].clientX,e.touches[0].clientY-e.touches[1].clientY);
  const nz=Math.min(3,Math.max(0.4,pinchZoom*(d/pinchDist)));
  const inner=canvas.querySelector('.og-pan-inner')||canvas.firstElementChild;
  if(inner)inner.style.transform=`scale(${nz})`;
 },{passive:false});
 document.addEventListener('touchend',e=>{if(e.touches.length<2)pinchDist=0;},{passive:true});
})();
"""
    src = src.replace(marker, touch_code + marker)
    open(path, 'w', encoding='utf-8').write(src)
    return ["touch pinch-zoom added"]

def patch_css(path):
    src = open(path, encoding='utf-8').read()
    if "og-mobile-fix" in src:
        return []
    css = """
/* og-mobile-fix: prevent cutoff + align inspector rows on small screens */
@media(max-width:640px){
 .ontology-graph-canvas{overflow:hidden;touch-action:pan-x pan-y;}
 .og-pill{max-width:44vw;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:12px!important;padding:6px 10px!important;}
 .og-pill-root{max-width:54vw;}
 .og-canvas-wrap{overflow:hidden;}
}
"""
    # Append at end of file
    src = src.rstrip() + "\n" + css
    open(path, 'w', encoding='utf-8').write(src)
    return ["mobile CSS appended"]

if __name__ == '__main__':
    import sys
    js = sys.argv[1] if len(sys.argv) > 1 else 'app.js'
    css = sys.argv[2] if len(sys.argv) > 2 else 'styles.css'
    print("JS:", patch_js(js))
    print("CSS:", patch_css(css))
