#!/usr/bin/env python3
"""PR3-B: Object 360 rebuild + shared graph engine (epic #359, issues #372/#373/#374).

Transform script: applies anchored replacements (each asserted to match exactly
once, or loudly fails) to app.js. Quote-style rule: HTML attrs and template
literals use PLAIN quotes; only single-quoted JS strings use \\' escapes.

Run:  python3 apply_ontology_pr3b.py [path-to-app.js]   (default PATH='app.js')
"""
import sys

PATH = sys.argv[1] if len(sys.argv) > 1 else 'app.js'
src = open(PATH, encoding='utf-8').read()


def rep(old, new, count=1):
    global src
    c = src.count(old)
    assert c == count, 'rep: expected %d, found %d for anchor %r' % (count, c, old[:90])
    src = src.replace(old, new)


def span_rep(start_anchor, end_anchor, new_text):
    """Replace from the start of start_anchor up to (not incl.) end_anchor."""
    global src
    assert src.count(start_anchor) == 1, 'span start not unique: %r' % start_anchor[:80]
    i = src.find(start_anchor)
    j = src.find(end_anchor, i)
    assert j > i, 'span end not found after start: %r' % end_anchor[:80]
    assert src.count(end_anchor) == 1, 'span end not unique: %r' % end_anchor[:80]
    src = src[:i] + new_text + src[j:]


def fn_body(name):
    """Extract the full `function name(...) {...}` text (string/comment aware)."""
    i = src.index('function ' + name + '(')
    k = src.index('{', i)
    depth = 0
    instr = None
    while True:
        ch = src[k]
        nxt = src[k + 1] if k + 1 < len(src) else ''
        if instr:
            if ch == '\\':
                k += 1
            elif ch == instr:
                instr = None
        elif ch == '/' and nxt == '/':
            while k < len(src) and src[k] != '\n':
                k += 1
            continue
        elif ch == '/' and nxt == '*':
            k = src.index('*/', k) + 1
        elif ch in ('"', "'", '`'):
            instr = ch
        elif ch == '{':
            depth += 1
        elif ch == '}':
            depth -= 1
            if depth == 0:
                return src[i:k + 1]
        k += 1


def assert_body_clean(name, banned, required=('driver.',)):
    body = fn_body(name)
    for b in banned:
        assert b not in body, '%s body still references %r' % (name, b)
    for r in required:
        assert r in body, '%s body missing %r' % (name, r)


# New-code blocks subject to the quote-style rule (template literals must use
# plain quotes; only single-quoted JS strings may use \' escapes).
QUOTE_CHECK_BLOCKS = []


def check_quotes(block, label):
    import re as _re
    for m in _re.finditer(r'`(?:[^`\\]|\\.)*`', block):
        seg = m.group(0)
        assert "\\'" not in seg, '%s: backslash-quote escape inside template literal: %r' % (label, seg[:80])


# ---------------------------------------------------------------------------
# 1. Shared engine header: depOntologyDriver, inserted before depGraphModel.
# ---------------------------------------------------------------------------
ENGINE_HEAD = r'''// ===== Shared graph engine (PR3-B, epic #359, issues #372/#373/#374) =====
// depGraphModel's traversal, depLayoutPositions' layout and depRenderGraph's
// renderer, generalized over a driver + ui contract so the Object 360 record
// graph reuses the exact same implementation instead of a second one.
// Behavior for the Ontology Graph is unchanged: depGraphModel,
// depLayoutPositions and depRenderGraph are thin wrappers below.
// Driver contract:
//   {rootType, rootId, nodeKey(t,i), splitKey(key)->[t,i], label(t,i),
//    origin(t,i), direct(t,i,dir)->[{type,id,edgeType}], dirs[], maxDepth,
//    expanded{}, collapsed{}, typeLabel(t)}
function depOntologyDriver(){
 const s=dependencyExplorerState;
 const bothOff=!s.showImpact&&!s.showLineage;
 return {
  rootType:s.type,rootId:s.id,
  nodeKey:depNodeKey,
  splitKey:depSplitKey,
  label:systemGraphNodeLabel,
  origin:depNodeOrigin,
  direct:systemGraphDirect,
  dirs:depActiveDirs(),
  maxDepth:bothOff?1:depMaxDepth(),
  expanded:s.expanded||{},
  collapsed:s.collapsed||{},
  typeLabel:(t)=>SYSTEM_NODE_TYPE_LABELS[t]||t
 };
}
// Unified graph model with real per-hop edges: BFS from the root honoring'''
check_quotes(ENGINE_HEAD, 'engine-head')
rep('// Unified graph model with real per-hop edges: BFS from the root honoring', ENGINE_HEAD)

# ---------------------------------------------------------------------------
# 2. depGraphModel -> graphModel(driver), targeted substitutions.
# ---------------------------------------------------------------------------
rep('function depGraphModel(){', 'function graphModel(driver){')
rep('''function graphModel(driver){
 const s=dependencyExplorerState;''', '''function graphModel(driver){''')
rep("const k=depNodeKey(type,id);const n=nodes.get(k);if(!n)nodes.set(k,{key:k,type,id,depth,label:systemGraphNodeLabel(type,id),origin:depNodeOrigin(type,id)});else if(depth<n.depth)n.depth=depth",
    "const k=driver.nodeKey(type,id);const n=nodes.get(k);if(!n)nodes.set(k,{key:k,type,id,depth,label:driver.label(type,id),origin:driver.origin(type,id)});else if(depth<n.depth)n.depth=depth")
rep("const k=ft+':'+fi+'>'+tt+':'+ti+':'+edgeType;if(edgeSeen.has(k))return;edgeSeen.add(k);edges.push({fromKey:depNodeKey(ft,fi),toKey:depNodeKey(tt,ti),edgeType})};",
    "const k=ft+':'+fi+'>'+tt+':'+ti+':'+edgeType;if(edgeSeen.has(k))return;edgeSeen.add(k);edges.push({fromKey:driver.nodeKey(ft,fi),toKey:driver.nodeKey(tt,ti),edgeType})};")
rep('addNode(s.type,s.id,0);', 'addNode(driver.rootType,driver.rootId,0);')
rep(''' const dirs=depActiveDirs();
 const bothOff=!s.showImpact&&!s.showLineage;
 const maxDepth=bothOff?1:depMaxDepth();''', ''' const dirs=driver.dirs;
 const maxDepth=driver.maxDepth;''')
rep('const seen=new Set([depNodeKey(s.type,s.id)]);',
    'const seen=new Set([driver.nodeKey(driver.rootType,driver.rootId)]);')
rep('let frontier=[{type:s.type,id:s.id}],truncated=0;',
    'let frontier=[{type:driver.rootType,id:driver.rootId}],truncated=0;')
rep('const hits=systemGraphDirect(node.type,node.id,dir);',
    'const hits=driver.direct(node.type,node.id,dir);')
rep('const k=depNodeKey(h.type,h.id);', 'const k=driver.nodeKey(h.type,h.id);', count=2)
rep('''for(const exKey of Object.keys(s.expanded||{})){
  const parts=depSplitKey(exKey),et=parts[0],ei=parts[1];''', '''for(const exKey of Object.keys(driver.expanded||{})){
  const parts=driver.splitKey(exKey),et=parts[0],ei=parts[1];''')
rep('for(const dir of dirs)for(const h of systemGraphDirect(et,ei,dir)){',
    'for(const dir of dirs)for(const h of driver.direct(et,ei,dir)){')
rep('depApplyCollapse(model);', 'depApplyCollapse(model,driver.collapsed,driver.typeLabel);')
rep(''' const model={nodes:Array.from(nodes.values()),edges,truncated};
 depApplyCollapse(model,driver.collapsed,driver.typeLabel);
 return model;
}
// Collapse cluster halos:''', ''' const model={nodes:Array.from(nodes.values()),edges,truncated};
 depApplyCollapse(model,driver.collapsed,driver.typeLabel);
 return model;
}
// Thin wrapper: the Ontology Graph's driver over the shared engine.
function depGraphModel(){
 return graphModel(depOntologyDriver());
}
// Collapse cluster halos:''')

assert_body_clean('graphModel', ('dependencyExplorerState', 'systemGraphDirect',
                                 'depNodeKey', 'depSplitKey', 'systemGraphNodeLabel',
                                 'depNodeOrigin', 'depActiveDirs', 'depMaxDepth'))

# ---------------------------------------------------------------------------
# 3. depApplyCollapse: accept the collapsed map + type-label fn (defaults keep
#    every existing call site behavior-identical).
# ---------------------------------------------------------------------------
rep('''function depApplyCollapse(model){
 const collapsed=dependencyExplorerState.collapsed||{};''', '''function depApplyCollapse(model,collapsed,typeLabel){
 collapsed=collapsed||dependencyExplorerState.collapsed||{};
 typeLabel=typeLabel||((t)=>SYSTEM_NODE_TYPE_LABELS[t]||t);''')
rep('const label=SYSTEM_NODE_TYPE_LABELS[t]||t;', 'const label=typeLabel(t);')

# ---------------------------------------------------------------------------
# 4. depLayoutPositions -> graphLayoutPositions(model, layout).
# ---------------------------------------------------------------------------
rep('function depLayoutPositions(model,W,H){', '''// layout: {rootKey, clusterOrder[], W, H} - rootKey/clusterOrder are the
// contract; W/H (canvas size) ride along since the body needs them.
function graphLayoutPositions(model,layout){''')
rep('''function graphLayoutPositions(model,layout){
 const visible=model.nodes;''', '''function graphLayoutPositions(model,layout){
 const W=layout.W,H=layout.H;
 const visible=model.nodes;''')
rep(' DEP_CLUSTER_ORDER.forEach((t,i)=>{', ' (layout.clusterOrder||[]).forEach((t,i)=>{')
rep('  const a=(i/DEP_CLUSTER_ORDER.length)*Math.PI*2-Math.PI/2;',
    '  const a=(i/layout.clusterOrder.length)*Math.PI*2-Math.PI/2;')
rep(' const rootKey=depNodeKey(dependencyExplorerState.type,dependencyExplorerState.id);',
    ' const rootKey=layout.rootKey;')
rep(''' return pos;
}
// Andrew's monotone chain convex hull, for cluster halos.''', ''' return pos;
}
function depLayoutPositions(model,W,H){
 return graphLayoutPositions(model,{rootKey:depNodeKey(dependencyExplorerState.type,dependencyExplorerState.id),clusterOrder:DEP_CLUSTER_ORDER,W:W,H:H});
}
// Andrew's monotone chain convex hull, for cluster halos.''')

assert_body_clean('graphLayoutPositions', ('DEP_CLUSTER_ORDER', 'dependencyExplorerState'),
                  required=('layout.',))

# ---------------------------------------------------------------------------
# 5. depRenderGraph -> graphRender(model, ui, W, H), targeted substitutions.
# ---------------------------------------------------------------------------
rep('''// Renders halos + edges + edge labels + nodes into the graph canvas.
function depRenderGraph(model,W,H){
 const s=dependencyExplorerState;
 const pos=depLayoutPositions(model,W,H);
 const rootKey=depNodeKey(s.type,s.id);
 const byKey=new Map(model.nodes.map(n=>[n.key,n]));''', '''// Renders halos + edges + edge labels + nodes into the graph canvas.
// Generalized over a ui contract (PR3-B) so Object 360 reuses this exact
// renderer at record scope. ui:
//   {colors{}, typeLabels{}, hiddenTypes:Set, rootKey, inspectType,
//    inspectId, inspectKey, inspectAttr, collapseAttr, edgeLabelFn,
//    isAccessEdgeFn, nodeSize, rootSize, rootClass, clusterOrder[],
//    highlight?{pathKeys[]|null, simKeys[]|null, simChange|null},
//    labelMode? 'all'|'neighborhood'}
// highlight.pathKeys dims everything off the path and rings path nodes
// (ontology-node-path); highlight.simKeys rings those nodes
// (ontology-node-sim, plus ontology-node-sim-<simChange> when simChange is
// set) and dims the rest; labelMode 'neighborhood' renders edge-label pills
// only for edges touching the inspected node (spec section 7.6 declutter).
function graphRender(model,ui,W,H){
 const pos=graphLayoutPositions(model,{rootKey:ui.rootKey,clusterOrder:ui.clusterOrder||[],W:W,H:H});
 const byKey=new Map(model.nodes.map(n=>[n.key,n]));
 const inspectKey=ui.inspectKey;
 const hl=ui.highlight||{};
 const pathSet=hl.pathKeys?new Set(hl.pathKeys):null;
 const simSet=hl.simKeys?new Set(hl.simKeys):null;
 const hlActive=!!(pathSet||simSet);''')
rep('if(depExplorerHiddenTypes.has(type))return;', 'if(ui.hiddenTypes.has(type))return;')
rep("const color=SYSTEM_NODE_TYPE_COLORS[type]||'#888';", "const color=ui.colors[type]||'#888';")
rep("const color=SYSTEM_NODE_TYPE_COLORS[n.type]||'#888';", "const color=ui.colors[n.type]||'#888';", count=2)
rep("const color=SYSTEM_NODE_TYPE_COLORS[an.type]||'#888';", "const color=ui.colors[an.type]||'#888';")
rep("const label=(SYSTEM_NODE_TYPE_LABELS[type]||type)+' · '+members.length;",
    "const label=(ui.typeLabels[type]||type)+' · '+members.length;")
rep('data-dep-collapse="\'+type+\'"', '\'+ui.collapseAttr+\'="\'+type+\'"')
rep('data-dep-collapse="\'+n.type+\'"', '\'+ui.collapseAttr+\'="\'+n.type+\'"', count=2)
rep('if(depExplorerHiddenTypes.has(an.type)||depExplorerHiddenTypes.has(bn.type))return;',
    'if(ui.hiddenTypes.has(an.type)||ui.hiddenTypes.has(bn.type))return;')
rep('''  const access=depIsAccessEdge(e.edgeType);''', '''  const access=ui.isAccessEdgeFn(e.edgeType);
  const eLabel=ui.edgeLabelFn(e.edgeType);
  const eOnPath=pathSet?pathSet.has(e.fromKey)&&pathSet.has(e.toKey):false;
  const eInSim=simSet?simSet.has(e.fromKey)&&simSet.has(e.toKey):false;
  const eDimmed=hlActive&&!eOnPath&&!eInSim;''')
rep('edgeSvg.push(\'<line x1="\'+a.x.toFixed(1)+\'" y1="\'+a.y.toFixed(1)+\'" x2="\'+b.x.toFixed(1)+\'" y2="\'+b.y.toFixed(1)+\'" stroke="\'+color+\'" stroke-opacity="0.5" stroke-width="1.5"\'+(access?\' stroke-dasharray="6 4"\':\'\')+\'><title>\'+an.label+\' — \'+edgeTypeLabel(e.edgeType)+\' → \'+bn.label+\'</title></line>\');',
    'edgeSvg.push(\'<line x1="\'+a.x.toFixed(1)+\'" y1="\'+a.y.toFixed(1)+\'" x2="\'+b.x.toFixed(1)+\'" y2="\'+b.y.toFixed(1)+\'" stroke="\'+color+\'" stroke-opacity="\'+(eDimmed?\'0.12\':\'0.5\')+\'" stroke-width="1.5"\'+(access?\' stroke-dasharray="6 4"\':\'\')+\'><title>\'+an.label+\' — \'+eLabel+\' → \'+bn.label+\'</title></line>\');')
rep('edgeLabels.push(\'<div class="ontology-edge-label\'+(access?\' ontology-edge-label-access\':\'\')+\'" style="left:\'+((a.x+b.x)/2).toFixed(1)+\'px;top:\'+((a.y+b.y)/2).toFixed(1)+\'px" title="\'+an.label+\' — \'+edgeTypeLabel(e.edgeType)+\' → \'+bn.label+\'">\'+(access?\'🔒 \':\'\')+edgeTypeLabel(e.edgeType)+\'</div>\');',
    '''if(ui.labelMode!=='neighborhood'||e.fromKey===inspectKey||e.toKey===inspectKey)
   edgeLabels.push('<div class="ontology-edge-label'+(access?' ontology-edge-label-access':'')+(eDimmed?' ontology-node-dimmed':'')+'" style="left:'+((a.x+b.x)/2).toFixed(1)+'px;top:'+((a.y+b.y)/2).toFixed(1)+'px" title="'+an.label+' — '+eLabel+' → '+bn.label+'">'+(access?'🔒 ':'')+eLabel+'</div>');''')
rep('''  const isRoot=n.key===rootKey;
  const isDimmed=depExplorerHiddenTypes.has(n.type);
  const isInspecting=!isRoot&&n.type===s.inspectType&&n.id===s.inspectId;''', '''  const isRoot=n.key===ui.rootKey;
  const onPath=pathSet?pathSet.has(n.key):false;
  const inSim=simSet?simSet.has(n.key):false;
  const isDimmed=ui.hiddenTypes.has(n.type)||(hlActive&&!onPath&&!inSim);
  const isInspecting=!isRoot&&n.type===ui.inspectType&&n.id===ui.inspectId;''')
rep('const size=isRoot?ONTOLOGY_ROOT_SIZE:ONTOLOGY_NODE_SIZE;',
    'const size=isRoot?ui.rootSize:ui.nodeSize;')
rep('nodeDivs.push(\'<div class="ontology-node-dot\'+(isRoot?\' ontology-node-root\':\'\')+(isInspecting?\' ontology-node-inspecting\':\'\')+(isDimmed?\' ontology-node-dimmed\':\'\')+\'" data-dep-inspect="\'+n.type+\':\'+n.id+\'" style="left:\'+(p.x-size/2).toFixed(1)+\'px;top:\'+(p.y-size/2).toFixed(1)+\'px;width:\'+size+\'px;height:\'+size+\'px;background:\'+color+\'" title="\'+(SYSTEM_NODE_TYPE_LABELS[n.type]||n.type)+\': \'+n.label+\'"></div>\');',
    'nodeDivs.push(\'<div class="ontology-node-dot\'+(isRoot?\' \'+ui.rootClass:\'\')+(isInspecting?\' ontology-node-inspecting\':\'\')+(isDimmed?\' ontology-node-dimmed\':\'\')+(onPath?\' ontology-node-path\':\'\')+(inSim?\' ontology-node-sim\':\'\')+(inSim&&hl.simChange?\' ontology-node-sim-\'+hl.simChange:\'\')+\'" \'+ui.inspectAttr+\'="\'+n.type+\':\'+n.id+\'" style="left:\'+(p.x-size/2).toFixed(1)+\'px;top:\'+(p.y-size/2).toFixed(1)+\'px;width:\'+size+\'px;height:\'+size+\'px;background:\'+color+\'" title="\'+(ui.typeLabels[n.type]||n.type)+\': \'+n.label+\'"></div>\');')
rep(''' return '<svg width="'+W+'" height="'+H+'" style="position:absolute;top:0;left:0;pointer-events:none">'+haloSvg.join('')+edgeSvg.join('')+'</svg>'+haloLabels.join('')+edgeLabels.join('')+nodeDivs.join('');
}
// Pinned right detail panel (replaces the floating inspector card): type,''', ''' return '<svg width="'+W+'" height="'+H+'" style="position:absolute;top:0;left:0;pointer-events:none">'+haloSvg.join('')+edgeSvg.join('')+'</svg>'+haloLabels.join('')+edgeLabels.join('')+nodeDivs.join('');
}
// Thin wrapper: the Ontology Graph's ui over the shared renderer.
// pathKeys/simKeys/simChange/labelMode are undefined on
// dependencyExplorerState today, so this renders exactly like the
// pre-PR3-B depRenderGraph.
function depRenderGraph(model,W,H){
 const s=dependencyExplorerState;
 return graphRender(model,{
  colors:SYSTEM_NODE_TYPE_COLORS,
  typeLabels:SYSTEM_NODE_TYPE_LABELS,
  hiddenTypes:depExplorerHiddenTypes,
  rootKey:depNodeKey(s.type,s.id),
  inspectType:s.inspectType,inspectId:s.inspectId,
  inspectKey:depNodeKey(s.inspectType,s.inspectId),
  inspectAttr:'data-dep-inspect',
  collapseAttr:'data-dep-collapse',
  edgeLabelFn:edgeTypeLabel,
  isAccessEdgeFn:depIsAccessEdge,
  nodeSize:ONTOLOGY_NODE_SIZE,rootSize:ONTOLOGY_ROOT_SIZE,
  rootClass:'ontology-node-root',
  clusterOrder:DEP_CLUSTER_ORDER,
  highlight:{pathKeys:s.pathKeys||null,simKeys:s.simKeys||null,simChange:s.simChange||null},
  labelMode:s.labelMode||'all'
 },W,H);
}
// Pinned right detail panel (replaces the floating inspector card): type,''')

assert_body_clean('graphRender', ('depExplorerHiddenTypes', 'SYSTEM_NODE_TYPE_COLORS',
                                  'SYSTEM_NODE_TYPE_LABELS', 'depIsAccessEdge',
                                  'data-dep-', 'ONTOLOGY_ROOT_SIZE', 'ONTOLOGY_NODE_SIZE',
                                  's.inspectType', 'edgeTypeLabel(e.edgeType)'),
                  required=('ui.',))

# ---------------------------------------------------------------------------
# 6. Record driver + Object 360 helpers, inserted before `let object360State`.
# ---------------------------------------------------------------------------
RECORD_BLOCK = r'''// ===== Object 360 record graph (PR3-B, issues #372/#373/#374) =====
// The shared graph engine driven at record scope: record nodes use the
// entity key itself (e.g. 'companies') plus pseudo-types 'business_rule',
// 'workflow', 'ai_agent' and 'orgUnit'. Record-level edge taxonomy
// (spec section 12.2 OB-4): linked_record, has_business_unit, applies_to,
// ran_on, scoped_agent, agent_can_read, agent_can_update.
function recordNodeLabel(type,id){
 if(type==='business_rule'){const r=(data.fieldRules||[]).find(x=>x.id===id);return r?(r.name||'Untitled rule'):id}
 if(type==='workflow'){const w=(data.workflowRules||[]).find(x=>x.id===id);return w?(w.name||'Untitled workflow'):id}
 if(type==='ai_agent'){const a=(data.aiAgents||[]).find(x=>x.id===id);return a?a.name:id}
 if(type==='orgUnit'){const o=(data.orgUnits||[]).find(x=>x.id===id);return o?o.name:id}
 const rec=byId(type,id);
 return rec?recordDisplayName(type,rec):id;
}
// Direct (depth-1) neighbors of one record. Related records are neighbors in
// both directions; the owning org unit is a dependency (record -> org unit);
// rules, workflows and scoped agents all point at the record (dependents).
function recordGraphDirect(entityType,entityId,direction){
 const out=[],seen=new Set();
 const push=(type,id,edgeType,label)=>{const k=type+':'+id+':'+edgeType;if(seen.has(k))return;seen.add(k);out.push({type,id,edgeType,label})};
 [...relatedRecordsFor(entityType,entityId),...nativeRelatedRecordsFor(entityType,entityId)]
  .forEach(r=>push(r.entityType,r.entityId,'linked_record',r.displayName));
 if(direction==='dependencies'){
  const rec=byId(entityType,entityId);
  const ou=rec&&rec.owningOrgUnitId?(data.orgUnits||[]).find(o=>o.id===rec.owningOrgUnitId):null;
  if(ou)push('orgUnit',ou.id,'has_business_unit',ou.name);
 }
 if(direction==='dependents'){
  (data.fieldRules||[]).filter(r=>r.entity===entityType&&r.active)
   .forEach(r=>push('business_rule',r.id,'applies_to',r.name||'Untitled rule'));
  (data.workflowRules||[]).filter(w=>w.entity===entityType&&w.active)
   .forEach(w=>push('workflow',w.id,'ran_on',w.name||'Untitled workflow'));
  (data.aiAgents||[])
   .filter(a=>a.isActive&&((a.objectTypeScopes||[]).length===0||(a.objectTypeScopes||[]).some(sc=>sc.objectType===entityType)))
   .forEach(a=>{
    push('ai_agent',a.id,'scoped_agent',a.name);
    systemGraphEdges()
     .filter(e=>(e.edgeType==='can_read'||e.edgeType==='can_update')&&e.fromType==='ai_agent'&&e.fromId===a.id&&e.toType==='custom_object'&&e.toId===entityType)
     .forEach(e=>push('ai_agent',a.id,e.edgeType==='can_read'?'agent_can_read':'agent_can_update',a.name));
   });
 }
 return out;
}
function o360RecordDriver(rootEntityType,rootEntityId){
 const s=object360State||{};
 const depth=parseInt(s.depth,10);
 return {
  rootType:rootEntityType,rootId:rootEntityId,
  nodeKey:(t,i)=>'rec:'+t+':'+i,
  splitKey:(k)=>{const m=/^rec:([^:]+):([\s\S]*)$/.exec(k);return m?[m[1],m[2]]:depSplitKey(k)},
  label:recordNodeLabel,
  origin:(t)=>ONTOLOGY_BUILTIN_TYPES.some(b=>b[0]===t)?'OOB':'Custom',
  direct:recordGraphDirect,
  dirs:[s.showLineage===false?null:'dependencies',s.showImpact===false?null:'dependents'].filter(Boolean),
  maxDepth:(depth>=1&&depth<=3)?depth:2,
  expanded:s.expanded||{},
  collapsed:s.collapsed||{},
  typeLabel:(t)=>t==='business_rule'?'Business Rule':t==='workflow'?'Workflow':t==='ai_agent'?'AI Agent':t==='orgUnit'?'Business Unit':(entityLabel(t)||t)
 };
}
function o360ClusterOrder(driver,model){
 const order=[driver.rootType];
 model.nodes.forEach(n=>{if(order.indexOf(n.type)<0)order.push(n.type)});
 return order;
}
function o360GraphModel(){
 const s=object360State;
 return graphModel(o360RecordDriver(s.root.entityType,s.root.entityId));
}
// Access chips for one agent row (issue #374): derived from the agent's
// capability edges onto this object type in systemGraphEdges().
function o360AgentChips(agent,entityType){
 const caps=systemGraphEdges().filter(e=>e.fromType==='ai_agent'&&e.fromId===agent.id&&e.toType==='custom_object'&&e.toId===entityType);
 const chips=[];
 if(caps.some(e=>e.edgeType==='can_read'))chips.push('can read');
 if(caps.some(e=>e.edgeType==='can_update'))chips.push('can update');
 if(caps.some(e=>e.edgeType==='can_delete'))chips.push('can delete');
 if(!chips.length)chips.push((agent.objectTypeScopes||[]).length===0?'unscoped':'scoped · no record access');
 return chips;
}
function o360TypeDot(type){
 return '<span style="display:inline-block;width:14px;height:14px;border-radius:50%;background:var(--chart-'+ontologyColorIndex(type)+');flex-shrink:0"></span>';
}
// Pivot (issue #374): push the current root onto the breadcrumb trail,
// re-center on the new record, and fix the modal title. newTrail overrides
// the push when pivoting back via a breadcrumb crumb.
function o360PivotTo(entityType,entityId,label,newTrail){
 const cur=object360State; if(!cur)return;
 const trail=newTrail||[...(cur.trail||[]),{entityType:cur.root.entityType,entityId:cur.root.entityId,label:cur.root.label}];
 object360State={...cur,trail,root:{entityType,entityId,label},inspect:{entityType,entityId,label}};
 if(typeof document!=='undefined'){
  const h2=document.querySelector('#modal .modal-head h2');
  if(h2)h2.textContent='Object 360: '+label;
 }
}
// Cross-links both ways (issue #374).
function openOntologyGraphFor(entityTypeKey){
 closeModal();
 dependencyExplorerState={type:'custom_object',id:entityTypeKey,depth:'2',inspectType:'custom_object',inspectId:entityTypeKey,showImpact:true,showLineage:true,expanded:{},collapsed:{}};
 current='admin';adminTab='dependencyExplorer';adminView='tool';renderView();
}
function openObject360ForEntity(entityTypeKey){
 let rec=null;
 const arr=data[entityTypeKey];
 if(Array.isArray(arr)&&arr.length)rec=arr[0];
 if(!rec){
  for(const o of (data.customObjects||[])){
   if(o.key!==entityTypeKey)continue;
   const a=data[o.key];
   if(Array.isArray(a)&&a.length){rec=a[0];break}
  }
 }
 if(!rec){toast('No records yet');return}
 object360Modal(entityTypeKey,rec.id,recordDisplayName(entityTypeKey,rec));
}
let object360State=null;'''
check_quotes(RECORD_BLOCK, 'record-block')
rep('let object360State=null;', RECORD_BLOCK)

# ---------------------------------------------------------------------------
# 7. object360Modal: breadcrumb trail starts empty.
# ---------------------------------------------------------------------------
rep('object360State={root:{entityType,entityId,label},inspect:{entityType,entityId,label}};',
    'object360State={root:{entityType,entityId,label},inspect:{entityType,entityId,label},trail:[]};')

# ---------------------------------------------------------------------------
# 8. renderObject360Body: shared engine at record scope (issues #372-#374).
# ---------------------------------------------------------------------------
BODY_360 = r'''function renderObject360Body(){
 const s=object360State; if(!s)return;
 const body=$('#object360Body'); if(!body)return;
 const driver=o360RecordDriver(s.root.entityType,s.root.entityId);
 const model=graphModel(driver);
 // Canvas scales with the model, like the Ontology Graph's.
 const W=Math.max(880,Math.min(1500,Math.round(560+model.nodes.length*5)));
 const H=Math.max(600,Math.min(940,Math.round(440+model.nodes.length*3.2)));
 const colors={},typeLabels={};
 model.nodes.forEach(n=>{
  if(!colors[n.type])colors[n.type]='var(--chart-'+ontologyColorIndex(n.type)+')';
  if(!typeLabels[n.type])typeLabels[n.type]=driver.typeLabel(n.type);
 });
 const graphHtml=graphRender(model,{
  colors:colors,typeLabels:typeLabels,
  hiddenTypes:object360HiddenTypes,
  rootKey:driver.nodeKey(s.root.entityType,s.root.entityId),
  inspectType:s.inspect.entityType,inspectId:s.inspect.entityId,
  inspectKey:driver.nodeKey(s.inspect.entityType,s.inspect.entityId),
  inspectAttr:'data-o360g-inspect',
  collapseAttr:'data-o360g-collapse',
  edgeLabelFn:edgeTypeLabel,
  isAccessEdgeFn:(t)=>t==='agent_can_read'||t==='agent_can_update',
  nodeSize:15,rootSize:26,
  rootClass:'ontology-node-root',
  clusterOrder:o360ClusterOrder(driver,model),
  labelMode:'all'
 },W,H);

 const rootRecord=byId(s.root.entityType,s.root.entityId);
 const related=[...relatedRecordsFor(s.root.entityType,s.root.entityId),...nativeRelatedRecordsFor(s.root.entityType,s.root.entityId)];
 const groups=new Map();
 related.forEach(r=>{const list=groups.get(r.defKey)||[];list.push(r);groups.set(r.defKey,list)});
 const groupEntries=Array.from(groups.entries());
 const orgUnit=rootRecord&&rootRecord.owningOrgUnitId?(data.orgUnits||[]).find(o=>o.id===rootRecord.owningOrgUnitId):null;

 const MAXPER=20;
 const inspectIsRoot=s.inspect.entityType===s.root.entityType&&s.inspect.entityId===s.root.entityId;
 const inspectIsPseudo=['business_rule','workflow','ai_agent'].indexOf(s.inspect.entityType)>=0;
 const o360CardPosStyle=object360CardPos?` style="left:${object360CardPos.left}px;top:${object360CardPos.top}px;right:auto"`:'';
 let inspectorHtml;
 if(s.inspect.entityType==='orgUnit'){
  inspectorHtml=`<div class="ontology-inspector-card" id="o360InspectorCard"${o360CardPosStyle}>
   <div class="ontology-inspector-head" id="o360InspectorHead">${o360TypeDot('orgUnit')}<div><strong>${s.inspect.label}</strong><div class="ontology-inspector-subtitle">Business Unit</div></div></div>
   <p class="muted" style="font-size:12px">Business Units aren't individually re-centerable here - see Admin → Organization Units for its own detail.</p>
  </div>`;
 }else if(inspectIsPseudo){
  // Rule / workflow / agent node: no record to open, so the card deep-links
  // to the owning admin screen instead of offering a broken Open button.
  const pseudoTab={business_rule:'rules',workflow:'workflow',ai_agent:'aiAgents'}[s.inspect.entityType];
  const pseudoTabLabel={business_rule:'Business Rules',workflow:'Workflows',ai_agent:'AI Agents'}[s.inspect.entityType];
  inspectorHtml=`<div class="ontology-inspector-card" id="o360InspectorCard"${o360CardPosStyle}>
   <div class="ontology-inspector-head" id="o360InspectorHead">${o360TypeDot(s.inspect.entityType)}<div><strong>${s.inspect.label}</strong><div class="ontology-inspector-subtitle">${driver.typeLabel(s.inspect.entityType)}</div></div></div>
   <div class="ontology-inspector-actions">
    <button type="button" class="btn btn-secondary" id="o360OpenPseudoAdmin">Open in Admin → ${pseudoTabLabel}</button>
   </div>
  </div>`;
 }else{
  let groupsHtml='';
  if(inspectIsRoot){
   const rules=(data.fieldRules||[]).filter(r=>r.entity===s.root.entityType&&r.active);
   const workflows=(data.workflowRules||[]).filter(w=>w.entity===s.root.entityType).sort((a,b)=>(b.active?1:0)-(a.active?1:0));
   const agents=(data.aiAgents||[]).filter(a=>a.isActive&&((a.objectTypeScopes||[]).length===0||(a.objectTypeScopes||[]).some(sc=>sc.objectType===s.root.entityType)));
   const activities=(data.activities||[]).filter(a=>a.entityType===s.root.entityType&&a.entityId===s.root.entityId).slice().sort((a,b)=>b.occurredAt.localeCompare(a.occurredAt)).slice(0,10);
   // Issue #372: live pass/fail per rule via ruleConditionsMatch, each row
   // deep-linking to Admin → Business Rules.
   const ruleRows=rules.map(r=>{
    const verdict=rootRecord?(ruleConditionsMatch(r,rootRecord)?'passing':'failing'):null;
    return `<button type="button" class="ontology-inspector-bullet" data-o360-admin="rules" title="Open in Admin → Business Rules"><span style="flex:1;text-align:left">${r.name||'Untitled rule'}</span>${verdict?`<span class="o360-chip ${verdict==='passing'?'o360-chip-pass':'o360-chip-fail'}">${verdict}</span>`:''}</button>`;
   }).join('');
   // Issue #372: Active/Inactive + a live trigger check per workflow. The
   // demo keeps no workflow run history, so this is honestly labeled
   // "Trigger check" - never a faked "last run" timestamp.
   const wfRows=workflows.map(w=>{
    const trig=rootRecord?conditionsMatch(w.matchType||'all',w.conditions||[],rootRecord):null;
    return `<button type="button" class="ontology-inspector-bullet" data-o360-admin="workflow" title="Open in Admin → Workflows"><span style="flex:1;text-align:left">${w.name||'Untitled workflow'}</span><span class="o360-chip ${w.active?'o360-chip-pass':'o360-chip-neutral'}">${w.active?'Active':'Inactive'}</span>${trig===null?'':`<span class="o360-chip o360-chip-neutral" title="Trigger check: this workflow's conditions evaluated against the record right now">${trig?'would trigger on this record':'conditions not met'}</span>`}</button>`;
   }).join('');
   // Issue #374: access chips per agent from its capability edges.
   const agentRows=agents.map(a=>{
    const chips=o360AgentChips(a,s.root.entityType);
    return `<div class="ontology-inspector-bullet" style="cursor:default"><span style="flex:1;text-align:left">${a.icon||''} ${a.name}</span>${chips.map(c=>`<span class="o360-chip ${c==='can read'||c==='can update'||c==='can delete'?'o360-chip-pass':'o360-chip-neutral'}">${c}</span>`).join('')}</div>`;
   }).join('');
   groupsHtml=`${groupEntries.map(([key,members])=>`<div class="ontology-inspector-group"><div class="ontology-inspector-group-label">${(members[0]&&members[0].groupLabel)||key}</div>${members.slice(0,MAXPER).map(m=>`<button type="button" class="ontology-inspector-bullet" data-o360g-inspect="${m.entityType}:${m.entityId}">${entityDotHtml(m.entityType)}${m.displayName}</button>`).join('')}${members.length>MAXPER?`<div class="muted" style="font-size:12px">+${members.length-MAXPER} more</div>`:''}</div>`).join('')}
    ${groupEntries.length===0?'<p class="muted" style="font-size:12px">No related records yet.</p>':''}
    ${orgUnit?`<div class="ontology-inspector-group"><div class="ontology-inspector-group-label">Business Unit</div><div style="font-size:13px">${orgUnit.name}</div></div>`:''}
    ${rules.length?`<div class="ontology-inspector-group"><div class="ontology-inspector-group-label">Business Rules that apply</div>${ruleRows}</div>`:''}
    ${workflows.length?`<div class="ontology-inspector-group"><div class="ontology-inspector-group-label">Workflows that apply</div><div class="o360-trigger-note">Trigger check evaluates each workflow's conditions against this record live — the demo keeps no run history, so there is no "last run" to show.</div>${wfRows}</div>`:''}
    ${agents.length?`<div class="ontology-inspector-group"><div class="ontology-inspector-group-label">AI Agents scoped to this object type</div>${agentRows}</div>`:''}
    ${activities.length?`<div class="ontology-inspector-group"><div class="ontology-inspector-group-label">Activity</div>${activities.map(a=>`<div style="font-size:12px" class="muted">${a.subject||a.channel}<br>${new Date(a.occurredAt).toLocaleString()}</div>`).join('')}</div>`:''}`;
  }
  inspectorHtml=`<div class="ontology-inspector-card" id="o360InspectorCard"${o360CardPosStyle}>
   <div class="ontology-inspector-head" id="o360InspectorHead">${entityDotHtml(s.inspect.entityType,14)}<div><strong>${s.inspect.label}</strong><div class="ontology-inspector-subtitle">${driver.typeLabel(s.inspect.entityType)}</div></div></div>
   <div class="ontology-inspector-actions">
    <button type="button" class="btn btn-primary" id="o360OpenRecord">Open record</button>
    <button type="button" class="btn btn-secondary" id="o360Recenter" ${inspectIsRoot?'disabled':''}>Show what's around it</button>
    ${inspectIsRoot?`<button type="button" class="btn btn-secondary" id="o360OpenOntology">View ${entityLabel(s.root.entityType)||s.root.entityType} in Ontology Graph</button>`:''}
   </div>
   ${inspectIsRoot?`<div class="ontology-inspector-groups">${groupsHtml}</div>`:''}
  </div>`;
 }

 // Legend doubles as the toggle-filter; built from the types actually in
 // this record's model, now including rules/workflows/agents/orgUnit.
 const legendTypes=[];
 model.nodes.forEach(n=>{if(legendTypes.indexOf(n.type)<0)legendTypes.push(n.type)});
 const legendHtmlStr=legendTypes.length?`<div class="ontology-legend">${legendTypes.map(k=>{
  const lbl=driver.typeLabel(k);
  const isOff=object360HiddenTypes.has(k);
  return `<button type="button" class="ontology-legend-item${isOff?' ontology-legend-item-off':''}" data-o360-legend-toggle="${k}" title="${isOff?'Show':'Hide'} ${lbl}"><span style="display:inline-block;flex-shrink:0;width:10px;height:10px;border-radius:50%;background:${colors[k]}"></span>${lbl}</button>`;
 }).join('')}</div>`:'';

 // Breadcrumb back through the pivot trail (issue #374).
 const trail=s.trail||[];
 const crumbHtml=trail.length?`<div class="o360-breadcrumb">${trail.map((t,i)=>`<button type="button" class="o360-crumb" data-o360-crumb="${i}" title="Pivot back to ${t.label}">${t.label}</button><span class="o360-crumb-sep">›</span>`).join('')}<span class="o360-crumb-current">${s.root.label}</span></div>`:'';

 body.innerHTML=`<p class="muted" style="font-size:13px;max-width:720px">Every record directly linked to ${s.root.label}, its Business Unit, the Business Rules/Workflows that apply to records like it, and which AI Agents are scoped to act on it.</p>
 ${crumbHtml}
 <div class="o360-toolbar">${legendHtmlStr}<label class="o360-depth">Depth <select id="o360Depth">${['1','2','3'].map(d=>`<option value="${d}"${String(s.depth||'2')===d?' selected':''}>${d}</option>`).join('')}</select></label></div>
 <div class="ontology-graph-wrap" style="height:${H+40}px">
  <div class="ontology-graph-canvas" style="width:${W}px;height:${H}px">${graphHtml}${inspectorHtml}</div>
 </div>`;

 body.querySelectorAll('[data-o360g-inspect]').forEach(el=>el.onclick=()=>{
  const v=el.dataset.o360gInspect,ci=v.indexOf(':');
  const entityType=v.slice(0,ci),entityId=v.slice(ci+1);
  object360State={...object360State,inspect:{entityType,entityId,label:driver.label(entityType,entityId)}};
  renderObject360Body();
 });
 body.querySelectorAll('[data-o360g-collapse]').forEach(el=>el.onclick=()=>{
  const t=el.dataset.o360gCollapse;
  const c={...(object360State.collapsed||{})};
  if(c[t])delete c[t];else c[t]=true;
  object360State={...object360State,collapsed:c};
  renderObject360Body();
 });
 body.querySelectorAll('[data-o360-legend-toggle]').forEach(b=>b.onclick=()=>{const t=b.dataset.o360LegendToggle;if(object360HiddenTypes.has(t))object360HiddenTypes.delete(t);else object360HiddenTypes.add(t);renderObject360Body()});
 body.querySelectorAll('[data-o360-crumb]').forEach(b=>b.onclick=()=>{
  const i=parseInt(b.dataset.o360Crumb,10);
  const t=object360State.trail[i];
  o360PivotTo(t.entityType,t.entityId,t.label,object360State.trail.slice(0,i));
  renderObject360Body();
 });
 body.querySelectorAll('[data-o360-admin]').forEach(b=>b.onclick=()=>{const t=b.dataset.o360Admin;closeModal();current='admin';adminTab=t;adminView='tool';renderView()});
 const openBtn=$('#o360OpenRecord'); if(openBtn)openBtn.onclick=()=>{const t=object360State.inspect;closeModal();openRecordDetail(t.entityType,t.entityId)};
 const recenterBtn=$('#o360Recenter'); if(recenterBtn)recenterBtn.onclick=()=>{const t=object360State.inspect;o360PivotTo(t.entityType,t.entityId,t.label);renderObject360Body()};
 const ontoBtn=$('#o360OpenOntology'); if(ontoBtn)ontoBtn.onclick=()=>openOntologyGraphFor(object360State.root.entityType);
 const pseudoBtn=$('#o360OpenPseudoAdmin'); if(pseudoBtn)pseudoBtn.onclick=()=>{const t={business_rule:'rules',workflow:'workflow',ai_agent:'aiAgents'}[object360State.inspect.entityType];closeModal();current='admin';adminTab=t;adminView='tool';renderView()};
 const depthSel=$('#o360Depth'); if(depthSel)depthSel.onchange=(e)=>{object360State={...object360State,depth:e.target.value};renderObject360Body()};
 makeInspectorCardDraggable($('#o360InspectorCard'),$('#o360InspectorHead'),(left,top)=>{object360CardPos={left,top}});
}
'''
check_quotes(BODY_360, 'renderObject360Body')
span_rep('function renderObject360Body(){', 'function companyDetail(id){', BODY_360)

open(PATH, 'w', encoding='utf-8').write(src)
print('PR3-B transform applied to', PATH)
