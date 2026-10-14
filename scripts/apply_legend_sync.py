#!/usr/bin/env python3
"""
LEGEND SYNC: When hover dimming is active on the graph, the legend chips
below the graph also refresh to reflect the selected neighborhood.
- Chips for types IN the hovered neighborhood get .legend-highlight
- Chips for types NOT in the neighborhood get .legend-dim
- On mouseout, classes are cleared

Works for both Ontology Graph (.graph-stage > .graph-legend) and
Object 360 (#object360Body .ontology-legend).

Idempotent.
"""

def patch_js(path):
    src = open(path, encoding='utf-8').read()
    if "__legendSync" in src:
        return []

    # 1. Add helper functions before the __graphHoverDelegated IIFE
    old_iife = "// Graph UX: hover spotlight + double-click to expand.\n// Uses document-level delegation so it survives graph re-renders.\n(function(){\n if(window.__graphHoverDelegated)return;window.__graphHoverDelegated=true;"
    new_iife = """// __legendSync: mirror graph spotlight in the legend chips.
function __findLegendForCanvas(canvas){
 if(!canvas)return null;
 // Ontology Graph: legend is a sibling within .graph-stage
 let legend=canvas.closest?canvas.closest('.graph-stage'):null;
 if(legend){legend=legend.querySelector('.ontology-legend');if(legend)return legend;}
 // Object 360: legend lives in #object360Body
 if(canvas.closest&&canvas.closest('#o360MiniGraph')){
  legend=document.querySelector('#object360Body .ontology-legend');
  if(legend)return legend;
 }
 return null;
}
function __syncLegend(canvas,typeSet){
 const legend=__findLegendForCanvas(canvas);
 if(!legend)return;
 legend.querySelectorAll('.ontology-legend-item').forEach(item=>{
  const t=item.dataset.legendToggle||item.dataset.o360LegendToggle;
  const inHood=t&&typeSet.has(t);
  item.classList.toggle('legend-highlight',!!inHood);
  item.classList.toggle('legend-dim',!inHood);
 });
}
function __clearLegend(canvas){
 const legend=__findLegendForCanvas(canvas);
 if(!legend)return;
 legend.querySelectorAll('.legend-highlight,.legend-dim').forEach(el=>{
  el.classList.remove('legend-highlight');el.classList.remove('legend-dim');
 });
}
// Graph UX: hover spotlight + double-click to expand.
// Uses document-level delegation so it survives graph re-renders.
(function(){
 if(window.__graphHoverDelegated)return;window.__graphHoverDelegated=true;"""

    if old_iife not in src:
        return ["hover IIFE not found"]
    src = src.replace(old_iife, new_iife, 1)

    # 2. In the cluster-label hover branch, sync legend for the cluster type
    old_cluster = """   canvas.querySelectorAll('ellipse[data-cluster-type]').forEach(el=>{
    el.classList.toggle('graph-hover-on',el.dataset.clusterType===ctype);
   });
   return;"""
    new_cluster = """   canvas.querySelectorAll('ellipse[data-cluster-type]').forEach(el=>{
    el.classList.toggle('graph-hover-on',el.dataset.clusterType===ctype);
   });
   __syncLegend(canvas,new Set([ctype]));
   return;"""
    if old_cluster in src:
        src = src.replace(old_cluster, new_cluster, 1)

    # 3. In the pill hover branch, sync legend with neighborhood types
    old_pill = """  canvas.querySelectorAll('.og-pill').forEach(el=>{
   el.classList.toggle('graph-hover-on',connected.has(el.dataset.nodeKey));
  });"""
    new_pill = """  canvas.querySelectorAll('.og-pill').forEach(el=>{
   el.classList.toggle('graph-hover-on',connected.has(el.dataset.nodeKey));
  });
  // __legendSync: highlight legend types in the hovered neighborhood
  const hoodTypes=new Set();
  canvas.querySelectorAll('.og-pill').forEach(el=>{
   if(connected.has(el.dataset.nodeKey)&&el.dataset.ogType)hoodTypes.add(el.dataset.ogType);
  });
  __syncLegend(canvas,hoodTypes);"""
    if old_pill in src:
        src = src.replace(old_pill, new_pill, 1)

    # 4. In clearCanvas, also clear the legend
    old_clear = "const clearCanvas=(canvas)=>{canvas.classList.remove('graph-hover-dim');canvas.querySelectorAll('.graph-hover-on').forEach(el=>el.classList.remove('graph-hover-on'))};"
    new_clear = "const clearCanvas=(canvas)=>{canvas.classList.remove('graph-hover-dim');canvas.querySelectorAll('.graph-hover-on').forEach(el=>el.classList.remove('graph-hover-on'));__clearLegend(canvas);};"
    if old_clear in src:
        src = src.replace(old_clear, new_clear, 1)

    # 5. In the mouseout handler's inline clear (for cluster labels), also clear legend
    # The mouseout handler calls clearCanvas(canvas) which now clears legend too.
    # But the cluster-label branch has its own inline clear; ensure it also clears legend.
    old_inline = "document.querySelectorAll('.ontology-graph-canvas.graph-hover-dim').forEach(c=>{if(c!==canvas){c.classList.remove('graph-hover-dim');c.querySelectorAll('.graph-hover-on').forEach(el=>el.classList.remove('graph-hover-on'));}});"
    # This appears twice (cluster branch and pill branch). Patch both to also clear legend.
    new_inline = "document.querySelectorAll('.ontology-graph-canvas.graph-hover-dim').forEach(c=>{if(c!==canvas){c.classList.remove('graph-hover-dim');c.querySelectorAll('.graph-hover-on').forEach(el=>el.classList.remove('graph-hover-on'));__clearLegend(c);}});"
    src = src.replace(old_inline, new_inline)

    open(path, 'w', encoding='utf-8').write(src)
    return ["legend sync JS added"]

def patch_css(path):
    src = open(path, encoding='utf-8').read()
    if "legend-dim" in src and "legend-highlight" in src and "__legendSync" in src:
        return []
    # Check if our specific block exists
    if "/* __legendSync:" in src:
        return []

    css = """
/* __legendSync: legend chips mirror graph hover spotlight */
.ontology-legend-item{transition:opacity .2s ease,box-shadow .2s ease,border-color .2s ease;}
.ontology-legend-item.legend-dim{opacity:.3;}
.ontology-legend-item.legend-highlight{box-shadow:0 0 0 2px rgba(79,70,229,.35);border-color:#4f46e5;}
"""
    src = src.rstrip() + "\n" + css
    open(path, 'w', encoding='utf-8').write(src)
    return ["legend sync CSS added"]

if __name__ == '__main__':
    import sys
    js_path = sys.argv[1] if len(sys.argv) > 1 else 'app.js'
    css_path = sys.argv[2] if len(sys.argv) > 2 else 'styles.css'
    print("JS:", patch_js(js_path))
    print("CSS:", patch_css(css_path))
