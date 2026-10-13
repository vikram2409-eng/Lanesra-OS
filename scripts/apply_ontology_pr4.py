#!/usr/bin/env python3
"""Apply PR 4 (Object 360 polish) edits to app.js.

Anchored replacements, each asserted to match exactly once.
"""
PATH = 'app.js'
src = open(PATH).read()
EDITS = [
    ('scatter',
     'pos.set(n.key,{x:a.x+(rng()-0.5)*170,y:a.y+(rng()-0.5)*170});',
     'pos.set(n.key,{x:a.x+(rng()-0.5)*90,y:a.y+(rng()-0.5)*90});'),
    ('cluster_gravity',
     '   d.x+=(a.x-p.x)*0.03+(W/2-p.x)*0.006;\n   d.y+=(a.y-p.y)*0.03+(H/2-p.y)*0.006;\n  });',
     '   d.x+=(a.x-p.x)*0.14+(W/2-p.x)*0.006;\n   d.y+=(a.y-p.y)*0.14+(H/2-p.y)*0.006;\n  });\n  // PR4: intra-cluster cohesion - same-type nodes also pull toward their\n  // type centroid each tick, so every cluster reads as one tight clump.\n  const pr4Centroids=new Map();\n  visible.forEach(n=>{if(n.key===rootKey)return;const c=pr4Centroids.get(n.type)||{x:0,y:0,k:0};const pp=pos.get(n.key);c.x+=pp.x;c.y+=pp.y;c.k++;pr4Centroids.set(n.type,c)});\n  pr4Centroids.forEach(c=>{c.x/=c.k;c.y/=c.k});\n  visible.forEach(n=>{\n   if(n.key===rootKey)return;\n   const c=pr4Centroids.get(n.type);if(!c||c.k<2)return;\n   const pp=pos.get(n.key),dd=disp.get(n.key);\n   dd.x+=(c.x-pp.x)*0.06;dd.y+=(c.y-pp.y)*0.06;\n  });'),
    ('root_edge_guarantee',
     'const model={nodes:Array.from(nodes.values()),edges,truncated};',
     " // PR4: Object 360 completeness - every depth-1 member must show a visible\n // edge to the central record. The drivers already emit one per member;\n // this is belt-and-braces so a cluster can never silently lose a member.\n if(driver.ensureRootEdges){\n  const pr4rk=driver.nodeKey(driver.rootType,driver.rootId);\n  const pr4touch=new Set();\n  edges.forEach(e=>{if(e.fromKey===pr4rk)pr4touch.add(e.toKey);if(e.toKey===pr4rk)pr4touch.add(e.fromKey)});\n  nodes.forEach(n=>{if(n.depth===1&&n.key!==pr4rk&&!pr4touch.has(n.key)){addEdge(driver.rootType,driver.rootId,n.type,n.id,'linked_record');pr4touch.add(n.key)}});\n }\n const model={nodes:Array.from(nodes.values()),edges,truncated};"),
    ('driver_flag',
     "  typeLabel:(t)=>t==='business_rule'?'Business Rule':t==='workflow'?'Workflow':t==='ai_agent'?'AI Agent':t==='orgUnit'?'Business Unit':(entityLabel(t)||t)\n };",
     "  typeLabel:(t)=>t==='business_rule'?'Business Rule':t==='workflow'?'Workflow':t==='ai_agent'?'AI Agent':t==='orgUnit'?'Business Unit':(entityLabel(t)||t),\n  ensureRootEdges:true\n };"),
    ('dep_fields_html',
     '>Copy link</button></div>${depPathBarHtml()}',
     '>Copy link</button><label class="ontology-fields-toggle" title="Field nodes can clutter the graph - hide them here"><input type="checkbox" id="depFieldsToggle"${depExplorerHiddenTypes.has(\'custom_field\')?\'\':\' checked\'}> Fields</label></div>${depPathBarHtml()}'),
    ('dep_fields_wire',
     "if(btn360)btn360.onclick=()=>{if(typeof openObject360ForEntity==='function')openObject360ForEntity(dependencyExplorerState.inspectId);else toast('Object 360 is not available here')};",
     "if(btn360)btn360.onclick=()=>{if(typeof openObject360ForEntity==='function')openObject360ForEntity(dependencyExplorerState.inspectId);else toast('Object 360 is not available here')};\n const depFieldsT=document.getElementById('depFieldsToggle');\n if(depFieldsT)depFieldsT.onchange=(e)=>{if(e.target.checked)depExplorerHiddenTypes.delete('custom_field');else depExplorerHiddenTypes.add('custom_field');renderAdminTab()};"),
    ('o360_fields_html',
     '</select></label></div>\n <div class="ontology-graph-wrap" style="height:${H+40}px">',
     '</select></label><label class="ontology-fields-toggle" title="Field nodes can clutter the graph - hide them here"><input type="checkbox" id="o360FieldsToggle"${object360HiddenTypes.has(\'custom_field\')?\'\':\' checked\'}> Fields</label></div>\n <div class="ontology-graph-wrap" style="height:${H+40}px">'),
    ('o360_fields_wire',
     "const depthSel=$('#o360Depth'); if(depthSel)depthSel.onchange=(e)=>{object360State={...object360State,depth:e.target.value};renderObject360Body()};",
     "const depthSel=$('#o360Depth'); if(depthSel)depthSel.onchange=(e)=>{object360State={...object360State,depth:e.target.value};renderObject360Body()};\n const o360FieldsT=$('#o360FieldsToggle'); if(o360FieldsT)o360FieldsT.onchange=(e)=>{if(e.target.checked)object360HiddenTypes.delete('custom_field');else object360HiddenTypes.add('custom_field');renderObject360Body()};"),
    ('modal_chrome',
     "if(modalEl)modalEl.style.cssText+=';width:min(1100px,95vw);max-height:92vh';",
     "if(modalEl)modalEl.style.cssText+=';width:min(1100px,95vw);max-height:92vh';\n o360WireModalChrome();"),
    ('modal_fns',
     'let object360CardPos=null;',
     "let object360CardPos=null;\n// PR4: Object 360 modal flexibility - maximize/restore + drag-resize state,\n// module-level like the card position so it survives re-renders.\nlet o360ModalMax=false;\nlet o360ModalSize=null;\n// PR4: wire the Object 360 modal chrome - an expand/reduce button in the\n// header plus a drag-resize grip pinned to the modal's bottom-right corner.\nfunction o360WireModalChrome(){\n if(typeof document==='undefined')return;\n const modalEl=document.querySelector('#modal .modal');\n const head=document.querySelector('#modal .modal-head');\n if(!modalEl||!head)return;\n modalEl.style.position='relative';\n if(!head.querySelector('#o360MaxBtn')){\n  const btn=document.createElement('button');\n  btn.type='button';btn.id='o360MaxBtn';btn.className='icon-btn';btn.textContent='\\u26F6';btn.title='Expand';\n  btn.style.marginRight='8px';\n  head.insertBefore(btn,head.lastElementChild);\n  btn.onclick=()=>{o360ModalMax=!o360ModalMax;o360ApplyModalSize()};\n }\n if(!document.querySelector('#o360ResizeGrip')){\n  const grip=document.createElement('div');\n  grip.id='o360ResizeGrip';grip.className='o360-resize-handle';grip.title='Drag to resize';\n  grip.style.position='fixed';grip.style.zIndex='101';\n  document.querySelector('#modal').appendChild(grip);\n  const place=()=>{const g=document.querySelector('#o360ResizeGrip');const m=document.querySelector('#modal .modal');if(!g||!m)return;const r=m.getBoundingClientRect();g.style.left=(r.right-36)+'px';g.style.top=(r.bottom-36)+'px'};\n  place();\n  modalEl.addEventListener('scroll',place,{passive:true});\n  window.addEventListener('resize',place);\n  const obs=new MutationObserver(place);\n  obs.observe(modalEl,{attributes:true,attributeFilter:['style']});\n  grip.onpointerdown=(e)=>{\n   e.preventDefault();\n   try{grip.setPointerCapture(e.pointerId)}catch(_){}\n   const r=modalEl.getBoundingClientRect(),sx=e.clientX,sy=e.clientY,sw=r.width,sh=r.height;\n   const move=(ev)=>{\n    o360ModalSize={w:Math.round(Math.max(480,Math.min(window.innerWidth-40,sw+ev.clientX-sx))),h:Math.round(Math.max(420,Math.min(window.innerHeight-40,sh+ev.clientY-sy)))};\n    o360ModalMax=false;o360ApplyModalSize();\n   };\n   const up=()=>{grip.removeEventListener('pointermove',move);grip.removeEventListener('pointerup',up);grip.removeEventListener('pointercancel',up)};\n   grip.addEventListener('pointermove',move);grip.addEventListener('pointerup',up);grip.addEventListener('pointercancel',up);\n  };\n }\n o360ApplyModalSize();\n}\nfunction o360ApplyModalSize(){\n if(typeof document==='undefined')return;\n const modalEl=document.querySelector('#modal .modal');if(!modalEl)return;\n const maxBtn=document.querySelector('#o360MaxBtn');\n if(o360ModalMax){\n  modalEl.style.width='calc(100vw - 40px)';modalEl.style.height='calc(100vh - 40px)';\n  modalEl.style.maxWidth='none';modalEl.style.maxHeight='none';\n  if(maxBtn){maxBtn.textContent='\\u26F7';maxBtn.title='Reduce'}\n }else if(o360ModalSize){\n  modalEl.style.width=o360ModalSize.w+'px';modalEl.style.height=o360ModalSize.h+'px';\n  modalEl.style.maxWidth='none';modalEl.style.maxHeight='none';\n  if(maxBtn){maxBtn.textContent='\\u26F6';maxBtn.title='Expand'}\n }else{\n  modalEl.style.width='min(1100px,95vw)';modalEl.style.height='';modalEl.style.maxWidth='';modalEl.style.maxHeight='92vh';\n  if(maxBtn){maxBtn.textContent='\\u26F6';maxBtn.title='Expand'}\n }\n const grip=document.querySelector('#o360ResizeGrip');\n if(grip){const r=modalEl.getBoundingClientRect();grip.style.left=(r.right-36)+'px';grip.style.top=(r.bottom-36)+'px'}\n}\n// PR4: dock the Object 360 inspector card left/right as an alternative to\n// dragging it - the card is no longer stuck at a fixed position.\nfunction o360WireCardDock(){\n const head=document.querySelector('#o360InspectorHead'),card=document.querySelector('#o360InspectorCard');\n if(!head||!card||head.querySelector('.o360-dock-btn'))return;\n const mk=(label,title,side)=>{\n  const b=document.createElement('button');\n  b.type='button';b.className='icon-btn o360-dock-btn';b.textContent=label;b.title=title;\n  b.style.cssText+=';padding:4px 7px;font-size:11px;margin-left:6px;flex-shrink:0';\n  b.onclick=(e)=>{e.stopPropagation();object360CardPos={dock:side};renderObject360Body()};\n  return b;\n };\n head.appendChild(mk('\\u21E4','Dock panel to the left','left'));\n head.appendChild(mk('\\u21E5','Dock panel to the right','right'));\n}"),
    ('cardpos_dock',
     'const o360CardPosStyle=object360CardPos?` style="left:${object360CardPos.left}px;top:${object360CardPos.top}px;right:auto"`:\'\';',
     'const o360CardPosStyle=object360CardPos?(object360CardPos.dock===\'left\'?` style="left:16px;top:16px;right:auto"`:(object360CardPos.dock===\'right\'?` style="right:16px;top:16px;left:auto"`:` style="left:${object360CardPos.left}px;top:${object360CardPos.top}px;right:auto"`)):\'\';'),
    ('carddock_wire',
     "makeInspectorCardDraggable($('#o360InspectorCard'),$('#o360InspectorHead'),(left,top)=>{object360CardPos={left,top}});",
     "makeInspectorCardDraggable($('#o360InspectorCard'),$('#o360InspectorHead'),(left,top)=>{object360CardPos={left,top}});\n o360WireCardDock();"),
]
for name, old, new in EDITS:
    c = src.count(old)
    assert c == 1, 'anchor %s matched %d times' % (name, c)
    src = src.replace(old, new)
    print('applied:', name)
open(PATH, 'w').write(src)
print('PR4 transform complete')
