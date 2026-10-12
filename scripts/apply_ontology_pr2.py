#!/usr/bin/env python3
"""Apply PR 2 (Ontology Graph clustered graph UI) edits to app.js.

Self-contained: the PR2 code block is inlined below (no external files).

Runs from the repo root: PATH='app.js'. Every replacement is anchored on
exact byte sequences from post-PR1 main (merge d27bb9c) and asserts an exact
match count, failing loudly otherwise. Ground truth on bytes (PR1 lesson):
HTML attrs and template literals use PLAIN quotes; only single-quoted JS
strings use \\' escapes.
"""
import sys

PATH = 'app.js'
src = open(PATH).read()
fails = []

def rep_once(old, new, name):
    global src
    n = src.count(old)
    if n != 1:
        fails.append('%s: expected 1 match, found %d' % (name, n))
        return
    src = src.replace(old, new, 1)
    print('ok: %s' % name)

def rep_slice(start_marker, end_marker, new, name):
    global src
    a = src.count(start_marker)
    b = src.count(end_marker)
    if a != 1 or b != 1:
        fails.append('%s: markers found %d/%d, expected 1/1' % (name, a, b))
        return
    i = src.index(start_marker)
    j = src.index(end_marker)
    if j <= i:
        fails.append('%s: end marker before start marker' % name)
        return
    src = src[:i] + new + src[j:]
    print('ok: %s' % name)

# --- 1. state: track incremental expansions + collapsed clusters ---
rep_once(
    "let dependencyExplorerState={type:'custom_object',id:null,depth:'2',inspectType:null,inspectId:null,showImpact:true,showLineage:true};",
    "let dependencyExplorerState={type:'custom_object',id:null,depth:'2',inspectType:null,inspectId:null,showImpact:true,showLineage:true,expanded:{},collapsed:{}};",
    'state expanded/collapsed')

# --- 2. legend: count badges per type ---
rep_once(
    """function legendHtml(hidden){
 hidden=hidden||new Set();
 return `<div class="ontology-legend">${Object.keys(SYSTEM_NODE_TYPE_LABELS).map(t=>`<button type="button" class="ontology-legend-item${hidden.has(t)?' ontology-legend-item-off':''}" data-legend-toggle="${t}" title="${hidden.has(t)?'Show':'Hide'} ${SYSTEM_NODE_TYPE_LABELS[t]}">${typeDotHtml(t)}${SYSTEM_NODE_TYPE_LABELS[t]}</button>`).join('')}</div>`;
}""",
    """function legendHtml(hidden,counts){
 hidden=hidden||new Set();counts=counts||{};
 return `<div class="ontology-legend">${Object.keys(SYSTEM_NODE_TYPE_LABELS).map(t=>`<button type="button" class="ontology-legend-item${hidden.has(t)?' ontology-legend-item-off':''}" data-legend-toggle="${t}" title="${hidden.has(t)?'Show':'Hide'} ${SYSTEM_NODE_TYPE_LABELS[t]}${counts[t]?' ('+counts[t]+' shown)':''}">${typeDotHtml(t)}${SYSTEM_NODE_TYPE_LABELS[t]}${counts[t]?`<span class="ontology-legend-count">${counts[t]}</span>`:''}</button>`).join('')}</div>`;
}""",
    'legend counts')

# --- 3. replace floating inspector (comment + inspectorCardHtml) with PR2 engine ---
pr2_block = r'''// ===== Ontology Graph PR2 (epic #359, issues #366/#367/#368): clustered
// force-directed graph UI. Nodes group by component type with labeled,
// collapsible cluster halos; edges render with typed pill labels on canvas
// (access edges dashed with a lock glyph); the floating inspector card is
// replaced by a pinned right detail panel; "Show what's around it"
// incrementally expands one hop around the selected node.
const DEP_ACCESS_EDGE_TYPES=['can_read','can_update','can_delete','can_invoke'];
// Cluster anchor order around the canvas - related types sit adjacent so
// their halos end up near each other without merging (screens/pages
// co-render; agents/teams orchestrate together).
const DEP_CLUSTER_ORDER=['custom_object','custom_field','relationship','business_rule','workflow','screen_layout','page_layout','ai_agent','execution_graph','business_glossary_term','metric_definition','test_case_definition','action_type_definition'];
function depNodeKey(type,id){return type+':'+id}
function depSplitKey(key){const i=key.indexOf(':');return [key.slice(0,i),key.slice(i+1)]}
function depHashStr(s){let h=2166136261;for(let i=0;i<s.length;i++){h^=s.charCodeAt(i);h=Math.imul(h,16777619)}return h>>>0}
function depSeededRandom(seed){let a=seed>>>0;return function(){a|=0;a=a+0x6D2B79F5|0;let t=Math.imul(a^a>>>15,1|a);t=t+Math.imul(t^t>>>7,61|t)^t;return((t^t>>>14)>>>0)/4294967296}}
function depNodeOrigin(type,id){
 if(type==='custom_object')return ONTOLOGY_BUILTIN_TYPES.some(b=>b[0]===id)?'OOB':'Custom';
 return 'Custom';
}
function depIsAccessEdge(edgeType){return DEP_ACCESS_EDGE_TYPES.indexOf(edgeType)>=0}
// Which traversal directions are active: the impact/lineage toggles, or -
// when both are off - both directions at depth 1 (the old "direct" view).
function depActiveDirs(){
 const s=dependencyExplorerState;
 if(s.showImpact||s.showLineage)return [s.showImpact?'dependents':null,s.showLineage?'dependencies':null].filter(Boolean);
 return ['dependents','dependencies'];
}
// Unified graph model with real per-hop edges: BFS from the root honoring
// toggles + depth selector (fan-out guard: >150 new nodes per expansion
// reports the rest via `truncated`), plus one-hop neighborhoods for
// incrementally expanded nodes. Collapsed cluster types become single
// stacked pseudo-nodes via depApplyCollapse.
function depGraphModel(){
 const s=dependencyExplorerState;
 const nodes=new Map(),edges=[];
 const edgeSeen=new Set();
 const addNode=(type,id,depth)=>{const k=depNodeKey(type,id);const n=nodes.get(k);if(!n)nodes.set(k,{key:k,type,id,depth,label:systemGraphNodeLabel(type,id),origin:depNodeOrigin(type,id)});else if(depth<n.depth)n.depth=depth};
 const addEdge=(ft,fi,tt,ti,edgeType)=>{const k=ft+':'+fi+'>'+tt+':'+ti+':'+edgeType;if(edgeSeen.has(k))return;edgeSeen.add(k);edges.push({fromKey:depNodeKey(ft,fi),toKey:depNodeKey(tt,ti),edgeType})};
 addNode(s.type,s.id,0);
 const dirs=depActiveDirs();
 const bothOff=!s.showImpact&&!s.showLineage;
 const maxDepth=bothOff?1:depMaxDepth();
 const seen=new Set([depNodeKey(s.type,s.id)]);
 let frontier=[{type:s.type,id:s.id}],truncated=0;
 for(let depth=1;depth<=maxDepth&&frontier.length;depth++){
  const next=[];
  for(const node of frontier)for(const dir of dirs){
   const hits=systemGraphDirect(node.type,node.id,dir);
   let added=0;
   for(const h of hits){
    if(dir==='dependents')addEdge(h.type,h.id,node.type,node.id,h.edgeType);
    else addEdge(node.type,node.id,h.type,h.id,h.edgeType);
    const k=depNodeKey(h.type,h.id);
    if(!seen.has(k)){
     if(added>=150){truncated++;continue}
     added++;seen.add(k);addNode(h.type,h.id,depth);next.push({type:h.type,id:h.id});
    }
   }
  }
  frontier=next;
 }
 // Incremental expansions ("Show what's around it"): one hop around each
 // expanded node, honoring the direction toggles.
 for(const exKey of Object.keys(s.expanded||{})){
  const parts=depSplitKey(exKey),et=parts[0],ei=parts[1];
  const exNode=nodes.get(exKey);
  const baseDepth=exNode?exNode.depth:1;
  for(const dir of dirs)for(const h of systemGraphDirect(et,ei,dir)){
   if(dir==='dependents')addEdge(h.type,h.id,et,ei,h.edgeType);
   else addEdge(et,ei,h.type,h.id,h.edgeType);
   const k=depNodeKey(h.type,h.id);
   if(!seen.has(k)){seen.add(k);addNode(h.type,h.id,baseDepth+1)}
  }
 }
 const model={nodes:Array.from(nodes.values()),edges,truncated};
 depApplyCollapse(model);
 return model;
}
// Collapse cluster halos: replace a collapsed type's nodes with one stacked
// pseudo-node; remap edges onto it, drop internal edges, dedupe.
function depApplyCollapse(model){
 const collapsed=dependencyExplorerState.collapsed||{};
 const types=Object.keys(collapsed).filter(t=>collapsed[t]);
 if(!types.length)return;
 const remap=new Map();
 types.forEach(t=>{
  const members=model.nodes.filter(n=>n.type===t&&n.id!=='__cluster__');
  if(!members.length)return;
  const label=SYSTEM_NODE_TYPE_LABELS[t]||t;
  const pseudo={key:'cluster:'+t,type:t,id:'__cluster__',clusterType:t,depth:Math.min.apply(null,members.map(n=>n.depth)),label:label+' · '+members.length,origin:'',count:members.length};
  model.nodes.push(pseudo);
  members.forEach(n=>remap.set(n.key,pseudo.key));
 });
 if(!remap.size)return;
 const liveKeys=new Set();
 model.nodes.forEach(n=>{if(!remap.has(n.key))liveKeys.add(n.key)});
 model.nodes=model.nodes.filter(n=>!remap.has(n.key));
 const seen=new Set();
 model.edges=model.edges
  .map(e=>({fromKey:remap.get(e.fromKey)||e.fromKey,toKey:remap.get(e.toKey)||e.toKey,edgeType:e.edgeType}))
  .filter(e=>e.fromKey!==e.toKey&&liveKeys.has(e.fromKey)&&liveKeys.has(e.toKey))
  .filter(e=>{const k=e.fromKey+'>'+e.toKey+':'+e.edgeType;if(seen.has(k))return false;seen.add(k);return true});
}
// Seeded force-directed layout with per-type cluster gravity: same-type
// nodes pull toward a shared anchor on an ellipse, so each type forms a
// visible clump. Deterministic per node-set (seeded PRNG), so re-renders
// (e.g. selecting a node) never reshuffle the graph.
function depLayoutPositions(model,W,H){
 const visible=model.nodes;
 const rng=depSeededRandom(depHashStr(visible.map(n=>n.key).sort().join('|')));
 const anchors={};
 DEP_CLUSTER_ORDER.forEach((t,i)=>{
  const a=(i/DEP_CLUSTER_ORDER.length)*Math.PI*2-Math.PI/2;
  anchors[t]={x:W/2+Math.cos(a)*W*0.36,y:H/2+Math.sin(a)*H*0.36};
 });
 const pos=new Map();
 const rootKey=depNodeKey(dependencyExplorerState.type,dependencyExplorerState.id);
 visible.forEach(n=>{
  if(n.key===rootKey){pos.set(n.key,{x:W/2,y:H/2});return}
  const a=anchors[n.type]||{x:W/2,y:H/2};
  pos.set(n.key,{x:a.x+(rng()-0.5)*170,y:a.y+(rng()-0.5)*170});
 });
 const TICKS=visible.length>260?60:110;
 const K=54,LINK_LEN=125;
 for(let t=0;t<TICKS;t++){
  const disp=new Map(visible.map(n=>[n.key,{x:0,y:0}]));
  for(let i=0;i<visible.length;i++)for(let j=i+1;j<visible.length;j++){
   const a=visible[i],b=visible[j];
   const pa=pos.get(a.key),pb=pos.get(b.key);
   let dx=pa.x-pb.x,dy=pa.y-pb.y,d2=dx*dx+dy*dy;
   if(d2<1){dx=rng()-0.5;dy=rng()-0.5;d2=1}
   const d=Math.sqrt(d2);
   const f=Math.min(K*K/d,9);
   const fx=f*dx/d,fy=f*dy/d;
   const da=disp.get(a.key),db=disp.get(b.key);
   da.x+=fx;da.y+=fy;db.x-=fx;db.y-=fy;
  }
  model.edges.forEach(e=>{
   const pa=pos.get(e.fromKey),pb=pos.get(e.toKey);
   if(!pa||!pb)return;
   const dx=pb.x-pa.x,dy=pb.y-pa.y;
   const d=Math.sqrt(dx*dx+dy*dy)||1;
   const f=(d-LINK_LEN)*0.055;
   const fx=f*dx/d,fy=f*dy/d;
   const da=disp.get(e.fromKey),db=disp.get(e.toKey);
   da.x+=fx;da.y+=fy;db.x-=fx;db.y-=fy;
  });
  visible.forEach(n=>{
   if(n.key===rootKey)return;
   const p=pos.get(n.key),d=disp.get(n.key);
   const a=anchors[n.type]||{x:W/2,y:H/2};
   d.x+=(a.x-p.x)*0.03+(W/2-p.x)*0.006;
   d.y+=(a.y-p.y)*0.03+(H/2-p.y)*0.006;
  });
  visible.forEach(n=>{
   if(n.key===rootKey)return;
   const p=pos.get(n.key),d=disp.get(n.key);
   p.x=Math.max(34,Math.min(W-34,p.x+d.x*0.5));
   p.y=Math.max(52,Math.min(H-30,p.y+d.y*0.5));
  });
 }
 return pos;
}
// Andrew's monotone chain convex hull, for cluster halos.
function depConvexHull(pts){
 if(pts.length<3)return pts.slice();
 const s=pts.slice().sort((a,b)=>a.x-b.x||a.y-b.y);
 const cross=(o,a,b)=>(a.x-o.x)*(b.y-o.y)-(a.y-o.y)*(b.x-o.x);
 const lower=[],upper=[];
 for(const p of s){while(lower.length>=2&&cross(lower[lower.length-2],lower[lower.length-1],p)<=0)lower.pop();lower.push(p)}
 for(let i=s.length-1;i>=0;i--){const p=s[i];while(upper.length>=2&&cross(upper[upper.length-2],upper[upper.length-1],p)<=0)upper.pop();upper.push(p)}
 lower.pop();upper.pop();
 return lower.concat(upper);
}
// Renders halos + edges + edge labels + nodes into the graph canvas.
function depRenderGraph(model,W,H){
 const s=dependencyExplorerState;
 const pos=depLayoutPositions(model,W,H);
 const rootKey=depNodeKey(s.type,s.id);
 const byKey=new Map(model.nodes.map(n=>[n.key,n]));
 const haloSvg=[],haloLabels=[];
 const clusters=new Map();
 model.nodes.forEach(n=>{if(n.id==='__cluster__')return;const l=clusters.get(n.type)||[];l.push(n);clusters.set(n.type,l)});
 clusters.forEach((members,type)=>{
  if(depExplorerHiddenTypes.has(type))return;
  const pts=members.map(n=>pos.get(n.key)).filter(Boolean);
  if(!pts.length)return;
  const color=SYSTEM_NODE_TYPE_COLORS[type]||'#888';
  const label=(SYSTEM_NODE_TYPE_LABELS[type]||type)+' · '+members.length;
  let d;
  if(pts.length===1){const p=pts[0];d='M '+(p.x-36).toFixed(1)+' '+p.y.toFixed(1)+' a 36 36 0 1 0 72 0 a 36 36 0 1 0 -72 0'}
  else{
   const hull=depConvexHull(pts);
   const cx=hull.reduce((a,p)=>a+p.x,0)/hull.length,cy=hull.reduce((a,p)=>a+p.y,0)/hull.length;
   const padded=hull.map(p=>{const dx=p.x-cx,dy=p.y-cy,dd=Math.sqrt(dx*dx+dy*dy)||1;return{x:p.x+dx/dd*28,y:p.y+dy/dd*28}});
   d='M '+padded.map(p=>p.x.toFixed(1)+' '+p.y.toFixed(1)).join(' L ')+' Z';
  }
  haloSvg.push('<path d="'+d+'" fill="'+color+'" fill-opacity="0.10" stroke="'+color+'" stroke-opacity="0.35" stroke-width="1.5"/>');
  const lx=(Math.min.apply(null,pts.map(p=>p.x))+Math.max.apply(null,pts.map(p=>p.x)))/2;
  const ly=Math.min.apply(null,pts.map(p=>p.y))-16;
  haloLabels.push('<button type="button" class="ontology-halo-label" data-dep-collapse="'+type+'" style="left:'+lx.toFixed(1)+'px;top:'+ly.toFixed(1)+'px" title="Collapse '+label+' into a single node">'+label+' ▾</button>');
 });
 // Collapsed clusters: dashed halo + expand affordance.
 model.nodes.filter(n=>n.id==='__cluster__').forEach(n=>{
  const p=pos.get(n.key);if(!p)return;
  const color=SYSTEM_NODE_TYPE_COLORS[n.type]||'#888';
  haloSvg.push('<circle cx="'+p.x.toFixed(1)+'" cy="'+p.y.toFixed(1)+'" r="36" fill="'+color+'" fill-opacity="0.10" stroke="'+color+'" stroke-opacity="0.45" stroke-width="1.5" stroke-dasharray="5 4"/>');
  haloLabels.push('<button type="button" class="ontology-halo-label" data-dep-collapse="'+n.type+'" style="left:'+p.x.toFixed(1)+'px;top:'+(p.y-48).toFixed(1)+'px" title="Expand '+n.label+'">'+n.label+' ▸</button>');
 });
 const edgeSvg=[],edgeLabels=[];
 model.edges.forEach(e=>{
  const a=pos.get(e.fromKey),b=pos.get(e.toKey);
  if(!a||!b)return;
  const an=byKey.get(e.fromKey),bn=byKey.get(e.toKey);
  if(!an||!bn)return;
  if(depExplorerHiddenTypes.has(an.type)||depExplorerHiddenTypes.has(bn.type))return;
  const access=depIsAccessEdge(e.edgeType);
  const color=SYSTEM_NODE_TYPE_COLORS[an.type]||'#888';
  edgeSvg.push('<line x1="'+a.x.toFixed(1)+'" y1="'+a.y.toFixed(1)+'" x2="'+b.x.toFixed(1)+'" y2="'+b.y.toFixed(1)+'" stroke="'+color+'" stroke-opacity="0.5" stroke-width="1.5"'+(access?' stroke-dasharray="6 4"':'')+'><title>'+an.label+' — '+edgeTypeLabel(e.edgeType)+' → '+bn.label+'</title></line>');
  edgeLabels.push('<div class="ontology-edge-label'+(access?' ontology-edge-label-access':'')+'" style="left:'+((a.x+b.x)/2).toFixed(1)+'px;top:'+((a.y+b.y)/2).toFixed(1)+'px" title="'+an.label+' — '+edgeTypeLabel(e.edgeType)+' → '+bn.label+'">'+(access?'🔒 ':'')+edgeTypeLabel(e.edgeType)+'</div>');
 });
 const nodeDivs=[];
 model.nodes.forEach(n=>{
  const p=pos.get(n.key);if(!p)return;
  const isRoot=n.key===rootKey;
  const isDimmed=depExplorerHiddenTypes.has(n.type);
  const isInspecting=!isRoot&&n.type===s.inspectType&&n.id===s.inspectId;
  const color=SYSTEM_NODE_TYPE_COLORS[n.type]||'#888';
  const short=n.label.length>18?n.label.slice(0,17)+'…':n.label;
  if(n.id==='__cluster__'){
   nodeDivs.push('<div class="ontology-node-cluster" data-dep-collapse="'+n.type+'" style="left:'+p.x.toFixed(1)+'px;top:'+p.y.toFixed(1)+'px;background:'+color+'" title="'+n.label+' — click to expand"><span>'+n.count+'</span></div>');
   nodeDivs.push('<div class="ontology-node-caption" style="left:'+p.x.toFixed(1)+'px;top:'+(p.y+26).toFixed(1)+'px">'+short+'</div>');
   return;
  }
  const size=isRoot?ONTOLOGY_ROOT_SIZE:ONTOLOGY_NODE_SIZE;
  nodeDivs.push('<div class="ontology-node-dot'+(isRoot?' ontology-node-root':'')+(isInspecting?' ontology-node-inspecting':'')+(isDimmed?' ontology-node-dimmed':'')+'" data-dep-inspect="'+n.type+':'+n.id+'" style="left:'+(p.x-size/2).toFixed(1)+'px;top:'+(p.y-size/2).toFixed(1)+'px;width:'+size+'px;height:'+size+'px;background:'+color+'" title="'+(SYSTEM_NODE_TYPE_LABELS[n.type]||n.type)+': '+n.label+'"></div>');
  nodeDivs.push('<div class="ontology-node-caption'+(isDimmed?' ontology-node-dimmed':'')+'" style="left:'+p.x.toFixed(1)+'px;top:'+(p.y+size/2+4).toFixed(1)+'px" title="'+n.label+'">'+short+'</div>');
 });
 return '<svg width="'+W+'" height="'+H+'" style="position:absolute;top:0;left:0;pointer-events:none">'+haloSvg.join('')+edgeSvg.join('')+'</svg>'+haloLabels.join('')+edgeLabels.join('')+nodeDivs.join('');
}
// Pinned right detail panel (replaces the floating inspector card): type,
// badges, relationships grouped by edge label with direction, fields for
// objects, and quick actions.
function depDetailPanelHtml(model){
 const s=dependencyExplorerState;
 if(!s.inspectType||!s.inspectId)return '<div class="ontology-detail-panel"><p class="muted" style="font-size:13px">Select a node to inspect it.</p></div>';
 const node=model.nodes.find(n=>n.type===s.inspectType&&n.id===s.inspectId);
 const label=node?node.label:systemGraphNodeLabel(s.inspectType,s.inspectId);
 const dependents=systemGraphDirect(s.inspectType,s.inspectId,'dependents');
 const dependencies=systemGraphDirect(s.inspectType,s.inspectId,'dependencies');
 const groups=new Map();
 [...dependents.map(h=>({type:h.type,id:h.id,edgeType:h.edgeType,dir:'in'})),...dependencies.map(h=>({type:h.type,id:h.id,edgeType:h.edgeType,dir:'out'}))].forEach(h=>{
  const list=groups.get(h.edgeType)||[];list.push(h);groups.set(h.edgeType,list);
 });
 const isRoot=s.inspectType===s.type&&s.inspectId===s.id;
 const definition=s.inspectType==='custom_object'?ontologyDescriptionFor(s.inspectId):null;
 const fields=s.inspectType==='custom_object'?(data.customFields||[]).filter(f=>f.entity===s.inspectId):[];
 const agent=s.inspectType==='ai_agent'?(data.aiAgents||[]).find(a=>a.id===s.inspectId):null;
 const constrained=!!agent&&systemGraphEdges().some(e=>e.edgeType==='constrained_by'&&e.toType==='ai_agent'&&e.toId===s.inspectId);
 const expandable=depExpandableCount(model,s.inspectType,s.inspectId);
 return '<div class="ontology-detail-panel">'
  +'<div class="ontology-inspector-head">'+typeDotHtml(s.inspectType,14)+'<div><strong>'+label+'</strong><div class="ontology-inspector-subtitle">'+(SYSTEM_NODE_TYPE_LABELS[s.inspectType]||s.inspectType)+'</div></div></div>'
  +'<div class="ontology-detail-badges"><span class="badge">'+depNodeOrigin(s.inspectType,s.inspectId)+'</span>'+(agent?'<span class="badge">'+((agent.objectTypeScopes||[]).length?'Scoped':'Unscoped')+'</span>':'')+(constrained?'<span class="badge" title="A business rule constrains what this agent may do">policy: constrained</span>':'')+'</div>'
  +(definition?'<p class="ontology-inspector-definition">'+definition+'</p>':'')
  +'<div class="ontology-detail-actions">'
  +'<button type="button" class="btn btn-primary" id="depOpenAdmin">Open in its admin screen</button>'
  +'<button type="button" class="btn btn-secondary" id="depExpand"'+(expandable?'':' disabled')+' title="'+(expandable?('Reveal '+expandable+' more neighbor'+(expandable===1?'':'s')+' around this node'):'Every neighbor is already shown')+'">Show what\'s around it'+(expandable?' ('+expandable+')':'')+'</button>'
  +(isRoot?'':'<button type="button" class="btn btn-secondary" id="depRecenter">Re-center graph here</button>')
  +'</div>'
  +(fields.length?'<div class="ontology-detail-section"><div class="ontology-detail-section-label">Fields · '+fields.length+'</div>'+fields.map(f=>'<button type="button" class="ontology-inspector-bullet" data-dep-inspect="custom_field:'+f.id+'">'+typeDotHtml('custom_field')+f.label+'</button>').join('')+'</div>':'')
  +'<div class="ontology-detail-section"><div class="ontology-detail-section-label">Relationships</div>'
  +(groups.size?Array.from(groups.entries()).map(([edgeType,members])=>{
   const ins=members.filter(m=>m.dir==='in'),outs=members.filter(m=>m.dir==='out');
   const row=(m,arrow,tip)=>'<button type="button" class="ontology-inspector-bullet" data-dep-bullet="'+m.type+':'+m.id+'" title="'+tip+'"><span class="ontology-dir-arrow">'+arrow+'</span>'+typeDotHtml(m.type)+systemGraphNodeLabel(m.type,m.id)+'</button>';
   return '<div class="ontology-inspector-group"><div class="ontology-inspector-group-label">'+edgeTypeLabel(edgeType)+' · '+members.length+'</div>'
    +ins.map(m=>row(m,'←','Points at this node — click to re-center the graph here')).join('')
    +outs.map(m=>row(m,'→','This node points at it — click to re-center the graph here')).join('')+'</div>';
  }).join(''):'<p class="muted" style="font-size:12px">Nothing else synced depends on or is depended on by this.</p>')
  +'</div></div>';
}
function depExpandableCount(model,type,id){
 const inModel=new Set(model.nodes.map(n=>n.key));
 let n=0;
 for(const dir of depActiveDirs())for(const h of systemGraphDirect(type,id,dir)){
  if(!inModel.has(depNodeKey(h.type,h.id)))n++;
 }
 return n;
}
// "Show what's around it": mark the inspected node expanded (one more hop),
// then lift the depth selector to the new max depth per the spec.
function depExpandAround(){
 const s=dependencyExplorerState;
 const expanded={...(s.expanded||{})};
 expanded[depNodeKey(s.inspectType,s.inspectId)]=true;
 dependencyExplorerState={...s,expanded};
 const model=depGraphModel();
 const maxDepth=model.nodes.reduce((m,n)=>Math.max(m,n.depth),0);
 let depth=s.depth;
 if(maxDepth>3)depth='all';
 else if(['1','2','3'].indexOf(depth)>=0&&parseInt(depth,10)<maxDepth)depth=String(maxDepth);
 dependencyExplorerState={...dependencyExplorerState,depth};
 renderAdminTab();
}
'''
rep_slice(
    "// The floating inspector targets `inspectType`/`inspectId` - which starts",
    "\nfunction dependencyExplorerTab(body){",
    pr2_block + "\n",
    'pr2 engine block (replaces floating inspector)')

# --- 4. replace graph render block with unified clustered render ---
rep_slice(
    " let graphHtml='';",
    "\n\n body.innerHTML=`<div class=\"panel\">",
    """ let graphHtml='';
 let truncatedHits=0;
 const depCounts={};
 if(dependencyExplorerState.id){
  const depModel=depGraphModel();
  truncatedHits=depModel.truncated;
  depModel.nodes.forEach(n=>{depCounts[n.type]=(depCounts[n.type]||0)+1});
  const depW=Math.max(880,Math.min(1500,Math.round(560+depModel.nodes.length*5)));
  const depH=Math.max(600,Math.min(940,Math.round(440+depModel.nodes.length*3.2)));
  graphHtml=`<div class="ontology-graph-layout"><div class="ontology-graph-wrap" style="flex:1;min-width:0"><div class="ontology-graph-canvas" style="width:${depW}px;height:${depH}px">${depRenderGraph(depModel,depW,depH)}</div></div>${depDetailPanelHtml(depModel)}</div>`;
 }
""",
    'unified clustered graph render')

# --- 5. legend call site: pass counts ---
rep_once(
    "  ${legendHtml(depExplorerHiddenTypes)}",
    "  ${legendHtml(depExplorerHiddenTypes,depCounts)}",
    'legend call site')

# --- 6. wiring: state resets, collapse toggle, incremental expand ---
rep_once(
    """ $('#depType').onchange=(e)=>{const t=e.target.value;const first=nodes.find(n=>n.type===t);dependencyExplorerState={type:t,id:first?first.id:null,depth:dependencyExplorerState.depth||'2',inspectType:t,inspectId:first?first.id:null,showImpact:true,showLineage:true};renderAdminTab()};
 $('#depComponent')&&($('#depComponent').onchange=(e)=>{dependencyExplorerState={...dependencyExplorerState,id:e.target.value,inspectType:dependencyExplorerState.type,inspectId:e.target.value,showImpact:true,showLineage:true};renderAdminTab()});
 $('#depDepth')&&($('#depDepth').onchange=(e)=>{dependencyExplorerState={...dependencyExplorerState,depth:e.target.value};renderAdminTab()});
 body.querySelectorAll('[data-dep-toggle]').forEach(b=>b.onclick=()=>{const k=b.dataset.depToggle;dependencyExplorerState={...dependencyExplorerState,[k]:!dependencyExplorerState[k]};renderAdminTab()});
 body.querySelectorAll('[data-dep-inspect]').forEach(b=>b.onclick=()=>{const [type,id]=b.dataset.depInspect.split(':');dependencyExplorerState={...dependencyExplorerState,inspectType:type,inspectId:id};renderAdminTab()});
 body.querySelectorAll('[data-dep-bullet]').forEach(b=>b.onclick=()=>{const [type,id]=b.dataset.depBullet.split(':');dependencyExplorerState={type,id,depth:dependencyExplorerState.depth||'2',inspectType:type,inspectId:id,showImpact:true,showLineage:true};renderAdminTab()});
 body.querySelectorAll('[data-legend-toggle]').forEach(b=>b.onclick=()=>{const t=b.dataset.legendToggle;if(depExplorerHiddenTypes.has(t))depExplorerHiddenTypes.delete(t);else depExplorerHiddenTypes.add(t);renderAdminTab()});
 const recenterBtn=$('#depRecenter'); if(recenterBtn)recenterBtn.onclick=()=>{dependencyExplorerState={...dependencyExplorerState,type:dependencyExplorerState.inspectType,id:dependencyExplorerState.inspectId,showImpact:true,showLineage:true};renderAdminTab()};
 const openBtn=$('#depOpenAdmin'); if(openBtn)openBtn.onclick=()=>{adminTab=NODE_TYPE_ADMIN_TAB[dependencyExplorerState.inspectType];adminView='tool';renderView()};
 makeInspectorCardDraggable($('#depInspectorCard'),$('#depInspectorHead'),(left,top)=>{depExplorerCardPos={left,top}});""",
    """ $('#depType').onchange=(e)=>{const t=e.target.value;const first=nodes.find(n=>n.type===t);dependencyExplorerState={type:t,id:first?first.id:null,depth:dependencyExplorerState.depth||'2',inspectType:t,inspectId:first?first.id:null,showImpact:true,showLineage:true,expanded:{},collapsed:{}};renderAdminTab()};
 $('#depComponent')&&($('#depComponent').onchange=(e)=>{dependencyExplorerState={...dependencyExplorerState,id:e.target.value,inspectType:dependencyExplorerState.type,inspectId:e.target.value,showImpact:true,showLineage:true,expanded:{},collapsed:{}};renderAdminTab()});
 $('#depDepth')&&($('#depDepth').onchange=(e)=>{dependencyExplorerState={...dependencyExplorerState,depth:e.target.value};renderAdminTab()});
 body.querySelectorAll('[data-dep-toggle]').forEach(b=>b.onclick=()=>{const k=b.dataset.depToggle;dependencyExplorerState={...dependencyExplorerState,[k]:!dependencyExplorerState[k]};renderAdminTab()});
 body.querySelectorAll('[data-dep-inspect]').forEach(b=>b.onclick=()=>{const [type,id]=b.dataset.depInspect.split(':');dependencyExplorerState={...dependencyExplorerState,inspectType:type,inspectId:id};renderAdminTab()});
 body.querySelectorAll('[data-dep-bullet]').forEach(b=>b.onclick=()=>{const [type,id]=b.dataset.depBullet.split(':');dependencyExplorerState={type,id,depth:dependencyExplorerState.depth||'2',inspectType:type,inspectId:id,showImpact:true,showLineage:true,expanded:{},collapsed:{}};renderAdminTab()});
 body.querySelectorAll('[data-legend-toggle]').forEach(b=>b.onclick=()=>{const t=b.dataset.legendToggle;if(depExplorerHiddenTypes.has(t))depExplorerHiddenTypes.delete(t);else depExplorerHiddenTypes.add(t);renderAdminTab()});
 body.querySelectorAll('[data-dep-collapse]').forEach(b=>b.onclick=()=>{const t=b.dataset.depCollapse;const c={...(dependencyExplorerState.collapsed||{})};if(c[t])delete c[t];else c[t]=true;dependencyExplorerState={...dependencyExplorerState,collapsed:c};renderAdminTab()});
 const expandBtn=$('#depExpand'); if(expandBtn)expandBtn.onclick=()=>depExpandAround();
 const recenterBtn=$('#depRecenter'); if(recenterBtn)recenterBtn.onclick=()=>{dependencyExplorerState={...dependencyExplorerState,type:dependencyExplorerState.inspectType,id:dependencyExplorerState.inspectId,inspectType:dependencyExplorerState.inspectType,inspectId:dependencyExplorerState.inspectId,showImpact:true,showLineage:true,expanded:{},collapsed:{}};renderAdminTab()};
 const openBtn=$('#depOpenAdmin'); if(openBtn)openBtn.onclick=()=>{adminTab=NODE_TYPE_ADMIN_TAB[dependencyExplorerState.inspectType];adminView='tool';renderView()};""",
    'wiring: resets + collapse + expand')

if fails:
    print('FAILURES:')
    for f in fails:
        print(' - ' + f)
    sys.exit(1)
open(PATH, 'w').write(src)
print('all sections done, fails: 0')
