#!/usr/bin/env python3
"""Apply PR 1 (Ontology Graph foundation + rename) edits to /tmp/lanesra-os/app.js.

Ground truth on bytes (verified via ordinals): HTML attrs and template
literals use PLAIN quotes. Only single-quoted JS strings use \' escapes.
"""
import sys

PATH = '/tmp/lanesra-os/app.js'
src = open(PATH).read()
fails = []

def rep(old, new, count=1):
    global src
    n = src.count(old)
    if n != count:
        fails.append('COUNT %d != %d: %r' % (n, count, old[:80]))
        return
    src = src.replace(old, new)

def rep_span(start_anchor, end_anchor, new_text):
    global src
    s = src.find(start_anchor)
    if s < 0:
        fails.append('START NOT FOUND: %r' % start_anchor[:60]); return
    e = src.find(end_anchor, s)
    if e < 0:
        fails.append('END NOT FOUND: %r' % end_anchor[:60]); return
    e += len(end_anchor)
    src = src[:s] + new_text + src[e:]

# ---------------- 1. RENAME ----------------
rep("['dependencyExplorer','Dependency Explorer']", "['dependencyExplorer','Ontology Graph']")
rep("behind the Ontology Explorer, down to", "behind the Ontology Graph, down to")
rep('Object Types\' "Open in Dependency Explorer" deep', 'Object Types\' "Open in Ontology Graph" deep')
rep('>Open in Dependency Explorer</button>', '>Open in Ontology Graph</button>')
rep("""// Ontology Explorer visual language (issue #339): which admin tab owns
// each System Graph node type, and a fixed --chart-N index per type -
// hand-mirrored from desktop's own `systemGraphNodeMeta.ts` (same values)
// so the legend/dots read identically on both surfaces.""",
"""// Ontology Graph visual language (issue #339, recolored #364): which admin
// tab owns each System Graph node type, and one fixed hex color per type
// (13 unique hues - the old 6-token --chart-N cycle made Object/Page Layout/
// Action Type indistinguishable).""")
rep("<h3>Dependency Explorer</h3>", "<h3>Ontology Graph</h3>")
rep("read-only Dependency Explorer admin screen", "read-only Ontology Graph admin screen", 2)
rep("a canonical dependency graph &amp; Dependency Explorer", "a canonical dependency graph &amp; Ontology Graph")
rep("Ontology Explorer \u2014 a radial dependency graph", "Ontology Graph \u2014 a radial dependency graph")
rep("The Dependency Explorer\\'s visual redesign", "The Ontology Graph\\'s visual redesign")
rep("the Ontology Explorer and Object 360", "the Ontology Graph and Object 360")
rep("The Dependency Explorer\\'s visual language is redesigned", "The Ontology Graph\\'s visual language is redesigned")
rep("a pre-filtered Dependency Explorer deep link", "a pre-filtered Ontology Graph deep link")
rep("traces to its exact source via the Dependency Explorer. Metrics are", "traces to its exact source via the Ontology Graph. Metrics are")
rep("with a one-click <b>Open in Dependency Explorer</b>", "with a one-click <b>Open in Ontology Graph</b>")
rep("{heading:'The Ontology Explorer \u2014 see the dependency graph'", "{heading:'The Ontology Graph \u2014 see the dependency graph'")
rep("<b>Admin \u2192 Data Model \u2192 Dependency Explorer</b>", "<b>Admin \u2192 Data Model \u2192 Ontology Graph</b>")
rep("\\u201cOpen in Dependency Explorer\\u201d link", "\\u201cOpen in Ontology Graph\\u201d link")
rep("The Dependency Explorer is schema-level", "The Ontology Graph is schema-level")
rep("The canonical dependency graph behind the Dependency Explorer", "The canonical dependency graph behind the Ontology Graph")
rep(">Ontology Explorer</a>'s dependency graph", ">Ontology Graph</a>'s dependency graph")
rep("in the Ontology Explorer's inspector", "in the Ontology Graph's inspector")
rep("show up in the Dependency Explorer too", "show up in the Ontology Graph too")
rep("the Ontology Explorer visual redesign", "the Ontology Graph visual redesign")
rep("traces to its exact source via the Dependency Explorer - metrics", "traces to its exact source via the Ontology Graph - metrics")
rep("open it in the Dependency Explorer to see precisely", "open it in the Ontology Graph to see precisely")

print('rename done, fails so far:', len(fails))

# ---------------- 2. Edge taxonomy comment + systemGraphEdges rewrite ----------------
rep("""// `{edgeType, fromType, fromId, toType, toId}` - 4 of the spec's 10 edge
// types (depends_on/invokes/delegates_to/derives_from) populated so far,
// derived from already-reliable fields only, matching the real edition's
// own sync_graph_node helpers in each owning Rust service.""",
"""// `{edgeType, fromType, fromId, toType, toId}` - every label comes from
// docs/RELATIONSHIP_TAXONOMY.md (epic #359, issue #360); the crawler must
// never emit a label that isn't in that file. Derived from
// already-reliable fields only, matching the real edition's own
// sync_graph_node helpers in each owning Rust service.""")

NEW_EDGES = '''function systemGraphEdges(){
 const edges=[];
 const add=(edgeType,fromType,fromId,toType,toId)=>{if(fromId&&toId)edges.push({edgeType,fromType,fromId,toType,toId})};
 // Object -> Field: the object HAS_FIELDs, so fields read as the object's
 // own downstream "lineage" (what it is built from).
 (data.customFields||[]).forEach(f=>add('has_field','custom_object',f.entity,'custom_field',f.id));
 (data.relationshipDefinitions||[]).forEach(r=>{
  add('related_via','custom_object',r.sourceEntity,'relationship',r.id);
  add('links_to','relationship',r.id,'custom_object',r.sourceEntity);
  if(!r.targetIsPolymorphic)add('links_to','relationship',r.id,'custom_object',r.targetEntity);
 });
 (data.fieldRules||[]).forEach(r=>{
  add('enforced_on','business_rule',r.id,'custom_object',r.entity);
  (r.actions||[]).forEach(a=>{
   if(a.targetField){
    const f=(data.customFields||[]).find(x=>x.entity===r.entity&&x.key===a.targetField);
    if(f)add('validates','business_rule',r.id,'custom_field',f.id);
   }
  });
 });
 (data.workflowRules||[]).forEach(w=>{
  add('triggered_by','workflow',w.id,'custom_object',w.entity);
  (w.actions||[]).forEach(a=>{
   // run_ai_agent is the only workflow action type that names another
   // component in this demo - create_task/notify carry no component ref,
   // so reads/writes edges have no demo mapping yet (see PR body).
   if(a.type==='run_ai_agent'&&a.targetType==='agent')add('can_invoke','ai_agent',a.targetId,'workflow',w.id);
  });
 });
 Object.entries(data.uiLayouts||{}).forEach(([entityKey,arr])=>(arr||[]).forEach(l=>add('renders','screen_layout',l.id,'custom_object',entityKey)));
 Object.entries(data.pageLayouts||{}).forEach(([entityKey,arr])=>(arr||[]).forEach(l=>add('displays','page_layout',l.id,'custom_object',entityKey)));
 // Field shown_on Screen Layout, derived from layout section field keys.
 // Built-in field keys match no custom field - only custom fields get edges.
 const layoutFieldEdges=(layoutId,tabs,entityKey)=>{(tabs||[]).forEach(t=>{(t.sections||[]).forEach(s=>{(s.fields||[]).forEach(f=>{
  const key=typeof f==='string'?f:f.key;
  const field=(data.customFields||[]).find(x=>x.entity===entityKey&&x.key===key);
  if(field)add('shown_on','custom_field',field.id,'screen_layout',layoutId);
 })})})};
 Object.entries(data.uiLayouts||{}).forEach(([entityKey,arr])=>(arr||[]).forEach(l=>{layoutFieldEdges(l.id,l.draftTabs,entityKey);layoutFieldEdges(l.id,l.publishedTabs,entityKey)}));
 (data.aiAgents||[]).forEach(a=>(a.delegateAgentIds||[]).forEach(d=>add('delegates_to','ai_agent',a.id,'ai_agent',d)));
 // Agent Ontology scopes become typed access edges. Only read/update/delete
 // have taxonomy labels - 'create'/'assign' capabilities have no demo
 // mapping yet (see PR body).
 (data.aiAgents||[]).forEach(a=>{
  (a.objectTypeScopes||[]).forEach(scope=>{
   const caps=scope.capabilities||[];
   if(caps.includes('read'))add('can_read','ai_agent',a.id,'custom_object',scope.objectType);
   if(caps.includes('update')||caps.includes('assign'))add('can_update','ai_agent',a.id,'custom_object',scope.objectType);
   if(caps.includes('delete'))add('can_delete','ai_agent',a.id,'custom_object',scope.objectType);
  });
  (a.actionTypeIds||[]).forEach(actionTypeId=>add('enables','action_type_definition',actionTypeId,'ai_agent',a.id));
 });
 // An Action Type acts on its target object type ('acts_on' was added to
 // docs/RELATIONSHIP_TAXONOMY.md in this same PR).
 (data.actionTypeDefinitions||[]).forEach(a=>add('acts_on','action_type_definition',a.id,'custom_object',a.targetObjectType));
 (data.aiExecutionGraphs||[]).forEach(g=>(g.nodes||[]).forEach(n=>{if(n.type==='agent'&&n.config&&n.config.agentId)add('includes','execution_graph',g.id,'ai_agent',n.config.agentId)}));
 (data.semanticMappings||[]).forEach(m=>{
  if(!m.glossaryTermId)return;
  if(m.fieldKey){
   const field=(data.customFields||[]).find(f=>f.entity===m.entityType&&f.key===m.fieldKey);
   if(field)add('defines','business_glossary_term',m.glossaryTermId,'custom_field',field.id);
   else add('defines','business_glossary_term',m.glossaryTermId,'custom_object',m.entityType);
  }else add('defines','business_glossary_term',m.glossaryTermId,'custom_object',m.entityType);
 });
 (data.metricDefinitions||[]).forEach(metric=>{
  add('computed_from','metric_definition',metric.id,'custom_object',metric.sourceEntityType);
  if(metric.sourceFieldKey){
   const field=(data.customFields||[]).find(f=>f.entity===metric.sourceEntityType&&f.key===metric.sourceFieldKey);
   if(field)add('computed_from','metric_definition',metric.id,'custom_field',field.id);
  }
  if(metric.glossaryTermId)add('defines','business_glossary_term',metric.glossaryTermId,'metric_definition',metric.id);
 });
 // FND-03's rule ("a built-in entity type never gets a test-case edge") is
 // relaxed for this demo: the demo's own nodes already include built-in
 // object types explicitly (see systemGraphNodes), and the taxonomy says a
 // Test Case covers any Component - so "Company Dedupe Test" can cover
 // Companies for real instead of pointing at nothing.
 const builtinKeys=ONTOLOGY_BUILTIN_TYPES.map(([key])=>key);
 (data.testCaseDefinitions||[]).forEach(c=>{
  if(['business_rule','workflow','access_security','screen_visibility'].includes(c.testType)){
   if((data.customObjects||[]).some(o=>o.key===c.targetId)||builtinKeys.includes(c.targetId))add('covers','test_case_definition',c.id,'custom_object',c.targetId);
  }else if(c.testType==='agent_eval')add('covers','test_case_definition',c.id,'ai_agent',c.targetId);
  else if(c.testType==='agent_team_eval')add('covers','test_case_definition',c.id,'execution_graph',c.targetId);
 });
 return edges;
}'''
rep_span('function systemGraphEdges(){', 'return edges;\n}', NEW_EDGES)

# ---------------- 3. systemGraphTransitive: fan-out guard ----------------
NEW_TRANSITIVE = '''function systemGraphTransitive(type,id,direction,maxDepth){
 const edges=systemGraphEdges();
 const visited=new Map();
 let truncated=0;
 let frontier=[{type,id}];
 for(let depth=1;depth<=maxDepth&&frontier.length;depth++){
  const next=[];
  frontier.forEach(node=>{
   const matches=direction==='dependents'?edges.filter(e=>e.toType===node.type&&e.toId===node.id):edges.filter(e=>e.fromType===node.type&&e.fromId===node.id);
   let addedThisNode=0;
   matches.forEach(e=>{
    const n=direction==='dependents'?{type:e.fromType,id:e.fromId}:{type:e.toType,id:e.toId};
    const key=n.type+':'+n.id;
    if(!visited.has(key)){
     // Fan-out guard: one expansion adding >150 new nodes renders the
     // first 150 and reports the rest - the caller shows a "+N more" chip.
     if(addedThisNode>=150){truncated++;return}
     addedThisNode++;
     visited.set(key,{type:n.type,id:n.id,edgeType:e.edgeType,depth});next.push(n)
    }
   });
  });
  frontier=next;
 }
 const out=Array.from(visited.values()).sort((a,b)=>a.depth-b.depth);
 out.truncated=truncated;
 return out;
}'''
rep_span('function systemGraphTransitive(type,id,direction,maxDepth){',
          'return Array.from(visited.values()).sort((a,b)=>a.depth-b.depth);\n}',
          NEW_TRANSITIVE)

# ---------------- 4. Colors: 13 unique ----------------
rep("const SYSTEM_NODE_TYPE_COLOR_INDEX={custom_object:1,custom_field:2,relationship:3,business_rule:4,workflow:5,screen_layout:6,page_layout:1,ai_agent:2,execution_graph:3,business_glossary_term:4,metric_definition:5,test_case_definition:6,action_type_definition:1};",
    "const SYSTEM_NODE_TYPE_COLORS={custom_object:'#4F46E5',custom_field:'#16A34A',relationship:'#D97706',business_rule:'#DC2626',workflow:'#2563EB',screen_layout:'#7C3AED',page_layout:'#0D9488',ai_agent:'#059669',execution_graph:'#EA580C',business_glossary_term:'#E11D48',metric_definition:'#0284C7',test_case_definition:'#9333EA',action_type_definition:'#475569'};")
rep('background:var(--chart-${SYSTEM_NODE_TYPE_COLOR_INDEX[nodeType]})',
    "background:${SYSTEM_NODE_TYPE_COLORS[nodeType]||'#888'}")
rep('background:var(--chart-${SYSTEM_NODE_TYPE_COLOR_INDEX[dependencyExplorerState.type]})',
    'background:${SYSTEM_NODE_TYPE_COLORS[dependencyExplorerState.type]}', 2)
rep('background:var(--chart-${SYSTEM_NODE_TYPE_COLOR_INDEX[h.type]})',
    'background:${SYSTEM_NODE_TYPE_COLORS[h.type]}')
rep('background:var(--chart-${SYSTEM_NODE_TYPE_COLOR_INDEX[hit.type]})',
    'background:${SYSTEM_NODE_TYPE_COLORS[hit.type]}')

# ---------------- 5. Edge type display labels ----------------
rep("const SYSTEM_NODE_TYPE_LABELS={custom_object:'Object'",
    "const SYSTEM_EDGE_TYPE_LABELS={has_field:'Has field',shown_on:'Shown on',renders:'Renders',displays:'Displays',embeds:'Embeds',related_via:'Related via',links_to:'Links to',has_many:'Has many',belongs_to:'Belongs to',many_to_many:'Many to many',enforced_on:'Enforced on',validates:'Validates',triggered_by:'Triggered by',reads:'Reads',writes:'Writes',covers:'Covers',enables:'Enables',acts_on:'Acts on',can_read:'Can read',can_update:'Can update',can_delete:'Can delete',can_invoke:'Can invoke',constrained_by:'Constrained by',delegates_to:'Delegates to',includes:'Includes',manages:'Manages',computed_from:'Computed from',defines:'Defines',linked_record:'Linked record',has_business_unit:'Has business unit',scoped_agent:'Scoped agent',applies_to:'Applies to',ran_on:'Ran on',agent_can_read:'Agent can read',agent_can_update:'Agent can update'};\nfunction edgeTypeLabel(t){return SYSTEM_EDGE_TYPE_LABELS[t]||String(t).replace(/_/g,' ')}\nconst SYSTEM_NODE_TYPE_LABELS={custom_object:'Object'")
rep('ontology-inspector-group-label">${edgeType}</div>',
    'ontology-inspector-group-label">${edgeTypeLabel(edgeType)}</div>')
rep('(${h.edgeType})', '(${edgeTypeLabel(h.edgeType)})')
rep('(${hit.edgeType})', '(${edgeTypeLabel(hit.edgeType)})')

# ---------------- 6. Depth selector ----------------
rep("let dependencyExplorerState={type:'custom_object',id:null,inspectType:null,inspectId:null,showImpact:true,showLineage:true};",
    "let dependencyExplorerState={type:'custom_object',id:null,depth:'2',inspectType:null,inspectId:null,showImpact:true,showLineage:true};\nfunction depMaxDepth(){const d=dependencyExplorerState.depth;return d==='all'?25:(parseInt(d,10)||2)}")
rep("if(first)dependencyExplorerState={type:first.type,id:first.id,inspectType:first.type,inspectId:first.id,showImpact:true,showLineage:true};",
    "if(first)dependencyExplorerState={type:first.type,id:first.id,depth:dependencyExplorerState.depth||'2',inspectType:first.type,inspectId:first.id,showImpact:true,showLineage:true};")
rep("const t=e.target.value;const first=nodes.find(n=>n.type===t);dependencyExplorerState={type:t,id:first?first.id:null,inspectType:t,inspectId:first?first.id:null,showImpact:true,showLineage:true};",
    "const t=e.target.value;const first=nodes.find(n=>n.type===t);dependencyExplorerState={type:t,id:first?first.id:null,depth:dependencyExplorerState.depth||'2',inspectType:t,inspectId:first?first.id:null,showImpact:true,showLineage:true};")
rep("dependencyExplorerState={type,id,inspectType:type,inspectId:id,showImpact:true,showLineage:true};",
    "dependencyExplorerState={type,id,depth:dependencyExplorerState.depth||'2',inspectType:type,inspectId:id,showImpact:true,showLineage:true};")
rep("""   const hits=[
    ...(dependencyExplorerState.showImpact?systemGraphTransitive(dependencyExplorerState.type,dependencyExplorerState.id,'dependents',25):[]),
    ...(dependencyExplorerState.showLineage?systemGraphTransitive(dependencyExplorerState.type,dependencyExplorerState.id,'dependencies',25):[]),
   ];""",
"""   const depDepth=depMaxDepth();
   const impactHits=dependencyExplorerState.showImpact?systemGraphTransitive(dependencyExplorerState.type,dependencyExplorerState.id,'dependents',depDepth):[];
   const lineageHits=dependencyExplorerState.showLineage?systemGraphTransitive(dependencyExplorerState.type,dependencyExplorerState.id,'dependencies',depDepth):[];
   const hits=[...impactHits,...lineageHits];
   truncatedHits=(impactHits.truncated||0)+(lineageHits.truncated||0);""")
rep(""" let graphHtml='';
 if(dependencyExplorerState.id){""",
""" let graphHtml='';
 let truncatedHits=0;
 if(dependencyExplorerState.id){""")
rep("""<label>Component<select id="depComponent">${options.map(o=>`<option value="${o.id}" ${o.id===dependencyExplorerState.id?'selected':''}>${o.label}</option>`).join('')}</select></label>""",
"""<label>Component<select id="depComponent">${options.map(o=>`<option value="${o.id}" ${o.id===dependencyExplorerState.id?'selected':''}>${o.label}</option>`).join('')}</select></label>
   <label>Depth<select id="depDepth">${['1','2','3','all'].map(d=>`<option value="${d}" ${String(dependencyExplorerState.depth)===d?'selected':''}>${d==='all'?'All':d+' hop'+(d==='1'?'':'s')}</option>`).join('')}</select></label>""")
rep("""Full lineage (everything downstream)</button>
  </div>""",
"""Full lineage (everything downstream)</button>
   ${truncatedHits?`<span class="ontology-more-chip" title="Narrow the depth or hide types to see the rest">+${truncatedHits} more</span>`:''}
  </div>""")
rep("""$('#depComponent')&&($('#depComponent').onchange=(e)=>{dependencyExplorerState={...dependencyExplorerState,id:e.target.value,inspectType:dependencyExplorerState.type,inspectId:e.target.value,showImpact:true,showLineage:true};renderAdminTab()});""",
"""$('#depComponent')&&($('#depComponent').onchange=(e)=>{dependencyExplorerState={...dependencyExplorerState,id:e.target.value,inspectType:dependencyExplorerState.type,inspectId:e.target.value,showImpact:true,showLineage:true};renderAdminTab()});
 $('#depDepth')&&($('#depDepth').onchange=(e)=>{dependencyExplorerState={...dependencyExplorerState,depth:e.target.value};renderAdminTab()});""")


# ---------------- 7. Demo-data enrichment (all 13 types selectable) ----------------
# Guarded: re-running never duplicates the block. (Sections 1-6 are
# single-shot by design - they fail loudly instead on a second run, so the
# workflow can never silently double-apply.)
OG_DEMODATA = """ // Ontology Graph (epic #359, PR 1): one-time demo-data enrichment so all
 // 13 graph component types have selectable components in the explorer.
 // Same one-time-flag convention as aiFoundrySeeded above - runs once for
 // existing workspaces too. Every push is id-guarded, so re-running never
 // duplicates.
 data.workspace=data.workspace||{};
 if(!data.workspace.ontologyGraphSeeded){
  data.workspace.ontologyGraphSeeded=true;
  const ogHasId=(arr,id)=>(arr||[]).some(x=>x.id===id);
  // Name the seeded rules/workflows - they render "Untitled ..." otherwise.
  const ogFr1=(data.fieldRules||[]).find(r=>r.id==='fr1');if(ogFr1&&!ogFr1.name)ogFr1.name='Require Industry';
  const ogWf1=(data.workflowRules||[]).find(w=>w.id==='wf1');if(ogWf1&&!ogWf1.name)ogWf1.name='New Company Onboarding';
  const ogWf2=(data.workflowRules||[]).find(w=>w.id==='wf2');if(ogWf2&&!ogWf2.name)ogWf2.name='Overdue Invoice Follow-up';
  // Extra Company fields (they cluster in the graph's Field group).
  if(!ogHasId(data.customFields,'cf_industry'))data.customFields.push(stampCreate({id:'cf_industry',entity:'companies',key:'industry',label:'Industry',type:'select',options:'Technology|Healthcare|Finance|Manufacturing|Retail',active:true,defaultValue:'',unique:false,helpText:'',placeholder:''}));
  if(!ogHasId(data.customFields,'cf_revenue'))data.customFields.push(stampCreate({id:'cf_revenue',entity:'companies',key:'annualRevenue',label:'Annual Revenue',type:'currency',options:'',active:true,defaultValue:'',unique:false,helpText:'',placeholder:''}));
  // Custom object: Partner Tier.
  if(!(data.customObjects||[]).some(o=>o.key==='partner_tiers'))data.customObjects.push(stampCreate({key:'partner_tiers',label:'Partner Tier',labelPlural:'Partner Tiers',icon:'🤝',description:'Partner tier classification for channel partners.',active:true,ownershipMode:'USER_TEAM_OWNED'}));
  if(!data['partner_tiers'])data['partner_tiers']=[];
  // Relationships: Company -> Contact / Opportunity / Quote.
  const ogRel=(id,key,source,target,fwd,rev)=>{if(!ogHasId(data.relationshipDefinitions,id))data.relationshipDefinitions.push(stampCreate({id,key,sourceEntity:source,targetEntity:target,relType:'one_to_many',forwardLabel:fwd,reverseLabel:rev,deleteBehavior:'restrict',showRelatedList:true,required:false,active:true,protected:false}))};
  ogRel('rel_cc1','company_contacts','companies','contacts','Contacts','Company');
  ogRel('rel_co1','company_opportunities','companies','opportunities','Opportunities','Company');
  ogRel('rel_cq1','company_quotes','companies','quotes','Quotes','Company');
  // Screen + page layouts for companies (factory fns = the exact shape the UI writes).
  const ogSl=ensureLayouts('companies');
  if(!ogSl.some(l=>l.name==='Company Detail Screen'))ogSl.push(freshLayout('companies','Company Detail Screen',false));
  const ogPl=ensurePageLayouts('companies');
  if(!ogPl.some(l=>l.name==='Company Record Page'))ogPl.push(freshPageLayout('Company Record Page',false));
  // Glossary term "Account vs Company", mapped to the Companies object.
  if(!ogHasId(data.businessGlossaryTerms,'gt_acct_co'))data.businessGlossaryTerms.push(stampCreate({id:'gt_acct_co',name:'Account vs Company',definition:'An Account is the commercial relationship; a Company is the legal entity behind it. One company can hold many accounts.',ownerUserId:null,synonyms:['account','company'],dataClassification:'standard',active:true}));
  if(!(data.semanticMappings||[]).some(m=>m.glossaryTermId==='gt_acct_co'))(data.semanticMappings=data.semanticMappings||[]).push(stampCreate({id:'sm_acct_co',glossaryTermId:'gt_acct_co',entityType:'companies',fieldKey:null}));
  // Metric: Pipeline by Company.
  if(!ogHasId(data.metricDefinitions,'met_pipe_co'))data.metricDefinitions.push(stampCreate({id:'met_pipe_co',key:'pipeline_by_company',name:'Pipeline by Company',description:'Open pipeline value grouped by company.',aggregation:'sum',sourceEntityType:'opportunities',sourceFieldKey:'',grain:'company',filters:[],timeLogic:'',ownerUserId:null,version:1,active:true,versions:[]}));
  // Test case: Company Dedupe Test (covers Companies - see the relaxed rule in systemGraphEdges).
  if(!ogHasId(data.testCaseDefinitions,'tc_dedupe'))data.testCaseDefinitions.push(stampCreate({id:'tc_dedupe',name:'Company Dedupe Test',description:'Flags probable duplicate company records.',testType:'business_rule',targetId:'companies',datasetJson:'{}',active:true}));
  // Action Type: Send Email, scoped to companies.
  if(!ogHasId(data.actionTypeDefinitions,'at_send_email'))data.actionTypeDefinitions.push(stampCreate({id:'at_send_email',targetObjectType:'companies',name:'Send Email',description:'Send a templated email to the primary contact.',requiredCapability:'update',parameters:[],effect:'send_email',effectConfigJson:'{}',isActive:true}));
  // Agent team: Sales Ops Team (execution graph with agent nodes).
  if(!ogHasId(data.aiExecutionGraphs,'team_sales_ops')){
   const ogSup=(data.aiAgents||[]).find(a=>a.name==='Ops Supervisor'),ogRes=(data.aiAgents||[]).find(a=>a.name==='Research Analyst');
   const ogNodes=[{key:'trigger_1',type:'trigger',config:{},x:40,y:40}];
   if(ogSup)ogNodes.push({key:'agent_1',type:'agent',config:{agentId:ogSup.id,inputTemplate:'{{trigger_input}}'},x:40,y:120});
   if(ogRes)ogNodes.push({key:'agent_2',type:'agent',config:{agentId:ogRes.id,inputTemplate:'{{previous_output}}'},x:40,y:200});
   ogNodes.push({key:'end_1',type:'end',config:{},x:40,y:280});
   data.aiExecutionGraphs.push({id:'team_sales_ops',name:'Sales Ops Team',description:'Supervised research and ops agents for the sales org.',status:'published',version:1,nodes:ogNodes,edges:[]});
  }
  // Agent ontology scopes -> typed access edges (can_read/can_update/...).
  const ogScope=(name,scopes,actionTypes)=>{const a=(data.aiAgents||[]).find(x=>x.name===name);if(a){a.objectTypeScopes=scopes;if(actionTypes)a.actionTypeIds=actionTypes}};
  ogScope('Research Analyst',[{objectType:'companies',capabilities:['read']}]);
  ogScope('Ops Supervisor',[{objectType:'companies',capabilities:['read','update']}],['at_send_email']);
  ogScope('Draft Writer',[]);
 }"""
if 'ontologyGraphSeeded' not in src:
    rep("  ],runs:[]}));\n }\n if(!data.aiSettings.vectorSearch)",
        "  ],runs:[]}));\n }\n\n" + OG_DEMODATA + "\n if(!data.aiSettings.vectorSearch)")

print('demo-data section done, fails so far:', len(fails))

open(PATH, 'w').write(src)
print('all sections done, fails:', len(fails))
for f in fails: print(' -', f)
sys.exit(1 if fails else 0)
