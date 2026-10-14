#!/usr/bin/env python3
"""
CLUSTER BOUNDARIES: dashed-ellipse cluster boundaries around groups of similar nodes.
Matches the Modern UI Concept design: "Contacts · 1", "AI Agents · 2", etc.
with colored dashed borders.

Changes:
1. graphRender(): replaces convex-hull <path> halos with dashed <ellipse> elements
   - Border: type color at 60% opacity, dasharray "8 6"
   - Label: type-colored "Type · N" at top of ellipse
2. Collapsed clusters: dashed circle now uses type color (was gray)
3. Hover: hovering a cluster label highlights all nodes of that type
4. CSS: cluster label + ellipse hover styles

Idempotent: checks for __clusterBoundaries marker before applying.
"""

import re
import sys


def patch_app_js(path):
    src = open(path, encoding='utf-8').read()
    orig = src
    changes = []

    # =====================================================================
    # 0. Add data-og-type to pill divs for robust cluster hover matching
    # =====================================================================
    old_pill = """nodeDivs.push('<div class="'+pillCls+'" '+ui.inspectAttr+'="'+n.type+':'+n.id+'" data-node-key="'+n.key+'" style="'+pillStyle+'" title="'+(ui.typeLabels[n.type]||n.type)+': '+n.label+'" role="button" tabindex="0">'+pillInner+'</div>');"""
    new_pill = """nodeDivs.push('<div class="'+pillCls+'" '+ui.inspectAttr+'="'+n.type+':'+n.id+'" data-node-key="'+n.key+'" data-og-type="'+n.type+'" style="'+pillStyle+'" title="'+(ui.typeLabels[n.type]||n.type)+': '+n.label+'" role="button" tabindex="0">'+pillInner+'</div>');"""

    if old_pill in src:
        # Check not already applied by looking for the new attribute pattern
        if 'data-og-type="\'+n.type' not in src:
            src = src.replace(old_pill, new_pill)
            changes.append("data-og-type added to pills")

    # =====================================================================
    # 1. Replace convex-hull halo <path> with dashed <ellipse>
    # =====================================================================
    old_halo = """  let d;
  if(pts.length===1){const p=pts[0];d='M '+(p.x-36).toFixed(1)+' '+p.y.toFixed(1)+' a 36 36 0 1 0 72 0 a 36 36 0 1 0 -72 0'}
  else{
   const hull=depConvexHull(pts);
   const cx=hull.reduce((a,p)=>a+p.x,0)/hull.length,cy=hull.reduce((a,p)=>a+p.y,0)/hull.length;
   const padded=hull.map(p=>{const dx=p.x-cx,dy=p.y-cy,dd=Math.sqrt(dx*dx+dy*dy)||1;return{x:p.x+dx/dd*28,y:p.y+dy/dd*28}});
   d='M '+padded.map(p=>p.x.toFixed(1)+' '+p.y.toFixed(1)).join(' L ')+' Z';
  }
  haloSvg.push('<path d="'+d+'" fill="'+color+'" fill-opacity="0.10" stroke="#cbd5e1" stroke-opacity="0.35" stroke-width="1.5"/>');
  const lx=(Math.min.apply(null,pts.map(p=>p.x))+Math.max.apply(null,pts.map(p=>p.x)))/2;
  const ly=Math.min.apply(null,pts.map(p=>p.y))-16;
  haloLabels.push('<button type="button" class="ontology-halo-label" '+ui.collapseAttr+'="'+type+'" style="left:'+lx.toFixed(1)+'px;top:'+ly.toFixed(1)+'px" title="Collapse '+label+' into a single node">'+label+' ▾</button>');"""

    new_halo = """  // __clusterBoundaries: dashed ellipse around same-type node groups
  const xs=pts.map(p=>p.x),ys=pts.map(p=>p.y);
  const minX=Math.min.apply(null,xs),maxX=Math.max.apply(null,xs);
  const minY=Math.min.apply(null,ys),maxY=Math.max.apply(null,ys);
  const padX=58,padY=44;
  const ecx=((minX+maxX)/2).toFixed(1),ecy=((minY+maxY)/2).toFixed(1);
  const erx=Math.max(70,((maxX-minX)/2+padX)).toFixed(1);
  const ery=Math.max(48,((maxY-minY)/2+padY)).toFixed(1);
  haloSvg.push('<ellipse data-cluster-type="'+type+'" cx="'+ecx+'" cy="'+ecy+'" rx="'+erx+'" ry="'+ery+'" fill="'+color+'" fill-opacity="0.04" stroke="'+color+'" stroke-opacity="0.6" stroke-width="1.5" stroke-dasharray="8 6"/>');
  const lx=(minX+maxX)/2;
  const ly=minY-14;
  haloLabels.push('<button type="button" class="ontology-halo-label og-cluster-label" data-cluster-type="'+type+'" '+ui.collapseAttr+'="'+type+'" style="left:'+lx.toFixed(1)+'px;top:'+ly.toFixed(1)+'px;color:'+color+'" title="Collapse '+label+' into a single node">'+label+' ▾</button>');"""

    if "__clusterBoundaries" not in src and old_halo in src:
        src = src.replace(old_halo, new_halo)
        changes.append("halo path -> dashed ellipse")

    # =====================================================================
    # 2. Collapsed clusters: use type color for dashed circle (was gray)
    # =====================================================================
    old_collapsed = """  haloSvg.push('<circle cx="'+p.x.toFixed(1)+'" cy="'+p.y.toFixed(1)+'" r="36" fill="'+color+'" fill-opacity="0.10" stroke="#cbd5e1" stroke-opacity="0.45" stroke-width="1.5" stroke-dasharray="5 4"/>');"""
    new_collapsed = """  haloSvg.push('<circle data-cluster-type="'+n.type+'" cx="'+p.x.toFixed(1)+'" cy="'+p.y.toFixed(1)+'" r="36" fill="'+color+'" fill-opacity="0.06" stroke="'+color+'" stroke-opacity="0.6" stroke-width="1.5" stroke-dasharray="8 6"/>');"""

    if old_collapsed in src:
        # Check specifically for the old gray collapsed circle
        if 'stroke="#cbd5e1" stroke-opacity="0.45"' in src:
            src = src.replace(old_collapsed, new_collapsed)
            changes.append("collapsed cluster circle uses type color")

    # =====================================================================
    # 3. Hover: hovering a cluster label highlights all nodes of that type
    # =====================================================================
    old_hover = """ document.addEventListener('mouseover',e=>{
  const dot=e.target.closest?e.target.closest('.og-pill'):null;
  const canvas=dot?dot.closest('.ontology-graph-canvas'):null;
  if(!dot||!canvas||!dot.dataset.nodeKey)return;"""

    new_hover = """ document.addEventListener('mouseover',e=>{
  // __clusterHover: hovering a cluster label highlights its type group
  const lbl=e.target.closest?e.target.closest('.og-cluster-label'):null;
  if(lbl){
   const canvas=lbl.closest('.ontology-graph-canvas');
   if(!canvas||!lbl.dataset.clusterType)return;
   const ctype=lbl.dataset.clusterType;
   document.querySelectorAll('.ontology-graph-canvas.graph-hover-dim').forEach(c=>{if(c!==canvas){c.classList.remove('graph-hover-dim');c.querySelectorAll('.graph-hover-on').forEach(el=>el.classList.remove('graph-hover-on'));}});
   canvas.classList.add('graph-hover-dim');
   canvas.querySelectorAll('.og-pill').forEach(el=>{
    el.classList.toggle('graph-hover-on',el.dataset.ogType===ctype);
   });
   canvas.querySelectorAll('ellipse[data-cluster-type]').forEach(el=>{
    el.classList.toggle('graph-hover-on',el.dataset.clusterType===ctype);
   });
   return;
  }
  const dot=e.target.closest?e.target.closest('.og-pill'):null;
  const canvas=dot?dot.closest('.ontology-graph-canvas'):null;
  if(!dot||!canvas||!dot.dataset.nodeKey)return;"""

    # Use a separate marker for hover to avoid conflict with halo marker
    if "__clusterHover" not in src and old_hover in src:
        src = src.replace(old_hover, new_hover)
        changes.append("cluster label hover highlighting")

    # Also make cluster ellipses participate in dimming (dim when not hovered)
    old_dim = """  canvas.querySelectorAll('.og-pill').forEach(el=>{
   el.classList.toggle('graph-hover-on',connected.has(el.dataset.nodeKey));
  });"""
    new_dim = """  canvas.querySelectorAll('.og-pill').forEach(el=>{
   el.classList.toggle('graph-hover-on',connected.has(el.dataset.nodeKey));
  });
  canvas.querySelectorAll('ellipse[data-cluster-type]').forEach(el=>{
   el.style.opacity=connected.size>1?'':'0.25';
  });"""

    if old_dim in src and 'ellipse[data-cluster-type]' not in src.split(old_dim)[0][-500:]:
        # Only apply if not already present after this block
        if src.count("el.style.opacity=connected.size>1") == 0:
            src = src.replace(old_dim, new_dim)
            changes.append("cluster ellipses dim with hover")

    if src != orig:
        open(path, 'w', encoding='utf-8').write(src)
    return changes


def patch_css(path):
    src = open(path, encoding='utf-8').read()
    if "__clusterBoundaries" in src:
        return []
    css = """
/* __clusterBoundaries: dashed cluster ellipse styles */
.og-cluster-label{
 border:1px solid color-mix(in srgb, currentColor 35%, transparent)!important;
 background:color-mix(in srgb, currentColor 8%, #fff)!important;
 font-size:10px!important;font-weight:700!important;letter-spacing:.02em;
}
.og-cluster-label:hover{border-color:currentColor!important;transform:translate(-50%,-50%) scale(1.05);}
.ontology-graph-canvas ellipse[data-cluster-type]{transition:opacity .2s ease;}
"""
    src = src.rstrip() + "\n" + css
    open(path, 'w', encoding='utf-8').write(src)
    return ["cluster CSS added"]


if __name__ == '__main__':
    js_path = sys.argv[1] if len(sys.argv) > 1 else 'app.js'
    css_path = sys.argv[2] if len(sys.argv) > 2 else 'styles.css'
    print("JS:", patch_app_js(js_path))
    print("CSS:", patch_css(css_path))
