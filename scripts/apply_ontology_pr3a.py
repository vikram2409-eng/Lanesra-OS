#!/usr/bin/env python3
"""Apply Ontology Graph PR3-A power tools (issues #369, #370, #371) to app.js.

Reproducible transform: each replacement is anchored on exact pristine text
and asserts a match count of exactly 1, failing loudly otherwise.
Run from the repo root:  python3 /tmp/pr3a/apply_ontology_pr3a.py
"""
import sys

PATH = 'app.js'


def rep(content, old, new, name):
    n = content.count(old)
    if n != 1:
        print(f'FAIL [{name}]: expected exactly 1 match, found {n}', file=sys.stderr)
        sys.exit(1)
    return content.replace(old, new, 1)


def main():
    with open(PATH, 'r', encoding='utf-8') as f:
        c = f.read()

    # R1: power-tool view state defaults + module scratch vars.
    c = rep(c,
        "let dependencyExplorerState={type:'custom_object',id:null,depth:'2',inspectType:null,inspectId:null,showImpact:true,showLineage:true,expanded:{},collapsed:{}};",
        "let dependencyExplorerState={type:'custom_object',id:null,depth:'2',inspectType:null,inspectId:null,showImpact:true,showLineage:true,expanded:{},collapsed:{},zoom:1,panX:0,panY:0,labelMode:'all',pathMode:false,pathFrom:null,pathTo:null,pathKeys:null,pathEdges:null,pathUnreachable:false,simKeys:null,simChange:null,simRoot:null};\n"
        "// PR3-A power tools: last canvas size (for centering) + one-time document-level pan listeners.\n"
        "let depLastCanvasW=0,depLastCanvasH=0,depPanWired=false,depDragPanEl=null,depDragState=null;",
        'R1 state defaults')

    # R2: top of dependencyExplorerTab - share-hash deep link + state normalization.
    c = rep(c,
        "function dependencyExplorerTab(body){\n const nodes=systemGraphNodes();",
        "function dependencyExplorerTab(body){\n"
        " depApplyShareHashIfNeeded();\n"
        " const nodes=systemGraphNodes();\n"
        " // PR3-A: normalize power-tool view state on every render - fresh resets\n"
        " // (which omit these keys) fall back to defaults, re-renders persist them.\n"
        " dependencyExplorerState={zoom:1,panX:0,panY:0,labelMode:'all',pathMode:false,pathFrom:null,pathTo:null,pathKeys:null,pathEdges:null,pathUnreachable:false,simKeys:null,simChange:null,simRoot:null,...dependencyExplorerState};",
        'R2 tab top')

    # R3: graph HTML - toolbar + zoom/pan viewport replaces the plain scroll wrap.
    c = rep(c,
        '  graphHtml=`<div class="ontology-graph-layout"><div class="ontology-graph-wrap" style="flex:1;min-width:0"><div class="ontology-graph-canvas" style="width:${depW}px;height:${depH}px">${depRenderGraph(depModel,depW,depH)}</div></div>${depDetailPanelHtml(depModel)}</div>`;',
        "  depLastCanvasW=depW;depLastCanvasH=depH;\n"
        "  const depZoom=dependencyExplorerState.zoom||1,depPanX=dependencyExplorerState.panX||0,depPanY=dependencyExplorerState.panY||0;\n"
        '  graphHtml=`<div class="ontology-graph-layout"><div style="flex:1;min-width:0">${depSimBannerHtml()}<div class="ontology-toolbar"><span class="ontology-toolbar-group"><button type="button" class="btn btn-secondary" id="depZoomIn" title="Zoom in">+</button><button type="button" class="btn btn-secondary" id="depZoomOut" title="Zoom out">\u2212</button><button type="button" class="btn btn-secondary" id="depFit" title="Reset zoom and pan">Fit</button><button type="button" class="btn btn-secondary" id="depResetView" title="Reset zoom, pan, path and simulation">Reset</button></span><span class="ontology-search-wrap"><input type="search" id="depSearch" class="ontology-search-input" placeholder="Search components\u2026" autocomplete="off"><div id="depSearchResults" class="ontology-search-results" hidden></div></span><button type="button" class="btn ${dependencyExplorerState.pathMode?\'btn-primary\':\'btn-secondary\'}" id="depPathToggle" title="Click two nodes on the canvas to trace the shortest path between them">Find path</button><button type="button" class="btn btn-secondary" id="depSimToggle" title="Preview what a change to the inspected node would affect" ${dependencyExplorerState.inspectId?\'\':\'disabled\'}>Simulate change</button><span id="depSimPicker" hidden><select id="depSimChange"><option value="rename">rename</option><option value="delete">delete</option><option value="revoke access">revoke access</option></select> <button type="button" class="btn btn-primary" id="depSimRun">Preview</button></span><button type="button" class="btn btn-secondary" id="depCopyLink" title="Copy a shareable link to this view">Copy link</button></div>${depPathBarHtml()}<div id="depGraphViewport" class="ontology-graph-viewport"><div id="depGraphPan" class="ontology-graph-canvas" style="width:${depW}px;height:${depH}px;transform:translate(${depPanX}px,${depPanY}px) scale(${depZoom});transform-origin:0 0">${depRenderGraph(depModel,depW,depH)}</div></div></div>${depDetailPanelHtml(depModel)}</div>`;',
        'R3 toolbar+viewport')

    # R4: branch canvas-node clicks into path mode.
    c = rep(c,
        " body.querySelectorAll('[data-dep-inspect]').forEach(b=>b.onclick=()=>{const [type,id]=b.dataset.depInspect.split(':');dependencyExplorerState={...dependencyExplorerState,inspectType:type,inspectId:id};renderAdminTab()});",
        " body.querySelectorAll('[data-dep-inspect]').forEach(b=>b.onclick=()=>{const [type,id]=b.dataset.depInspect.split(':');if(dependencyExplorerState.pathMode){depPathPick(type,id);return}dependencyExplorerState={...dependencyExplorerState,inspectType:type,inspectId:id};renderAdminTab()});",
        'R4 path-mode click branch')

    # R5: end of dependencyExplorerTab - wire toolbar, persist share hash.
    c = rep(c,
        " const openBtn=$('#depOpenAdmin'); if(openBtn)openBtn.onclick=()=>{adminTab=NODE_TYPE_ADMIN_TAB[dependencyExplorerState.inspectType];adminView='tool';renderView()};\n}",
        " const openBtn=$('#depOpenAdmin'); if(openBtn)openBtn.onclick=()=>{adminTab=NODE_TYPE_ADMIN_TAB[dependencyExplorerState.inspectType];adminView='tool';renderView()};\n"
        " // PR3-A power tools: wire toolbar (zoom/pan, search, path, simulation, share link).\n"
        " depWireToolbar(body);\n"
        " if(dependencyExplorerState.id)depWriteShareHash();\n"
        "}",
        'R5 toolbar wiring+share hash')

    # R6: detail panel actions - Object 360 cross-link for object nodes.
    c = rep(c,
        "  +'<button type=\"button\" class=\"btn btn-primary\" id=\"depOpenAdmin\">Open in its admin screen</button>'",
        "  +'<button type=\"button\" class=\"btn btn-primary\" id=\"depOpenAdmin\">Open in its admin screen</button>'\n"
        "  +(s.inspectType==='custom_object'?'<button type=\"button\" class=\"btn btn-secondary\" id=\"depOpen360\">View records in Object 360</button>':'')",
        'R6 Object 360 button')

    # R7: Path section + all new top-level functions.
    c = rep(c,
        "  +'</div></div>';\n}\nfunction depExpandableCount(model,type,id){",
        NEW_FUNCTIONS,
        'R7 path section + functions')

    with open(PATH, 'w', encoding='utf-8') as f:
        f.write(c)
    print(f'OK: applied 7 replacements to {PATH}')


NEW_FUNCTIONS = """  +'</div>'+depPathSectionHtml()+'</div>';
}
// ---------------------------------------------------------------------------
// PR3-A power tools (issues #369, #370, #371; spec 7.6, FR-9/10/11): zoom/pan
// toolbar, component search, find-path, change-impact simulation, shareable
// view state. These only SET dependencyExplorerState fields - the render
// layer (depRenderGraph, owned by the parallel worker) honors pathKeys,
// simKeys, simChange and labelMode.
// ---------------------------------------------------------------------------
// Shareable view state -------------------------------------------------------
function depReadShareHash(){
 try{
  const h=(typeof location==='undefined'?'':location.hash)||'';
  if(h.indexOf('#/ontology')!==0)return null;
  const params=new URLSearchParams(h.slice('#/ontology'.length).replace(/^\\?/,''));
  const out={type:params.get('type')||'',id:params.get('id')||'',depth:params.get('depth')||'2',showImpact:params.get('impact')!=='0',showLineage:params.get('lineage')!=='0',hidden:[],collapsed:{}};
  const hidden=params.get('hidden');
  if(hidden)out.hidden=hidden.split(',').filter(Boolean);
  const collapsed=params.get('collapsed');
  if(collapsed)collapsed.split(',').filter(Boolean).forEach(k=>{out.collapsed[k]=true});
  return out;
 }catch(e){return null}
}
function depWriteShareHash(){
 try{
  const s=dependencyExplorerState;
  if(!s.id)return;
  const hidden=Array.from(depExplorerHiddenTypes).sort().join(',');
  const collapsed=Object.keys(s.collapsed||{}).filter(k=>s.collapsed[k]).sort().join(',');
  const parts=[['type',s.type||''],['id',s.id||''],['depth',s.depth||'2'],['impact',s.showImpact?1:0],['lineage',s.showLineage?1:0],['hidden',hidden],['collapsed',collapsed]];
  history.replaceState(null,'','#/ontology?'+parts.map(p=>p[0]+'='+encodeURIComponent(p[1])).join('&'));
 }catch(e){}
}
function depApplyShareHashIfNeeded(){
 const s=dependencyExplorerState;
 if(s.id)return;
 if(typeof location==='undefined'||!location.hash||location.hash.indexOf('#/ontology')!==0)return;
 const p=depReadShareHash();
 if(!p||!p.type||!p.id)return;
 dependencyExplorerState={...s,type:p.type,id:p.id,depth:p.depth,inspectType:p.type,inspectId:p.id,showImpact:p.showImpact,showLineage:p.showLineage};
 (p.hidden||[]).forEach(t=>depExplorerHiddenTypes.add(t));
 if(p.collapsed&&Object.keys(p.collapsed).length)dependencyExplorerState={...dependencyExplorerState,collapsed:p.collapsed};
}
// Zoom / pan -----------------------------------------------------------------
function depSetZoom(z,focusX,focusY){
 const s=dependencyExplorerState;
 z=Math.min(3,Math.max(0.25,z));
 let panX=s.panX||0,panY=s.panY||0;
 if(focusX!==undefined){
  const oz=s.zoom||1;
  const cx=(focusX-panX)/oz,cy=(focusY-panY)/oz;
  panX=focusX-z*cx;panY=focusY-z*cy;
 }
 dependencyExplorerState={...s,zoom:z,panX:Math.round(panX),panY:Math.round(panY),labelMode:z<0.8?'neighborhood':'all'};
 renderAdminTab();
}
function depZoomStep(dir,focusX,focusY){depSetZoom((dependencyExplorerState.zoom||1)*(dir>0?1.2:1/1.2),focusX,focusY)}
function depResetZoomPan(){
 const s=dependencyExplorerState;
 dependencyExplorerState={...s,zoom:1,panX:0,panY:0,labelMode:'all'};
 renderAdminTab();
}
function depCenterOnNode(type,id){
 const model=depGraphModel();
 const pos=depLayoutPositions(model,depLastCanvasW,depLastCanvasH);
 const p=pos.get(depNodeKey(type,id));
 const vp=document.getElementById('depGraphViewport');
 if(!p||!vp)return;
 const z=dependencyExplorerState.zoom||1;
 dependencyExplorerState={...dependencyExplorerState,panX:Math.round(vp.clientWidth/2-z*p.x),panY:Math.round(vp.clientHeight/2-z*p.y)};
 renderAdminTab();
}
function depWirePanZoom(){
 const vp=document.getElementById('depGraphViewport'),pan=document.getElementById('depGraphPan');
 if(!vp||!pan)return;
 vp.addEventListener('wheel',e=>{
  e.preventDefault();
  const r=vp.getBoundingClientRect();
  depZoomStep(e.deltaY<0?1:-1,e.clientX-r.left,e.clientY-r.top);
 },{passive:false});
 pan.addEventListener('mousedown',e=>{
  if(e.button!==0||e.target.closest('button,[data-dep-inspect]'))return;
  depDragState={x:e.clientX,y:e.clientY,panX:dependencyExplorerState.panX||0,panY:dependencyExplorerState.panY||0};
  depDragPanEl=pan;
  e.preventDefault();
 });
 if(depPanWired)return;
 depPanWired=true;
 document.addEventListener('mousemove',e=>{
  if(!depDragState||!depDragPanEl)return;
  depDragPanEl.style.transform='translate('+(depDragState.panX+e.clientX-depDragState.x)+'px,'+(depDragState.panY+e.clientY-depDragState.y)+'px) scale('+(dependencyExplorerState.zoom||1)+')';
 });
 document.addEventListener('mouseup',e=>{
  if(!depDragState)return;
  const panX=Math.round(depDragState.panX+e.clientX-depDragState.x),panY=Math.round(depDragState.panY+e.clientY-depDragState.y);
  depDragState=null;depDragPanEl=null;
  dependencyExplorerState={...dependencyExplorerState,panX,panY};
  renderAdminTab();
 });
}
// Search ---------------------------------------------------------------------
function depSearchNodes(q){
 q=(q||'').trim().toLowerCase();
 if(!q)return[];
 return systemGraphNodes().filter(n=>(n.label||'').toLowerCase().indexOf(q)>=0).slice(0,8);
}
function depWireSearch(inputId,resultsId,onPick){
 const input=document.getElementById(inputId),box=document.getElementById(resultsId);
 if(!input||!box)return;
 input.addEventListener('input',()=>{
  const hits=depSearchNodes(input.value);
  if(!hits.length){box.hidden=true;box.innerHTML='';return}
  box.innerHTML=hits.map(n=>'<button type="button" class="ontology-search-hit" data-type="'+n.type+'" data-id="'+n.id+'">'+typeDotHtml(n.type,10)+'<span><strong>'+n.label+'</strong><br><small class="muted">'+(SYSTEM_NODE_TYPE_LABELS[n.type]||n.type)+'</small></span></button>').join('');
  box.hidden=false;
  box.querySelectorAll('.ontology-search-hit').forEach(b=>b.onclick=()=>{box.hidden=true;onPick(b.dataset.type,b.dataset.id)});
 });
 input.addEventListener('blur',()=>{setTimeout(()=>{box.hidden=true},150)});
 input.addEventListener('focus',()=>{if(box.innerHTML)box.hidden=false});
}
function depSelectSearchNode(type,id){
 dependencyExplorerState={type,id,depth:'2',inspectType:type,inspectId:id,showImpact:true,showLineage:true,expanded:{},collapsed:{}};
 renderAdminTab();
 depCenterOnNode(type,id);
}
// Find path ------------------------------------------------------------------
function depFindShortestPath(fromType,fromId,toType,toId){
 const start=depNodeKey(fromType,fromId),goal=depNodeKey(toType,toId);
 if(start===goal)return{keys:[start],edges:[]};
 const prev=new Map(),seen=new Set([start]);
 let frontier=[{type:fromType,id:fromId}],found=false;
 for(let depth=1;depth<=6&&frontier.length&&!found;depth++){
  const next=[];
  for(const node of frontier){
   const nbrs=[];
   systemGraphDirect(node.type,node.id,'dependents').forEach(h=>nbrs.push({type:h.type,id:h.id,edge:{edgeType:h.edgeType,fromType:h.type,fromId:h.id,toType:node.type,toId:node.id}}));
   systemGraphDirect(node.type,node.id,'dependencies').forEach(h=>nbrs.push({type:h.type,id:h.id,edge:{edgeType:h.edgeType,fromType:node.type,fromId:node.id,toType:h.type,toId:h.id}}));
   for(const nb of nbrs){
    const key=depNodeKey(nb.type,nb.id);
    if(seen.has(key))continue;
    seen.add(key);
    prev.set(key,{prevKey:depNodeKey(node.type,node.id),edge:nb.edge});
    if(key===goal){found=true;break}
    next.push({type:nb.type,id:nb.id});
   }
   if(found)break;
  }
  frontier=next;
 }
 if(!found)return null;
 const keys=[goal],edges=[];
 let cur=goal;
 while(cur!==start){const p=prev.get(cur);edges.unshift(p.edge);cur=p.prevKey;keys.unshift(cur)}
 return{keys,edges};
}
function depPathPick(type,id){
 const s=dependencyExplorerState;
 if(!s.pathFrom||(s.pathFrom&&s.pathTo)){
  dependencyExplorerState={...s,pathFrom:{type,id},pathTo:null,pathKeys:null,pathEdges:null,pathUnreachable:false,inspectType:type,inspectId:id};
  renderAdminTab();
  return;
 }
 if(s.pathFrom.type===type&&s.pathFrom.id===id)return;
 const to={type,id};
 const res=depFindShortestPath(s.pathFrom.type,s.pathFrom.id,type,id);
 dependencyExplorerState={...s,pathTo:to,pathKeys:res?res.keys:null,pathEdges:res?res.edges:null,pathUnreachable:!res,inspectType:type,inspectId:id};
 renderAdminTab();
}
function depPathEndpointPick(which,type,id){
 const s=dependencyExplorerState;
 const from=which==='from'?{type,id}:s.pathFrom;
 const to=which==='to'?{type,id}:s.pathTo;
 let st={...s,pathFrom:from,pathTo:to,pathKeys:null,pathEdges:null,pathUnreachable:false,inspectType:type,inspectId:id};
 if(from&&to){
  const res=depFindShortestPath(from.type,from.id,to.type,to.id);
  st={...st,pathKeys:res?res.keys:null,pathEdges:res?res.edges:null,pathUnreachable:!res};
 }
 dependencyExplorerState=st;
 renderAdminTab();
}
function depPathBarHtml(){
 const s=dependencyExplorerState;
 if(!s.pathMode)return '';
 const badge=v=>v?'<span class="badge">'+systemGraphNodeLabel(v.type,v.id)+'</span>':'<span class="muted">\u2014</span>';
 return '<div class="ontology-path-bar"><span class="muted" style="font-size:12px">Path mode: click two nodes on the canvas, or pick them here.</span>'
  +'<span class="ontology-search-wrap"><input type="search" id="depPathFrom" class="ontology-search-input" placeholder="From\u2026" autocomplete="off"><div id="depPathFromResults" class="ontology-search-results" hidden></div></span>'
  +badge(s.pathFrom)
  +'<span class="muted">\u2192</span>'
  +'<span class="ontology-search-wrap"><input type="search" id="depPathTo" class="ontology-search-input" placeholder="To\u2026" autocomplete="off"><div id="depPathToResults" class="ontology-search-results" hidden></div></span>'
  +badge(s.pathTo)
  +'<button type="button" class="btn btn-secondary" id="depClearPath">Clear path</button></div>';
}
function depPathSectionHtml(){
 const s=dependencyExplorerState;
 if(!s.pathMode&&!s.pathKeys&&!s.pathUnreachable)return '';
 let inner='';
 if(s.pathUnreachable){
  inner='<p class="muted" style="font-size:12px">No path within 6 hops between the two selected nodes.</p>';
 }else if(s.pathKeys&&s.pathKeys.length>1){
  inner='<div class="ontology-path-chain">'+(s.pathEdges||[]).map(e=>'<div class="ontology-path-hop">'+systemGraphNodeLabel(e.fromType,e.fromId)+' \u2014'+edgeTypeLabel(e.edgeType)+'\u2192 '+systemGraphNodeLabel(e.toType,e.toId)+'</div>').join('')+'</div>';
 }else if(s.pathFrom){
  inner='<p class="muted" style="font-size:12px">From <strong>'+systemGraphNodeLabel(s.pathFrom.type,s.pathFrom.id)+'</strong> \u2014 pick a second node.</p>';
 }else{
  inner='<p class="muted" style="font-size:12px">Pick two nodes to trace a path.</p>';
 }
 return '<div class="ontology-detail-section"><div class="ontology-detail-section-label">Path</div>'+inner+'<button type="button" class="btn btn-secondary" id="depClearPathBtn" style="margin-top:8px">Clear path</button></div>';
}
// Change-impact simulation ----------------------------------------------------
// Read-only: traverses the graph via systemGraphDirect only. It never writes
// to `data`, never calls save(), and never touches localStorage.
function depSimulateImpact(type,id){
 const seen=new Set(),out=[];
 const stack=[{type,id}];
 while(stack.length){
  const n=stack.pop();
  const key=depNodeKey(n.type,n.id);
  if(seen.has(key))continue;
  seen.add(key);
  out.push(key);
  const dependents=systemGraphDirect(n.type,n.id,'dependents');
  for(let i=0;i<dependents.length;i++)stack.push({type:dependents[i].type,id:dependents[i].id});
 }
 return out;
}
function depSimBannerHtml(){
 const s=dependencyExplorerState;
 if(!s.simKeys||!s.simRoot)return '';
 const n=s.simKeys.length;
 const label=systemGraphNodeLabel(s.simRoot.type,s.simRoot.id);
 return '<div class="ontology-sim-banner"><span><strong>Preview only \u2014 no changes applied.</strong> '+n+' component'+(n===1?'':'s')+' would be affected by '+s.simChange+' of '+label+'.</span><button type="button" class="btn btn-secondary" id="depSimOpenAdmin">Open in its admin screen</button><button type="button" class="btn btn-secondary" id="depSimClear">Clear simulation</button></div>';
}
// Toolbar wiring ---------------------------------------------------------------
function depWireToolbar(body){
 const zi=document.getElementById('depZoomIn');if(zi)zi.onclick=()=>depZoomStep(1);
 const zo=document.getElementById('depZoomOut');if(zo)zo.onclick=()=>depZoomStep(-1);
 const fit=document.getElementById('depFit');if(fit)fit.onclick=()=>depResetZoomPan();
 const rv=document.getElementById('depResetView');
 if(rv)rv.onclick=()=>{dependencyExplorerState={...dependencyExplorerState,zoom:1,panX:0,panY:0,labelMode:'all',pathMode:false,pathFrom:null,pathTo:null,pathKeys:null,pathEdges:null,pathUnreachable:false,simKeys:null,simChange:null,simRoot:null};renderAdminTab()};
 depWirePanZoom();
 depWireSearch('depSearch','depSearchResults',(t,i)=>depSelectSearchNode(t,i));
 const pt=document.getElementById('depPathToggle');
 if(pt)pt.onclick=()=>{const s=dependencyExplorerState;dependencyExplorerState={...s,pathMode:!s.pathMode,pathFrom:null,pathTo:null,pathKeys:null,pathEdges:null,pathUnreachable:false};renderAdminTab()};
 depWireSearch('depPathFrom','depPathFromResults',(t,i)=>depPathEndpointPick('from',t,i));
 depWireSearch('depPathTo','depPathToResults',(t,i)=>depPathEndpointPick('to',t,i));
 body.querySelectorAll('#depClearPath,#depClearPathBtn').forEach(b=>b.onclick=()=>{dependencyExplorerState={...dependencyExplorerState,pathMode:false,pathFrom:null,pathTo:null,pathKeys:null,pathEdges:null,pathUnreachable:false};renderAdminTab()});
 const simToggle=document.getElementById('depSimToggle');
 const simPicker=document.getElementById('depSimPicker');
 if(simToggle)simToggle.onclick=()=>{if(simPicker)simPicker.hidden=!simPicker.hidden};
 const simRun=document.getElementById('depSimRun');
 if(simRun)simRun.onclick=()=>{
  const s=dependencyExplorerState;
  if(!s.inspectType||!s.inspectId){toast('Select a node first');return}
  const chg=document.getElementById('depSimChange').value;
  dependencyExplorerState={...s,simKeys:depSimulateImpact(s.inspectType,s.inspectId),simChange:chg,simRoot:{type:s.inspectType,id:s.inspectId}};
  renderAdminTab();
 };
 const simClear=document.getElementById('depSimClear');
 if(simClear)simClear.onclick=()=>{dependencyExplorerState={...dependencyExplorerState,simKeys:null,simChange:null,simRoot:null};renderAdminTab()};
 const simOpen=document.getElementById('depSimOpenAdmin');
 if(simOpen)simOpen.onclick=()=>{const s=dependencyExplorerState;if(s.simRoot){adminTab=NODE_TYPE_ADMIN_TAB[s.simRoot.type];adminView='tool';renderView()}};
 const copyBtn=document.getElementById('depCopyLink');
 if(copyBtn)copyBtn.onclick=()=>{
  depWriteShareHash();
  const done=()=>toast('Link copied');
  if(navigator.clipboard&&navigator.clipboard.writeText)navigator.clipboard.writeText(location.href).then(done,()=>toast('Copy failed'));
  else toast('Copy not available');
 };
 const btn360=document.getElementById('depOpen360');
 if(btn360)btn360.onclick=()=>{if(typeof openObject360ForEntity==='function')openObject360ForEntity(dependencyExplorerState.inspectId);else toast('Object 360 is not available here')};
}
function depExpandableCount(model,type,id){"""


if __name__ == '__main__':
    main()
