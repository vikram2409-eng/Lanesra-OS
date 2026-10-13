#!/usr/bin/env python3
"""Graph UX overhaul: dropdown hover fix, depth default 1, Palantir-style interactions.
Applies to app.js and styles.css. Idempotent-ish: safe to run once per branch.
"""
import re, sys

def patch_app_js(path):
    src = open(path, encoding='utf-8').read()
    orig_len = len(src)
    changes = []
    depth_patches = [
        ("let dependencyExplorerState={type:'custom_object',id:null,depth:'2',",
         "let dependencyExplorerState={type:'custom_object',id:null,depth:'1',"),
        ("dependencyExplorerState={type:'custom_object',id:entityTypeKey,depth:'2',",
         "dependencyExplorerState={type:'custom_object',id:entityTypeKey,depth:'1',"),
        ("String(s.depth||'2')===d?' selected':''",
         "String(s.depth||'1')===d?' selected':''"),
        ("['depth',s.depth||'2'],", "['depth',s.depth||'1'],"),
        ("dependencyExplorerState={type,id,depth:'2',",
         "dependencyExplorerState={type,id,depth:'1',"),
        ("depth:dependencyExplorerState.depth||'2',",
         "depth:dependencyExplorerState.depth||'1',"),
        ("String(dependencyExplorerState.depth||'2')",
         "String(dependencyExplorerState.depth||'1')"),
        ("maxDepth:(depth>=1&&depth<=3)?depth:2,",
         "maxDepth:(depth>=1&&depth<=3)?depth:1,"),
    ]
    for old, new in depth_patches:
        c = src.count(old)
        if c:
            src = src.replace(old, new)
            changes.append(f"depth: replaced {c}x :: {old[:50]}")
    old_dot = '\'" \'+ui.inspectAttr+\'="\'+n.type+\':\'+n.id+\'" style="left:\''
    new_dot = '\'" \'+ui.inspectAttr+\'="\'+n.type+\':\'+n.id+\'" data-node-key="\'+n.key+\'" style="left:\''
    c = src.count(old_dot)
    assert c >= 1, "node dot inspect attr not found"
    src = src.replace(old_dot, new_dot)
    changes.append(f"graphRender: node dot data-node-key ({c}x)")
    old_cap2 = '\'"+(isDimmed?\' ontology-node-dimmed\':\'\')+\'" data-node-key="\''
    assert src.count(old_cap2) < 1, "caption already patched"
    old_cap = '\'"+(isDimmed?\' ontology-node-dimmed\':\'\')+\'" style="left:\''
    new_cap = '\'"+(isDimmed?\' ontology-node-dimmed\':\'\')+\'" data-node-key="\'+n.key+\'" style="left:\''
    c = src.count("ontology-node-caption")
    src = src.replace('nodeDivs.push(\'<div class="ontology-node-caption\'', 'nodeDivs.push(\'<div class="ontology-node-caption\'')
    # targeted: only the caption push with isDimmed
    target_old = '\'<div class="ontology-node-caption\'+(isDimmed?\' ontology-node-dimmed\':\'\')+\'" style="left:\''
    target_new = '\'<div class="ontology-node-caption\'+(isDimmed?\' ontology-node-dimmed\':\'\')+\'" data-node-key="\'+n.key+\'" style="left:\''
    assert src.count(target_old) >= 1, "caption target not found"
    src = src.replace(target_old, target_new)
    changes.append("graphRender: caption data-node-key")
    old_line = 'edgeSvg.push(\'<line x1="\''
    new_line = 'edgeSvg.push(\'<line data-edge-from="\'+e.fromKey+\'" data-edge-to="\'+e.toKey+\'" x1="\''
    assert src.count(old_line) >= 1, "edge line not found"
    src = src.replace(old_line, new_line)
    changes.append("graphRender: edge line data attrs")
    old_el = '(eDimmed?\' ontology-node-dimmed\':\'\')+\'" style="left:\'+((a.x+b.x)/2)'
    new_el = '(eDimmed?\' ontology-node-dimmed\':\'\')+\'" data-edge-from="\'+e.fromKey+\'" data-edge-to="\'+e.toKey+\'" style="left:\'+((a.x+b.x)/2)'
    assert src.count(old_el) >= 1, "edge label not found"
    src = src.replace(old_el, new_el)
    changes.append("graphRender: edge label data attrs")
    marker = "// Thin wrapper: the Ontology Graph's ui over the shared renderer."
    assert marker in src, "marker not found"
    hover_fn = """
// Graph UX: Palantir-style hover highlighting + double-click to expand.
function wireGraphHoverFx(host, onExpand){
 const canvas=host?host.querySelector('.ontology-graph-canvas'):null;
 if(!canvas||canvas.dataset.hoverWired)return;canvas.dataset.hoverWired='1';
 const clear=()=>{canvas.classList.remove('graph-hover-dim');canvas.querySelectorAll('.graph-hover-on').forEach(el=>el.classList.remove('graph-hover-on'))};
 canvas.addEventListener('mouseover',e=>{
  const dot=e.target.closest('.ontology-node-dot');if(!dot||!dot.dataset.nodeKey)return;
  const key=dot.dataset.nodeKey;
  const connected=new Set([key]);
  canvas.querySelectorAll('line[data-edge-from]').forEach(l=>{
   if(l.dataset.edgeFrom===key)connected.add(l.dataset.edgeTo);
   else if(l.dataset.edgeTo===key)connected.add(l.dataset.edgeFrom);
  });
  canvas.classList.add('graph-hover-dim');
  canvas.querySelectorAll('.ontology-node-dot,.ontology-node-caption').forEach(el=>{
   el.classList.toggle('graph-hover-on',connected.has(el.dataset.nodeKey));
  });
  canvas.querySelectorAll('line[data-edge-from],.ontology-edge-label[data-edge-from]').forEach(el=>{
   const f=el.dataset.edgeFrom,t=el.dataset.edgeTo;
   el.classList.toggle('graph-hover-on',f===key||t===key);
  });
 });
 canvas.addEventListener('mouseleave',clear);
 canvas.addEventListener('mouseout',e=>{if(!e.relatedTarget||!canvas.contains(e.relatedTarget))clear()});
 if(onExpand){
  canvas.addEventListener('dblclick',e=>{
   const dot=e.target.closest('.ontology-node-dot');if(!dot||!dot.dataset.nodeKey)return;
   e.preventDefault();e.stopPropagation();onExpand(dot.dataset.nodeKey);
  });
 }
}
"""
    src = src.replace(marker, hover_fn + marker)
    changes.append("added wireGraphHoverFx")
    old_o360 = "object360State={...object360State,collapsed:c};\n  renderObject360Body();\n });"
    assert src.count(old_o360) >= 1, "o360 wiring not found"
    src = src.replace(old_o360, old_o360 + """
 // Graph UX: hover spotlight + double-click to expand one hop.
 wireGraphHoverFx(body,(nodeKey)=>{
  const ex={...(object360State.expanded||{})};ex[nodeKey]=true;
  object360State={...object360State,expanded:ex};renderObject360Body();
 });""", 1)
    changes.append("wired hover/dblclick in Object 360")
    old_dep = "dependencyExplorerState={...dependencyExplorerState,inspectType:type,inspectId:id};renderAdminTab()});"
    assert src.count(old_dep) >= 1, "dep wiring not found"
    src = src.replace(old_dep, old_dep + """
 // Graph UX: hover spotlight + double-click to expand one hop.
 wireGraphHoverFx(body,(nodeKey)=>{
  const ex={...(dependencyExplorerState.expanded||{})};ex[nodeKey]=true;
  dependencyExplorerState={...dependencyExplorerState,expanded:ex};renderAdminTab();
 });""", 1)
    changes.append("wired hover/dblclick in Ontology Graph")
    open(path, 'w', encoding='utf-8').write(src)
    print(f"app.js: {len(changes)} change groups")
    for c in changes: print("  -", c)

def patch_styles_css(path):
    src = open(path, encoding='utf-8').read()
    orig_len = len(src)
    additions = """
/* ===== Graph UX: dropdown hover bridge (website nav) ===== */
@media(min-width:901px){.nav-dropdown::before{content:'';position:absolute;top:-8px;left:0;right:0;height:8px}}

/* ===== Graph UX: Palantir-style interactions ===== */
.ontology-node-dot{transition:transform .18s ease,box-shadow .18s ease,opacity .2s ease;cursor:pointer}
.ontology-node-dot:hover{transform:scale(1.32);z-index:6}
.ontology-node-caption{transition:opacity .2s ease}
.ontology-edge-label{transition:opacity .2s ease}
.ontology-graph-canvas.graph-hover-dim .ontology-node-dot:not(.graph-hover-on){opacity:.14}
.ontology-graph-canvas.graph-hover-dim .ontology-node-caption:not(.graph-hover-on){opacity:.1}
.ontology-graph-canvas.graph-hover-dim .ontology-edge-label:not(.graph-hover-on){opacity:.05}
.ontology-graph-canvas.graph-hover-dim svg line:not(.graph-hover-on){stroke-opacity:.06!important}
.ontology-node-dot.graph-hover-on{box-shadow:0 0 0 5px color-mix(in srgb,currentColor 22%,transparent),0 4px 12px rgba(0,0,0,.28)}
.ontology-node-caption.graph-hover-on{font-weight:700}
"""
    import re as _re
    clean = _re.sub(r'/\*.*?\*/', '', src, flags=_re.S)
    o, c = clean.count('{'), clean.count('}')
    assert o == c, f"styles.css braces unbalanced before patch: {o} vs {c}"
    if '.nav-dropdown::before' not in src:
        src = src.rstrip('\n') + '\n' + additions
    open(path, 'w', encoding='utf-8').write(src)
    print(f"styles.css: {orig_len} -> {len(src)} bytes")

if __name__ == '__main__':
    patch_app_js('app.js')
    patch_styles_css('styles.css')
    print("done")
