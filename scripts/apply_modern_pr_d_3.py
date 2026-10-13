#!/usr/bin/env python3
"""Modern UI PR-D/3 - Ontology Graph workbench + Object 360 modal artifact fidelity.

Worker 3 of the PR-D fidelity pass (issue #395) for vikram2409-eng/Lanesra-OS.
User directive: "I want the screen and ui to be exactly same as the artifact."
This is a STRUCTURAL rebuild, not a styling pass.

Reference:
  ~/workspace/lanesra-build/prd-chain/reference/artifact-body.html  (ontology + object360 sections)
  ~/workspace/lanesra-build/prd-chain/reference/artifact-style.css

Region:
  1. Ontology Graph workbench - dependencyExplorerTab()'s chrome restructured to
     artifact .workbench > aside.graph-tools + .graph-stage + aside.graph-detail.
     graphModel()/depGraphModel()/graphRender() internals, traversal and layout
     algorithms are NOT touched (byte-identical). All toolbar ids + data-*
     hooks (depSearch, depZoomIn/Out/Fit/ResetView, depPathToggle, depSimToggle,
     depSimPicker, depCopyLink, depFieldsToggle, depType/Component/Depth,
     data-dep-toggle, depGraphViewport/depGraphPan) are preserved.
  2. Object 360 modal - object360Modal() emitter + o360WireModalChrome() +
     o360ApplyModalSize() + renderObject360Body() restructured to artifact
     .modal-stage > .record-backdrop + .object-modal.card >
     .modal-head (.modal-title h3 + .pivot-trail, maximize/close .btn-icon) +
     .modal-grid (graph side .mini-graph + .object-inspector side with
     .inspector-section blocks). Resizable/maximizable, resize grip, dockable
     + draggable inspector card, fields toggle, legend toggles, depth select,
     pivot trail, recenter, open-record, admin deep-links: all ids and handlers
     intact. o360RecordDriver()/o360GraphModel() untouched.

Rules honored:
  - app.js: exact ordinal string anchors, assert count==1 or fail loudly.
  - styles.css: APPEND ONLY, token-mapped artifact CSS (hsl(var(--X)) ->
    var(--X); hsl(var(--X) / A) -> color-mix(in srgb, var(--X) A%, transparent)).
    Zero hardcoded hex/radius/shadow. Canvas/SVG node colors keep the app's
    existing token mapping.

Usage:
  python3 apply_modern_pr_d_3.py [--app-in P] [--css-in P] [--app-out P] [--css-out P]
Defaults: pristine /tmp/lanesra-os/{app.js,styles.css} -> /tmp/prd-w3/{app.js,styles.css}
Byte-reproducible: pristine + this script -> byte-identical output across runs.
"""
import argparse
import hashlib
import sys

APPLIED = []


def rep(src, old, new, name):
    n = src.count(old)
    if n != 1:
        raise SystemExit("ANCHOR FAIL [%s]: expected 1 occurrence, found %d" % (name, n))
    APPLIED.append(name)
    return src.replace(old, new, 1)


def rep_span(src, start_anchor, end_anchor, new, name):
    """Replace the span from start_anchor through the end of end_anchor."""
    ns = src.count(start_anchor)
    ne = src.count(end_anchor)
    if ns != 1 or ne != 1:
        raise SystemExit("ANCHOR FAIL [%s]: start x%d, end x%d (want 1, 1)" % (name, ns, ne))
    i = src.find(start_anchor)
    j = src.find(end_anchor, i) + len(end_anchor)
    if j <= i:
        raise SystemExit("ANCHOR FAIL [%s]: end anchor not after start" % name)
    APPLIED.append(name)
    return src[:i] + new + src[j:]


def rep_line(src, prefix, new_line, name):
    """Replace the whole source line that starts with `prefix`."""
    n = src.count(prefix)
    if n != 1:
        raise SystemExit("ANCHOR FAIL [%s]: prefix x%d (want 1)" % (name, n))
    i = src.find(prefix)
    j = src.find("\n", i)
    if j < 0:
        raise SystemExit("ANCHOR FAIL [%s]: no end of line" % name)
    APPLIED.append(name)
    return src[:i] + new_line + src[j:]


# ---------------------------------------------------------------- app.js ---
def transform_app(src):
    # -- A1: chrome selectors -> .object-modal (artifact modal element) ------
    src = rep(src, r'''// PR4: wire the Object 360 modal chrome - an expand/reduce button in the
// header plus a drag-resize grip pinned to the modal's bottom-right corner.
function o360WireModalChrome(){
 if(typeof document==='undefined')return;
 const modalEl=document.querySelector('#modal .modal');
 const head=document.querySelector('#modal .modal-head');''',
r''' // PR-D/3: wire the Object 360 modal chrome - the artifact's .modal-head
// (title + pivot trail + maximize/close), the drag handle, and the
// drag-resize grip pinned to the modal's bottom-right corner.
function o360WireModalChrome(){
 if(typeof document==='undefined')return;
 const modalEl=document.querySelector('#modal .object-modal');
 const head=document.querySelector('#modal .modal-head');''', "A1 chrome selectors")

    # -- A2: maximize button gets artifact button classes --------------------
    src = rep(src,
        r''' btn.type='button';btn.id='o360MaxBtn';btn.className='icon-btn';btn.textContent='\u26F6';btn.title='Expand';''',
        r''' btn.type='button';btn.id='o360MaxBtn';btn.className='btn btn-outline btn-icon';btn.setAttribute('data-maximize','');btn.textContent='\u26F6';btn.title='Expand';''',
        "A2 max button classes")

    # -- A3: resize-grip placement selector ----------------------------------
    src = rep(src,
        r'''const m=document.querySelector('#modal .modal');if(!g||!m)return;''',
        r'''const m=document.querySelector('#modal .object-modal');if(!g||!m)return;''',
        "A3 grip place selector")

    # -- A4: wire the artifact drag handle (moves the modal in the stage) ----
    src = rep(src, r'''   grip.addEventListener('pointermove',move);grip.addEventListener('pointerup',up);grip.addEventListener('pointercancel',up);
  };
 }
 o360ApplyModalSize();
}''',
r'''   grip.addEventListener('pointermove',move);grip.addEventListener('pointerup',up);grip.addEventListener('pointercancel',up);
  };
 }
 // PR-D/3: the artifact's drag handle moves the modal inside the stage.
 const dragHandle=head.querySelector('#o360DragHandle');
 if(dragHandle&&!dragHandle.dataset.wired){
  dragHandle.dataset.wired='1';
  dragHandle.style.touchAction='none';
  dragHandle.onpointerdown=(e)=>{
   e.preventDefault();
   try{dragHandle.setPointerCapture(e.pointerId)}catch(_){}
   const sx=e.clientX,sy=e.clientY;
   const m=/translate\((-?\d+(?:\.\d+)?)px,\s*(-?\d+(?:\.\d+)?)px\)/.exec(modalEl.style.transform||'');
   const ox=m?parseFloat(m[1]):0,oy=m?parseFloat(m[2]):0;
   const move=(ev)=>{modalEl.style.transform='translate('+(ox+ev.clientX-sx)+'px,'+(oy+ev.clientY-sy)+'px)'};
   const up=()=>{dragHandle.removeEventListener('pointermove',move);dragHandle.removeEventListener('pointerup',up);dragHandle.removeEventListener('pointercancel',up)};
   dragHandle.addEventListener('pointermove',move);dragHandle.addEventListener('pointerup',up);dragHandle.addEventListener('pointercancel',up);
  };
 }
 o360ApplyModalSize();
}''', "A4 drag handle wiring")

    # -- B1: applyModalSize selector -----------------------------------------
    src = rep(src,
        r''' const modalEl=document.querySelector('#modal .modal');if(!modalEl)return;''',
        r''' const modalEl=document.querySelector('#modal .object-modal');if(!modalEl)return;''',
        "B1 applyModalSize selector")

    # -- B2: default size matches the artifact modal geometry ----------------
    src = rep(src,
        r'''  modalEl.style.width='min(1100px,95vw)';modalEl.style.height='';modalEl.style.maxWidth='';modalEl.style.maxHeight='92vh';''',
        r'''  modalEl.style.width='min(1120px,calc(100vw - 72px))';modalEl.style.height='min(760px,calc(100vh - 136px))';modalEl.style.maxWidth='none';modalEl.style.maxHeight='none';''',
        "B2 default modal size")

    # -- D1: pivot title anchor -> artifact h3 -------------------------------
    src = rep(src, r''' if(typeof document!=='undefined'){
  const h2=document.querySelector('#modal .modal-head h2');
  if(h2)h2.textContent='Object 360: '+label;
 }''',
r''' if(typeof document!=='undefined'){
  const h3=document.querySelector('#o360Title');
  if(h3)h3.textContent='Object 360 · '+label;
 }''', "D1 pivot title anchor")

    # -- E: object360Modal emitter -> artifact .modal-stage structure --------
    src = rep(src, r'''let object360State=null;
function object360Modal(entityType,entityId,label){
 object360State={root:{entityType,entityId,label},inspect:{entityType,entityId,label},trail:[]};
 modal('Object 360: '+label,'<div id="object360Body"></div>');
 const modalEl=document.querySelector('#modal .modal');
 // Modern UI PR-B: adopt the dialog component's chrome classes for the
 // Object 360 modal. The shared modal() markup stays untouched - these
 // classes only add token-driven chrome and sit below the modal's own
 // inline width/max-height.
 const overlayEl=document.querySelector('#modal');
 if(overlayEl)overlayEl.classList.add('dialog-overlay');
 if(modalEl){modalEl.classList.add('dialog-content');modalEl.style.cssText+=';width:min(1100px,95vw);max-height:92vh'}
 o360WireModalChrome();
 renderObject360Body();
}''',
r'''let object360State=null;
function object360Modal(entityType,entityId,label){
 object360State={root:{entityType,entityId,label},inspect:{entityType,entityId,label},trail:[]};
 // PR-D/3: artifact-fidelity modal chrome - .modal-stage > .record-backdrop
 // + .object-modal (.modal-head with title + pivot trail + maximize/close,
 // then .modal-grid with the graph side and the .object-inspector side).
 // The backdrop keeps id="modal" so the chrome wiring (maximize, resize
 // grip) and closeModal() keep working unchanged.
 document.body.insertAdjacentHTML('beforeend','<div class="modal-backdrop modal-stage" id="modal"><div class="record-backdrop" aria-hidden="true"><div class="backdrop-line" style="width:32%"></div><div class="backdrop-line" style="width:58%"></div><div class="card" style="height:260px;margin-top:36px"></div><div class="card" style="height:180px;margin-top:18px;width:66%"></div></div><article class="object-modal card" data-object-modal><header class="modal-head"><span class="drag-handle" id="o360DragHandle" title="Drag modal"><i></i><i></i><i></i></span><div class="modal-title"><h3 id="o360Title">Object 360 · '+label+'</h3><div class="pivot-trail" id="o360PivotTrail"></div></div><button type="button" class="btn btn-ghost btn-icon" id="o360CloseBtn" aria-label="Close preview">×</button></header><div class="modal-grid"><div class="mini-graph" id="o360MiniGraph"></div><aside class="object-inspector" id="object360Body"></aside></div></article></div>');
 const closeBtn=document.querySelector('#o360CloseBtn');
 if(closeBtn)closeBtn.onclick=()=>closeModal();
 o360WireModalChrome();
 renderObject360Body();
}''', "E modal emitter")

    # -- F6a: legend + Graph layers section computed before the card variants
    src = rep(src, r''' const o360CardPosStyle=object360CardPos?(object360CardPos.dock==='left'?` style="left:16px;top:16px;right:auto"`:(object360CardPos.dock==='right'?` style="right:16px;top:16px;left:auto"`:` style="left:${object360CardPos.left}px;top:${object360CardPos.top}px;right:auto"`)):'';
 let inspectorHtml;''',
r''' const o360CardPosStyle=object360CardPos?(object360CardPos.dock==='left'?` style="left:16px;top:16px;right:auto"`:(object360CardPos.dock==='right'?` style="right:16px;top:16px;left:auto"`:` style="left:${object360CardPos.left}px;top:${object360CardPos.top}px;right:auto"`)):'';
 // PR-D/3: legend + view controls are computed before the inspector card so
 // every variant can include the Graph layers .inspector-section.
 const o360LegendTypes=[];
 model.nodes.forEach(n=>{if(o360LegendTypes.indexOf(n.type)<0)o360LegendTypes.push(n.type)});
 const o360LegendCounts={};
 model.nodes.forEach(n=>{o360LegendCounts[n.type]=(o360LegendCounts[n.type]||0)+1});
 const o360LegendHtmlStr=o360LegendTypes.length?`<div class="ontology-legend">${o360LegendTypes.map(k=>{
  const lbl=driver.typeLabel(k);
  const isOff=object360HiddenTypes.has(k);
  return `<button type="button" class="ontology-legend-item${isOff?' ontology-legend-item-off':''}" data-o360-legend-toggle="${k}" title="${isOff?'Show':'Hide'} ${lbl} (${o360LegendCounts[k]||0} shown)"><span style="display:inline-block;flex-shrink:0;width:10px;height:10px;border-radius:50%;background:${colors[k]}"></span>${lbl}<span class="ontology-legend-count">${o360LegendCounts[k]||0}</span></button>`;
 }).join('')}</div>`:'';
 const viewSectionHtml=`<div class="inspector-section"><div class="panel-title"><div><h3>Graph layers</h3><p class="card-desc">Toggle what the graph shows</p></div></div>${o360LegendHtmlStr}<div class="o360-view-controls"><label class="o360-depth">Depth <select id="o360Depth" class="select">${['1','2','3'].map(d=>`<option value="${d}"${String(s.depth||'2')===d?' selected':''}>${d}</option>`).join('')}</select></label><label class="ontology-fields-toggle" title="Field nodes can clutter the graph - hide them here"><input type="checkbox" id="o360FieldsToggle"${object360HiddenTypes.has('custom_field')?'':' checked'}> Fields</label></div></div>`;
 let inspectorHtml;''', "F6a legend/viewSection hoist")

    # -- F6c: orgUnit variant gains the Graph layers section -----------------
    src = rep(src, r'''   <div class="ontology-inspector-head" id="o360InspectorHead">${o360TypeDot('orgUnit')}<div><strong>${s.inspect.label}</strong><div class="ontology-inspector-subtitle">Business Unit</div></div></div>
   <p class="muted" style="font-size:12px">Business Units aren't individually re-centerable here - see Admin → Organization Units for its own detail.</p>
  </div>`;''',
r'''   <div class="ontology-inspector-head" id="o360InspectorHead">${o360TypeDot('orgUnit')}<div><strong>${s.inspect.label}</strong><div class="ontology-inspector-subtitle">Business Unit</div></div></div>
   ${viewSectionHtml}
   <div class="inspector-section"><p class="muted" style="font-size:12px">Business Units aren't individually re-centerable here - see Admin → Organization Units for its own detail.</p></div>
  </div>`;''', "F6c orgUnit variant")

    # -- F6d: pseudo variant gains the Graph layers section ------------------
    src = rep(src, r'''    <button type="button" class="btn btn-secondary" id="o360OpenPseudoAdmin">Open in Admin → ${pseudoTabLabel}</button>
   </div>
  </div>`;''',
r'''    <button type="button" class="btn btn-secondary" id="o360OpenPseudoAdmin">Open in Admin → ${pseudoTabLabel}</button>
   </div>
   ${viewSectionHtml}
  </div>`;''', "F6d pseudo variant")

    # -- F2: rule rows -> artifact .rule-row ---------------------------------
    src = rep(src,
        r'''<button type="button" class="ontology-inspector-bullet" data-o360-admin="rules"''',
        r'''<button type="button" class="rule-row" data-o360-admin="rules"''',
        "F2 rule rows")

    # -- F3: workflow rows -> artifact .flow-row ------------------------------
    src = rep(src,
        r'''<button type="button" class="ontology-inspector-bullet" data-o360-admin="workflow"''',
        r'''<button type="button" class="flow-row" data-o360-admin="workflow"''',
        "F3 workflow rows")

    # -- F4: agent rows -> artifact .access-chip list -------------------------
    src = rep(src, r'''   const agentRows=agents.map(a=>{
    const chips=o360AgentChips(a,s.root.entityType);
    return `<div class="ontology-inspector-bullet" style="cursor:default"><span style="flex:1;text-align:left">${a.icon||''} ${a.name}</span>${chips.map(c=>`<span class="badge ${c==='can read'||c==='can update'||c==='can delete'?'badge-info':'badge-secondary'}">${c}</span>`).join('')}</div>`;
   }).join('');''',
r'''   const agentChipsHtml=agents.map(a=>{
    const caps=o360AgentChips(a,s.root.entityType);
    const pretty=caps.map(c=>c==='can read'?'Read':c==='can update'?'Act':c==='can delete'?'Delete':c).join(' + ');
    return `<span class="access-chip">${a.icon||''} ${a.name} · ${pretty}</span>`;
   }).join('');''', "F4 agent access chips")

    # -- F5: groups -> artifact .inspector-section blocks --------------------
    src = rep_span(src,
        r'''   groupsHtml=`${groupEntries.map''',
        r'''${new Date(a.occurredAt).toLocaleString()}</div>`).join('')}</div>`:''}`;''',
r'''   sectionsHtml=`${groupEntries.length?`<div class="inspector-section"><div class="panel-title"><div><h3>Related records</h3><p class="card-desc">Linked to this record</p></div><span class="settings-count">${groupEntries.reduce((n,g)=>n+g[1].length,0)} linked</span></div>${groupEntries.map(([key,members])=>`<div class="ontology-inspector-group"><div class="ontology-inspector-group-label">${(members[0]&&members[0].groupLabel)||key}</div>${members.slice(0,MAXPER).map(m=>`<button type="button" class="ontology-inspector-bullet" data-o360g-inspect="${m.entityType}:${m.entityId}">${entityDotHtml(m.entityType)}${m.displayName}</button>`).join('')}${members.length>MAXPER?`<div class="muted" style="font-size:12px">+${members.length-MAXPER} more</div>`:''}</div>`).join('')}</div>`:`<div class="inspector-section"><p class="muted" style="font-size:12px">No related records yet.</p></div>`}
    ${orgUnit?`<div class="inspector-section"><div class="panel-title"><div><h3>Business Unit</h3></div></div><div style="font-size:13px">${orgUnit.name}</div></div>`:''}
    ${rules.length?`<div class="inspector-section"><div class="panel-title"><div><h3>Business Rules</h3><p class="card-desc">Evaluated for this record</p></div><span class="settings-count">${rules.length} rule${rules.length===1?'':'s'}</span></div>${ruleRows}</div>`:''}
    ${workflows.length?`<div class="inspector-section"><div class="panel-title"><div><h3>Workflows</h3><p class="card-desc">Connected automation</p></div></div><div class="o360-trigger-note">Trigger check evaluates each workflow's conditions against this record live — the demo keeps no run history, so there is no "last run" to show.</div>${wfRows}</div>`:''}
    ${agents.length?`<div class="inspector-section"><h3>Agent effective access</h3><p class="card-desc">Inherited and direct permissions</p><div class="access-chips">${agentChipsHtml}</div></div>`:''}
    ${activities.length?`<div class="inspector-section"><div class="panel-title"><div><h3>Activity</h3><p class="card-desc">Recent work on this record</p></div></div>${activities.map(a=>`<div style="font-size:12px" class="muted">${a.subject||a.channel}<br>${new Date(a.occurredAt).toLocaleString()}</div>`).join('')}</div>`:''}`;''',
        "F5 inspector sections")

    # -- F5b: the groupsHtml declaration becomes sectionsHtml ---------------
    src = rep(src, ' }else{\n  let groupsHtml=\'\';\n  if(inspectIsRoot){',
        ' }else{\n  let sectionsHtml=\'\';\n  if(inspectIsRoot){', "F5b groupsHtml->sectionsHtml")

    # -- F6e: default variant uses sections + Graph layers -------------------
    src = rep(src, r'''   ${inspectIsRoot?`<div class="ontology-inspector-groups">${groupsHtml}</div>`:''}
  </div>`;''',
r'''   ${viewSectionHtml}
   ${inspectIsRoot?`${sectionsHtml}`:''}
  </div>`;''', "F6e default variant")

    # -- F6b: drop the old legend block (moved up into F6a) ------------------
    src = rep(src, r''' // Legend doubles as the toggle-filter; built from the types actually in
 // this record's model, now including rules/workflows/agents/orgUnit.
 const legendTypes=[];
 model.nodes.forEach(n=>{if(legendTypes.indexOf(n.type)<0)legendTypes.push(n.type)});
 const legendCounts={};
 model.nodes.forEach(n=>{legendCounts[n.type]=(legendCounts[n.type]||0)+1});
 const legendHtmlStr=legendTypes.length?`<div class="ontology-legend">${legendTypes.map(k=>{
  const lbl=driver.typeLabel(k);
  const isOff=object360HiddenTypes.has(k);
  return `<button type="button" class="ontology-legend-item${isOff?' ontology-legend-item-off':''}" data-o360-legend-toggle="${k}" title="${isOff?'Show':'Hide'} ${lbl} (${legendCounts[k]||0} shown)"><span style="display:inline-block;flex-shrink:0;width:10px;height:10px;border-radius:50%;background:${colors[k]}"></span>${lbl}<span class="ontology-legend-count">${legendCounts[k]||0}</span></button>`;
 }).join('')}</div>`:'';

''', '', "F6b drop old legend block")

    # -- F7: split body render -> head trail / graph side / inspector side ---
    src = rep_span(src,
        r''' // Breadcrumb back through the pivot trail (issue #374).''',
        r'''<div class="ontology-graph-canvas" style="width:${W}px;height:${H}px">${graphHtml}${inspectorHtml}</div>
 </div>`;''',
r''' // Pivot trail (issue #374) - PR-D/3 renders it into the modal head's
 // .pivot-trail; the graph fills the .mini-graph side.
 const trail=s.trail||[];
 const trailEl=document.querySelector('#o360PivotTrail');
 if(trailEl)trailEl.innerHTML=trail.length?trail.map((t,i)=>`<button type="button" class="o360-crumb" data-o360-crumb="${i}" title="Pivot back to ${t.label}">${t.label}</button><span class="o360-crumb-sep">›</span>`).join('')+`<span class="o360-crumb-current">${s.root.label}</span>`:'';
 const graphHost=document.querySelector('#o360MiniGraph');
 if(graphHost)graphHost.innerHTML=`<div class="ontology-graph-canvas" style="width:${W}px;height:${H}px">${graphHtml}</div>`;

 body.innerHTML=`${inspectorHtml}`;''', "F7 split body render")

    # -- F8: crumb wiring targets the head trail -----------------------------
    src = rep(src,
        r''' body.querySelectorAll('[data-o360-crumb]').forEach(b=>b.onclick=()=>{''',
        r''' (trailEl||body).querySelectorAll('[data-o360-crumb]').forEach(b=>b.onclick=()=>{''',
        "F8 crumb wiring")

    # -- G0: artifact tools rail (ids + data-* intact) ------------------------
    src = rep(src, r''' let graphHtml='';
 let truncatedHits=0;
 const depCounts={};''',
r''' // PR-D/3: artifact workbench tools rail - .graph-tools with labeled tool
 // sections. Every id and data-* hook is unchanged, so the toolbar wiring
 // (zoom/pan/search/path/sim/share), the depth select, the impact/lineage
 // toggles and the fields toggle keep working exactly as before.
 const depToolsHtml=`<aside class="graph-tools" aria-label="Graph tools">
  <div class="tool-section"><div class="tool-label">Component</div><div class="path-fields"><label>Component type <select id="depType" class="select">${types.map(t=>`<option value="${t}" ${t===dependencyExplorerState.type?'selected':''}>${SYSTEM_NODE_TYPE_LABELS[t]}</option>`).join('')}</select></label><label>Component <select id="depComponent" class="select">${options.map(o=>`<option value="${o.id}" ${o.id===dependencyExplorerState.id?'selected':''}>${o.label}</option>`).join('')}</select></label></div></div>
  <div class="tool-section"><div class="tool-label">Search graph</div><div class="search-wrap" style="width:100%"><input type="search" id="depSearch" class="input ontology-search-input" placeholder="Search components…" autocomplete="off"><div id="depSearchResults" class="ontology-search-results" hidden></div></div></div>
  <div class="tool-section"><div class="tool-label">Find path</div><div class="path-fields"><button type="button" class="btn ${dependencyExplorerState.pathMode?'btn-primary':'btn-secondary'} btn-sm" id="depPathToggle" title="Click two nodes on the canvas to trace the shortest path between them">Find path</button>${depPathBarHtml()}</div></div>
  <div class="tool-section"><div class="tool-label">Viewport</div><div class="zoom-row"><button type="button" class="btn btn-outline btn-sm" id="depZoomOut" title="Zoom out" aria-label="Zoom out">−</button><button type="button" class="btn btn-outline btn-sm" id="depFit" title="Reset zoom and pan">Fit</button><button type="button" class="btn btn-outline btn-sm" id="depZoomIn" title="Zoom in" aria-label="Zoom in">+</button><button type="button" class="btn btn-outline btn-sm" id="depResetView" title="Reset zoom, pan, path and simulation">Reset</button></div><div class="zoom-readout" data-zoom-readout>${Math.round((dependencyExplorerState.zoom||1)*100)}%</div></div>
  <div class="tool-section"><div class="tool-label">Depth</div><div class="chip-row"><label>Depth <select id="depDepth" class="select" aria-label="Depth">${['1','2','3','all'].map(d=>`<option value="${d}" ${String(dependencyExplorerState.depth)===d?'selected':''}>${d==='all'?'All':d+' hop'+(d==='1'?'':'s')}</option>`).join('')}</select></label></div></div>
  <div class="tool-section"><div class="tool-label">Analysis</div><div class="chip-row"><button type="button" class="toggle-chip${dependencyExplorerState.showImpact?' active':''}" data-dep-toggle="showImpact" title="Toggle full impact">Impact</button><button type="button" class="toggle-chip${dependencyExplorerState.showLineage?' active':''}" data-dep-toggle="showLineage" title="Toggle full lineage">Lineage</button></div></div>
  <div class="tool-section"><div class="tool-label">Content</div><div class="chip-row"><label class="ontology-fields-toggle" title="Field nodes can clutter the graph - hide them here"><input type="checkbox" id="depFieldsToggle"${depExplorerHiddenTypes.has('custom_field')?'':' checked'}> Fields</label></div></div>
  <div class="tool-section"><div class="tool-label">Simulate</div><div class="path-fields"><button type="button" class="btn btn-secondary btn-sm" id="depSimToggle" title="Preview what a change to the inspected node would affect" ${dependencyExplorerState.inspectId?'':'disabled'}>Simulate change</button><span id="depSimPicker" hidden><select id="depSimChange" class="select"><option value="rename">rename</option><option value="delete">delete</option><option value="revoke access">revoke access</option></select> <button type="button" class="btn btn-primary btn-sm" id="depSimRun">Preview</button></span><button type="button" class="btn btn-secondary btn-sm" id="depCopyLink" title="Copy a shareable link to this view">Copy link</button></div></div>
 </aside>`;
 let graphHtml='';
 let truncatedHits=0;
 const depCounts={};''', "G0 tools rail")

    # -- G1: stage + detail chrome (whole line replaced) ----------------------
    src = rep_line(src, r'''  graphHtml=`<div class="ontology-graph-layout">''',
r'''  graphHtml=`<div class="graph-stage" data-graph-stage><div class="graph-stage-head"><div><div class="eyebrow">Ontology Graph</div><h3>${systemGraphNodeLabel(dependencyExplorerState.type,dependencyExplorerState.id)} neighborhood</h3></div><div class="chip-row"><span class="badge badge-outline">Depth ${String(dependencyExplorerState.depth)==='all'?'All':String(dependencyExplorerState.depth||'2')}</span><span class="badge badge-secondary">${depModel.edges.length} relationships</span>${truncatedHits?`<span class="ontology-more-chip" title="Narrow the depth or hide types to see the rest">+${truncatedHits} more</span>`:''}</div></div>${depSimBannerHtml()}<div class="graph-canvas"><div id="depGraphViewport" class="ontology-graph-viewport"><div id="depGraphPan" class="ontology-graph-canvas" style="width:${depW}px;height:${depH}px;transform:translate(${depPanX}px,${depPanY}px) scale(${depZoom});transform-origin:0 0">${depRenderGraph(depModel,depW,depH)}</div></div></div><div class="graph-legend">${legendHtml(depExplorerHiddenTypes,depCounts)}</div></div><aside class="graph-detail" aria-label="Selected node">${depDetailPanelHtml(depModel)}</aside>`;''',
        "G1 stage chrome")

    # -- G2: workbench assembly ----------------------------------------------
    src = rep_span(src,
        ' body.innerHTML=`<div class="panel">\n  <h3>Ontology Graph</h3>',
        '  ${graphHtml}`}\n </div>`;',
r''' body.innerHTML=`<div class="workbench">${depToolsHtml}${graphHtml||`<div class="graph-stage" data-graph-stage><div class="graph-stage-head"><div><div class="eyebrow">Ontology Graph</div><h3>Pick a component</h3></div></div><div class="graph-canvas"><div class="empty" style="padding:48px 24px;text-align:center;color:var(--muted-foreground)">Nothing synced yet for this type.</div></div></div>`}</div>`;''',
        "G2 workbench assembly")

    return src


# -------------------------------------------------------------- styles.css --
CSS_APPEND = r'''/* ===== Modern UI PR-D/3 graph+object360 — artifact fidelity ===== */
.workbench{min-height:calc(100vh - 64px);display:grid;grid-template-columns:228px minmax(0,1fr) 286px;background:var(--background)}
.graph-tools,.graph-detail{padding:18px;background:var(--card);overflow-y:auto}
.graph-tools{border-right:1px solid var(--border)}
.graph-detail{border-left:1px solid var(--border);min-width:0}
.tool-section+.tool-section{margin-top:20px;padding-top:18px;border-top:1px solid var(--border)}
.tool-label{margin-bottom:7px;color:var(--muted-foreground);font-size:11px;font-weight:700}
.path-fields{display:grid;gap:8px}
.path-fields>label{display:grid;gap:5px;font-size:12px;font-weight:600;color:var(--muted-foreground)}
.zoom-row,.graph-tools .chip-row,.graph-stage-head .chip-row{display:flex;flex-wrap:wrap;gap:6px}
.zoom-row .btn{flex:1;min-width:0}
.toggle-chip{border:1px solid var(--border);background:var(--background);color:var(--muted-foreground);border-radius:var(--radius-full);min-height:30px;padding:4px 10px;font-size:12px;font-weight:650;cursor:pointer}
.toggle-chip:hover{color:var(--foreground);border-color:var(--muted-foreground)}
.toggle-chip.active{color:var(--accent-foreground);border-color:color-mix(in srgb,var(--primary) 35%,transparent);background:var(--accent)}
.zoom-readout{margin-top:6px;text-align:center;font-size:11px;color:var(--muted-foreground);font-variant-numeric:tabular-nums}
.search-wrap{position:relative}
.graph-stage{min-width:0;position:relative;display:flex;flex-direction:column;background-color:color-mix(in srgb,var(--muted) 28%,transparent);background-image:radial-gradient(color-mix(in srgb,var(--muted-foreground) 18%,transparent) 1px,transparent 1px);background-size:18px 18px}
.graph-stage-head{min-height:64px;display:flex;align-items:center;justify-content:space-between;gap:12px;padding:12px 18px;background:color-mix(in srgb,var(--background) 86%,transparent);border-bottom:1px solid var(--border);backdrop-filter:blur(10px)}
.graph-stage-head h3{margin:3px 0 0;font-size:18px;font-weight:700;letter-spacing:-.01em}
.graph-canvas{min-height:620px;position:relative;overflow:hidden;flex:1}
.graph-canvas .ontology-graph-viewport{position:absolute;inset:0;border:0;border-radius:0;max-height:none;background:none}
.graph-legend{display:flex;flex-wrap:wrap;gap:7px;padding:12px 16px;border-top:1px solid var(--border);background:var(--background)}
.graph-legend .ontology-legend{display:flex;flex-wrap:wrap;gap:7px;margin:0;padding:0;background:none;border:0;width:auto;max-width:none}
.graph-legend .ontology-legend-item{border:1px solid var(--border);background:var(--card);color:var(--foreground);border-radius:var(--radius-sm);font-size:11px;font-weight:650;padding:5px 8px}
.graph-legend .ontology-legend-count{background:var(--muted);border-radius:var(--radius-sm);min-width:18px;height:18px;display:inline-grid;place-items:center;font-size:10px;margin-left:2px}
.graph-detail .ontology-detail-panel{width:auto;max-height:none;border:0;background:transparent;border-radius:0;padding:0;box-shadow:none;overflow:visible}
#modal.modal-stage{position:fixed;inset:0;z-index:100;display:grid;place-items:center;padding:36px;overflow:hidden;background:color-mix(in srgb,var(--foreground) 72%,transparent)}
.record-backdrop{position:absolute;inset:0;padding:44px;background:var(--background);filter:blur(1.5px);opacity:.3;pointer-events:none}
.backdrop-line{height:13px;margin-bottom:12px;border-radius:var(--radius-sm);background:color-mix(in srgb,var(--muted-foreground) 30%,transparent)}
.object-modal{position:relative;z-index:2;width:min(1120px,calc(100vw - 72px));height:min(760px,calc(100vh - 136px));min-height:560px;margin:0 auto;overflow:auto;background:var(--background);color:var(--foreground);border:1px solid var(--border);border-radius:var(--radius-lg);box-shadow:var(--shadow-lg)}
.object-modal.maximized{width:calc(100vw - 72px);height:calc(100vh - 72px)}
.object-modal .modal-head{position:sticky;top:0;z-index:5;display:flex;align-items:center;gap:12px;min-height:66px;padding:10px 18px;border-bottom:1px solid var(--border);background:color-mix(in srgb,var(--background) 94%,transparent);backdrop-filter:blur(12px)}
.drag-handle{display:grid;gap:3px;padding:7px;cursor:grab;flex-shrink:0}
.drag-handle i{display:block;width:18px;height:1px;background:var(--muted-foreground)}
.object-modal .modal-title{flex:1;min-width:0}
.object-modal .modal-title h3{margin:0;font-size:18px;font-weight:700;letter-spacing:-.01em}
.pivot-trail{display:flex;flex-wrap:wrap;gap:5px;align-items:center;margin-top:7px;color:var(--muted-foreground);font-size:12px}
.pivot-trail strong{color:var(--foreground)}
.modal-grid{display:grid;grid-template-columns:minmax(390px,1.15fr) minmax(320px,.85fr);min-height:calc(100% - 66px)}
.mini-graph{min-width:0;border-right:1px solid var(--border);background-color:color-mix(in srgb,var(--muted) 28%,transparent);background-image:radial-gradient(color-mix(in srgb,var(--muted-foreground) 18%,transparent) 1px,transparent 1px);background-size:18px 18px;overflow:auto}
.mini-graph .ontology-graph-canvas{position:relative}
.object-inspector{padding:18px;position:relative;min-width:0}
.object-inspector .ontology-inspector-card{max-width:calc(100% - 32px)}
.inspector-section+.inspector-section{margin-top:20px;padding-top:18px;border-top:1px solid var(--border)}
.inspector-section h3{margin:0 0 2px;font-size:15px;font-weight:700}
.panel-title{display:flex;align-items:center;justify-content:space-between;gap:12px}
.panel-title h3{margin:0;font-size:15px;font-weight:700}
.card-desc{color:var(--muted-foreground);margin-top:5px;font-size:12px}
.settings-count{padding:4px 7px;border-radius:var(--radius-sm);background:var(--muted);color:var(--muted-foreground);font-size:11px;font-weight:650;white-space:nowrap}
.rule-row,.flow-row{display:flex;align-items:center;justify-content:space-between;gap:10px;width:100%;padding:10px 0;background:none;border:0;cursor:pointer;font:inherit;color:inherit;text-align:left}
.rule-row+.rule-row,.flow-row+.flow-row{border-top:1px solid var(--border)}
.access-chips{display:flex;flex-wrap:wrap;gap:7px;margin-top:10px}
.access-chip{display:inline-flex;align-items:center;gap:5px;padding:5px 8px;border:1px solid var(--border);border-radius:var(--radius-sm);background:color-mix(in srgb,var(--muted) 50%,transparent);font-size:11px;font-weight:650}
.o360-view-controls{display:grid;gap:8px;margin-top:10px}
.object-inspector .ontology-legend{display:flex;flex-wrap:wrap;gap:7px;margin:0 0 4px;padding:0;background:none;border:0;width:auto;max-width:none}
.object-inspector .ontology-legend-item{border:1px solid var(--border);background:var(--card);color:var(--foreground);border-radius:var(--radius-sm);font-size:11px;font-weight:650;padding:5px 8px}
@media (max-width:1040px){
 .workbench{grid-template-columns:190px minmax(0,1fr)}
 .graph-detail{grid-column:1/-1;border-left:0;border-top:1px solid var(--border)}
 #modal.modal-stage{padding:24px}
 .modal-grid{grid-template-columns:1fr}
 .mini-graph{border-right:0;border-bottom:1px solid var(--border)}
}
@media (max-width:760px){
 .workbench{display:block}
 .graph-tools,.graph-detail{border:0;border-bottom:1px solid var(--border)}
 .graph-tools{display:grid;grid-template-columns:1fr;gap:14px}
 .tool-section+.tool-section{margin:0;padding:0;border:0}
 .graph-detail{display:block}
 .graph-canvas{min-height:470px}
 .graph-stage-head{align-items:flex-start;flex-direction:column}
 #modal.modal-stage{padding:12px}
 .object-modal,.object-modal.maximized{width:calc(100vw - 24px);height:auto;min-height:0;max-height:none;border-radius:var(--radius-md)}
 .object-modal .modal-head{align-items:flex-start}
 .mini-graph .ontology-graph-canvas{min-height:390px}
 .object-inspector{padding:16px}
}
'''


def transform_css(src):
    marker = "/* ===== Modern UI PR-D/3 graph+object360 — artifact fidelity ===== */"
    if marker in src:
        raise SystemExit("CSS FAIL: marker already present - refusing to double-append")
    if not src.endswith("\n"):
        src += "\n"
    return src + CSS_APPEND


def main():
    ap = argparse.ArgumentParser(description="PR-D/3: Ontology Graph workbench + Object 360 modal artifact fidelity")
    ap.add_argument("--app-in", default="/tmp/lanesra-os/app.js")
    ap.add_argument("--css-in", default="/tmp/lanesra-os/styles.css")
    ap.add_argument("--app-out", default="/tmp/prd-w3/app.js")
    ap.add_argument("--css-out", default="/tmp/prd-w3/styles.css")
    args = ap.parse_args()

    with open(args.app_in, "r", encoding="utf-8") as f:
        app = f.read()
    with open(args.css_in, "r", encoding="utf-8") as f:
        css = f.read()

    app = transform_app(app)
    css = transform_css(css)

    with open(args.app_out, "w", encoding="utf-8") as f:
        f.write(app)
    with open(args.css_out, "w", encoding="utf-8") as f:
        f.write(css)

    print("anchors applied: %d" % len(APPLIED))
    for a in APPLIED:
        print("  ok", a)
    print("app.js  sha256:", hashlib.sha256(app.encode("utf-8")).hexdigest())
    print("styles.css sha256:", hashlib.sha256(css.encode("utf-8")).hexdigest())


if __name__ == "__main__":
    main()
