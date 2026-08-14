const $ = (s, el=document) => el.querySelector(s);
const money = n => new Intl.NumberFormat('en-US',{style:'currency',currency:'USD',maximumFractionDigits:2}).format(Number(n||0));
const seed = {
 companies:[
  {id:'c1',name:'BrightPath Logistics',industry:'Logistics',city:'Toronto',owner:'Maya Chen',status:'Customer'},
  {id:'c2',name:'Harbour Health Group',industry:'Healthcare',city:'Vancouver',owner:'Noah Williams',status:'Prospect'},
  {id:'c3',name:'Atlas Construction',industry:'Construction',city:'Calgary',owner:'Maya Chen',status:'Customer'},
  {id:'c4',name:'Maple & Co Retail',industry:'Retail',city:'Ottawa',owner:'Liam Singh',status:'Prospect'},
  {id:'c5',name:'Northern Peak Foods',industry:'Food & Beverage',city:'Milton',owner:'Liam Singh',status:'Customer'},
  {id:'c6',name:'Bluewave Legal',industry:'Professional Services',city:'Toronto',owner:'Maya Chen',status:'Lead'}
 ],
 contacts:[
  {id:'p1',name:'Ava Martin',companyId:'c1',email:'ava@brightpath.example',phone:'416-555-0101',role:'COO',status:'Active'},
  {id:'p2',name:'Ethan Wong',companyId:'c2',email:'ethan@harbour.example',phone:'604-555-0130',role:'Director, Operations',status:'Active'},
  {id:'p3',name:'Sophia Patel',companyId:'c3',email:'sophia@atlas.example',phone:'403-555-0188',role:'Finance Manager',status:'Active'},
  {id:'p4',name:'Oliver Brown',companyId:'c4',email:'oliver@mapleco.example',phone:'613-555-0118',role:'Founder',status:'Active'}
 ],
 opportunities:[
  {id:'o1',title:'CRM Modernization',companyId:'c1',contactId:'p1',value:42000,stage:'Proposal',probability:70,close:'2026-08-28',owner:'Maya Chen',status:'Open'},
  {id:'o2',title:'Managed IT Support',companyId:'c2',contactId:'p2',value:28800,stage:'Discovery',probability:40,close:'2026-09-15',owner:'Noah Williams',status:'Open'},
  {id:'o3',title:'Cloud Migration',companyId:'c3',contactId:'p3',value:68000,stage:'Negotiation',probability:85,close:'2026-08-18',owner:'Maya Chen',status:'Open'},
  {id:'o4',title:'Retail Analytics',companyId:'c4',contactId:'p4',value:35500,stage:'Qualified',probability:30,close:'2026-10-01',owner:'Liam Singh',status:'Open'},
  {id:'o5',title:'Security Assessment',companyId:'c5',contactId:'',value:12000,stage:'Won',probability:100,close:'2026-07-20',owner:'Liam Singh',status:'Won'},
  {id:'o6',title:'Website Redesign',companyId:'c6',contactId:'',value:18500,stage:'Lead',probability:15,close:'2026-10-20',owner:'Maya Chen',status:'Open'}
 ],
 products:[
  {id:'pr1',name:'CRM Implementation',sku:'SVC-CRM',type:'Service',category:'Professional Service',price:35000,tax:13,status:'Active'},
  {id:'pr2',name:'Managed IT Support',sku:'SVC-MSP',type:'Service',category:'Subscription',price:2400,tax:13,status:'Active'},
  {id:'pr3',name:'Security Assessment',sku:'SVC-SEC',type:'Service',category:'Professional Service',price:7500,tax:13,status:'Active'},
  {id:'pr4',name:'Cloud Migration',sku:'SVC-CLD',type:'Service',category:'Professional Service',price:48000,tax:13,status:'Active'},
  {id:'pr5',name:'Wireless Access Point',sku:'PRD-WAP',type:'Product',category:'Hardware',price:850,tax:13,status:'Active'}
 ],
 quotes:[
  {id:'q1',number:'Q-2026-041',companyId:'c1',contactId:'p1',opportunityId:'o1',status:'Sent',date:'2026-07-28',valid:'2026-08-28',items:[{productId:'pr1',quantity:1,unitPrice:35000},{productId:'pr5',quantity:8,unitPrice:875}]},
  {id:'q2',number:'Q-2026-042',companyId:'c3',contactId:'p3',opportunityId:'o3',status:'Accepted',date:'2026-07-22',valid:'2026-08-22',items:[{productId:'pr4',quantity:1,unitPrice:48000},{productId:'pr1',quantity:0.57,unitPrice:35000}]},
  {id:'q3',number:'Q-2026-043',companyId:'c4',contactId:'p4',opportunityId:'',status:'Draft',date:'2026-07-30',valid:'2026-08-30',items:[{productId:'pr1',quantity:1,unitPrice:35500}]}
 ],
 orders:[
  {id:'so1',number:'SO-2026-019',companyId:'c3',contactId:'p3',quoteId:'q2',status:'In Progress',date:'2026-07-24',items:[{productId:'pr4',quantity:1,unitPrice:48000},{productId:'pr1',quantity:0.57,unitPrice:35000}]},
  {id:'so2',number:'SO-2026-018',companyId:'c5',contactId:'',quoteId:'',status:'Completed',date:'2026-07-12',items:[{productId:'pr3',quantity:1.6,unitPrice:7500}]}
 ],
 invoices:[
  {id:'i1',number:'INV-2026-081',companyId:'c5',orderId:'so2',status:'Paid',due:'2026-07-31',items:[{productId:'pr3',quantity:1.6,unitPrice:7500}]},
  {id:'i2',number:'INV-2026-082',companyId:'c3',orderId:'so1',status:'Sent',due:'2026-08-15',items:[{productId:'pr4',quantity:0.7083,unitPrice:48000}]},
  {id:'i3',number:'INV-2026-079',companyId:'c1',orderId:'',status:'Overdue',due:'2026-07-20',items:[{productId:'pr2',quantity:3.5417,unitPrice:2400}]}
 ],
 contracts:[
  {id:'ct1',number:'CTR-2026-011',companyId:'c5',contactId:'',title:'Security Services Agreement',value:12000,status:'Active',start:'2026-07-15',end:'2026-10-15'},
  {id:'ct2',number:'CTR-2026-009',companyId:'c1',contactId:'p1',title:'Support Retainer',value:28800,status:'Renewal Due',start:'2025-09-01',end:'2026-08-31'}
 ],
 tasks:[
  {id:'t1',title:'Follow up on CRM proposal',relatedType:'Quote',relatedId:'q1',owner:'Maya Chen',due:'2026-08-02',priority:'High',status:'Open'},
  {id:'t2',title:'Prepare discovery workshop',relatedType:'Opportunity',relatedId:'o2',owner:'Noah Williams',due:'2026-08-04',priority:'Medium',status:'Open'},
  {id:'t3',title:'Review contract renewal',relatedType:'Contract',relatedId:'ct2',owner:'Maya Chen',due:'2026-08-07',priority:'High',status:'Open'},
  {id:'t4',title:'Send final invoice',relatedType:'Invoice',relatedId:'i1',owner:'Liam Singh',due:'2026-07-30',priority:'Low',status:'Completed'}
 ],
 workspace:{name:'Northstar Digital Solutions',address:'120 Bay Street, Suite 400',city:'Toronto, ON',phone:'416-555-0142',logo:''},
 users:[
  {id:'u1',name:'Maya Chen',email:'maya@northstar.example',role:'Administrator',status:'Active'},
  {id:'u2',name:'Noah Williams',email:'noah@northstar.example',role:'Sales Rep',status:'Active'},
  {id:'u3',name:'Liam Singh',email:'liam@northstar.example',role:'Sales Rep',status:'Active'}
 ],
 customFields:[
  {id:'cf1',entity:'opportunities',key:'leadSource',label:'Lead Source',type:'select',options:'Referral|Website|Event|Cold Outreach|Partner',active:true,defaultValue:'',unique:false,helpText:'',placeholder:''},
  {id:'cf2',entity:'companies',key:'externalId',label:'External ID',type:'text',options:'',active:true,defaultValue:'',unique:true,helpText:'Must match the ID used in your accounting system.',placeholder:'e.g. ACCT-10432'}
 ],
 fieldRules:[
  {id:'fr1',entity:'opportunities',matchType:'all',
   conditions:[{fieldKey:'stage',operator:'equals',value:'Won',compareField:null,groupId:null}],
   actions:[{type:'require',targetField:'leadSource',value:'',message:''}],active:true}
 ],
 workflowRules:[
  {id:'wf1',entity:'opportunities',triggerField:'stage',toValue:'Won',operator:'equals',conditions:[],matchType:'all',
   actions:[{type:'create_task',taskTitle:'Kick off onboarding',daysOffset:2}],notify:true,active:true},
  {id:'wf2',entity:'invoices',triggerField:'status',toValue:'Overdue',operator:'equals',conditions:[],matchType:'all',
   actions:[{type:'create_task',taskTitle:'Follow up on overdue invoice',daysOffset:0}],notify:false,active:true}
 ],
 // Each row restricts exactly one from -> to move (from:'' means "from any
 // status") and carries its own active toggle - the same granularity as
 // the desktop edition's status_transitions rules, rather than one big
 // grouped rule per entity.
 statusTransitionRules:[
  {id:'st1',entity:'opportunities',active:true,from:'',to:'Qualified'},
  {id:'st2',entity:'opportunities',active:true,from:'Qualified',to:'Discovery'},
  {id:'st3',entity:'opportunities',active:true,from:'Discovery',to:'Proposal'},
  {id:'st4',entity:'opportunities',active:true,from:'Proposal',to:'Negotiation'},
  {id:'st5',entity:'opportunities',active:true,from:'Negotiation',to:'Won'},
  {id:'st6',entity:'opportunities',active:true,from:'Negotiation',to:'Lost'},
  {id:'st7',entity:'opportunities',active:true,from:'',to:'Lost'}
 ],
 numberingOverrides:{},
 kpiPrefs:[],
 notifications:[]
};
const storeKey='lanesra-os-demo-v10';
let data = JSON.parse(localStorage.getItem(storeKey)||'null') || structuredClone(seed);
let current='dashboard';
let viewFilter=null;
// Customer/Contact 360 (Phase 5): which record's detail page is open, if
// any - {type:'companies'|'contacts', id}. Cleared by any real navigation
// so the sidebar/breadcrumbs always return to the list.
let detailRecord=null;
const save=()=>localStorage.setItem(storeKey,JSON.stringify(data));
const uid=()=>Math.random().toString(36).slice(2,10);
const pad=(n,w=4)=>String(n).padStart(w,'0');
const year=()=>new Date().getFullYear();
ensureAdminData();
const numberRules={
 companies:{field:'customerNumber',prefix:'CUS',year:false,width:4},
 contacts:{field:'contactNumber',prefix:'CON',year:false,width:4},
 opportunities:{field:'opportunityNumber',prefix:'OPP',year:true,width:3},
 products:{field:'productNumber',prefix:'PRD',year:false,width:4},
 quotes:{field:'number',prefix:'Q',year:true,width:3},
 orders:{field:'number',prefix:'SO',year:true,width:3},
 invoices:{field:'number',prefix:'INV',year:true,width:3},
 contracts:{field:'number',prefix:'CTR',year:true,width:3},
 tasks:{field:'taskNumber',prefix:'TSK',year:false,width:4}
};
ensureNumbers(); // backfills any seed/imported record missing its auto-generated ID
function effectiveRule(key){
 const base=numberRules[key];
 if(base){
  const o=(data.numberingOverrides||{})[key];
  return o?{prefix:o.prefix,width:o.width||base.width,field:base.field,custom:true}:{...base,custom:false};
 }
 // Custom objects aren't in numberRules (their prefix/digits live on the
 // object definition itself, edited from the Custom Objects admin tab, not
 // through the built-ins-only Numbering-override screen) - but they still
 // get real auto-numbering through this same function.
 const co=customObjectByKey(key);
 if(!co)return null;
 return {prefix:co.prefix,width:co.digits,field:'number',year:false,custom:false};
}
function nextNumber(key){
 const r=effectiveRule(key); if(!r)return '';
 const base=r.custom?r.prefix:(r.year?`${r.prefix}-${year()}-`:`${r.prefix}-`);
 const nums=(data[key]||[]).map(x=>String(x[r.field]||'')).filter(v=>v.startsWith(base)).map(v=>Number(v.slice(base.length))).filter(Number.isFinite);
 return base+pad((nums.length?Math.max(...nums):0)+1,r.width);
}
function ensureNumbers(){
 Object.entries(numberRules).forEach(([key,r])=>{
  (data[key]||[]).slice().reverse().forEach(x=>{if(!x[r.field])x[r.field]=nextNumber(key)});
 });
 save();
}
// Second Admin Automation & Customization addendum round: business rules
// and workflows both moved from a single condition/single effect shape to
// conditions[]/actions[] arrays (with one level of OR-grouping via
// group_id - see conditionsMatch). These migrations upgrade any rule saved
// before this round in place, idempotently (a rule that already has
// `conditions` is left untouched) - so existing localStorage data and the
// seed both keep evaluating exactly as they always did until an admin
// edits the rule through the new builder.
function migrateFieldRule(r){
 if(r.active===undefined)r.active=true;
 if(r.conditions)return;
 const fieldKey=r.triggerField||transitionFieldFor(r.entity);
 r.conditions=[{fieldKey,operator:r.operator||'equals',value:r.triggerValue||'',compareField:r.compareField||null,groupId:null}];
 r.matchType='all';
 r.actions=[{type:r.effect==='hide'?'hide':'require',targetField:r.fieldKey,value:'',message:''}];
}
// v0.25 bug-report round: the separate "Trigger" (triggerField/operator/
// toValue/compareField) and "Extra conditions" sections were confusing -
// two places to define what amounts to one thing, unlike Salesforce/
// Dynamics' single unified entry-criteria list. They're now folded into
// one `conditions[]` (+ `matchType`), one-time and idempotent (guarded by
// `conditionsMerged`, so a rule already saved under the new unified
// builder is left untouched). The old trigger becomes the first,
// ungrouped condition (always AND'd in) so it's still mandatory; if the
// old extra conditions used matchType 'any' with more than one condition,
// they're bundled into a single OR-group so "trigger AND (extra1 OR
// extra2)" keeps meaning exactly what it did before - see
// groupConditionUnits/conditionsMatch for how a group is evaluated as one
// unit. `actions` replaces the single top-level actionType/taskTitle/etc.
// fields with an array, same as before.
function migrateWorkflowRule(r){
 if(r.active===undefined)r.active=true;
 if(!r.conditions)r.conditions=[];
 if(!r.matchType)r.matchType='all';
 if(!r.conditionsMerged){
  if(!r.operator)r.operator='equals';
  if(!r.triggerField)r.triggerField=transitionFieldFor(r.entity);
  const trigger={fieldKey:r.triggerField,operator:r.operator,value:r.toValue||'',compareField:r.compareField||null,groupId:null};
  const extras=r.conditions.map(c=>({...c}));
  if(extras.length>1&&r.matchType==='any'){const gid=newGroupId();extras.forEach(c=>c.groupId=gid)}
  r.conditions=[trigger,...extras];
  r.matchType='all';
  r.conditionsMerged=true;
 }
 if(r.actions)return;
 const actionType=r.actionType||'create_task';
 const action={type:actionType};
 if(actionType==='create_task'){action.taskTitle=r.taskTitle||'';action.daysOffset=Number(r.daysOffset||0)}
 else if(actionType==='create_record'){action.recordTargetEntity=r.recordTargetEntity||'';action.recordNameTemplate=r.recordNameTemplate||''}
 else if(actionType==='update_related_record'){action.relTargetEntity=r.relTargetEntity||'';action.relTargetField=r.relTargetField||'';action.relValue=r.relValue||''}
 else if(actionType==='update_field'){action.updateFieldKey=r.updateFieldKey||'';action.updateValue=r.updateValue||'';action.updateCopyFrom=r.updateCopyFrom||''}
 r.actions=[action];
}
function ensureAdminData(){
 if(!data.workspace)data.workspace={name:'Northstar Digital Solutions',address:'120 Bay Street, Suite 400',city:'Toronto, ON',phone:'416-555-0142',logo:''};
 if(!data.users)data.users=[{id:'u1',name:'Maya Chen',email:'maya@northstar.example',role:'Administrator',status:'Active'}];
 if(!data.customFields)data.customFields=[];
 if(!data.fieldRules)data.fieldRules=[];
 if(!data.workflowRules)data.workflowRules=[];
 if(!data.statusTransitionRules)data.statusTransitionRules=[];
 if(!data.numberingOverrides)data.numberingOverrides={};
 if(!data.kpiPrefs)data.kpiPrefs=[];
 if(!data.notifications)data.notifications=[];
 if(!data.customObjects)data.customObjects=[];
 (data.customObjects||[]).forEach(o=>{if(o.active===undefined)o.active=true;if(!data[o.key])data[o.key]=[]});
 if(!data.relationshipDefinitions)data.relationshipDefinitions=[];
 if(!data.relationshipInstances)data.relationshipInstances=[];
 (data.relationshipDefinitions||[]).forEach(d=>{if(d.active===undefined)d.active=true;if(d.showRelatedList===undefined)d.showRelatedList=true;if(d.required===undefined)d.required=false});
 if(!data.customReports)data.customReports=[];
 if(!data.uiLayouts)data.uiLayouts={};
 if(!data.integrationJobs)data.integrationJobs=[];
 if(!data.apiEndpoints)data.apiEndpoints=[];
 if(!data.externalConnections)data.externalConnections=[];
 (data.integrationJobs||[]).forEach(j=>{if(j.active===undefined)j.active=true;if(!j.runs)j.runs=[]});
 (data.apiEndpoints||[]).forEach(e=>{if(e.active===undefined)e.active=true});
 (data.externalConnections||[]).forEach(c=>{if(c.active===undefined)c.active=true;if(!c.calls)c.calls=[]});
 (data.fieldRules||[]).forEach(migrateFieldRule);
 (data.workflowRules||[]).forEach(migrateWorkflowRule);
 (data.customFields||[]).forEach(f=>{if(f.defaultValue===undefined)f.defaultValue='';if(f.unique===undefined)f.unique=false;if(f.helpText===undefined)f.helpText='';if(f.placeholder===undefined)f.placeholder='';if(f.required===undefined)f.required=false;if(f.maxLength===undefined)f.maxLength=null;if(f.pattern===undefined)f.pattern='';if(f.minValue===undefined)f.minValue='';if(f.maxValue===undefined)f.maxValue='';if(f.searchable===undefined)f.searchable=false;if(f.filterable===undefined)f.filterable=false;if(f.reportable===undefined)f.reportable=true;if(f.hiddenByDefault===undefined)f.hiddenByDefault=false});
 save();
}
const icons={dashboard:'▦',companies:'◫',contacts:'◎',pipeline:'⌁',products:'◇',quotes:'▤',orders:'▣',invoices:'$',contracts:'▧',tasks:'✓',reports:'▥'};
const labels={dashboard:'Dashboard',companies:'Companies',contacts:'Contacts',pipeline:'Sales Pipeline',products:'Products',quotes:'Quotes',orders:'Orders',invoices:'Invoices',contracts:'Contracts',tasks:'Tasks',reports:'Reports'};
// Admin extensibility: an admin-defined Custom Object (data.customObjects)
// gets its own sidebar entry, list/create/edit screens and record array
// (data[key]) exactly like a built-in entity - all it needs is an entry in
// labels/icons and an array at data[key]. syncCustomObjectRegistry keeps
// those in sync with data.customObjects and is called once at load, after
// any Custom Objects admin action, and after "Reset demo". The reserved
// set is captured from labels' built-in keys before any custom object can
// be added, so it never accidentally grows.
const RESERVED_ENTITY_KEYS=[...Object.keys(labels),'admin'];
function customObjectByKey(key){return (data.customObjects||[]).find(o=>o.key===key)}
function activeCustomObjects(){return (data.customObjects||[]).filter(o=>o.active)}
function activeCustomObjectKeys(){return activeCustomObjects().map(o=>o.key)}
function syncCustomObjectRegistry(){
 const activeKeys=activeCustomObjectKeys();
 activeCustomObjects().forEach(o=>{labels[o.key]=o.labelPlural;icons[o.key]=o.icon;if(!data[o.key])data[o.key]=[]});
 // Drop stale labels/icons for objects that were deactivated or deleted
 // since the last sync, so the sidebar never shows a ghost entry.
 Object.keys(labels).forEach(k=>{if(!RESERVED_ENTITY_KEYS.includes(k)&&!activeKeys.includes(k)){delete labels[k];delete icons[k]}});
}
syncCustomObjectRegistry();
// ---- Custom Relationships (admin extensibility, Phase B) ------------------
// Mirrors desktop's relationship_service exactly: an Administrator connects
// any two object types - built-in or custom - with a cardinality and a
// delete-behavior; any user can then link/unlink actual records through it
// from that record's edit form. "source" is the owning/"many" side (e.g.
// Contact -> Company already works this way via companyId); one_to_many
// and many_to_one are the same physical shape read from either direction,
// which is why forward/reverse labels exist instead of a 4th type value.
const RELATIONSHIP_TYPES=['many_to_one','one_to_one','many_to_many'];
const RELATIONSHIP_TYPE_LABELS={many_to_one:'Many-to-one (many source records, one target each)',one_to_one:'One-to-one',many_to_many:'Many-to-many'};
const DELETE_BEHAVIORS=['restrict','archive'];
const DELETE_BEHAVIOR_LABELS={restrict:'Restrict — block deleting a linked record',archive:'Archive — drop the link, keep both records'};
// Every entity type a relationship (or its record pickers) can reference -
// the same "built-in or any active custom object" vocabulary customFieldsFor
// etc. already use.
function allEntityTypeKeys(){return [...Object.keys(numberRules),...activeCustomObjectKeys()]}
// A record's natural display name, regardless of entity type - custom
// objects always have .name (see customObjectFields), built-ins each have
// their own primary field.
function recordDisplayName(entityType,r){
 if(!r)return '—';
 if(customObjectByKey(entityType))return r.name||r.number||'—';
 return r.name||r.title||r.number||'—';
}
function relationshipDefsFor(entityType){return (data.relationshipDefinitions||[]).filter(d=>d.active&&d.showRelatedList&&(d.sourceEntity===entityType||d.targetEntity===entityType))}
// Every related record for one record, across every active relationship it
// participates in from either direction - mirrors
// relationship_service::related_records_for. A relationship's forward
// label is shown from the source side, reverse label from the target side.
function relatedRecordsFor(entityType,entityId){
 const out=[];
 relationshipDefsFor(entityType).forEach(def=>{
  if(def.sourceEntity===entityType){
   (data.relationshipInstances||[]).filter(i=>i.definitionId===def.id&&i.sourceEntity===entityType&&i.sourceId===entityId).forEach(inst=>{
    const rec=byId(inst.targetEntity,inst.targetId); if(!rec)return;
    out.push({instanceId:inst.id,defKey:def.key,groupLabel:def.forwardLabel,entityType:inst.targetEntity,entityId:inst.targetId,displayName:recordDisplayName(inst.targetEntity,rec),status:rec.status||''});
   });
  }
  if(def.targetEntity===entityType){
   (data.relationshipInstances||[]).filter(i=>i.definitionId===def.id&&i.targetEntity===entityType&&i.targetId===entityId).forEach(inst=>{
    const rec=byId(inst.sourceEntity,inst.sourceId); if(!rec)return;
    out.push({instanceId:inst.id,defKey:def.key,groupLabel:def.reverseLabel,entityType:inst.sourceEntity,entityId:inst.sourceId,displayName:recordDisplayName(inst.sourceEntity,rec),status:rec.status||''});
   });
  }
 });
 return out;
}
function relationshipInstancesForRecord(entityType,entityId){
 return (data.relationshipInstances||[]).filter(i=>(i.sourceEntity===entityType&&i.sourceId===entityId)||(i.targetEntity===entityType&&i.targetId===entityId));
}
// Checked before a record is deleted: any 'restrict' relationship still
// linking to it blocks the delete (return a message); an 'archive'
// relationship just has its link rows silently cleared instead - the
// linked *other* record is never touched, only the link (matches
// relationship_service::enforce_delete_behavior / ADM-CR-06).
function relationshipDeleteCheck(entityType,entityId){
 for(const inst of relationshipInstancesForRecord(entityType,entityId)){
  const def=(data.relationshipDefinitions||[]).find(d=>d.id===inst.definitionId);
  const restrict=def?def.deleteBehavior==='restrict':true;
  if(restrict)return `This record is still linked to other records through a custom relationship${def?` (${def.forwardLabel} / ${def.reverseLabel})`:''} — unlink it first, or change the relationship's delete behavior to Archive.`;
 }
 return null;
}
function clearArchivableRelationshipInstances(entityType,entityId){
 const ids=relationshipInstancesForRecord(entityType,entityId).map(i=>i.id);
 if(ids.length)data.relationshipInstances=(data.relationshipInstances||[]).filter(i=>!ids.includes(i.id));
}
// Cardinality enforcement on link - mirrors relationship_service::link's
// match on relationship_type. many_to_many has no extra constraint beyond
// "not already linked", which the final check below always applies.
function relationshipLinkError(def,sourceId,targetId){
 const instances=data.relationshipInstances||[];
 if(def.relType==='many_to_one'&&instances.some(i=>i.definitionId===def.id&&i.sourceId===sourceId))
  return `This record is already linked as ${def.forwardLabel} — unlink it first to relink.`;
 if(def.relType==='one_to_one'&&instances.some(i=>i.definitionId===def.id&&(i.sourceId===sourceId||i.targetId===targetId)))
  return 'One or both records are already linked through this one-to-one relationship.';
 if(instances.some(i=>i.definitionId===def.id&&i.sourceId===sourceId&&i.targetId===targetId))
  return 'These two records are already linked.';
 return null;
}
// Admin panel: entities that support custom fields & business rules are every object with numbering.
// Workflow automation is limited to the entities Tasks can relate to (see relatedTypeFor).
let adminTab='profile';
let cfEntity='companies';
let ruleEntity='companies';
let wfEntity='companies';
let trEntity='companies';
let testingRules=false;
let testingWorkflow=false;
// null = show the list; 'create' = the new-rule builder; any other string
// is the id of an existing rule/workflow being edited in the builder.
let ruleBuilderMode=null;
let wfBuilderMode=null;
const ENTITY_SINGULAR={companies:'company',contacts:'contact',opportunities:'opportunity',products:'product',quotes:'quote',orders:'order',invoices:'invoice',contracts:'contract',tasks:'task'};
const relatedTypeFor={companies:'Company',contacts:'Contact',opportunities:'Opportunity',quotes:'Quote',orders:'Order',invoices:'Invoice',contracts:'Contract'};
// The demo has no generic relationship system (unlike the desktop
// edition's admin-defined relationships), but every entity already
// carries fixed foreign keys - this is that graph: for a given entity,
// which other entities have a field pointing back at it (the "down"
// direction, e.g. Company -> its Contacts), and which field.
const REVERSE_RELATIONS={
 companies:[['contacts','companyId'],['opportunities','companyId'],['quotes','companyId'],['orders','companyId'],['invoices','companyId'],['contracts','companyId']],
 contacts:[['opportunities','contactId'],['quotes','contactId'],['orders','contactId'],['contracts','contactId']],
 opportunities:[['quotes','opportunityId']],
 quotes:[['orders','quoteId']],
 orders:[['invoices','orderId']],
};
// Bidirectional relation graph powering "update_related_record": every
// entity reachable from a trigger entity through a single foreign-key
// hop, in either direction. Built automatically from REVERSE_RELATIONS -
// each [parent, [child,fk]] entry also registers the inverse ("up") hop,
// so a Contact can update its parent Company just as a Company can
// update its Contacts, without declaring every pair twice. Tasks' own
// relatedType/relatedId polymorphic link (both the "which task points at
// me" and "which record does this task point at" directions) is layered
// on top since it isn't a plain per-entity foreign key.
const RELATIONS={};
function addRelation(from,to,fk,direction){(RELATIONS[from]=RELATIONS[from]||[]).push({target:to,fk,direction})}
Object.entries(REVERSE_RELATIONS).forEach(([parent,children])=>{
 children.forEach(([child,fk])=>{
  addRelation(parent,child,fk,'down'); // parent -> child: child rows carry fk===parent.id
  addRelation(child,parent,fk,'up');   // child -> parent: parent row's id===child[fk]
 });
});
Object.keys(relatedTypeFor).forEach(entityKey=>{
 addRelation(entityKey,'tasks','relatedId','taskBack'); // entity -> its tasks: task.relatedType/relatedId point at this record
 addRelation('tasks',entityKey,'relatedId','taskLink');  // task -> the record it's related to
});
// Every built-in entity can be a "create_record" workflow target - matches
// the desktop edition's full creatable set.
const CREATABLE_RECORD_TYPES=['companies','contacts','opportunities','products','quotes','orders','invoices','contracts','tasks'];
// Targets that need a companyId to save at all (mirrors each entity's own
// field definition, see companyFields()/contactFields()/etc above).
const COMPANY_DEPENDENT_TYPES=['contacts','opportunities','quotes','orders','invoices','contracts'];
// Quotes/orders/invoices have no name/title field of their own - just an
// auto-generated number - so they're created from sensible defaults
// instead of a typed name/title template.
const UNNAMED_RECORD_TYPES=['quotes','orders','invoices'];
function createRecordTargetsFor(entityKey){
 // Company-dependent targets need a companyId - offer them only when the
 // triggering record itself carries (or is) one.
 return CREATABLE_RECORD_TYPES.filter(t=>!COMPANY_DEPENDENT_TYPES.includes(t)||entityKey==='companies'||fieldsFnFor(entityKey)?.().some(f=>f[0]==='companyId'));
}
function transitionFieldFor(key){return key==='opportunities'?'stage':'status'}
// The valid values for an entity's status/stage field, read straight off
// its own field definition - so the Status Transitions editor's from/to
// pickers always match whatever that select field actually allows.
function transitionOptionsFor(entityKey){
 const fn=fieldsFnFor(entityKey); if(!fn)return [];
 const field=fn().find(f=>f[0]===transitionFieldFor(entityKey));
 return field&&field[3]?field[3].split('|'):[];
}
function fieldsFnFor(key){
 const fn={companies:companyFields,contacts:contactFields,opportunities:opportunityFields,products:productFields,quotes:quoteFields,orders:orderFields,invoices:invoiceFields,contracts:contractFields,tasks:taskFields}[key];
 return fn||(customObjectByKey(key)?customObjectFields:undefined);
}
// A custom object's records all share this one fixed shape (matches the
// desktop edition's custom_records table exactly: auto number, name,
// status, owner, notes) - everything object-specific comes from custom
// fields, the same system every built-in entity already uses.
function customObjectFields(){return [['number','Record ID','auto'],['name','Name'],['status','Status','select','Active|Inactive|Archived'],['owner','Owner'],['notes','Notes']]}
function slugify(label){
 const parts=String(label).trim().split(/[^a-zA-Z0-9]+/).filter(Boolean);
 if(!parts.length)return 'field'+uid();
 return parts[0].toLowerCase()+parts.slice(1).map(w=>w[0].toUpperCase()+w.slice(1).toLowerCase()).join('');
}
// Custom object keys use the desktop edition's lowercase_underscore
// convention (distinct from slugify()'s camelCase, which is for field
// keys) since a custom object's key is a visible, permanent identifier
// shown in the admin UI - "vendor", not "vendorObject" or similar.
function slugifyObjectKey(label){return String(label).trim().toLowerCase().replace(/[^a-z0-9]+/g,'_').replace(/^_+|_+$/g,'')}
// Tuple shape is [key,label,type,opts,extra] - extra (index 4) carries the
// Phase 4 extensibility settings (default value/unique/help text/
// placeholder) that only custom fields have; built-in field tuples simply
// omit it, and every reader treats a missing extra as "no extras".
function customFieldsFor(entityKey){
 return (data.customFields||[]).filter(f=>f.entity===entityKey&&f.active).map(f=>{
  const extra={
   defaultValue:f.defaultValue||'',unique:!!f.unique,helpText:f.helpText||'',placeholder:f.placeholder||'',
   required:!!f.required,maxLength:f.maxLength||null,pattern:f.pattern||'',minValue:f.minValue??'',maxValue:f.maxValue??'',
   searchable:!!f.searchable,filterable:!!f.filterable,reportable:f.reportable!==false,
   hiddenByDefault:!!f.hiddenByDefault,
  };
  if(f.type==='boolean')return [f.key,f.label,'select','Yes|No',extra];
  if(f.type==='select')return [f.key,f.label,'select',f.options||'',extra];
  if(f.type==='number')return [f.key,f.label,'number',undefined,extra];
  if(f.type==='date')return [f.key,f.label,'date',undefined,extra];
  return [f.key,f.label,'text',undefined,extra];
 });
}
function fieldsFor(entityKey,builtinFn){return [...builtinFn(),...customFieldsFor(entityKey)]}
// Fields safe to use as a business-rule/workflow condition or action target:
// excludes the generated ID field and every relationship-picker field (those
// need a record picker, not a plain value comparison) - mirrors the desktop
// edition's core::domain::builtin_fields registry.
const NON_TARGETABLE_TYPES=['auto','relation','filteredContact','filteredOpportunity','filteredQuote','filteredOrder','dynamicRelation'];
function builtinFieldsFor(entityKey,{actionable}={}){
 const fn=fieldsFnFor(entityKey); if(!fn)return [];
 const tf=transitionFieldFor(entityKey);
 return fn().filter(f=>!NON_TARGETABLE_TYPES.includes(f[2])&&!(actionable&&f[0]===tf));
}
// Every field a rule/workflow can condition on: any built-in field plus any
// active custom field for that entity - not just the one status/stage field.
function conditionFieldsFor(entityKey){return [...builtinFieldsFor(entityKey),...customFieldsFor(entityKey)]}
// Every field a business rule's "Then" can require/hide: same set, minus the
// entity's status/stage field itself - that one keeps its own dedicated
// trigger mechanism rather than being a settable target, same as the desktop
// edition.
function actionableFieldsFor(entityKey){return [...builtinFieldsFor(entityKey,{actionable:true}),...customFieldsFor(entityKey)]}
function fieldLabelFor(entityKey,key){return (conditionFieldsFor(entityKey).find(f=>f[0]===key)||[])[1]||key}
// A condition's right-hand side, for display - another field's label when
// it's a field-to-field comparison, otherwise the literal value as a badge.
function describeComparand(entityKey,compareField,literalValue){return compareField?fieldLabelFor(entityKey,compareField):badgeMaybe(literalValue)}
const OPERATOR_LABELS={equals:'is',not_equals:'is not',contains:'contains',not_contains:'does not contain',starts_with:'starts with',ends_with:'ends with',in_list:'is one of',not_in_list:'is not one of',is_empty:'is empty',is_not_empty:'is not empty',greater_than:'is greater than',less_than:'is less than'};
const MATCH_TYPES=['all','any'];
// `in_list`/`not_in_list` split their value on this - the same convention
// select-field options already use ("Option A|Option B|Option C").
const LIST_SEPARATOR='|';
function operatorsForType(type){
 if(type==='select')return ['equals','not_equals','in_list','not_in_list'];
 if(type==='number'||type==='date')return ['equals','not_equals','greater_than','less_than','in_list','not_in_list','is_empty','is_not_empty'];
 return ['equals','not_equals','contains','not_contains','starts_with','ends_with','in_list','not_in_list','is_empty','is_not_empty'];
}
function operatorNeedsValue(op){return op!=='is_empty'&&op!=='is_not_empty'}
function operatorMatch(op,value,target){
 const v=value??'', t=target??'';
 switch(op){
  case 'not_equals':return String(v)!==String(t);
  case 'contains':return String(v).toLowerCase().includes(String(t).toLowerCase());
  case 'not_contains':return !String(v).toLowerCase().includes(String(t).toLowerCase());
  case 'starts_with':return t!==''&&String(v).toLowerCase().startsWith(String(t).toLowerCase());
  case 'ends_with':return t!==''&&String(v).toLowerCase().endsWith(String(t).toLowerCase());
  case 'in_list':return String(t).split(LIST_SEPARATOR).some(o=>o===String(v));
  case 'not_in_list':return !String(t).split(LIST_SEPARATOR).some(o=>o===String(v));
  case 'is_empty':return String(v).trim()==='';
  case 'is_not_empty':return String(v).trim()!=='';
  case 'greater_than':return Number(v)>Number(t);
  case 'less_than':return Number(v)<Number(t);
  default:return String(v)===String(t);
 }
}
// A field's value input, matching its type - a <select> for select fields,
// a typed <input> otherwise. Used for both a rule/trigger's comparison
// value and (via conditionFieldsFor) a workflow's "reaches" value, so every
// condition compares against a widget that matches the field, not a plain
// text box for everything.
function fieldValueHtml(name,field,value=''){
 if(!field)return `<input name="${name}" value="${value}">`;
 const [,,type,opts]=field;
 if(type==='select')return `<select name="${name}">${(opts||'').split('|').map(o=>`<option value="${o}" ${value===o?'selected':''}>${o}</option>`).join('')}</select>`;
 if(type==='number')return `<input name="${name}" type="number" step="any" value="${value}">`;
 if(type==='date')return `<input name="${name}" type="date" value="${value}">`;
 return `<input name="${name}" type="text" value="${value}">`;
}
// ---- Condition groups (Admin Automation & Customization, second addendum
// round) - shared by business rules and workflow automation, mirroring
// desktop's core::domain::conditions::conditions_match exactly. A
// condition with no groupId participates directly in the rule's top-level
// matchType; conditions sharing a groupId are OR'd together into one
// sub-unit first, and that sub-unit's result then participates in the
// top-level matchType alongside the ungrouped conditions - one level of
// nested OR-grouping ("+ Add condition" vs "+ OR group" in the builders).
function conditionsMatch(matchType,conditions,ctx){
 if(!conditions||!conditions.length)return false;
 const units=[]; const groups=new Map(); const groupOrder=[];
 conditions.forEach(c=>{
  const compareValue=c.compareField?(ctx[c.compareField]??''):(c.value??'');
  const matched=operatorMatch(c.operator,ctx[c.fieldKey],compareValue);
  if(c.groupId){
   if(!groups.has(c.groupId))groupOrder.push(c.groupId);
   groups.set(c.groupId,(groups.get(c.groupId)||false)||matched);
  }else units.push(matched);
 });
 groupOrder.forEach(g=>units.push(groups.get(g)));
 return matchType==='any'?units.some(Boolean):units.every(Boolean);
}
// Groups a flat conditions array into top-level units, in first-occurrence
// order - used to render an OR-group as one bordered box in the builders
// and to parenthesize a read-only summary correctly.
function groupConditionUnits(conditions){
 const units=[]; const byGroup=new Map();
 conditions.forEach((c,i)=>{
  if(c.groupId){
   let g=byGroup.get(c.groupId);
   if(!g){g={kind:'group',groupId:c.groupId,indices:[]};byGroup.set(c.groupId,g);units.push(g)}
   g.indices.push(i);
  }else units.push({kind:'single',index:i});
 });
 return units;
}
function describeCondition(entityKey,c){
 return `${fieldLabelFor(entityKey,c.fieldKey)} ${OPERATOR_LABELS[c.operator]||'is'}${operatorNeedsValue(c.operator)?' '+describeComparand(entityKey,c.compareField,c.value):''}`;
}
// Plain-language description of a whole conditions list, honoring
// OR-groups - "(a OR b) AND c" rather than the flat "a OR b AND c" a naive
// join would produce.
function describeConditions(entityKey,conditions,matchType){
 if(!conditions||!conditions.length)return 'no conditions';
 const units=groupConditionUnits(conditions);
 const parts=units.map(u=>u.kind==='single'?describeCondition(entityKey,conditions[u.index]):`(${u.indices.map(i=>describeCondition(entityKey,conditions[i])).join(' OR ')})`);
 return parts.join(matchType==='any'?' OR ':' AND ');
}
function newGroupId(){return 'g'+Math.random().toString(36).slice(2,9)}

// ---- Business rules (fieldRules) - condition/action evaluation ------------
// Field-behavior action types (continuous, live-evaluated against the open
// form) vs. save-time-only actions (set_default/set_value/clear_value/
// restrict_choices' value/block_save/show_error/show_warning) - mirrors
// desktop's business_rule_service::evaluate split exactly.
const FIELD_EFFECT_ACTIONS=['require','hide','show','lock','editable'];
// Client-side, cosmetic-only mirror of the field-effect actions - re-run on
// every change to any condition/compare field, exactly like desktop's
// lib/businessRules.ts. Later (later in the array) matching rule's action
// wins on a shared target field - "last matching rule wins" map-insert-
// overwrite semantics, same as the original single-effect version.
function fieldEffectsFor(rules,ctx){
 const effects={};
 rules.forEach(r=>{
  if(!conditionsMatch(r.matchType||'all',r.conditions,ctx))return;
  (r.actions||[]).forEach(a=>{if(FIELD_EFFECT_ACTIONS.includes(a.type)&&a.targetField)effects[a.targetField]=a.type});
 });
 return effects;
}
function restrictedChoicesFor(rules,ctx){
 const choices={};
 rules.forEach(r=>{
  if(!conditionsMatch(r.matchType||'all',r.conditions,ctx))return;
  (r.actions||[]).forEach(a=>{if(a.type==='restrict_choices'&&a.targetField&&a.value)choices[a.targetField]=a.value});
 });
 return choices;
}
// Live form wiring: require/hide/show/lock/editable + restrict_choices +
// a custom field's own is_hidden_by_default flag, re-evaluated on every
// change to a watched condition/compare field. set_default/set_value/
// clear_value/block_save/show_error/show_warning are save-time-only (see
// evaluateFieldRulesForSave, called from the record save handler) since
// they mutate or reject the save itself rather than just how the form
// looks - same split the desktop edition's businessRules.ts documents.
function applyFieldRules(entityKey,form){
 const rules=(data.fieldRules||[]).filter(r=>r.entity===entityKey&&r.active);
 const allFields=actionableFieldsFor(entityKey);
 function ctxFromForm(){
  const ctx={};
  conditionFieldsFor(entityKey).forEach(f=>{const el=form.elements[f[0]];if(el)ctx[f[0]]=el.value});
  return ctx;
 }
 function apply(){
  const ctx=ctxFromForm();
  const effects=fieldEffectsFor(rules,ctx);
  const choices=restrictedChoicesFor(rules,ctx);
  allFields.forEach(f=>{
   const [key,,,,extra]=f;
   const input=form.elements[key]; if(!input)return;
   const wrap=input.closest('.field'); const label=wrap?.querySelector('label');
   const effect=effects[key];
   const baseRequired=!!extra?.required||['name','title','number'].includes(key);
   const hiddenByDefault=!!extra?.hiddenByDefault;
   const hidden=effect==='hide'||(effect!=='show'&&hiddenByDefault);
   if(wrap)wrap.style.display=hidden?'none':'';
   input.disabled=effect==='lock';
   const required=!hidden&&(baseRequired||effect==='require');
   input.required=required;
   if(label){const base=label.textContent.replace(/\s*\*$/,'');label.textContent=required?base+' *':base}
   if(input.tagName==='SELECT'){
    const allowed=choices[key]?choices[key].split(LIST_SEPARATOR):null;
    [...input.options].forEach(o=>{if(o.value)o.hidden=allowed?!allowed.includes(o.value):false});
   }
  });
 }
 const watchFields=[...new Set(rules.flatMap(r=>(r.conditions||[]).flatMap(c=>[c.fieldKey,c.compareField].filter(Boolean))))];
 watchFields.forEach(fk=>{const el=form.elements[fk]; if(el){el.addEventListener('change',apply);el.addEventListener('input',apply)}});
 apply();
}
// Save-time-only business rule actions - mirrors
// custom_field_service::set_entity_values / business_rule_service::evaluate.
// Applied once, right before a record save's own default-value/uniqueness
// pass, on the same `obj` about to be written.
function evaluateFieldRulesForSave(entityKey,obj){
 const rules=(data.fieldRules||[]).filter(r=>r.entity===entityKey&&r.active);
 const result={blocked:null,setValues:{},errors:[],warnings:[]};
 rules.forEach(r=>{
  if(!conditionsMatch(r.matchType||'all',r.conditions,obj))return;
  (r.actions||[]).forEach(a=>{
   if(a.type==='set_default'){if(a.targetField&&!obj[a.targetField])result.setValues[a.targetField]=a.value??''}
   else if(a.type==='set_value'){if(a.targetField)result.setValues[a.targetField]=a.value??''}
   else if(a.type==='clear_value'){if(a.targetField)result.setValues[a.targetField]=''}
   else if(a.type==='block_save'){result.blocked=a.message||'This save is blocked by a business rule.'}
   else if(a.type==='show_error'){if(a.message)result.errors.push(a.message)}
   else if(a.type==='show_warning'){if(a.message)result.warnings.push(a.message)}
  });
 });
 return result;
}
const KPI_DEFS=[
 {key:'openPipeline',label:'Open pipeline',nav:'pipeline',filter:'open',value:()=>money(data.opportunities.filter(o=>!['Won','Lost'].includes(o.stage)).reduce((s,o)=>s+Number(o.value||0),0))},
 {key:'wonRevenue',label:'Won revenue',nav:'pipeline',filter:'won',value:()=>money(data.opportunities.filter(o=>o.stage==='Won').reduce((s,o)=>s+Number(o.value||0),0))},
 {key:'outstandingInvoices',label:'Outstanding invoices',nav:'invoices',filter:'outstanding',value:()=>money(data.invoices.filter(i=>!['Paid','Cancelled'].includes(i.status)).reduce((s,i)=>s+docBalance(i),0))},
 {key:'openTasks',label:'Open tasks',nav:'tasks',filter:'open',value:()=>String(data.tasks.filter(t=>!['Completed','Cancelled'].includes(t.status)).length)}
];
function visibleKpis(){const prefs=data.kpiPrefs||[]; return prefs.length?KPI_DEFS.filter(k=>prefs.includes(k.key)):KPI_DEFS}

function landing(){
 document.title='Lanesra OS — Open-source sales management you can shape yourself';
 $('#app').innerHTML=`
 ${publicNav()}
 <main>
 <section class="hero"><div class="container hero-grid"><div><div class="eyebrow">Open-source business software</div><h1>Run your business without complicated software — then shape it into exactly what your business needs.</h1><p>Lanesra OS gives small businesses one modern workspace for customers, opportunities, products, quotes, orders, invoices, contracts and daily follow-ups — plus an admin panel that lets you define your own record types, relationships, business rules and automations, no code required.</p><div class="hero-actions"><a class="btn btn-primary" href="/demo">Try the live demo →</a><a class="btn btn-secondary" href="/download">Desktop edition — Windows installer available</a></div><div class="trust-row"><span>✓ Free to use</span><span>✓ No licence key</span><span>✓ No-code customization</span><span>✓ Own your data</span></div></div><div class="mock"><div class="mock-top"><span class="dot"></span><span class="dot"></span><span class="dot"></span></div><div class="mock-body"><div class="mock-grid"><div class="mock-card"><small class="muted">Pipeline</small><br><strong>$192K</strong></div><div class="mock-card"><small class="muted">Revenue</small><br><strong>$84K</strong></div><div class="mock-card mock-chart">${[40,70,52,88,62,100].map(h=>`<div class="bar" style="height:${h}%"></div>`).join('')}</div></div></div></div></div></section>
 <section id="features" class="section"><div class="container"><div class="section-head"><div class="eyebrow">Complete sales journey</div><h2>Everything connected from first conversation to invoice.</h2><p class="muted">No maze of modules. No enterprise setup project. Just the essentials your team uses every day.</p></div><div class="feature-grid">${[
 ['◎','Companies & Contacts','Keep customer profiles, people, notes and activities together.'],['⌁','Sales Pipeline','Move opportunities visually from lead to won.'],['◇','Products & Services','Maintain reusable pricing, categories and tax settings.'],['▤','Quotes','Create professional commercial proposals and track acceptance.'],['▣','Orders','Convert approved quotes into trackable sales orders.'],['$','Invoices','Issue invoices and monitor paid, open and overdue balances.'],['▧','Contracts','Track agreement values, dates, files and renewals.'],['✓','Tasks & Activities','Manage calls, meetings, follow-ups and priorities.'],['▦','Sales Dashboard','See pipeline, revenue, customers and next actions instantly.']].map(x=>`<article class="feature-card"><div class="feature-icon">${x[0]}</div><h3>${x[1]}</h3><p class="muted">${x[2]}</p></article>`).join('')}</div></div></section>
 <section id="extensibility" class="section" style="background:var(--surface-alt,#f7f8fc)"><div class="container"><div class="section-head"><div class="eyebrow">Make it yours — no code required</div><h2>Every business outgrows a fixed data model. Lanesra doesn't have one.</h2><p class="muted">An Administrator can reshape the workspace itself from a settings screen — not a developer, not a support ticket.</p></div><div class="feature-grid">${[
 ['⬡','Custom Objects','Define an entirely new record type — Vendors, Assets, Projects, anything — with its own fields, ID format and navigation section.'],['⇄','Custom Relationships','Connect any two record types with one-to-one, many-to-one or many-to-many links, and a related-records list that appears automatically.'],['◈','Business Rules','Require, show, hide, lock, unlock, set or clear a field\'s value with multi-condition AND/OR logic (plus nested OR-groups) and 10 comparison operators — restrict a select field\'s choices, or block a save with an error or warning message.'],['⚙','Workflow Automation','Trigger on a status/field change, a due date, or a schedule; create a task, assign an owner, create a related record, or post a notification.'],['▥','Custom Reports & Fields','Add validated custom fields to any object, then build reports that group and sum on them — no separate reporting tool.'],['🔔','Notifications & Admin Panel','An in-app notification center, user roles, branding, numbering formats and dashboard KPIs — one place to configure the whole workspace.']].map(x=>`<article class="feature-card"><div class="feature-icon">${x[0]}</div><h3>${x[1]}</h3><p class="muted">${x[2]}</p></article>`).join('')}</div></div></section>
 <section id="desktop" class="section"><div class="container split"><div class="choice-card"><div class="eyebrow">Try online</div><h2>Explore a working business</h2><p class="muted">Open the live demo with realistic sample customers, opportunities, quotes, invoices and contracts. No registration required.</p><ul><li>Sample company included</li><li>Create and edit records</li><li>Reset demo anytime</li></ul><a class="btn btn-primary" href="/demo">Open live demo</a></div><div class="choice-card dark"><div class="eyebrow" style="color:#a5b4fc">Desktop edition</div><h2>Your software. Your computer. Your data.</h2><p style="color:#cbd5e1">A private desktop edition is available now for Windows (Early Access, unsigned installer), with macOS and Linux to follow. The source is public on GitHub today.</p><ul><li>No cloud account required</li><li>Works without internet</li><li>No activation or subscription</li></ul><a class="btn btn-secondary" href="/download">Desktop status — Windows installer available</a></div></div></section>
 <section id="open-source" class="section"><div class="container cta"><div class="eyebrow" style="color:#a5b4fc">Open source by design</div><h2>Inspect it. Run it. Improve it.</h2><p style="color:#cbd5e1;max-width:700px;margin:0 auto 24px">Lanesra OS is designed to be transparent, community-driven and free from licence keys or mandatory telemetry.</p><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank" rel="noopener">View GitHub repository</a></div></section>
 </main>${publicFooter()}`;
 bindPublicNav();
}


function appShell(){
 document.title='Lanesra OS Demo';
 $('#app').innerHTML=`<div class="demo-banner">You are exploring the sample workspace. Changes stay in this browser. <button class="link-btn" id="resetDemo">Reset demo</button><a class="link-btn" href="/">Product website</a></div><div class="app-shell"><aside class="sidebar"><div class="side-brand"><span class="brand-mark">L</span><span>Lanesra OS</span><span class="demo-pill">DEMO</span></div><nav class="side-nav" id="sideNav">${Object.keys(labels).map(k=>`<button data-nav="${k}"><b>${icons[k]}</b><span>${labels[k]}</span></button>`).join('')}<button data-nav="admin" class="admin-nav-btn"><b>⚙</b><span>Admin</span></button></nav><div class="side-bottom"><div class="side-meta"><strong>Early Access v0.24.1</strong><div class="side-product-links"><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/roadmap">Roadmap</a><a href="/releases">Releases</a></div><span>Created by <a href="https://vikramgrover.com">Vikram Grover</a></span></div><button class="btn btn-secondary" style="width:100%" onclick="location.href='/'">← Website</button></div></aside><main class="app-main"><header class="topbar"><div class="search"><input id="globalSearch" autocomplete="off" placeholder="Search companies, contacts, deals…  ⌘K"><div id="searchResults" class="search-results" hidden></div></div><div class="top-actions"><div class="notif-wrap"><button class="icon-btn" id="notifButton" aria-label="Notifications">🔔<span id="notifBadge" class="notif-badge" hidden></span></button><div id="notifPanel" class="notif-panel" hidden></div></div><button class="icon-btn" id="helpButton" aria-label="Help">?</button><div class="avatar">MC</div></div></header><div class="content" id="view"></div></main></div>`;
 document.querySelectorAll('[data-nav]').forEach(b=>b.onclick=()=>{current=b.dataset.nav;viewFilter=null;detailRecord=null;renderView()});
 $('#resetDemo').onclick=()=>{data=structuredClone(seed);ensureAdminData();syncCustomObjectRegistry();renderSidebarNav();current='dashboard';detailRecord=null;save();toast('Demo data restored');refreshNotifBadge();renderView()};
 const searchInput=$('#globalSearch'), searchBox=$('#searchResults');
 const searchable=[['companies','Company'],['contacts','Contact'],['opportunities','Opportunity'],['products','Product / Service'],['quotes','Quote'],['orders','Order'],['invoices','Invoice'],['contracts','Contract'],['tasks','Task']];
 function closeSearch(){searchBox.hidden=true;searchBox.innerHTML=''}
 function runSearch(){
  const q=searchInput.value.trim().toLowerCase();
  if(q.length<2){closeSearch();return}
  const matches=[];
  searchable.forEach(([key,type])=>(data[key]||[]).forEach(x=>{if(JSON.stringify(x).toLowerCase().includes(q))matches.push({key,type,record:x})}));
  const shown=matches.slice(0,12);
  searchBox.innerHTML=shown.length?shown.map((m,i)=>`<button type="button" class="search-result" data-result="${i}"><span><strong>${m.record.name||m.record.title||m.record.number||m.record.customerNumber||m.record.contactNumber}</strong><small>${m.type}${m.record.companyId?' · '+companyName(m.record.companyId):''}</small></span><span class="search-arrow">→</span></button>`).join(''):'<div class="search-empty">No matching records</div>';
  searchBox.hidden=false;
  searchBox.querySelectorAll('[data-result]').forEach(b=>b.onclick=()=>{const m=shown[Number(b.dataset.result)]; closeSearch(); searchInput.value='';
   if(m.key==='companies')return openCompanyDetail(m.record.id);
   if(m.key==='contacts')return openContactDetail(m.record.id);
   current=m.key==='opportunities'?'pipeline':m.key; detailRecord=null; renderView()});
 }
 searchInput.addEventListener('input',runSearch);
 searchInput.addEventListener('keydown',e=>{if(e.key==='Escape'){closeSearch();searchInput.blur()}});
 document.addEventListener('click',e=>{if(!e.target.closest('.search'))closeSearch()});
 document.addEventListener('keydown',e=>{if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='k'){e.preventDefault();searchInput.focus();runSearch()}});
 $('#helpButton').onclick=()=>modal('Help & product links',`<div class="help-list"><a href="/principles">Product principles</a><a href="/compare">Compare Lanesra</a><a href="/roadmap">Roadmap & backlog</a><a href="/releases">Releases</a><a href="/">Product website</a><button class="btn btn-secondary" onclick="document.getElementById('modal').remove()">Close</button></div>`);
 const notifBtn=$('#notifButton'), notifPanel=$('#notifPanel');
 notifBtn.onclick=e=>{e.stopPropagation();notifPanel.hidden=!notifPanel.hidden;if(!notifPanel.hidden)renderNotifPanel()};
 document.addEventListener('click',e=>{if(!e.target.closest('.notif-wrap'))notifPanel.hidden=true});
 refreshNotifBadge();
 renderView();
}
function refreshNotifBadge(){
 const badge=$('#notifBadge'); if(!badge)return;
 const unread=(data.notifications||[]).filter(n=>!n.read).length;
 badge.hidden=unread===0; badge.textContent=String(unread);
}
function renderNotifPanel(){
 const panel=$('#notifPanel'); if(!panel)return;
 const items=(data.notifications||[]).slice(0,20);
 panel.innerHTML=`<div class="notif-head"><strong>Notifications</strong><button class="link-btn" id="notifMarkAll">Mark all read</button></div><div class="notif-list">${items.length?items.map(n=>`<div class="notif-item ${n.read?'':'unread'}" data-notif="${n.id}"><span>${n.message}</span><small class="muted">${new Date(n.createdAt).toLocaleString()}</small></div>`).join(''):'<div class="empty">No notifications yet — they appear here when a workflow rule with "notify admins" fires.</div>'}</div>`;
 $('#notifMarkAll').onclick=()=>{(data.notifications||[]).forEach(n=>n.read=true);save();refreshNotifBadge();renderNotifPanel()};
 panel.querySelectorAll('[data-notif]').forEach(el=>el.onclick=()=>{const n=data.notifications.find(x=>x.id===el.dataset.notif);if(n){n.read=true;save();refreshNotifBadge();renderNotifPanel()}});
}
const byId=(key,id)=>data[key]?.find(x=>x.id===id);
const companyName=id=>byId('companies',id)?.name||'—';
const contactName=id=>byId('contacts',id)?.name||'—';
const opportunityName=id=>byId('opportunities',id)?.title||'—';
const productName=id=>byId('products',id)?.name||'—';
const quoteName=id=>byId('quotes',id)?.number||'—';
const orderName=id=>byId('orders',id)?.number||'—';
const lineTotal=i=>Number(i.quantity||0)*Number(i.unitPrice||0);
// Discount % (new quote/order/invoice field) now actually reduces the
// document total, matching the desktop edition's discount_cents feeding
// into total_cents - previously a document's total was line items only.
const docTotal=r=>{const raw=(r.items||[]).reduce((s,i)=>s+lineTotal(i),0);return raw-raw*(Number(r.discount||0)/100)};
// Outstanding balance on an invoice - total minus whatever's been recorded
// against the new "Amount paid" field, floored at zero. Closes the demo's
// long-standing "no balance_cents field... stand in with the full total"
// gap (see the AR aging report) now that partial payment is trackable.
const docBalance=r=>Math.max(0,docTotal(r)-Number(r.amountPaid||0));
const relatedLabel=t=>({Company:companyName,Contact:contactName,Opportunity:opportunityName,Quote:quoteName,Order:orderName,Invoice:id=>byId('invoices',id)?.number||'—',Contract:id=>byId('contracts',id)?.number||'—',General:()=> 'General'}[t.relatedType]?.(t.relatedId)||'General');
function options(list,value,labelFn=x=>x.name){return `<option value="">Select…</option>`+list.map(x=>`<option value="${x.id}" ${x.id===value?'selected':''}>${labelFn(x)}</option>`).join('')}
function optionalOptions(list,value,emptyLabel='None',labelFn=x=>x.name){return `<option value="">${emptyLabel}</option>`+list.map(x=>`<option value="${x.id}" ${x.id===value?'selected':''}>${labelFn(x)}</option>`).join('')}
function selectHtml(name,label,items,value,required=true){return `<div class="field"><label>${label}</label><select name="${name}" ${required?'required':''}>${options(items,value)}</select></div>`}
// The sidebar's <nav> is only built once, inside appShell()'s initial
// innerHTML - unlike #view/#adminBody it's never re-rendered by
// renderView()/renderAdminTab(), so a Custom Objects create/edit/delete
// (or Reset demo) needs to explicitly rebuild it or its entry would never
// appear/disappear from navigation.
function renderSidebarNav(){
 const nav=$('#sideNav'); if(!nav)return;
 nav.innerHTML=`${Object.keys(labels).map(k=>`<button data-nav="${k}"><b>${icons[k]}</b><span>${labels[k]}</span></button>`).join('')}<button data-nav="admin" class="admin-nav-btn"><b>⚙</b><span>Admin</span></button>`;
 document.querySelectorAll('[data-nav]').forEach(b=>{b.onclick=()=>{current=b.dataset.nav;viewFilter=null;detailRecord=null;renderView()};b.classList.toggle('active',b.dataset.nav===current)});
}
function renderView(){
 document.querySelectorAll('[data-nav]').forEach(b=>b.classList.toggle('active',b.dataset.nav===current));
 if(current==='dashboard') return dashboard();
 if(current==='pipeline') return pipeline();
 if(current==='reports') return reportsPage();
 if(current==='admin') return adminPage();
 if(detailRecord&&detailRecord.type===current)return detailRecord.type==='companies'?companyDetail(detailRecord.id):detailRecord.type==='contacts'?contactDetail(detailRecord.id):genericRecordDetail(detailRecord.type,detailRecord.id);
 // Admin-defined Custom Objects reuse the exact same generic tablePage +
 // recordModal flow every built-in entity uses - only the columns/fields
 // are built from the object's definition instead of being hardcoded.
 const co=customObjectByKey(current);
 if(co)return tablePage(current,{cols:[['number','ID'],['name','Name'],['status','Status'],['owner','Owner']],fields:()=>fieldsFor(current,customObjectFields)});
 // Every ID column below is 'idLink' - v0.25 round, so every list can
 // click straight into that record's own detail page, not just
 // Companies/Contacts' Name column as before (see cellValue/openRecordDetail).
 const configs={
 companies:{cols:[['customerNumber','Customer ID','idLink'],['name','Company','companyLink'],['industry','Industry'],['city','City'],['owner','Owner'],['status','Status']],fields:()=>fieldsFor('companies',companyFields)},
 contacts:{cols:[['contactNumber','Contact ID','idLink'],['name','Contact','contactLink'],['companyId','Company','company'],['role','Role'],['email','Email'],['status','Status']],fields:()=>fieldsFor('contacts',contactFields)},
 products:{cols:[['productNumber','Product ID','idLink'],['name','Product / Service'],['type','Type'],['sku','SKU'],['price','Price','money'],['status','Status']],fields:()=>fieldsFor('products',productFields)},
 quotes:{cols:[['number','Quote','idLink'],['companyId','Customer','company'],['opportunityId','Opportunity','opportunity'],['amount','Amount','docmoney'],['status','Status']],fields:()=>fieldsFor('quotes',quoteFields),document:true},
 orders:{cols:[['number','Order','idLink'],['companyId','Customer','company'],['quoteId','Quote','quote'],['amount','Amount','docmoney'],['status','Status']],fields:()=>fieldsFor('orders',orderFields),document:true},
 invoices:{cols:[['number','Invoice','idLink'],['companyId','Customer','company'],['orderId','Order','order'],['amount','Amount','docmoney'],['status','Status']],fields:()=>fieldsFor('invoices',invoiceFields),document:true},
 contracts:{cols:[['number','Contract','idLink'],['companyId','Customer','company'],['title','Title'],['value','Value','money'],['status','Status'],['end','End date']],fields:()=>fieldsFor('contracts',contractFields)},
 tasks:{cols:[['taskNumber','Task ID','idLink'],['title','Task'],['relatedId','Related to','related'],['owner','Owner'],['due','Due'],['priority','Priority'],['status','Status']],fields:()=>fieldsFor('tasks',taskFields)}
 };
 tablePage(current,configs[current]);
}
function dashboard(){
 $('#view').innerHTML=`<div class="page-head"><div><div class="eyebrow">${data.workspace.name}</div><h1>Good afternoon, Maya</h1><p class="muted">Here is what needs your attention today.</p></div><div class="quick-create"><button class="btn btn-primary" id="quickNew">+ New</button><div class="quick-menu" id="quickMenu" hidden>${[['companies','Company'],['contacts','Contact'],['opportunities','Opportunity'],['quotes','Quote'],['orders','Order'],['invoices','Invoice'],['contracts','Contract'],['tasks','Task']].map(x=>`<button data-create="${x[0]}">${x[1]}</button>`).join('')}</div></div></div><div class="kpi-grid">${visibleKpis().map(k=>`<button class="kpi kpi-link" data-kpi-nav="${k.nav}" data-kpi-filter="${k.filter}"><div class="kpi-label">${k.label}</div><div class="kpi-value">${k.value()}</div><span>View ${k.label.toLowerCase()} →</span></button>`).join('')}</div><div class="grid-2"><section class="panel"><div class="panel-head"><h3>Pipeline snapshot</h3><button class="link-btn" data-nav2="pipeline" data-filter2="open">Open pipeline</button></div>${data.opportunities.filter(o=>!['Won','Lost'].includes(o.stage)).slice(0,5).map(o=>`<div class="deal"><div style="display:flex;justify-content:space-between"><strong>${o.title}</strong><strong>${money(o.value)}</strong></div><small class="muted">${companyName(o.companyId)} · ${o.stage}</small></div>`).join('')}</section><section class="panel"><div class="panel-head"><h3>Tasks requiring attention</h3><button class="link-btn" data-nav2="tasks" data-filter2="open">View tasks</button></div>${data.tasks.filter(t=>!['Completed','Cancelled'].includes(t.status)).map(t=>`<div class="deal"><strong>${t.title}</strong><small class="muted">${relatedLabel(t)} · ${t.due}</small></div>`).join('')}</section></div>`;
 document.querySelectorAll('[data-kpi-nav]').forEach(b=>b.onclick=()=>{current=b.dataset.kpiNav;viewFilter=b.dataset.kpiFilter;detailRecord=null;renderView()});
 document.querySelectorAll('[data-nav2]').forEach(b=>b.onclick=()=>{current=b.dataset.nav2;viewFilter=b.dataset.filter2||null;detailRecord=null;renderView()});
 const quick=$('#quickNew'),menu=$('#quickMenu'); quick.onclick=e=>{e.stopPropagation();menu.hidden=!menu.hidden};
 menu.querySelectorAll('[data-create]').forEach(b=>b.onclick=()=>{const k=b.dataset.create;menu.hidden=true;if(k==='opportunities')recordModal('opportunities',fieldsFor('opportunities',opportunityFields));else{const fn={companies:companyFields,contacts:contactFields,quotes:quoteFields,orders:orderFields,invoices:invoiceFields,contracts:contractFields,tasks:taskFields}[k];recordModal(k,fieldsFor(k,fn))}});
 document.addEventListener('click',()=>{if(menu)menu.hidden=true},{once:true});
}
// v0.25 record-detail-page round: a handful of built-in fields added to
// every core entity, matching what the desktop edition's Rust models
// already carry (job_title/mobile/next_step/description/discount_cents/
// terms/payment_terms/owner_user_id/type/renewal_date/... - see
// core/src/models/*.rs) so the demo's field set catches up to desktop's
// instead of drifting further apart. Optional and additive - every new
// field is undefined on existing seed/localStorage records, which
// fieldHtml already renders as blank, so nothing migrates.
function companyFields(){return [['customerNumber','Customer ID','auto'],['name','Company name'],['industry','Industry'],['city','City'],['owner','Owner'],['status','Status','select','Lead|Prospect|Customer|Inactive'],['phone','Phone'],['email','Email'],['website','Website'],['annualRevenue','Annual revenue','number'],['employeeCount','Employees','number'],['preferredContactMethod','Preferred contact method','select','Email|Phone|Text']]}
function contactFields(){return [['contactNumber','Contact ID','auto'],['name','Full name'],['companyId','Company','relation','companies'],['role','Role'],['email','Email'],['phone','Phone'],['status','Status','select','Active|Inactive'],['mobile','Mobile'],['department','Department'],['preferredContactMethod','Preferred contact method','select','Email|Phone|Text'],['linkedin','LinkedIn (optional)']]}
function opportunityFields(){return [['opportunityNumber','Opportunity ID','auto'],['title','Opportunity title'],['companyId','Customer','relation','companies'],['contactId','Primary contact (optional)','filteredContact'],['value','Value','number'],['stage','Stage','select','Lead|Qualified|Discovery|Proposal|Negotiation|Won|Lost'],['probability','Probability %','number'],['close','Expected close','date'],['owner','Owner'],['status','Status','select','Open|On Hold|Won|Lost'],['lostReason','Lost reason (optional)'],['nextStep','Next step (optional)']]}
function productFields(){return [['productNumber','Product ID','auto'],['name','Name'],['sku','SKU'],['type','Type','select','Product|Service'],['category','Category'],['price','Unit price','number'],['tax','Tax %','number'],['status','Status','select','Active|Inactive'],['description','Description (optional)']]}
function quoteFields(){return [['number','Quote number','auto'],['companyId','Customer','relation','companies'],['contactId','Contact (optional)','filteredContact'],['opportunityId','Opportunity (optional)','filteredOpportunity'],['status','Status','select','Draft|Sent|Accepted|Rejected|Expired'],['date','Quote date','date'],['valid','Valid until','date'],['discount','Discount %','number'],['terms','Terms (optional)']]}
function orderFields(){return [['number','Order number','auto'],['companyId','Customer','relation','companies'],['contactId','Contact (optional)','filteredContact'],['quoteId','Source quote (optional)','filteredQuote'],['status','Status','select','Draft|Confirmed|In Progress|Completed|Cancelled'],['date','Order date','date'],['discount','Discount %','number']]}
function invoiceFields(){return [['number','Invoice number','auto'],['companyId','Customer','relation','companies'],['orderId','Source order (optional)','filteredOrder'],['status','Status','select','Draft|Sent|Partially Paid|Paid|Overdue|Cancelled'],['due','Due date','date'],['discount','Discount %','number'],['paymentTerms','Payment terms (optional)'],['amountPaid','Amount paid','number']]}
function contractFields(){return [['number','Contract number','auto'],['companyId','Customer','relation','companies'],['contactId','Contact (optional)','filteredContact'],['title','Title'],['value','Value','number'],['status','Status','select','Draft|Active|Renewal Due|Expired|Terminated'],['start','Start','date'],['end','End','date'],['owner','Owner'],['type','Contract type','select','Service Agreement|Support|License|NDA|Other'],['renewalDate','Renewal date (optional)','date'],['noticePeriodDays','Notice period (days)','number']]}
function taskFields(){return [['taskNumber','Task ID','auto'],['title','Task title'],['relatedType','Related record type','select','General|Company|Contact|Opportunity|Quote|Order|Invoice|Contract'],['relatedId','Related record','dynamicRelation'],['owner','Owner'],['due','Due date','date'],['priority','Priority','select','Low|Medium|High|Urgent'],['status','Status','select','Open|In Progress|Completed|Cancelled'],['description','Description (optional)']]}
function pipeline(){
 let stages=['Lead','Qualified','Discovery','Proposal','Negotiation','Won','Lost'];
 if(viewFilter==='open')stages=['Lead','Qualified','Discovery','Proposal','Negotiation'];
 if(viewFilter==='won')stages=['Won'];
 $('#view').innerHTML=`<div class="page-head"><div><h1>${viewFilter==='won'?'Won Opportunities':viewFilter==='open'?'Open Pipeline':'Sales Pipeline'}</h1><p class="muted">Opportunities are optional sales records linked to a customer and, when useful, a primary contact.</p></div><button class="btn btn-primary" id="addDeal">+ New opportunity</button></div><div class="kanban">${stages.map(s=>{const items=data.opportunities.filter(o=>o.stage===s);return `<div class="kanban-col"><div class="kanban-head"><span>${s}</span><span>${items.length}</span></div>${items.map(o=>`<article class="deal"><div class="deal-title">${o.title}</div><small class="muted">${companyName(o.companyId)}${o.contactId?' · '+contactName(o.contactId):''}</small><div class="deal-value">${money(o.value)}</div><small class="muted">${o.probability}% · ${o.close}</small><div class="actions"><button class="icon-btn" data-edit="${o.id}">Edit</button><button class="icon-btn" data-del="${o.id}">Delete</button></div></article>`).join('')}</div>`}).join('')}</div>`;
 $('#addDeal').onclick=()=>recordModal('opportunities',fieldsFor('opportunities',opportunityFields));
 document.querySelectorAll('[data-edit]').forEach(b=>b.onclick=()=>recordModal('opportunities',fieldsFor('opportunities',opportunityFields),byId('opportunities',b.dataset.edit)));
 document.querySelectorAll('[data-del]').forEach(b=>b.onclick=()=>remove('opportunities',b.dataset.del));
}
// ---- Reports (Phase 3 demo parity) -----------------------------------------
// Mirrors desktop's fixed report gallery (report_service.rs: Revenue by
// month, Win rate by owner, Lost reasons, AR aging, Sales by owner) plus its
// generic custom report builder (custom_report_service.rs: pick any object -
// built-in or custom - group by its status/stage or an active+reportable
// custom field, count or sum). Two schema gaps this demo's simpler data
// model has that desktop doesn't: invoices only carry a due date (no
// separate issue date) and don't track a running balance apart from the
// full document total. Rather than fake either, both reports below say so
// in a subtitle and use the closest real field (due date / full total).
let reportsTab='revenue';
let reportsFrom='';
let reportsTo='';
let reportsAsOf='';
let selectedCustomReportId=null;
function inRange(dateStr){if(!dateStr)return false;if(reportsFrom&&dateStr<reportsFrom)return false;if(reportsTo&&dateStr>reportsTo)return false;return true}
function reportBarHtml(value,max){const pct=max>0?Math.max(2,Math.round(value/max*100)):0;return `<div style="background:#eef2ff;border-radius:5px;width:130px;height:9px;overflow:hidden"><div style="width:${pct}%;height:100%;background:var(--brand)"></div></div>`}
function downloadCsv(filename,headers,rows){
 const esc=v=>{const s=String(v??'');return /[",\n]/.test(s)?'"'+s.replace(/"/g,'""')+'"':s};
 const csv=[headers.map(esc).join(','),...rows.map(r=>r.map(esc).join(','))].join('\n');
 const blob=new Blob([csv],{type:'text/csv'});
 const url=URL.createObjectURL(blob);
 const a=document.createElement('a');
 a.href=url;a.download=filename;document.body.appendChild(a);a.click();a.remove();
 URL.revokeObjectURL(url);
}
function reportRevenueByMonth(){
 const rows=data.invoices.filter(i=>!['Draft','Cancelled'].includes(i.status)&&inRange(i.due));
 const byMonth={};const order=[];
 rows.forEach(i=>{const m=(i.due||'').slice(0,7);if(!m)return;if(!byMonth[m]){byMonth[m]={month:m,count:0,total:0};order.push(m)}byMonth[m].count++;byMonth[m].total+=docTotal(i)});
 return order.sort().map(m=>byMonth[m]);
}
function reportWinRateByOwner(){
 const rows=data.opportunities.filter(o=>['Won','Lost'].includes(o.status)&&inRange(o.close));
 const byOwner={};const order=[];
 rows.forEach(o=>{const owner=o.owner||'Unassigned';if(!byOwner[owner]){byOwner[owner]={owner,won:0,lost:0,wonValue:0};order.push(owner)}if(o.status==='Won'){byOwner[owner].won++;byOwner[owner].wonValue+=Number(o.value||0)}else byOwner[owner].lost++});
 return order.map(o=>byOwner[o]).sort((a,b)=>b.wonValue-a.wonValue);
}
function reportLostReasons(){
 const rows=data.opportunities.filter(o=>o.status==='Lost'&&inRange(o.close));
 const byReason={};const order=[];
 rows.forEach(o=>{const reason=(o.lostReason||'').trim()||'No reason given';if(!byReason[reason]){byReason[reason]={reason,count:0,value:0};order.push(reason)}byReason[reason].count++;byReason[reason].value+=Number(o.value||0)});
 return order.map(r=>byReason[r]).sort((a,b)=>b.count-a.count);
}
function reportArAging(asOf){
 const cutoff=asOf||new Date().toISOString().slice(0,10);
 // Now backed by the "Amount paid" field (new this round) via docBalance -
 // still treats Paid the same as "no balance left" (its balance is 0 or
 // close to it either way once amountPaid catches up to the total).
 const rows=data.invoices.filter(i=>!['Draft','Cancelled','Paid'].includes(i.status));
 const order=['Not yet due','1-30 days overdue','31-60 days overdue','61-90 days overdue','90+ days overdue','No due date'];
 const buckets=Object.fromEntries(order.map(b=>[b,{bucket:b,count:0,balance:0}]));
 rows.forEach(i=>{
  const bal=docBalance(i);
  let key;
  if(!i.due)key='No due date';
  else{
   const days=Math.floor((new Date(cutoff)-new Date(i.due))/86400000);
   key=days<=0?'Not yet due':days<=30?'1-30 days overdue':days<=60?'31-60 days overdue':days<=90?'61-90 days overdue':'90+ days overdue';
  }
  buckets[key].count++;buckets[key].balance+=bal;
 });
 return order.map(b=>buckets[b]).filter(b=>b.count>0);
}
function reportSalesByOwner(){
 const rows=data.invoices.filter(i=>!['Draft','Cancelled'].includes(i.status)&&inRange(i.due));
 const byOwner={};const order=[];
 rows.forEach(i=>{const c=byId('companies',i.companyId);const owner=c?.owner||'Unassigned';if(!byOwner[owner]){byOwner[owner]={owner,count:0,total:0};order.push(owner)}byOwner[owner].count++;byOwner[owner].total+=docTotal(i)});
 return order.map(o=>byOwner[o]).sort((a,b)=>b.total-a.total);
}
// Fields an admin flagged reportable, mirroring ADM-CF-05 - a field marked
// "not reportable" is off-limits as a group-by or sum target here too.
function reportableCustomFields(entityKey){return customFieldsFor(entityKey).filter(f=>f[4]&&f[4].reportable)}
function reportableNumericCustomFields(entityKey){return reportableCustomFields(entityKey).filter(f=>f[2]==='number')}
function runCustomReport(report){
 const arr=data[report.entityKey]||[];
 const groups={};const order=[];
 arr.forEach(r=>{
  const group=report.groupBySource==='builtin'
   ? (r[transitionFieldFor(report.entityKey)]||'(none)')
   : ((r[report.groupByField]===undefined||r[report.groupByField]===null||r[report.groupByField]==='')?'(none)':String(r[report.groupByField]));
  if(!(group in groups)){groups[group]=0;order.push(group)}
  groups[group]+=report.aggregate==='sum'?Number(r[report.sumFieldKey]||0):1;
 });
 return order.map(g=>({group:g,value:groups[g]}));
}
function reportsPage(){
 document.title='Reports — Lanesra OS Demo';
 if(!reportsTo)reportsTo=new Date().toISOString().slice(0,10);
 if(!reportsAsOf)reportsAsOf=new Date().toISOString().slice(0,10);
 const tabs=[['revenue','Revenue by month'],['winRate','Win rate by owner'],['lostReasons','Lost reasons'],['arAging','AR aging'],['salesByOwner','Sales by owner'],['custom','Custom reports']];
 $('#view').innerHTML=`<div class="page-head"><div><h1>Reports</h1><p class="muted">Beyond the dashboard's KPI tiles — revenue, pipeline outcomes, aging receivables, sales by owner, and admin-built custom reports.</p></div></div><div class="tabs">${tabs.map(t=>`<button class="tab ${reportsTab===t[0]?'active':''}" data-report-tab="${t[0]}">${t[1]}</button>`).join('')}</div><div id="reportsBody"></div>`;
 document.querySelectorAll('[data-report-tab]').forEach(b=>b.onclick=()=>{reportsTab=b.dataset.reportTab;renderReportsTab()});
 renderReportsTab();
}
function renderReportsTab(){
 document.querySelectorAll('[data-report-tab]').forEach(b=>b.classList.toggle('active',b.dataset.reportTab===reportsTab));
 const body=$('#reportsBody');
 ({revenue:revenueReportTab,winRate:winRateReportTab,lostReasons:lostReasonsReportTab,arAging:arAgingReportTab,salesByOwner:salesByOwnerReportTab,custom:customReportsTab}[reportsTab])(body);
}
function rangeControlsHtml(){return `<div class="form-grid" style="grid-template-columns:repeat(3,max-content);align-items:end;margin-bottom:16px"><div class="field"><label>From</label><input type="date" id="reportsFromInput" value="${reportsFrom}"></div><div class="field"><label>To</label><input type="date" id="reportsToInput" value="${reportsTo}"></div><div class="field"><button class="btn btn-secondary" type="button" id="reportsClearRange">Clear range</button></div></div>`}
function wireRangeControls(rerender){
 $('#reportsFromInput').onchange=e=>{reportsFrom=e.target.value;rerender()};
 $('#reportsToInput').onchange=e=>{reportsTo=e.target.value;rerender()};
 $('#reportsClearRange').onclick=()=>{reportsFrom='';reportsTo='';rerender()};
}
function revenueReportTab(body){
 const rows=reportRevenueByMonth();
 const max=Math.max(0,...rows.map(r=>r.total));
 body.innerHTML=`${rangeControlsHtml()}<div class="panel"><div class="panel-head"><h3>Revenue by month</h3><button class="btn btn-secondary" id="exportReport">Export CSV</button></div><p class="muted" style="margin-top:-8px;font-size:13px">Grouped by each invoice's due date — this demo doesn't track a separate issue date.</p>${rows.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Month</th><th>Invoices</th><th></th><th>Revenue</th></tr></thead><tbody>${rows.map(r=>`<tr><td>${r.month}</td><td>${r.count}</td><td>${reportBarHtml(r.total,max)}</td><td>${money(r.total)}</td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No invoices in this range.</div>'}</div>`;
 wireRangeControls(()=>revenueReportTab(body));
 $('#exportReport').onclick=()=>downloadCsv('revenue-by-month.csv',['Month','Invoices','Revenue'],rows.map(r=>[r.month,r.count,r.total.toFixed(2)]));
}
function winRateReportTab(body){
 const rows=reportWinRateByOwner();
 body.innerHTML=`${rangeControlsHtml()}<div class="panel"><div class="panel-head"><h3>Win rate by owner</h3><button class="btn btn-secondary" id="exportReport">Export CSV</button></div><p class="muted" style="margin-top:-8px;font-size:13px">Uses each opportunity's expected close date — this demo doesn't track a separate closed-date timestamp.</p>${rows.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Owner</th><th>Won</th><th>Lost</th><th>Win rate</th><th>Won value</th></tr></thead><tbody>${rows.map(r=>{const total=r.won+r.lost;const rate=total>0?Math.round(r.won/total*100)+'%':'—';return `<tr><td>${r.owner}</td><td>${r.won}</td><td>${r.lost}</td><td>${rate}</td><td>${money(r.wonValue)}</td></tr>`}).join('')}</tbody></table></div>`:'<div class="empty">No won or lost opportunities in this range.</div>'}</div>`;
 wireRangeControls(()=>winRateReportTab(body));
 $('#exportReport').onclick=()=>downloadCsv('win-rate-by-owner.csv',['Owner','Won','Lost','Win rate','Won value'],rows.map(r=>{const total=r.won+r.lost;return [r.owner,r.won,r.lost,total>0?Math.round(r.won/total*100)+'%':'—',r.wonValue.toFixed(2)]}));
}
function lostReasonsReportTab(body){
 const rows=reportLostReasons();
 const max=Math.max(0,...rows.map(r=>r.count));
 body.innerHTML=`${rangeControlsHtml()}<div class="panel"><div class="panel-head"><h3>Lost reasons</h3><button class="btn btn-secondary" id="exportReport">Export CSV</button></div><p class="muted" style="margin-top:-8px;font-size:13px">Uses each opportunity's expected close date — this demo doesn't track a separate closed-date timestamp.</p>${rows.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Reason</th><th>Count</th><th></th><th>Value</th></tr></thead><tbody>${rows.map(r=>`<tr><td>${r.reason}</td><td>${r.count}</td><td>${reportBarHtml(r.count,max)}</td><td>${money(r.value)}</td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No lost opportunities in this range.</div>'}</div>`;
 wireRangeControls(()=>lostReasonsReportTab(body));
 $('#exportReport').onclick=()=>downloadCsv('lost-reasons.csv',['Reason','Count','Value'],rows.map(r=>[r.reason,r.count,r.value.toFixed(2)]));
}
function arAgingReportTab(body){
 const rows=reportArAging(reportsAsOf);
 const max=Math.max(0,...rows.map(r=>r.balance));
 body.innerHTML=`<div class="form-grid" style="grid-template-columns:max-content;align-items:end;margin-bottom:16px"><div class="field"><label>As of</label><input type="date" id="reportsAsOfInput" value="${reportsAsOf}"></div></div><div class="panel"><div class="panel-head"><h3>AR aging</h3><button class="btn btn-secondary" id="exportReport">Export CSV</button></div><p class="muted" style="margin-top:-8px;font-size:13px">Uses each invoice's full amount as its balance — this demo doesn't track partial payments separately.</p>${rows.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Bucket</th><th>Invoices</th><th></th><th>Balance</th></tr></thead><tbody>${rows.map(r=>`<tr><td>${r.bucket}</td><td>${r.count}</td><td>${reportBarHtml(r.balance,max)}</td><td>${money(r.balance)}</td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No outstanding balances.</div>'}</div>`;
 $('#reportsAsOfInput').onchange=e=>{reportsAsOf=e.target.value;arAgingReportTab(body)};
 $('#exportReport').onclick=()=>downloadCsv('ar-aging.csv',['Bucket','Invoices','Balance'],rows.map(r=>[r.bucket,r.count,r.balance.toFixed(2)]));
}
function salesByOwnerReportTab(body){
 const rows=reportSalesByOwner();
 const max=Math.max(0,...rows.map(r=>r.total));
 body.innerHTML=`${rangeControlsHtml()}<div class="panel"><div class="panel-head"><h3>Sales by owner</h3><button class="btn btn-secondary" id="exportReport">Export CSV</button></div><p class="muted" style="margin-top:-8px;font-size:13px">Attributed via each invoice's Company owner — invoices have no owner of their own. Grouped by due date, since this demo doesn't track a separate issue date.</p>${rows.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Owner</th><th>Invoices</th><th></th><th>Revenue</th></tr></thead><tbody>${rows.map(r=>`<tr><td>${r.owner}</td><td>${r.count}</td><td>${reportBarHtml(r.total,max)}</td><td>${money(r.total)}</td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No invoices in this range.</div>'}</div>`;
 wireRangeControls(()=>salesByOwnerReportTab(body));
 $('#exportReport').onclick=()=>downloadCsv('sales-by-owner.csv',['Owner','Invoices','Revenue'],rows.map(r=>[r.owner,r.count,r.total.toFixed(2)]));
}
function customReportsTab(body){
 const list=data.customReports||[];
 body.innerHTML=`<div class="panel"><div class="panel-head"><h3>Custom reports</h3><button class="btn btn-primary" id="addCustomReport">+ New report</button></div><p class="muted" style="font-size:13px">Pick an entity, a field to group by, and an aggregate — a small alternative to the fixed reports above for questions those don't answer.</p>${list.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Entity</th><th>Group by</th><th>Aggregate</th><th>Actions</th></tr></thead><tbody>${list.map(r=>`<tr><td><a class="cell-link" data-run-report="${r.id}">${r.name}</a></td><td>${entityLabel(r.entityKey)}</td><td>${r.groupBySource==='builtin'?(transitionFieldFor(r.entityKey)==='stage'?'Stage':'Status'):r.groupByField}</td><td>${r.aggregate==='sum'?'Sum of '+r.sumFieldKey:'Count'}</td><td><div class="actions"><button class="icon-btn" data-del-report="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No custom reports yet.</div>'}</div><div id="customReportResults"></div>`;
 $('#addCustomReport').onclick=()=>customReportModal();
 body.querySelectorAll('[data-run-report]').forEach(a=>a.onclick=()=>{selectedCustomReportId=a.dataset.runReport;renderCustomReportResults()});
 body.querySelectorAll('[data-del-report]').forEach(b=>b.onclick=()=>{if(!confirm('Delete this report?'))return;data.customReports=data.customReports.filter(r=>r.id!==b.dataset.delReport);if(selectedCustomReportId===b.dataset.delReport)selectedCustomReportId=null;save();customReportsTab(body)});
 renderCustomReportResults();
}
function renderCustomReportResults(){
 const box=$('#customReportResults'); if(!box)return;
 const report=(data.customReports||[]).find(r=>r.id===selectedCustomReportId);
 if(!report){box.innerHTML='';return}
 const rows=runCustomReport(report);
 const max=Math.max(0,...rows.map(r=>r.value));
 box.innerHTML=`<div class="panel" style="margin-top:16px"><div class="panel-head"><h3>${report.name}</h3><button class="btn btn-secondary" id="exportCustomReport">Export CSV</button></div>${rows.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Group</th><th></th><th>Value</th></tr></thead><tbody>${rows.map(r=>`<tr><td>${r.group}</td><td>${reportBarHtml(r.value,max)}</td><td>${report.aggregate==='sum'?r.value.toLocaleString():r.value}</td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No data yet.</div>'}</div>`;
 $('#exportCustomReport').onclick=()=>downloadCsv(`${report.name.toLowerCase().replace(/\s+/g,'-')}.csv`,['Group','Value'],rows.map(r=>[r.group,r.value]));
}
function customReportModal(){
 const keys=allEntityTypeKeys();
 const body=`<form id="customReportForm" class="form-grid">
 <div class="field full"><label>Report name</label><input name="name" required></div>
 <div class="field"><label>Entity</label><select name="entityKey" id="crEntitySelect">${keys.map(k=>`<option value="${k}">${entityLabel(k)}</option>`).join('')}</select></div>
 <div class="field"><label>Group by</label><select name="groupBy" id="crGroupBySelect"></select></div>
 <div class="field"><label>Aggregate</label><select name="aggregate" id="crAggregateSelect"><option value="count">Count of records</option><option value="sum">Sum of a numeric field</option></select></div>
 <div class="field" id="crSumFieldWrap" hidden><label>Sum field</label><select name="sumFieldKey" id="crSumFieldSelect"></select></div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Create report</button></div>
 </form>`;
 modal('New custom report',body);
 function refreshGroupBy(){
  const entityKey=$('#crEntitySelect').value;
  const statusLabel=transitionFieldFor(entityKey)==='stage'?'Stage':'Status';
  const fields=reportableCustomFields(entityKey);
  $('#crGroupBySelect').innerHTML=`<option value="__builtin__">${statusLabel}</option>${fields.map(f=>`<option value="${f[0]}">${f[1]}</option>`).join('')}`;
  refreshSumField();
 }
 function refreshSumField(){
  const entityKey=$('#crEntitySelect').value;
  const numeric=reportableNumericCustomFields(entityKey);
  $('#crSumFieldSelect').innerHTML=numeric.length?numeric.map(f=>`<option value="${f[0]}">${f[1]}</option>`).join(''):'<option value="">— none available —</option>';
 }
 refreshGroupBy();
 $('#crEntitySelect').onchange=refreshGroupBy;
 $('#crAggregateSelect').onchange=e=>{$('#crSumFieldWrap').hidden=e.target.value!=='sum'};
 $('[data-close]').onclick=closeModal;
 $('#customReportForm').onsubmit=e=>{
  e.preventDefault();
  const fd=Object.fromEntries(new FormData(e.target).entries());
  if(fd.aggregate==='sum'&&!fd.sumFieldKey)return alert('Pick a numeric field to sum, or choose Count of records instead.');
  const report={id:uid(),name:fd.name,entityKey:fd.entityKey,groupBySource:fd.groupBy==='__builtin__'?'builtin':'custom',groupByField:fd.groupBy==='__builtin__'?transitionFieldFor(fd.entityKey):fd.groupBy,aggregate:fd.aggregate,sumFieldKey:fd.aggregate==='sum'?fd.sumFieldKey:''};
  data.customReports.push(report);
  save();closeModal();
  selectedCustomReportId=report.id;
  toast('Custom report created');
  customReportsTab($('#reportsBody'));
 };
}
// Phase 5 Customer/Contact 360, generalized in the v0.25 round: a
// company/contact reference anywhere in a list becomes a clickable link
// into its 360 page, and - new this round - every list's own ID column
// does too via the 'idLink' column type (data-open-record="entityKey:id",
// handled by openRecordDetail, the same generic router genericRecordDetail
// uses). `entityKey` is the list's own entity (tablePage's `key`), not a
// column value, since the ID column always points at its own row.
function cellValue(r,c,entityKey){const [colKey,,type]=c;if(type==='money')return money(r[colKey]);if(type==='docmoney')return money(docTotal(r));if(type==='company')return r[colKey]?`<a class="cell-link" data-open-company="${r[colKey]}">${companyName(r[colKey])}</a>`:'—';if(type==='companyLink')return `<a class="cell-link" data-open-company="${r.id}">${r[colKey]}</a>`;if(type==='contactLink')return `<a class="cell-link" data-open-contact="${r.id}">${r[colKey]}</a>`;if(type==='idLink')return `<a class="cell-link" data-open-record="${entityKey}:${r.id}">${r[colKey]}</a>`;if(type==='opportunity')return opportunityName(r[colKey]);if(type==='quote')return quoteName(r[colKey]);if(type==='order')return orderName(r[colKey]);if(type==='related')return relatedLabel(r);return badgeMaybe(r[colKey])}
function wireCellLinks(scope){
 scope.querySelectorAll('[data-open-company]').forEach(a=>a.onclick=(e)=>{e.stopPropagation();openCompanyDetail(a.dataset.openCompany)});
 scope.querySelectorAll('[data-open-contact]').forEach(a=>a.onclick=(e)=>{e.stopPropagation();openContactDetail(a.dataset.openContact)});
 scope.querySelectorAll('[data-open-record]').forEach(a=>a.onclick=(e)=>{e.stopPropagation();const [k,id]=a.dataset.openRecord.split(':');openRecordDetail(k,id)});
}
function tablePage(key,cfg){
 let arr=data[key];
 if(key==='tasks'&&viewFilter==='open')arr=arr.filter(x=>!['Completed','Cancelled'].includes(x.status));
 if(key==='invoices'&&viewFilter==='outstanding')arr=arr.filter(x=>!['Paid','Cancelled'].includes(x.status));
 $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><span>${viewFilter?viewFilter.charAt(0).toUpperCase()+viewFilter.slice(1):labels[key]}</span></div><h1>${viewFilter==='open'&&key==='tasks'?'Open Tasks':viewFilter==='outstanding'?'Outstanding Invoices':labels[key]}</h1><p class="muted">${arr.length} connected records in the sample workspace</p></div><button class="btn btn-primary" id="addRecord">+ New ${labels[key].replace(/s$/,'')}</button></div><div class="table-wrap"><table class="table"><thead><tr>${cfg.cols.map(c=>`<th>${c[1]}</th>`).join('')}<th>Actions</th></tr></thead><tbody>${arr.map(r=>`<tr>${cfg.cols.map(c=>`<td>${cellValue(r,c,key)}</td>`).join('')}<td><div class="actions"><button class="icon-btn" data-edit="${r.id}">Edit</button><button class="icon-btn" data-del="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${arr.length?'':'<div class="empty">No records yet</div>'}</div>`;
 document.querySelector('[data-clear-filter]')?.addEventListener('click',()=>{current='dashboard';viewFilter=null;detailRecord=null;renderView()});
 wireCellLinks($('#view'));
 $('#addRecord').onclick=()=>recordModal(key,cfg.fields());
 document.querySelectorAll('[data-edit]').forEach(b=>b.onclick=()=>recordModal(key,cfg.fields(),byId(key,b.dataset.edit)));
 document.querySelectorAll('[data-del]').forEach(b=>b.onclick=()=>remove(key,b.dataset.del));
}
function badgeMaybe(v){const vals=['Active','Inactive','Customer','Prospect','Lead','Sent','Accepted','Draft','Paid','Overdue','Open','Completed','High','Medium','Low','Urgent','Renewal Due','In Progress','Won','Lost','Confirmed','Cancelled'];return vals.includes(String(v))?`<span class="badge">${v}</span>`:(v??'—')}
function fieldHtml(f,record){const [name,label,type,opts]=f;const extra=f[4];const val=record[name]??(!record.id&&extra?.defaultValue?extra.defaultValue:'');const help=extra?.helpText?`<small class="field-help">${extra.helpText}</small>`:'';const req=extra?.required?'required':(['name','title','number'].includes(name)?'required':'');if(type==='auto')return `<div class="field"><label>${label}</label><input name="${name}" value="${val}" readonly placeholder="Generated automatically"><small class="field-help">Generated when the record is saved</small></div>`;if(type==='select')return `<div class="field"><label>${label}</label><select name="${name}" ${req}>${opts.split('|').map(o=>`<option value="${o}" ${val===o?'selected':''}>${o}</option>`).join('')}</select>${help}</div>`;if(type==='relation')return selectHtml(name,label,data[opts],val);if(type==='filteredContact')return `<div class="field"><label>${label}</label><select name="${name}" data-filter="contact">${optionalOptions(data.contacts.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No contact')}</select></div>`;if(type==='filteredOpportunity')return `<div class="field"><label>${label}</label><select name="${name}" data-filter="opportunity">${optionalOptions(data.opportunities.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No opportunity',x=>x.title)}</select></div>`;if(type==='filteredQuote')return `<div class="field"><label>${label}</label><select name="${name}" data-filter="quote">${optionalOptions(data.quotes.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No source quote',x=>x.number+' · '+money(docTotal(x)))}</select></div>`;if(type==='filteredOrder')return `<div class="field"><label>${label}</label><select name="${name}" data-filter="order">${optionalOptions(data.orders.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No source order',x=>x.number+' · '+money(docTotal(x)))}</select></div>`;if(type==='dynamicRelation')return `<div class="field"><label>${label}</label><select name="${name}" data-dynamic-related></select></div>`;
 // Custom fields carry their own HTML5-native validation constraints - a
 // maxlength/pattern for text, min/max for number - so the browser blocks
 // an invalid save the same way desktop's server-side validation does,
 // without a separate client-side validator to keep in sync.
 const extraAttrs=type==='number'
  ?`${extra?.minValue!==''&&extra?.minValue!==undefined?`min="${extra.minValue}"`:''} ${extra?.maxValue!==''&&extra?.maxValue!==undefined?`max="${extra.maxValue}"`:''}`
  :`${extra?.maxLength?`maxlength="${extra.maxLength}"`:''} ${extra?.pattern?`pattern="${extra.pattern}"`:''}`;
 return `<div class="field ${name==='title'?'full':''}"><label>${label}</label><input name="${name}" type="${type||'text'}" value="${val}" placeholder="${extra?.placeholder||''}" ${req} ${extraAttrs}>${help}</div>`}
function lineItemsHtml(items=[]){const rows=(items.length?items:[{productId:'',quantity:1,unitPrice:0}]).map(lineRow).join('');return `<div class="full line-items"><div class="line-head"><h3>Products & services</h3><button type="button" class="btn btn-secondary" id="addLine">+ Add line</button></div><div id="lineRows">${rows}</div><div class="line-total">Total <strong id="docTotal">${money(items.reduce((s,i)=>s+lineTotal(i),0))}</strong></div></div>`}
function lineRow(i={productId:'',quantity:1,unitPrice:0}){return `<div class="line-row"><div class="field"><label>Product / service</label><select class="line-product">${options(data.products.filter(p=>p.status==='Active'),i.productId)}</select></div><div class="field"><label>Quantity</label><input class="line-qty" type="number" min="0.01" step="0.01" value="${i.quantity??1}"></div><div class="field"><label>Unit price</label><input class="line-price" type="number" min="0" step="0.01" value="${i.unitPrice??0}"></div><div class="line-subtotal">${money(lineTotal(i))}</div><button type="button" class="icon-btn line-remove">Remove</button></div>`}
// ---- Customer 360 / Contact 360 (Phase 5), generalized in the v0.25
// round to every entity that now has a detail page. ------------------------
function openCompanyDetail(id){current='companies';detailRecord={type:'companies',id};renderView()}
function openContactDetail(id){current='contacts';detailRecord={type:'contacts',id};renderView()}
// The generic entry point every ID-column link and every related-record row
// (data-open-record / data-nav-related) goes through - opportunities have
// no detail page (kanban pipeline only), so that one still lands on its
// list instead.
const DETAIL_PAGE_ENTITIES=new Set(['companies','contacts','products','quotes','orders','invoices','contracts','tasks']);
function openRecordDetail(key,id){
 if(!DETAIL_PAGE_ENTITIES.has(key)){current=key==='opportunities'?'pipeline':key;viewFilter=null;detailRecord=null;renderView();return}
 if(key==='companies')return openCompanyDetail(id);
 if(key==='contacts')return openContactDetail(id);
 current=key;detailRecord={type:key,id};renderView();
}
// A card of related records for the 360 page's right column - each row
// navigates to that record's own list (or its own 360 page, for another
// company/contact), the same click-through pattern as a cell-link.
function relatedCardHtml(title,items,navKey,labelFn,metaFn){
 return `<div class="rule360-related-card"><h4>${title} (${items.length})</h4>${items.length?items.map(x=>`<div class="rule360-related-row"><a class="cell-link" data-nav-related="${navKey}:${x.id}">${labelFn(x)}</a><span class="muted">${metaFn?metaFn(x):''}</span></div>`).join(''):'<div class="muted" style="padding:6px 0">None yet</div>'}</div>`;
}
function wireRelatedRows(scope){
 scope.querySelectorAll('[data-nav-related]').forEach(a=>a.onclick=()=>{
  const [navKey,id]=a.dataset.navRelated.split(':');
  openRecordDetail(navKey,id);
 });
}
function detail360Header(breadcrumbLabel,title,eyebrow,metaHtml){
 return `<div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><button data-back-list>${breadcrumbLabel}</button><span>›</span><span>${title}</span></div>
 <div class="rule360-header">
  <div><div class="eyebrow">${eyebrow||''}</div><h1>${title}</h1><div class="rule360-meta">${metaHtml}</div></div>
  <button class="btn btn-secondary" id="editDetailRecord">Edit</button>
 </div>`;
}
function wireDetail360Nav(editHandler){
 document.querySelector('[data-clear-filter]')?.addEventListener('click',()=>{current='dashboard';viewFilter=null;detailRecord=null;renderView()});
 $('[data-back-list]').onclick=()=>{detailRecord=null;renderView()};
 $('#editDetailRecord').onclick=editHandler;
 wireRelatedRows($('#view'));
}
function companyDetail(id){
 const c=byId('companies',id);
 if(!c){current='companies';detailRecord=null;return renderView()}
 const overviewFields=fieldsFor('companies',companyFields).filter(f=>f[0]!=='name');
 const contacts=data.contacts.filter(x=>x.companyId===id);
 const opportunities=data.opportunities.filter(x=>x.companyId===id);
 const quotes=data.quotes.filter(x=>x.companyId===id);
 const orders=data.orders.filter(x=>x.companyId===id);
 const invoices=data.invoices.filter(x=>x.companyId===id);
 const contracts=data.contracts.filter(x=>x.companyId===id);
 const tasks=data.tasks.filter(x=>x.relatedType==='Company'&&x.relatedId===id);
 $('#view').innerHTML=`${detail360Header('Companies',c.name,c.customerNumber,`${badgeMaybe(c.status)}<span>Owner: ${c.owner||'Unassigned'}</span>`)}
 <div class="rule360-grid">
  <div><div class="panel"><h3 style="margin-top:0">Overview</h3><div class="form-grid" style="margin-top:0">${overviewFields.map(f=>`<div class="field"><label>${f[1]}</label><div>${badgeMaybe(c[f[0]])}</div></div>`).join('')}</div></div></div>
  <div>
   ${relatedCardHtml('Contacts',contacts,'contacts',x=>x.name,x=>x.role||'')}
   ${relatedCardHtml('Sales Pipeline',opportunities,'opportunities',x=>x.title,x=>money(x.value))}
   ${relatedCardHtml('Quotes',quotes,'quotes',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Orders',orders,'orders',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Invoices',invoices,'invoices',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Contracts',contracts,'contracts',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Tasks',tasks,'tasks',x=>x.title,x=>x.status)}
  </div>
 </div>`;
 wireDetail360Nav(()=>recordModal('companies',fieldsFor('companies',companyFields),c));
}
function contactDetail(id){
 const c=byId('contacts',id);
 if(!c){current='contacts';detailRecord=null;return renderView()}
 const overviewFields=fieldsFor('contacts',contactFields).filter(f=>!['auto','relation'].includes(f[2])&&f[0]!=='name');
 const opportunities=data.opportunities.filter(x=>x.contactId===id);
 const quotes=data.quotes.filter(x=>x.contactId===id);
 const orders=data.orders.filter(x=>x.contactId===id);
 const contracts=data.contracts.filter(x=>x.contactId===id);
 const tasks=data.tasks.filter(x=>x.relatedType==='Contact'&&x.relatedId===id);
 $('#view').innerHTML=`${detail360Header('Contacts',c.name,c.contactNumber,`${badgeMaybe(c.status)}<span>${c.role||'—'}</span><a class="cell-link" data-nav-related="companies:${c.companyId}">${companyName(c.companyId)}</a>`)}
 <div class="rule360-grid">
  <div><div class="panel"><h3 style="margin-top:0">Overview</h3><div class="form-grid" style="margin-top:0">${overviewFields.map(f=>`<div class="field"><label>${f[1]}</label><div>${badgeMaybe(c[f[0]])}</div></div>`).join('')}</div></div></div>
  <div>
   ${relatedCardHtml('Sales Pipeline',opportunities,'opportunities',x=>x.title,x=>money(x.value))}
   ${relatedCardHtml('Quotes',quotes,'quotes',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Orders',orders,'orders',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Contracts',contracts,'contracts',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Tasks',tasks,'tasks',x=>x.title,x=>x.status)}
  </div>
 </div>`;
 wireDetail360Nav(()=>recordModal('contacts',fieldsFor('contacts',contactFields),c));
}
// ---- Generic record detail pages for Products/Quotes/Orders/Invoices/
// Contracts/Tasks (v0.25 round) - Companies/Contacts keep their existing
// bespoke companyDetail/contactDetail above (entity-specific enough
// already); everything else shares one config-driven renderer, the same
// "config once, one generic function" pattern tablePage's `configs`
// already uses for list columns.
const DETAIL_FIELDS_FN={products:productFields,quotes:quoteFields,orders:orderFields,invoices:invoiceFields,contracts:contractFields,tasks:taskFields};
const DETAIL_BREADCRUMB={products:'Products',quotes:'Quotes',orders:'Orders',invoices:'Invoices',contracts:'Contracts',tasks:'Tasks'};
const DETAIL_TITLE_FIELD={products:'name',quotes:'number',orders:'number',invoices:'number',contracts:'title',tasks:'title'};
const DETAIL_EYEBROW=(key,r)=>key==='products'?r.productNumber:key==='tasks'?r.taskNumber:key==='contracts'?r.number:(ENTITY_SINGULAR[key]||'').replace(/^./,c=>c.toUpperCase());
// The badges/links row under the H1 - status plus whatever this record
// points at (company/contact/source document), each a live link into
// that record's own detail page via data-nav-related.
function recordEyebrowMeta(key,r){
 const relLink=(navKey,id,label)=>id?`<a class="cell-link" data-nav-related="${navKey}:${id}">${label}</a>`:'';
 switch(key){
  case 'products':return `${badgeMaybe(r.status)}<span>${r.type||''}</span>`;
  case 'quotes':return `${badgeMaybe(r.status)}${relLink('companies',r.companyId,companyName(r.companyId))}${relLink('contacts',r.contactId,contactName(r.contactId))}${relLink('opportunities',r.opportunityId,opportunityName(r.opportunityId))}`;
  case 'orders':return `${badgeMaybe(r.status)}${relLink('companies',r.companyId,companyName(r.companyId))}${relLink('contacts',r.contactId,contactName(r.contactId))}${relLink('quotes',r.quoteId,r.quoteId?`from ${quoteName(r.quoteId)}`:'')}`;
  case 'invoices':return `${badgeMaybe(r.status)}${relLink('companies',r.companyId,companyName(r.companyId))}${relLink('orders',r.orderId,r.orderId?`from ${orderName(r.orderId)}`:'')}`;
  case 'contracts':return `${badgeMaybe(r.status)}${relLink('companies',r.companyId,companyName(r.companyId))}${relLink('contacts',r.contactId,contactName(r.contactId))}`;
  case 'tasks':{
   const navKey={Company:'companies',Contact:'contacts',Opportunity:'opportunities',Quote:'quotes',Order:'orders',Invoice:'invoices',Contract:'contracts'}[r.relatedType];
   return `${badgeMaybe(r.status)}<span>${r.priority||''}</span>${navKey?relLink(navKey,r.relatedId,relatedLabel(r)):'<span>General</span>'}`;
  }
  default:return '';
 }
}
// Which related-record cards to show, per entity - built from data
// already loaded client-side (no new query needed), same filter-in-JS
// approach companyDetail/contactDetail already use.
function recordRelatedDefs(key,r){
 switch(key){
  case 'products':return [
   ['Quotes',data.quotes.filter(q=>(q.items||[]).some(i=>i.productId===r.id)),'quotes',x=>x.number,x=>x.status],
   ['Orders',data.orders.filter(o=>(o.items||[]).some(i=>i.productId===r.id)),'orders',x=>x.number,x=>x.status],
   ['Invoices',data.invoices.filter(i=>(i.items||[]).some(li=>li.productId===r.id)),'invoices',x=>x.number,x=>x.status],
  ];
  case 'quotes':return [
   ['Orders created from this quote',data.orders.filter(o=>o.quoteId===r.id),'orders',x=>x.number,x=>x.status],
   ['Tasks',data.tasks.filter(t=>t.relatedType==='Quote'&&t.relatedId===r.id),'tasks',x=>x.title,x=>x.status],
  ];
  case 'orders':return [
   ['Invoices created from this order',data.invoices.filter(i=>i.orderId===r.id),'invoices',x=>x.number,x=>x.status],
   ['Tasks',data.tasks.filter(t=>t.relatedType==='Order'&&t.relatedId===r.id),'tasks',x=>x.title,x=>x.status],
  ];
  case 'invoices':return [['Tasks',data.tasks.filter(t=>t.relatedType==='Invoice'&&t.relatedId===r.id),'tasks',x=>x.title,x=>x.status]];
  case 'contracts':return [['Tasks',data.tasks.filter(t=>t.relatedType==='Contract'&&t.relatedId===r.id),'tasks',x=>x.title,x=>x.status]];
  default:return [];
 }
}
// Overview panels are built generically from each entity's field tuples,
// which don't carry a distinct "money" type the way tablePage's column
// configs do (a form input for a dollar amount is just type "number") - so
// money-valued fields are called out explicitly here to render through
// money() instead of the plain badgeMaybe() every other field gets.
const MONEY_OVERVIEW_FIELDS={companies:['annualRevenue'],contracts:['value'],invoices:['amountPaid']};
function overviewValueHtml(key,f,r){
 if((MONEY_OVERVIEW_FIELDS[key]||[]).includes(f[0])&&r[f[0]]!==undefined&&r[f[0]]!=='')return money(r[f[0]]);
 return badgeMaybe(r[f[0]]);
}
function genericRecordDetail(key,id){
 const r=byId(key,id);
 if(!r){current=key;detailRecord=null;return renderView()}
 const fieldsFn=DETAIL_FIELDS_FN[key];
 const relationTypes=['auto','relation','filteredContact','filteredOpportunity','filteredQuote','filteredOrder','dynamicRelation'];
 const overviewFields=fieldsFor(key,fieldsFn).filter(f=>!relationTypes.includes(f[2])&&f[0]!==DETAIL_TITLE_FIELD[key]);
 const isDoc=['quotes','orders','invoices'].includes(key);
 const linesHtml=isDoc?`<div class="panel" style="margin-bottom:16px"><h3 style="margin-top:0">Products & services</h3><div class="table-wrap"><table class="table"><thead><tr><th>Product / service</th><th>Quantity</th><th>Unit price</th><th>Line total</th></tr></thead><tbody>${(r.items||[]).map(i=>`<tr><td>${productName(i.productId)}</td><td>${i.quantity}</td><td>${money(i.unitPrice)}</td><td>${money(lineTotal(i))}</td></tr>`).join('')}</tbody></table></div><div class="line-total">Total <strong>${money(docTotal(r))}</strong>${key==='invoices'?` <span class="muted" style="font-size:13px;font-weight:400">· Balance ${money(docBalance(r))}</span>`:''}</div></div>`:'';
 $('#view').innerHTML=`${detail360Header(DETAIL_BREADCRUMB[key],r[DETAIL_TITLE_FIELD[key]]||'—',DETAIL_EYEBROW(key,r),recordEyebrowMeta(key,r))}
 <div class="rule360-grid">
  <div>
   ${linesHtml}
   <div class="panel"><h3 style="margin-top:0">Overview</h3><div class="form-grid" style="margin-top:0">${overviewFields.map(f=>`<div class="field"><label>${f[1]}</label><div>${overviewValueHtml(key,f,r)}</div></div>`).join('')}</div></div>
  </div>
  <div>${recordRelatedDefs(key,r).map(([title,items,navKey,labelFn,metaFn])=>relatedCardHtml(title,items,navKey,labelFn,metaFn)).join('')}</div>
 </div>`;
 wireDetail360Nav(()=>recordModal(key,fieldsFor(key,fieldsFn),r));
}
function recordModal(key,fields,record={}){
 const isDoc=['quotes','orders','invoices'].includes(key);
 if(!record.id){const r0=effectiveRule(key);if(r0)record={...record,[r0.field]:nextNumber(key)}}
 // Phase 4 no-code layout designer: a published layout (Admin -> Screen
 // layouts) groups these same fields into admin-drag-ordered sections; with
 // no published layout the fields render in their plain default order,
 // exactly as before this feature existed.
 const fieldsHtml=orderedFieldGroupsFor(key,fields).map(g=>(g.title?`<div class="field full"><h4 style="margin:14px 0 0">${g.title}</h4></div>`:'')+g.fields.map(f=>fieldHtml(f,record)).join('')).join('');
 const form=`<form id="recordForm"><div class="form-grid">${fieldsHtml}${isDoc?lineItemsHtml(record.items||[]):''}</div><div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Save record</button></div></form>${record.id?'<div id="relatedRecordsPanel"></div>':''}`;
 modal(record.id?'Edit record':'Create record',form); $('[data-close]').onclick=closeModal;
 wireRelations(record); if(isDoc)wireLines();
 applyFieldRules(key,$('#recordForm'));
 // Custom Relationships (admin extensibility, Phase B): a record being
 // edited shows every linked record across every applicable relationship,
 // with inline link/unlink - the same place desktop puts RelatedRecordsCard
 // (below the edit form), since this demo has no separate detail page for
 // most entities.
 if(record.id){relLinkingKey=null;renderRelatedRecordsPanel(key,record.id)}
 $('#recordForm').onsubmit=e=>{e.preventDefault();const obj=Object.fromEntries(new FormData(e.target).entries());
 const relationError=validateRelationships(key,obj);if(relationError)return alert(relationError);
 // Phase 4 custom field extensibility: a save that leaves a custom field
 // empty gets its definition's default value filled in, and a field
 // flagged unique is rejected if another record on this entity already
 // has that value - mirrors custom_field_service::set_entity_values.
 fields.forEach(f=>{const extra=f[4];if(extra?.defaultValue&&!obj[f[0]])obj[f[0]]=extra.defaultValue});
 for(const f of fields){const extra=f[4];if(extra?.unique&&obj[f[0]]){const dup=data[key].some(x=>x.id!==record.id&&String(x[f[0]]||'')===String(obj[f[0]]));if(dup)return alert(`${f[1]} must be unique — "${obj[f[0]]}" is already used by another record.`)}}
 // Business rule save-time actions (block_save/set_default/set_value/
 // clear_value/show_error/show_warning) - require/hide/show/lock/editable/
 // restrict_choices are already reflected live by applyFieldRules; this is
 // the save-time-only subset, mirrors custom_field_service::set_entity_values.
 const ruleOutcome=evaluateFieldRulesForSave(key,obj);
 if(ruleOutcome.blocked)return alert(ruleOutcome.blocked);
 Object.assign(obj,ruleOutcome.setValues);
 fields.filter(f=>f[2]==='number').forEach(f=>obj[f[0]]=Number(obj[f[0]]||0));if(isDoc){obj.items=[...document.querySelectorAll('.line-row')].map(r=>({productId:$('.line-product',r).value,quantity:Number($('.line-qty',r).value||1),unitPrice:Number($('.line-price',r).value||0)})).filter(i=>i.productId);if(!obj.items.length)return alert('Add at least one product or service.')}
 const wasEdit=!!record.id, before=wasEdit?{...record}:null;
 // Status Transition Editor (Phase 2): if this entity has any active
 // transition rules, the status/stage field can only move along a listed
 // from -> to pair (a rule with from:'' matches any starting status). No
 // active rules at all means the field stays fully unrestricted, and
 // resaving the same status is never blocked since it's not a change.
 if(wasEdit&&before){
  const tf=transitionFieldFor(key), fromVal=before[tf], toVal=obj[tf];
  if(fromVal!==undefined&&toVal!==undefined&&fromVal!==toVal){
   const activeRules=(data.statusTransitionRules||[]).filter(r=>r.entity===key&&r.active);
   if(activeRules.length&&!activeRules.some(r=>r.to===toVal&&(r.from===''||r.from===fromVal)))
    return alert(`"${fromVal||'—'} → ${toVal}" is not an allowed ${fieldLabelFor(key,tf)} transition. Configure allowed transitions in Admin → Status transitions.`);
  }
 }
 if(wasEdit)Object.assign(byId(key,record.id),obj);else{const rule=effectiveRule(key);if(rule&&!obj[rule.field])obj[rule.field]=nextNumber(key);data[key].unshift({id:uid(),...obj})}
 // Workflow execution isn't limited to relatedTypeFor's built-ins anymore -
 // create_record/update_related_record/update_field don't need a Task
 // relatedType at all, and create_task itself already falls back to
 // 'General' for anything relatedTypeFor doesn't cover (see
 // executeWorkflowAction below), so custom objects fire workflows too.
 if(wasEdit&&before){
  // v0.25: one unified Conditions list per rule (see migrateWorkflowRule) -
  // fires only when at least one field the conditions actually reference
  // changed on this save (create has no "before" snapshot, so only edits
  // can fire, same as the old single-trigger-field check) AND the full
  // condition set matches the just-saved record. This is the standard
  // Salesforce/Dynamics "entry criteria" pattern: one list of conditions,
  // re-evaluated on save, instead of a separate mandatory trigger plus
  // optional extra filters.
  (data.workflowRules||[]).filter(r=>r.entity===key&&r.active).forEach(r=>{
   const conditions=r.conditions||[];
   if(!conditions.length)return;
   const watchedFields=new Set();
   conditions.forEach(c=>{watchedFields.add(c.fieldKey);if(c.compareField)watchedFields.add(c.compareField)});
   const anyChanged=[...watchedFields].some(f=>obj[f]!==undefined&&obj[f]!==before[f]);
   if(!anyChanged)return;
   if(!conditionsMatch(r.matchType||'all',conditions,obj))return;
   const descriptions=(r.actions||[]).map(a=>executeWorkflowAction(a,key,record)).filter(Boolean);
   if(!descriptions.length)return; // e.g. update_related_record with nothing linked yet - a silent no-op, same as desktop
   if(r.notify){
    const label=obj.name||obj.title||obj.number||entityLabel(key);
    data.notifications.unshift({id:uid(),message:`${entityLabel(key).replace(/s$/,'')} "${label}" — ${describeConditions(key,conditions,r.matchType||'all')} — ${descriptions.join('; ')}`,createdAt:new Date().toISOString(),read:false});
   }
  });
 }
 save();closeModal();toast('Record saved');refreshNotifBadge();renderView();
 // show_error/show_warning notices (non-blocking - the save already
 // succeeded) - mirrors the desktop edition's showRuleMessages.
 const notices=[...ruleOutcome.errors.map(m=>`⚠ ${m}`),...ruleOutcome.warnings];
 if(notices.length)alert(notices.join('\n'))};
}
function wireRelations(record){
 const form=$('#recordForm');
 const company=form.elements.companyId;
 function refresh(){
  const cid=company?.value||'';
  const maps={
   contact:['contacts',x=>x.name,'No contact'],
   opportunity:['opportunities',x=>x.title,'No opportunity'],
   quote:['quotes',x=>x.number+' · '+money(docTotal(x)),'No source quote'],
   order:['orders',x=>x.number+' · '+money(docTotal(x)),'No source order']
  };
  Object.entries(maps).forEach(([k,[arr,label,empty]])=>{
   const el=form.querySelector(`[data-filter="${k}"]`);if(!el)return;
   const old=el.value;
   const filtered=data[arr].filter(x=>!cid||x.companyId===cid);
   el.innerHTML=optionalOptions(filtered,old,empty,label);
  });
 }
 if(company){company.addEventListener('change',refresh);refresh()}

 // Optional source records streamline entry but never become mandatory.
 const opportunity=form.elements.opportunityId;
 if(opportunity){opportunity.addEventListener('change',()=>{
  const source=byId('opportunities',opportunity.value);if(!source)return;
  if(company){company.value=source.companyId;refresh()}
  if(form.elements.contactId)form.elements.contactId.value=source.contactId||'';
 })}
 const quote=form.elements.quoteId;
 if(quote){quote.addEventListener('change',()=>{
  const source=byId('quotes',quote.value);if(!source)return;
  if(company){company.value=source.companyId;refresh()}
  if(form.elements.contactId)form.elements.contactId.value=source.contactId||'';
  replaceLineItems(source.items||[]);
 })}
 const order=form.elements.orderId;
 if(order){order.addEventListener('change',()=>{
  const source=byId('orders',order.value);if(!source)return;
  if(company){company.value=source.companyId;refresh()}
  replaceLineItems(source.items||[]);
 })}

 const type=form.elements.relatedType, rel=form.querySelector('[data-dynamic-related]');
 function refreshRelated(){if(!type||!rel)return;const t=type.value;const map={Company:['companies',x=>x.name],Contact:['contacts',x=>`${x.name} · ${companyName(x.companyId)}`],Opportunity:['opportunities',x=>x.title],Quote:['quotes',x=>x.number],Order:['orders',x=>x.number],Invoice:['invoices',x=>x.number],Contract:['contracts',x=>x.number]};if(!map[t]){rel.innerHTML='<option value="">General</option>';rel.disabled=true;return}rel.disabled=false;const [arr,label]=map[t];rel.innerHTML=options(data[arr],record.relatedId,label)}if(type){type.addEventListener('change',refreshRelated);refreshRelated()}
}
// Which relationship group (by definition key) currently has its inline
// link picker open in the record modal - reset to null every time the
// modal opens (see recordModal), scoped to whichever modal is on screen.
let relLinkingKey=null;
function linkPickerHtml(g){
 const optionsHtml=(data[g.otherType]||[]).map(r=>`<option value="${r.id}">${recordDisplayName(g.otherType,r)}</option>`).join('');
 return `<div style="display:flex;gap:8px;align-items:center;margin-top:8px;flex-wrap:wrap"><select data-link-select><option value="">Select a record…</option>${optionsHtml}</select><button type="button" class="btn btn-primary" data-link-submit>Link</button><button type="button" class="btn btn-secondary" data-link-cancel>Cancel</button></div>`;
}
// Renders every related record for entityType/entityId across every
// applicable relationship, with inline link/unlink - mirrors desktop's
// RelatedRecordsCard, just re-rendering itself in place on every change
// instead of a query-client refetch.
function renderRelatedRecordsPanel(entityType,entityId){
 const panel=$('#relatedRecordsPanel'); if(!panel)return;
 const defs=relationshipDefsFor(entityType);
 if(!defs.length){panel.innerHTML='';return}
 const related=relatedRecordsFor(entityType,entityId);
 const groups=defs.map(def=>{
  const isSource=def.sourceEntity===entityType;
  return {def,label:isSource?def.forwardLabel:def.reverseLabel,otherType:isSource?def.targetEntity:def.sourceEntity,isSource,rows:related.filter(r=>r.defKey===def.key)};
 });
 panel.innerHTML=`<div class="panel" style="margin-top:16px"><h3 style="margin-top:0">Related records</h3>${groups.map(g=>`<div style="margin-bottom:14px"><div class="panel-head" style="margin-bottom:4px"><strong>${g.label}</strong><button type="button" class="btn btn-secondary" data-link-group="${g.def.key}">+ Link</button></div>${g.rows.length?g.rows.map(r=>`<div class="deal" style="display:flex;justify-content:space-between;align-items:center"><span>${r.displayName} ${badgeMaybe(r.status)}</span><button type="button" class="icon-btn" data-unlink="${r.instanceId}">Unlink</button></div>`).join(''):'<div class="empty">None linked</div>'}${relLinkingKey===g.def.key?linkPickerHtml(g):''}</div>`).join('')}</div>`;
 groups.forEach(g=>{
  const btn=panel.querySelector(`[data-link-group="${g.def.key}"]`);
  if(btn)btn.onclick=()=>{relLinkingKey=relLinkingKey===g.def.key?null:g.def.key;renderRelatedRecordsPanel(entityType,entityId)};
 });
 panel.querySelectorAll('[data-unlink]').forEach(b=>b.onclick=()=>{
  data.relationshipInstances=(data.relationshipInstances||[]).filter(i=>i.id!==b.dataset.unlink);
  save();toast('Unlinked');renderRelatedRecordsPanel(entityType,entityId);
 });
 const linkGroup=groups.find(g=>g.def.key===relLinkingKey);
 if(linkGroup){
  const select=panel.querySelector('[data-link-select]'), linkBtn=panel.querySelector('[data-link-submit]'), cancelBtn=panel.querySelector('[data-link-cancel]');
  if(cancelBtn)cancelBtn.onclick=()=>{relLinkingKey=null;renderRelatedRecordsPanel(entityType,entityId)};
  if(linkBtn)linkBtn.onclick=()=>{
   const otherId=select.value; if(!otherId)return;
   const sourceEntity=linkGroup.isSource?entityType:linkGroup.otherType, sourceId=linkGroup.isSource?entityId:otherId;
   const targetEntity=linkGroup.isSource?linkGroup.otherType:entityType, targetId=linkGroup.isSource?otherId:entityId;
   const err=relationshipLinkError(linkGroup.def,sourceId,targetId);
   if(err)return alert(err);
   data.relationshipInstances.push({id:uid(),definitionId:linkGroup.def.id,sourceEntity,sourceId,targetEntity,targetId});
   save();toast('Linked');relLinkingKey=null;renderRelatedRecordsPanel(entityType,entityId);
  };
 }
}

function validateRelationships(key,obj){
 const cid=obj.companyId||'';
 const contact=obj.contactId?byId('contacts',obj.contactId):null;
 if(contact&&contact.companyId!==cid)return 'The selected contact does not belong to the selected customer.';
 if(key==='quotes'&&obj.opportunityId){const opp=byId('opportunities',obj.opportunityId);if(!opp||opp.companyId!==cid)return 'The selected opportunity does not belong to the selected customer.';if(contact&&opp.contactId&&opp.contactId!==contact.id)return 'The selected contact does not match the opportunity primary contact.'}
 if(key==='orders'&&obj.quoteId){const q=byId('quotes',obj.quoteId);if(!q||q.companyId!==cid)return 'The selected quote does not belong to the selected customer.';if(contact&&q.contactId&&q.contactId!==contact.id)return 'The selected contact does not match the source quote.'}
 if(key==='invoices'&&obj.orderId){const o=byId('orders',obj.orderId);if(!o||o.companyId!==cid)return 'The selected order does not belong to the selected customer.'}
 return '';
}
function replaceLineItems(items){
 const rows=$('#lineRows');if(!rows)return;
 rows.innerHTML=(items.length?items:[{productId:'',quantity:1,unitPrice:0}]).map(lineRow).join('');
 wireLines();
}
function wireLines(){const rows=$('#lineRows');function recalc(){let total=0;rows.querySelectorAll('.line-row').forEach(r=>{const p=byId('products',$('.line-product',r).value);if(p&&Number($('.line-price',r).value)===0)$('.line-price',r).value=p.price;const sub=Number($('.line-qty',r).value||0)*Number($('.line-price',r).value||0);$('.line-subtotal',r).textContent=money(sub);total+=sub});$('#docTotal').textContent=money(total)}function bind(r){$('.line-product',r).onchange=()=>{const p=byId('products',$('.line-product',r).value);if(p){$('.line-price',r).value=p.price;if(p.type==='Service'&&!$('.line-qty',r).value)$('.line-qty',r).value=1}recalc()};$('.line-qty',r).oninput=recalc;$('.line-price',r).oninput=recalc;$('.line-remove',r).onclick=()=>{r.remove();recalc()}}rows.querySelectorAll('.line-row').forEach(bind);$('#addLine').onclick=()=>{rows.insertAdjacentHTML('beforeend',lineRow());bind(rows.lastElementChild);recalc()};recalc()}
function dependencies(key,id){const refs=[];if(key==='companies'){['contacts','opportunities','quotes','orders','invoices','contracts'].forEach(k=>{const n=data[k].filter(x=>x.companyId===id).length;if(n)refs.push(`${n} ${labels[k]||k}`)})}if(key==='contacts'){const maps=[['opportunities','contactId'],['quotes','contactId'],['orders','contactId'],['contracts','contactId']];maps.forEach(([k,f])=>{const n=data[k].filter(x=>x[f]===id).length;if(n)refs.push(`${n} ${labels[k]||k}`)});const n=data.tasks.filter(x=>x.relatedType==='Contact'&&x.relatedId===id).length;if(n)refs.push(`${n} tasks`)}if(key==='opportunities'){const n=data.quotes.filter(x=>x.opportunityId===id).length;if(n)refs.push(`${n} quotes`);const t=data.tasks.filter(x=>x.relatedType==='Opportunity'&&x.relatedId===id).length;if(t)refs.push(`${t} tasks`)}if(key==='products'){['quotes','orders','invoices'].forEach(k=>{const n=data[k].filter(x=>(x.items||[]).some(i=>i.productId===id)).length;if(n)refs.push(`${n} ${labels[k]}`)})}if(key==='quotes'){const n=data.orders.filter(x=>x.quoteId===id).length;if(n)refs.push(`${n} orders`)}if(key==='orders'){const n=data.invoices.filter(x=>x.orderId===id).length;if(n)refs.push(`${n} invoices`)}return refs}
function remove(key,id){
 const refs=dependencies(key,id);if(refs.length)return alert(`This record is connected to ${refs.join(', ')}. Update or delete those records first.`);
 const relBlock=relationshipDeleteCheck(key,id);if(relBlock)return alert(relBlock);
 if(confirm('Delete this record?')){clearArchivableRelationshipInstances(key,id);data[key]=data[key].filter(x=>x.id!==id);save();toast('Record deleted');renderView()}
}
function modal(title,body){document.body.insertAdjacentHTML('beforeend',`<div class="modal-backdrop" id="modal"><div class="modal"><div class="modal-head"><h2>${title}</h2><button class="icon-btn" onclick="document.getElementById('modal').remove()">✕</button></div>${body}</div></div>`)}
function closeModal(){document.getElementById('modal')?.remove()}
function toast(msg){document.body.insertAdjacentHTML('beforeend',`<div class="toast">${msg}</div>`);setTimeout(()=>$('.toast')?.remove(),2200)}

// ---- Admin panel ---------------------------------------------------------
function entityLabel(key){return key==='opportunities'?labels.pipeline:labels[key]}
function entityPills(keys,active){return `<div class="entity-tabs">${keys.map(k=>`<button class="pill-tab ${k===active?'active':''}" data-entity="${k}">${entityLabel(k)}</button>`).join('')}</div>`}
function adminPage(){
 document.title='Admin — Lanesra OS Demo';
 const tabs=[['profile','Business profile'],['users','Users & roles'],['objects','Custom Objects'],['relationships','Relationships'],['fields','Custom fields'],['rules','Business rules'],['workflow','Workflow automation'],['transitions','Status transitions'],['layouts','Screen layouts'],['integrations','Integrations'],['numbering','Numbering'],['kpis','Dashboard KPIs']];
 $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><span>Admin</span></div><h1>Admin panel</h1><p class="muted">Configure your workspace, users and automation. Changes save immediately in this browser.</p></div></div><div class="tabs">${tabs.map(t=>`<button class="tab ${adminTab===t[0]?'active':''}" data-admin-tab="${t[0]}">${t[1]}</button>`).join('')}</div><div id="adminBody" class="admin-body"></div>`;
 $('[data-clear-filter]').onclick=()=>{current='dashboard';viewFilter=null;renderView()};
 document.querySelectorAll('[data-admin-tab]').forEach(b=>b.onclick=()=>{adminTab=b.dataset.adminTab;renderAdminTab()});
 renderAdminTab();
}
function renderAdminTab(){
 document.querySelectorAll('[data-admin-tab]').forEach(b=>b.classList.toggle('active',b.dataset.adminTab===adminTab));
 const body=$('#adminBody');
 ({profile:profileTab,users:usersTab,objects:objectsTab,relationships:relationshipsTab,fields:fieldsTab,rules:rulesTab,workflow:workflowTab,transitions:transitionsTab,layouts:layoutsTab,integrations:integrationsTab,numbering:numberingTab,kpis:kpisTab}[adminTab])(body);
}
function profileTab(body){
 const w=data.workspace;
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Business profile</h3><p class="muted">Shown across the workspace.</p><form id="profileForm" class="form-grid">
 <div class="field"><label>Company name</label><input name="name" value="${w.name}" required></div>
 <div class="field"><label>Phone</label><input name="phone" value="${w.phone||''}"></div>
 <div class="field full"><label>Address</label><input name="address" value="${w.address||''}"></div>
 <div class="field"><label>City / region</label><input name="city" value="${w.city||''}"></div>
 <div class="field"><label>Logo URL (optional)</label><input name="logo" value="${w.logo||''}" placeholder="https://…"></div>
 <div class="field full"><button class="btn btn-primary" type="submit">Save business profile</button></div>
 </form></div>`;
 $('#profileForm').onsubmit=e=>{e.preventDefault();const obj=Object.fromEntries(new FormData(e.target).entries());Object.assign(data.workspace,obj);save();toast('Business profile updated');renderView()};
}
function userFields(){return [['name','Full name'],['email','Email'],['role','Role','select','Administrator|Sales Rep|Viewer'],['status','Status','select','Active|Inactive']]}
function usersTab(body){
 const arr=data.users;
 body.innerHTML=`<div class="panel"><div class="panel-head"><h3>Users & roles</h3><button class="btn btn-primary" id="addUser">+ New user</button></div><div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Status</th><th>Actions</th></tr></thead><tbody>${arr.map(u=>`<tr><td>${u.name}</td><td>${u.email}</td><td>${u.role}</td><td>${badgeMaybe(u.status)}</td><td><div class="actions"><button class="icon-btn" data-edit="${u.id}">Edit</button><button class="icon-btn" data-del="${u.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${arr.length?'':'<div class="empty">No users yet</div>'}</div><p class="muted" style="margin-top:12px">Roles are illustrative in this browser demo — the desktop edition enforces per-role access control server-side.</p></div>`;
 $('#addUser').onclick=()=>recordModal('users',userFields());
 body.querySelectorAll('[data-edit]').forEach(b=>b.onclick=()=>recordModal('users',userFields(),byId('users',b.dataset.edit)));
 body.querySelectorAll('[data-del]').forEach(b=>b.onclick=()=>remove('users',b.dataset.del));
}
// ---- Custom Objects (admin extensibility) ---------------------------------
// Lets an Administrator define a whole new business object at runtime -
// Vendors, Assets, Projects - with no code change. Mirrors the desktop
// edition's custom_object_service exactly: a stable lowercase_underscore
// key, a fixed record-number prefix/digit width, and once created it's a
// full citizen of custom fields, business rules and status transitions -
// see fieldsFnFor/effectiveRule/syncCustomObjectRegistry above.
const OBJECT_ICON_CHOICES=['◆','🏭','📦','🚗','🏢','🔧','📋','🗂️','💼','🏗️'];
function objectsTab(body){
 const arr=data.customObjects||[];
 body.innerHTML=`<div class="panel"><div class="panel-head"><h3>Custom Objects</h3><button class="btn btn-primary" id="addObject">+ New object</button></div><p class="muted">Add a whole new business object — Vendors, Assets, Projects — without a code change. Once created it gets its own place in the sidebar and works with custom fields, business rules, status transitions and workflow automation exactly like a built-in object.</p><div class="table-wrap"><table class="table"><thead><tr><th></th><th>Name</th><th>Key</th><th>Numbering</th><th>Records</th><th>Status</th><th>Actions</th></tr></thead><tbody>${arr.map(o=>`<tr><td>${o.icon}</td><td>${o.labelPlural}</td><td><code>${o.key}</code></td><td><code>${o.prefix}-${'0'.repeat(o.digits)}</code></td><td>${(data[o.key]||[]).length}</td><td>${badgeMaybe(o.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-edit-object="${o.id}">Edit</button></div></td></tr>`).join('')}</tbody></table>${arr.length?'':'<div class="empty">No custom objects yet</div>'}</div></div>`;
 $('#addObject').onclick=()=>customObjectModal();
 body.querySelectorAll('[data-edit-object]').forEach(b=>b.onclick=()=>customObjectModal(arr.find(o=>o.id===b.dataset.editObject)));
}
function customObjectModal(obj){
 const isEdit=!!obj;
 const body=`<form id="objForm" class="form-grid">
 <div class="field"><label>Singular name</label><input name="singular" value="${obj?.label||''}" placeholder="Vendor" required></div>
 <div class="field"><label>Plural name</label><input name="plural" value="${obj?.labelPlural||''}" placeholder="Vendors" required></div>
 <div class="field"><label>Icon</label><select name="icon">${OBJECT_ICON_CHOICES.map(i=>`<option value="${i}" ${obj?.icon===i?'selected':''}>${i}</option>`).join('')}</select></div>
 <div class="field"><label>Record-number prefix</label><input name="prefix" value="${obj?.prefix||''}" placeholder="VEN" maxlength="20" required></div>
 <div class="field"><label>Digit width</label><input name="digits" type="number" min="1" max="10" value="${obj?.digits??6}"></div>
 ${isEdit?`<div class="field"><label>Active</label><select name="active"><option value="true" ${obj.active?'selected':''}>Active</option><option value="false" ${!obj.active?'selected':''}>Inactive</option></select></div><div class="field full"><small class="field-help">Key: <code>${obj.key}</code> (fixed — every custom field, business rule and record keys off this)</small></div>`:''}
 <div class="modal-actions">${isEdit?`<button type="button" class="btn btn-secondary" data-delete-object>Delete</button>`:''}<button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">${isEdit?'Save object':'Create object'}</button></div>
 </form>`;
 modal(isEdit?`Edit ${obj.labelPlural}`:'New custom object',body);
 $('[data-close]').onclick=closeModal;
 $('#objForm').onsubmit=e=>{
  e.preventDefault();
  const fd=Object.fromEntries(new FormData(e.target).entries());
  if(!fd.singular.trim()||!fd.plural.trim())return alert('Singular and plural names are required.');
  const prefix=fd.prefix.trim();
  if(!prefix||prefix.length>20)return alert('Record-number prefix must be 1-20 characters.');
  const digits=Math.min(10,Math.max(1,Number(fd.digits)||1));
  if(isEdit){
   Object.assign(obj,{label:fd.singular,labelPlural:fd.plural,icon:fd.icon,prefix,digits,active:fd.active==='true'});
  }else{
   const rawKey=slugifyObjectKey(fd.singular);
   if(!rawKey)return alert('Object name must contain at least one letter or number.');
   // A custom object literally named e.g. "Company" would slug-collide in
   // spirit with a built-in entity - block it outright, same as desktop.
   if(RESERVED_ENTITY_KEYS.includes(rawKey))return alert(`"${fd.singular}" is too close to a built-in object name — choose another.`);
   let key=rawKey,suffix=2;
   while((data.customObjects||[]).some(o=>o.key===key))key=`${rawKey}_${suffix++}`;
   data.customObjects.push({id:uid(),key,label:fd.singular,labelPlural:fd.plural,icon:fd.icon,prefix,digits,active:true});
   data[key]=[];
  }
  syncCustomObjectRegistry();
  renderSidebarNav();
  save();closeModal();toast(isEdit?'Custom object saved':'Custom object created');renderAdminTab();
 };
 if(isEdit){
  // Hard-delete is blocked while any record exists (matches desktop's
  // custom_object_service::delete) - deactivating is always safe instead,
  // since it just hides the object from nav/creation without touching data.
  $('[data-delete-object]').onclick=()=>{
   const count=(data[obj.key]||[]).length;
   if(count)return alert(`Cannot delete '${obj.labelPlural}' — ${count} record(s) still exist. Delete or archive them first, or deactivate the object instead.`);
   if(!confirm(`Delete '${obj.labelPlural}'? This only works because it has no records.`))return;
   data.customObjects=data.customObjects.filter(o=>o.id!==obj.id);
   delete data[obj.key];
   syncCustomObjectRegistry();
   renderSidebarNav();
   if(current===obj.key){current='dashboard';detailRecord=null}
   save();closeModal();toast('Custom object deleted');renderAdminTab();
  };
 }
}
// ---- Custom Relationships (admin extensibility, Phase B) ------------------
function relationshipsTab(body){
 const arr=data.relationshipDefinitions||[];
 body.innerHTML=`<div class="panel"><div class="panel-head"><h3>Relationships</h3><button class="btn btn-primary" id="addRelationship">+ New relationship</button></div><p class="muted">Connect any two object types — built-in or custom. Once created, both sides automatically show a related list on that record's edit form, and link/unlink from there.</p><div class="table-wrap"><table class="table"><thead><tr><th>Connects</th><th>Type</th><th>Labels</th><th>On delete</th><th>Status</th><th>Actions</th></tr></thead><tbody>${arr.map(d=>`<tr><td>${entityLabel(d.sourceEntity)} → ${entityLabel(d.targetEntity)}</td><td>${RELATIONSHIP_TYPE_LABELS[d.relType]}</td><td><span title="Forward label, shown on the source record">${d.forwardLabel}</span> / <span title="Reverse label, shown on the target record">${d.reverseLabel}</span></td><td>${DELETE_BEHAVIOR_LABELS[d.deleteBehavior]}</td><td>${badgeMaybe(d.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-edit-rel="${d.id}">Edit</button></div></td></tr>`).join('')}</tbody></table>${arr.length?'':'<div class="empty">No relationships defined yet</div>'}</div></div>`;
 $('#addRelationship').onclick=()=>relationshipModal();
 body.querySelectorAll('[data-edit-rel]').forEach(b=>b.onclick=()=>relationshipModal(arr.find(d=>d.id===b.dataset.editRel)));
}
function relationshipModal(def){
 const isEdit=!!def;
 const keys=allEntityTypeKeys();
 const source=def?.sourceEntity||keys[0], target=def?.targetEntity||keys[1]||keys[0];
 const body=`<form id="relForm" class="form-grid">
 <div class="field"><label>Source (the "many"/owning side)</label><select name="source" ${isEdit?'disabled':''}>${keys.map(k=>`<option value="${k}" ${k===source?'selected':''}>${entityLabel(k)}</option>`).join('')}</select></div>
 <div class="field"><label>Target</label><select name="target" ${isEdit?'disabled':''}>${keys.map(k=>`<option value="${k}" ${k===target?'selected':''}>${entityLabel(k)}</option>`).join('')}</select></div>
 <div class="field"><label>Relationship type</label><select name="relType" ${isEdit?'disabled':''}>${RELATIONSHIP_TYPES.map(t=>`<option value="${t}" ${def?.relType===t?'selected':''}>${RELATIONSHIP_TYPE_LABELS[t]}</option>`).join('')}</select></div>
 <div class="field"><label>On delete</label><select name="deleteBehavior">${DELETE_BEHAVIORS.map(b=>`<option value="${b}" ${(def?.deleteBehavior||'restrict')===b?'selected':''}>${DELETE_BEHAVIOR_LABELS[b]}</option>`).join('')}</select></div>
 <div class="field"><label>Forward label (shown on the source record)</label><input name="forwardLabel" value="${def?.forwardLabel||''}" placeholder="${entityLabel(target)}" required></div>
 <div class="field"><label>Reverse label (shown on the target record)</label><input name="reverseLabel" value="${def?.reverseLabel||''}" placeholder="${entityLabel(source)}" required></div>
 <div class="field full"><label class="checkbox-row" style="padding:0"><input type="checkbox" name="showRelatedList" value="true" ${def?.showRelatedList!==false?'checked':''}> Show as a related list on both records</label></div>
 <div class="field full"><label class="checkbox-row" style="padding:0"><input type="checkbox" name="required" value="true" ${def?.required?'checked':''}> Source record should have a target linked</label></div>
 ${isEdit?`<div class="field"><label>Active</label><select name="active"><option value="true" ${def.active?'selected':''}>Active</option><option value="false" ${!def.active?'selected':''}>Inactive</option></select></div>`:''}
 <div class="modal-actions">${isEdit?`<button type="button" class="btn btn-secondary" data-delete-rel>Delete</button>`:''}<button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">${isEdit?'Save relationship':'Create relationship'}</button></div>
 </form>`;
 modal(isEdit?`Edit ${entityLabel(source)} → ${entityLabel(target)}`:'New relationship',body);
 $('[data-close]').onclick=closeModal;
 $('#relForm').onsubmit=e=>{
  e.preventDefault();
  const fd=Object.fromEntries(new FormData(e.target).entries());
  if(!fd.forwardLabel.trim()||!fd.reverseLabel.trim())return alert('Both direction labels are required.');
  if(isEdit){
   Object.assign(def,{forwardLabel:fd.forwardLabel,reverseLabel:fd.reverseLabel,deleteBehavior:fd.deleteBehavior,showRelatedList:fd.showRelatedList==='true',required:fd.required==='true',active:fd.active==='true'});
  }else{
   if(fd.source===fd.target)return alert('A relationship must connect two different object types.');
   const base=`${fd.source}_${fd.target}`;
   let key=base,suffix=2;
   while((data.relationshipDefinitions||[]).some(d=>d.key===key))key=`${base}_${suffix++}`;
   data.relationshipDefinitions.push({id:uid(),key,sourceEntity:fd.source,targetEntity:fd.target,relType:fd.relType,forwardLabel:fd.forwardLabel,reverseLabel:fd.reverseLabel,deleteBehavior:fd.deleteBehavior,showRelatedList:fd.showRelatedList==='true',required:fd.required==='true',active:true,protected:false});
  }
  save();closeModal();toast(isEdit?'Relationship saved':'Relationship created');renderAdminTab();
 };
 if(isEdit){
  // Hard-delete is blocked while any link exists (matches
  // relationship_service::delete) - deactivating is always safe instead.
  $('[data-delete-rel]').onclick=()=>{
   const count=(data.relationshipInstances||[]).filter(i=>i.definitionId===def.id).length;
   if(count)return alert(`Cannot delete this relationship — ${count} record(s) are still linked through it. Unlink them first, or deactivate the relationship instead.`);
   if(!confirm('Delete this relationship? This only works if no records are linked through it.'))return;
   data.relationshipDefinitions=data.relationshipDefinitions.filter(d=>d.id!==def.id);
   save();closeModal();toast('Relationship deleted');renderAdminTab();
  };
 }
}
// ---- Screen layouts (Phase 4: no-code UI layout designer) -----------------
// A new capability, not a desktop port (desktop has no layout designer
// either): an admin arranges any object's create/edit fields into
// drag-ordered sections. Editing only ever touches the *draft* - the live
// record form keeps using the plain field order until Publish copies the
// draft to publishedSections, and Unpublish clears it back to that default.
// A published layout never hides a field it doesn't know about: any field
// missing from the layout (new custom field added after publishing, a
// stale key from a deleted one) is filtered out or auto-appended to a
// trailing "Other fields" group, so a layout change can never silently
// drop something off the live form.
let layoutsEntityKey=null;
function allFieldsFor(entityKey){return fieldsFor(entityKey,fieldsFnFor(entityKey))}
function ensureLayoutDraft(entityKey){
 if(!data.uiLayouts[entityKey]){
  data.uiLayouts[entityKey]={draftSections:[{id:uid(),title:'Details',fields:allFieldsFor(entityKey).map(f=>f[0])}],publishedSections:null,updatedAt:null};
  save();
 }
 return data.uiLayouts[entityKey];
}
function orderedFieldGroupsFor(entityKey,fields){
 const layout=data.uiLayouts&&data.uiLayouts[entityKey];
 if(!layout||!layout.publishedSections)return [{title:null,fields}];
 const byKey=Object.fromEntries(fields.map(f=>[f[0],f]));
 const used=new Set();
 const groups=layout.publishedSections.map(s=>{
  const secFields=s.fields.map(k=>byKey[k]).filter(Boolean);
  secFields.forEach(f=>used.add(f[0]));
  return {title:s.title,fields:secFields};
 }).filter(g=>g.fields.length);
 const rest=fields.filter(f=>!used.has(f[0]));
 if(rest.length)groups.push({title:groups.length?'Other fields':null,fields:rest});
 return groups.length?groups:[{title:null,fields}];
}
function layoutsTab(body){
 const keys=allEntityTypeKeys();
 if(!layoutsEntityKey||!keys.includes(layoutsEntityKey))layoutsEntityKey=keys[0];
 const entityKey=layoutsEntityKey;
 const layout=ensureLayoutDraft(entityKey);
 const isPublished=!!layout.publishedSections;
 const hasDraftChanges=JSON.stringify(layout.draftSections)!==JSON.stringify(layout.publishedSections);
 body.innerHTML=`<div class="panel">
 <div class="panel-head"><h3>Screen layouts</h3><select id="layoutsEntitySelect">${keys.map(k=>`<option value="${k}" ${k===entityKey?'selected':''}>${entityLabel(k)}</option>`).join('')}</select></div>
 <p class="muted" style="font-size:13px">Drag fields to reorder them or move them between sections on ${entityLabel(entityKey)}'s create/edit form. Editing here only changes the draft — the live form keeps its default order until you Publish.</p>
 <div style="margin-bottom:14px"><span class="badge">${isPublished?(hasDraftChanges?'Published — unpublished draft changes':'Published'):'Not published — using default field order'}</span></div>
 <div id="layoutSections"></div>
 <div class="actions" style="margin-top:16px;flex-wrap:wrap">
  <button class="btn btn-secondary" id="addSection" type="button">+ Add section</button>
  <button class="btn btn-secondary" id="previewLayout" type="button">Preview draft</button>
  <button class="btn btn-secondary" id="revertLayout" type="button" ${isPublished?'':'disabled'}>Revert draft to published</button>
  <button class="btn btn-primary" id="publishLayout" type="button">Publish</button>
  ${isPublished?'<button class="btn btn-secondary" id="unpublishLayout" type="button">Unpublish</button>':''}
 </div>
 </div>`;
 $('#layoutsEntitySelect').onchange=e=>{layoutsEntityKey=e.target.value;layoutsTab(body)};
 renderLayoutSections(entityKey);
 $('#addSection').onclick=()=>{layout.draftSections.push({id:uid(),title:'New section',fields:[]});save();renderLayoutSections(entityKey)};
 $('#previewLayout').onclick=()=>layoutPreviewModal(entityKey);
 $('#publishLayout').onclick=()=>{layout.publishedSections=structuredClone(layout.draftSections);layout.updatedAt=new Date().toISOString();save();toast('Layout published');layoutsTab(body)};
 $('#revertLayout').onclick=()=>{if(!layout.publishedSections)return;layout.draftSections=structuredClone(layout.publishedSections);save();toast('Draft reverted to the published layout');layoutsTab(body)};
 const unpub=$('#unpublishLayout'); if(unpub)unpub.onclick=()=>{if(!confirm('Unpublish this layout? The live form goes back to the default field order until you publish again.'))return;layout.publishedSections=null;save();toast('Layout unpublished');layoutsTab(body)};
}
function renderLayoutSections(entityKey){
 const layout=data.uiLayouts[entityKey];
 const allFields=allFieldsFor(entityKey);
 const fieldLabel=k=>{const f=allFields.find(x=>x[0]===k);return f?f[1]:k};
 const placedKeys=new Set(layout.draftSections.flatMap(s=>s.fields));
 const unplaced=allFields.map(f=>f[0]).filter(k=>!placedKeys.has(k));
 const chip=(k,idx)=>`<span class="layout-field-chip" draggable="true" data-field-key="${k}" data-section-idx="${idx}" style="border:1px solid var(--line);border-radius:8px;padding:6px 10px;background:${idx>=0?'#f9fafb':'#fff'};cursor:grab;font-size:13px;display:inline-block">⠿ ${fieldLabel(k)}</span>`;
 const listHtml=(fieldsArr,idx)=>`<div class="layout-field-list" data-section-idx="${idx}" style="display:flex;flex-wrap:wrap;gap:8px;min-height:34px">${fieldsArr.map(k=>chip(k,idx)).join('')||'<span class="muted" style="font-size:12px">Drag fields here</span>'}</div>`;
 const box=$('#layoutSections'); if(!box)return;
 box.innerHTML=`${layout.draftSections.map((s,idx)=>`<div class="layout-section" style="border:1px solid var(--line);border-radius:12px;padding:12px;margin-bottom:12px">
  <div style="display:flex;justify-content:space-between;align-items:center;gap:8px;margin-bottom:8px">
   <input class="layout-section-title" data-section-idx="${idx}" value="${s.title}" style="border:1px solid var(--line);border-radius:8px;padding:6px 9px;font-weight:700;flex:1">
   <button class="icon-btn" data-remove-section="${idx}" type="button" ${layout.draftSections.length<=1?'disabled':''}>Delete section</button>
  </div>
  ${listHtml(s.fields,idx)}
 </div>`).join('')}
 <div class="layout-section" style="border:1px dashed var(--line);border-radius:12px;padding:12px">
  <div class="muted" style="font-weight:700;margin-bottom:8px">Unplaced fields — not shown on the form</div>
  ${listHtml(unplaced,-1)}
 </div>`;
 box.querySelectorAll('.layout-section-title').forEach(inp=>inp.onchange=e=>{const idx=Number(e.target.dataset.sectionIdx);layout.draftSections[idx].title=e.target.value.trim()||'Section';save()});
 box.querySelectorAll('[data-remove-section]').forEach(b=>b.onclick=()=>{const idx=Number(b.dataset.removeSection);if(layout.draftSections.length<=1)return;layout.draftSections.splice(idx,1);save();renderLayoutSections(entityKey)});
 wireLayoutDragDrop(entityKey);
}
function wireLayoutDragDrop(entityKey){
 const layout=data.uiLayouts[entityKey];
 let dragKey=null,dragFromIdx=null;
 function moveField(toIdx,beforeKey){
  if(dragKey===null)return;
  if(dragFromIdx>=0)layout.draftSections[dragFromIdx].fields=layout.draftSections[dragFromIdx].fields.filter(k=>k!==dragKey);
  if(toIdx>=0){
   const s=layout.draftSections[toIdx];
   s.fields=s.fields.filter(k=>k!==dragKey);
   const insertAt=beforeKey?s.fields.indexOf(beforeKey):-1;
   s.fields.splice(insertAt<0?s.fields.length:insertAt,0,dragKey);
  }
  save();dragKey=null;dragFromIdx=null;
  renderLayoutSections(entityKey);
 }
 document.querySelectorAll('.layout-field-chip').forEach(el=>{
  el.ondragstart=e=>{dragKey=el.dataset.fieldKey;dragFromIdx=Number(el.dataset.sectionIdx);e.dataTransfer.effectAllowed='move'};
  el.ondragover=e=>{e.preventDefault();e.stopPropagation()};
  el.ondrop=e=>{e.preventDefault();e.stopPropagation();moveField(Number(el.dataset.sectionIdx),el.dataset.fieldKey)};
 });
 document.querySelectorAll('.layout-field-list').forEach(list=>{
  list.ondragover=e=>{e.preventDefault();e.dataTransfer.dropEffect='move'};
  list.ondrop=e=>{e.preventDefault();moveField(Number(list.dataset.sectionIdx),null)};
 });
}
function layoutPreviewModal(entityKey){
 const fields=allFieldsFor(entityKey);
 const layout=data.uiLayouts[entityKey];
 const byKey=Object.fromEntries(fields.map(f=>[f[0],f]));
 const used=new Set();
 const groups=layout.draftSections.map(s=>{const fs=s.fields.map(k=>byKey[k]).filter(Boolean);fs.forEach(f=>used.add(f[0]));return {title:s.title,fields:fs}}).filter(g=>g.fields.length);
 const rest=fields.filter(f=>!used.has(f[0]));
 if(rest.length)groups.push({title:'Other fields (not placed in a section)',fields:rest});
 const sample={};
 const body=`<div class="form-grid">${groups.map(g=>`<div class="field full"><h4 style="margin:14px 0 0">${g.title}</h4></div>${g.fields.map(f=>fieldHtml(f,sample)).join('')}`).join('')}</div><p class="muted" style="font-size:12px;margin-top:12px">Preview of the draft layout only — nothing here is saved, and the live form is unaffected until you Publish.</p><div class="modal-actions"><button class="btn btn-secondary" type="button" data-close>Close</button></div>`;
 modal(`Preview: ${entityLabel(entityKey)} form`,body);
 $('[data-close]').onclick=closeModal;
}
// ---- Integrations (Phase 5: UI-only simulation) ---------------------------
// New to the demo (and not on desktop either): scheduled data jobs, exposed
// API endpoints, and configured external API connections. The static demo
// has no server, so nothing here makes a real network call or runs on a
// real schedule - every "run"/"test" is a local simulation against this
// browser's own demo data, saved to localStorage like everything else,
// matching the honesty level the rest of this demo already holds to.
let integrationsSubTab='jobs';
const JOB_TYPES=['export','import','sync'];
const JOB_TYPE_LABELS={export:'Export data out',import:'Import data in',sync:'Two-way sync'};
const SCHEDULE_OPTIONS=['manual','hourly','daily','weekly'];
const SCHEDULE_LABELS={manual:'Manual only',hourly:'Every hour',daily:'Once a day',weekly:'Once a week'};
const FORMAT_OPTIONS=['csv','json'];
const EXTERNAL_AUTH_TYPES=['none','apiKey','bearer'];
const EXTERNAL_AUTH_LABELS={none:'None',apiKey:'API key',bearer:'Bearer token'};
function integrationsTab(body){
 const subTabs=[['jobs','Scheduled jobs'],['endpoints','API endpoints'],['external','Consume external APIs']];
 body.innerHTML=`<div class="panel">
 <h3 style="margin-top:0">Integrations</h3>
 <p class="muted" style="font-size:13px">Schedule data import/export jobs, expose API endpoints for other systems to call, and configure external APIs this workspace would consume. This is a UI-only simulation: everything is saved to this browser and "runs"/"test calls" produce a realistic result against your demo data, but no real network request or scheduled job ever actually fires — there's no server behind the online demo to run one.</p>
 <div class="tabs">${subTabs.map(t=>`<button class="tab ${integrationsSubTab===t[0]?'active':''}" data-integrations-tab="${t[0]}">${t[1]}</button>`).join('')}</div>
 <div id="integrationsBody"></div>
 </div>`;
 document.querySelectorAll('[data-integrations-tab]').forEach(b=>b.onclick=()=>{integrationsSubTab=b.dataset.integrationsTab;renderIntegrationsSubTab()});
 renderIntegrationsSubTab();
}
function renderIntegrationsSubTab(){
 document.querySelectorAll('[data-integrations-tab]').forEach(b=>b.classList.toggle('active',b.dataset.integrationsTab===integrationsSubTab));
 const body=$('#integrationsBody');
 ({jobs:jobsSubTab,endpoints:endpointsSubTab,external:externalSubTab}[integrationsSubTab])(body);
}
function jobsSubTab(body){
 const list=data.integrationJobs||[];
 body.innerHTML=`<div class="panel-head" style="margin-top:16px"><h3 style="margin:0;font-size:16px">Scheduled jobs</h3><button class="btn btn-primary" id="addJob">+ New job</button></div>${list.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Type</th><th>Entity</th><th>Schedule</th><th>Format</th><th>Status</th><th>Last run</th><th>Actions</th></tr></thead><tbody>${list.map(j=>`<tr><td>${j.name}</td><td>${JOB_TYPE_LABELS[j.type]}</td><td>${entityLabel(j.entityKey)}</td><td>${SCHEDULE_LABELS[j.schedule]}</td><td>${j.format.toUpperCase()}</td><td>${badgeMaybe(j.active?'Active':'Inactive')}</td><td>${j.lastRun?new Date(j.lastRun).toLocaleString():'Never run'}</td><td><div class="actions"><button class="icon-btn" data-run-job="${j.id}">Run now</button><button class="icon-btn" data-history-job="${j.id}">History</button><button class="icon-btn" data-edit-job="${j.id}">Edit</button><button class="icon-btn" data-del-job="${j.id}">Delete</button></div></td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No scheduled jobs yet.</div>'}`;
 $('#addJob').onclick=()=>jobModal();
 body.querySelectorAll('[data-run-job]').forEach(b=>b.onclick=()=>runIntegrationJob(b.dataset.runJob));
 body.querySelectorAll('[data-history-job]').forEach(b=>b.onclick=()=>jobHistoryModal(b.dataset.historyJob));
 body.querySelectorAll('[data-edit-job]').forEach(b=>b.onclick=()=>jobModal(list.find(j=>j.id===b.dataset.editJob)));
 body.querySelectorAll('[data-del-job]').forEach(b=>b.onclick=()=>{if(!confirm('Delete this job? Its run history goes with it - this does not affect any real data.'))return;data.integrationJobs=data.integrationJobs.filter(j=>j.id!==b.dataset.delJob);save();jobsSubTab(body)});
}
function jobModal(job){
 const isEdit=!!job;
 const keys=allEntityTypeKeys();
 const defaultKey=job?.entityKey||keys[0];
 const body=`<form id="jobForm" class="form-grid">
 <div class="field full"><label>Job name</label><input name="name" value="${job?.name||''}" required></div>
 <div class="field"><label>Type</label><select name="type">${JOB_TYPES.map(t=>`<option value="${t}" ${(job?.type||'export')===t?'selected':''}>${JOB_TYPE_LABELS[t]}</option>`).join('')}</select></div>
 <div class="field"><label>Entity</label><select name="entityKey">${keys.map(k=>`<option value="${k}" ${defaultKey===k?'selected':''}>${entityLabel(k)}</option>`).join('')}</select></div>
 <div class="field"><label>Schedule</label><select name="schedule">${SCHEDULE_OPTIONS.map(s=>`<option value="${s}" ${(job?.schedule||'manual')===s?'selected':''}>${SCHEDULE_LABELS[s]}</option>`).join('')}</select></div>
 <div class="field"><label>Format</label><select name="format">${FORMAT_OPTIONS.map(f=>`<option value="${f}" ${(job?.format||'csv')===f?'selected':''}>${f.toUpperCase()}</option>`).join('')}</select></div>
 ${isEdit?`<div class="field"><label>Status</label><select name="active"><option value="true" ${job.active?'selected':''}>Active</option><option value="false" ${!job.active?'selected':''}>Inactive</option></select></div>`:''}
 <div class="modal-actions">${isEdit?'<button type="button" class="btn btn-secondary" data-delete-job>Delete</button>':''}<button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">${isEdit?'Save job':'Create job'}</button></div>
 </form>`;
 modal(isEdit?`Edit job: ${job.name}`:'New scheduled job',body);
 $('[data-close]').onclick=closeModal;
 $('#jobForm').onsubmit=e=>{
  e.preventDefault();
  const fd=Object.fromEntries(new FormData(e.target).entries());
  if(isEdit){Object.assign(job,{name:fd.name,type:fd.type,entityKey:fd.entityKey,schedule:fd.schedule,format:fd.format,active:fd.active==='true'})}
  else{data.integrationJobs.push({id:uid(),name:fd.name,type:fd.type,entityKey:fd.entityKey,schedule:fd.schedule,format:fd.format,active:true,lastRun:null,runs:[]})}
  save();closeModal();toast(isEdit?'Job saved':'Job created');renderAdminTab();
 };
 if(isEdit){$('[data-delete-job]').onclick=()=>{if(!confirm('Delete this job? This does not affect any real data - only the job definition is removed.'))return;data.integrationJobs=data.integrationJobs.filter(j=>j.id!==job.id);save();closeModal();toast('Job deleted');renderAdminTab()}}
}
// Simulated run: for export/sync, the "record count" is the entity's real
// current count in this demo's data (so it feels grounded, not random); for
// import there's nothing to count yet, so a plausible small batch size
// stands in for it. Always simulates success - there's no real failure mode
// to reproduce honestly here, so this doesn't invent one.
function runIntegrationJob(id){
 const job=(data.integrationJobs||[]).find(j=>j.id===id); if(!job)return;
 const count=job.type==='import'?Math.floor(Math.random()*30)+3:(data[job.entityKey]||[]).length;
 const startedAt=new Date().toISOString();
 const verb=job.type==='import'?'would be imported':job.type==='export'?'exported':'synced';
 const run={id:uid(),startedAt,status:'success',recordCount:count,message:`${JOB_TYPE_LABELS[job.type]} completed — ${count} ${entityLabel(job.entityKey).toLowerCase()} record${count===1?'':'s'} ${verb} as ${job.format.toUpperCase()}. Simulated — no real file or network call was made.`};
 job.runs.unshift(run); if(job.runs.length>10)job.runs.length=10;
 job.lastRun=startedAt;
 save();toast('Job run simulated');renderAdminTab();
}
function jobHistoryModal(id){
 const job=(data.integrationJobs||[]).find(j=>j.id===id); if(!job)return;
 const body=`${job.runs.length?`<div class="table-wrap"><table class="table"><thead><tr><th>When</th><th>Status</th><th>Records</th><th>Details</th></tr></thead><tbody>${job.runs.map(r=>`<tr><td>${new Date(r.startedAt).toLocaleString()}</td><td>${badgeMaybe('Completed')}</td><td>${r.recordCount}</td><td style="font-size:12px" class="muted">${r.message}</td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No runs yet — click Run now to simulate one.</div>'}<div class="modal-actions"><button class="btn btn-secondary" type="button" data-close>Close</button></div>`;
 modal(`Run history: ${job.name}`,body);
 $('[data-close]').onclick=closeModal;
}
function endpointsSubTab(body){
 const list=data.apiEndpoints||[];
 body.innerHTML=`<div class="panel-head" style="margin-top:16px"><h3 style="margin:0;font-size:16px">API endpoints</h3><button class="btn btn-primary" id="addEndpoint">+ New endpoint</button></div><p class="muted" style="font-size:13px">Expose a read/write endpoint backed by a built-in or custom object. Test call simulates the request/response locally against your demo data — nothing actually leaves your browser.</p>${list.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Method</th><th>Path</th><th>Entity</th><th>Auth</th><th>Status</th><th>Actions</th></tr></thead><tbody>${list.map(e=>`<tr><td>${e.name}</td><td><code>${e.method}</code></td><td><code>${e.path}</code></td><td>${entityLabel(e.entityKey)}</td><td>${e.authType==='apiKey'?'API key':'None'}</td><td>${badgeMaybe(e.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-test-endpoint="${e.id}">Test call</button><button class="icon-btn" data-edit-endpoint="${e.id}">Edit</button><button class="icon-btn" data-del-endpoint="${e.id}">Delete</button></div></td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No API endpoints yet.</div>'}`;
 $('#addEndpoint').onclick=()=>endpointModal();
 body.querySelectorAll('[data-test-endpoint]').forEach(b=>b.onclick=()=>testEndpointModal(list.find(e=>e.id===b.dataset.testEndpoint)));
 body.querySelectorAll('[data-edit-endpoint]').forEach(b=>b.onclick=()=>endpointModal(list.find(e=>e.id===b.dataset.editEndpoint)));
 body.querySelectorAll('[data-del-endpoint]').forEach(b=>b.onclick=()=>{if(!confirm('Delete this endpoint?'))return;data.apiEndpoints=data.apiEndpoints.filter(x=>x.id!==b.dataset.delEndpoint);save();endpointsSubTab(body)});
}
function endpointModal(endpoint){
 const isEdit=!!endpoint;
 const keys=allEntityTypeKeys();
 const defaultKey=endpoint?.entityKey||keys[0];
 const body=`<form id="endpointForm" class="form-grid">
 <div class="field full"><label>Endpoint name</label><input name="name" value="${endpoint?.name||''}" required></div>
 <div class="field"><label>Method</label><select name="method"><option value="GET" ${(endpoint?.method||'GET')==='GET'?'selected':''}>GET (read records)</option><option value="POST" ${endpoint?.method==='POST'?'selected':''}>POST (create a record)</option></select></div>
 <div class="field"><label>Entity</label><select name="entityKey" id="endpointEntitySelect">${keys.map(k=>`<option value="${k}" ${defaultKey===k?'selected':''}>${entityLabel(k)}</option>`).join('')}</select></div>
 <div class="field full"><label>Path</label><input name="path" id="endpointPathInput" value="${endpoint?.path||'/api/v1/'+defaultKey}" required></div>
 <div class="field"><label>Auth</label><select name="authType"><option value="apiKey" ${(endpoint?.authType||'apiKey')==='apiKey'?'selected':''}>API key</option><option value="none" ${endpoint?.authType==='none'?'selected':''}>None (public)</option></select></div>
 ${isEdit?`<div class="field"><label>Status</label><select name="active"><option value="true" ${endpoint.active?'selected':''}>Active</option><option value="false" ${!endpoint.active?'selected':''}>Inactive</option></select></div>`:''}
 <div class="modal-actions">${isEdit?'<button type="button" class="btn btn-secondary" data-delete-endpoint>Delete</button>':''}<button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">${isEdit?'Save endpoint':'Create endpoint'}</button></div>
 </form>`;
 modal(isEdit?`Edit endpoint: ${endpoint.name}`:'New API endpoint',body);
 $('[data-close]').onclick=closeModal;
 if(!isEdit)$('#endpointEntitySelect').onchange=e=>{$('#endpointPathInput').value='/api/v1/'+e.target.value};
 $('#endpointForm').onsubmit=e=>{
  e.preventDefault();
  const fd=Object.fromEntries(new FormData(e.target).entries());
  if(!fd.path.trim().startsWith('/'))return alert('Path must start with /, e.g. /api/v1/companies');
  if(isEdit){Object.assign(endpoint,{name:fd.name,method:fd.method,entityKey:fd.entityKey,path:fd.path,authType:fd.authType,active:fd.active==='true'})}
  else{data.apiEndpoints.push({id:uid(),name:fd.name,method:fd.method,entityKey:fd.entityKey,path:fd.path,authType:fd.authType,apiKey:'demo_'+uid()+uid(),active:true})}
  save();closeModal();toast(isEdit?'Endpoint saved':'Endpoint created');renderAdminTab();
 };
 if(isEdit){$('[data-delete-endpoint]').onclick=()=>{if(!confirm('Delete this endpoint?'))return;data.apiEndpoints=data.apiEndpoints.filter(x=>x.id!==endpoint.id);save();closeModal();toast('Endpoint deleted');renderAdminTab()}}
}
function testEndpointModal(endpoint){
 if(!endpoint)return;
 const records=(data[endpoint.entityKey]||[]).slice(0,3);
 const respBody=endpoint.method==='GET'?{data:records,count:(data[endpoint.entityKey]||[]).length}:{created:{...(records[0]||{}),id:'sim_'+uid()},note:'Simulated — no record was actually created.'};
 const headerLines=[`${endpoint.method} ${endpoint.path} HTTP/1.1`,'Host: demo.lanesraos.com',endpoint.authType==='apiKey'?`Authorization: Bearer ${endpoint.apiKey}`:null].filter(Boolean).join('\n');
 const body=`<p class="muted" style="font-size:13px">Simulated locally — this is what calling this endpoint would return against your current demo data. No real HTTP request was made.</p>
 <div><strong>Request</strong><pre style="background:#0f172a;color:#e2e8f0;border-radius:10px;padding:12px;overflow:auto;font-size:12px">${headerLines}</pre></div>
 <div style="margin-top:12px"><strong>Response</strong> <span class="badge">200 OK</span><pre style="background:#0f172a;color:#e2e8f0;border-radius:10px;padding:12px;overflow:auto;font-size:12px;max-height:280px">${JSON.stringify(respBody,null,2)}</pre></div>
 <div class="modal-actions"><button class="btn btn-secondary" type="button" data-close>Close</button></div>`;
 modal(`Test call: ${endpoint.name}`,body);
 $('[data-close]').onclick=closeModal;
}
function externalSubTab(body){
 const list=data.externalConnections||[];
 body.innerHTML=`<div class="panel-head" style="margin-top:16px"><h3 style="margin:0;font-size:16px">Consume external APIs</h3><button class="btn btn-primary" id="addConnection">+ New connection</button></div><p class="muted" style="font-size:13px">Configure an external API this workspace would call. Test request simulates a response shape locally — the online demo can't make outbound network calls, so nothing is actually sent.</p>${list.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Method</th><th>Base URL</th><th>Auth</th><th>Status</th><th>Actions</th></tr></thead><tbody>${list.map(c=>`<tr><td>${c.name}</td><td><code>${c.method}</code></td><td style="max-width:260px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap"><code>${c.baseUrl}</code></td><td>${EXTERNAL_AUTH_LABELS[c.authType]}</td><td>${badgeMaybe(c.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-test-conn="${c.id}">Test request</button><button class="icon-btn" data-history-conn="${c.id}">History</button><button class="icon-btn" data-edit-conn="${c.id}">Edit</button><button class="icon-btn" data-del-conn="${c.id}">Delete</button></div></td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No external connections yet.</div>'}`;
 $('#addConnection').onclick=()=>connectionModal();
 body.querySelectorAll('[data-test-conn]').forEach(b=>b.onclick=()=>testConnection(b.dataset.testConn));
 body.querySelectorAll('[data-history-conn]').forEach(b=>b.onclick=()=>connectionHistoryModal(b.dataset.historyConn));
 body.querySelectorAll('[data-edit-conn]').forEach(b=>b.onclick=()=>connectionModal(list.find(c=>c.id===b.dataset.editConn)));
 body.querySelectorAll('[data-del-conn]').forEach(b=>b.onclick=()=>{if(!confirm('Delete this connection?'))return;data.externalConnections=data.externalConnections.filter(x=>x.id!==b.dataset.delConn);save();externalSubTab(body)});
}
function connectionModal(conn){
 const isEdit=!!conn;
 const authType=conn?.authType||'none';
 const body=`<form id="connectionForm" class="form-grid">
 <div class="field full"><label>Connection name</label><input name="name" value="${conn?.name||''}" required></div>
 <div class="field full"><label>Base URL</label><input name="baseUrl" type="url" value="${conn?.baseUrl||''}" placeholder="https://api.example.com/v1/orders" required></div>
 <div class="field"><label>Method</label><select name="method"><option value="GET" ${(conn?.method||'GET')==='GET'?'selected':''}>GET</option><option value="POST" ${conn?.method==='POST'?'selected':''}>POST</option></select></div>
 <div class="field"><label>Auth</label><select name="authType" id="connAuthSelect">${EXTERNAL_AUTH_TYPES.map(a=>`<option value="${a}" ${authType===a?'selected':''}>${EXTERNAL_AUTH_LABELS[a]}</option>`).join('')}</select></div>
 <div class="field full" id="connAuthValueWrap" ${authType==='none'?'hidden':''}><label id="connAuthValueLabel">${authType==='bearer'?'Bearer token':'API key'}</label><input name="authValue" value="${conn?.authValue||''}" placeholder="Stored in this browser only"></div>
 ${isEdit?`<div class="field"><label>Status</label><select name="active"><option value="true" ${conn.active?'selected':''}>Active</option><option value="false" ${!conn.active?'selected':''}>Inactive</option></select></div>`:''}
 <div class="modal-actions">${isEdit?'<button type="button" class="btn btn-secondary" data-delete-conn>Delete</button>':''}<button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">${isEdit?'Save connection':'Create connection'}</button></div>
 </form>`;
 modal(isEdit?`Edit connection: ${conn.name}`:'New external connection',body);
 $('[data-close]').onclick=closeModal;
 $('#connAuthSelect').onchange=e=>{const wrap=$('#connAuthValueWrap');wrap.hidden=e.target.value==='none';$('#connAuthValueLabel').textContent=e.target.value==='bearer'?'Bearer token':'API key'};
 $('#connectionForm').onsubmit=e=>{
  e.preventDefault();
  const fd=Object.fromEntries(new FormData(e.target).entries());
  if(isEdit){Object.assign(conn,{name:fd.name,baseUrl:fd.baseUrl,method:fd.method,authType:fd.authType,authValue:fd.authValue||'',active:fd.active==='true'})}
  else{data.externalConnections.push({id:uid(),name:fd.name,baseUrl:fd.baseUrl,method:fd.method,authType:fd.authType,authValue:fd.authValue||'',active:true,calls:[]})}
  save();closeModal();toast(isEdit?'Connection saved':'Connection created');renderAdminTab();
 };
 if(isEdit){$('[data-delete-conn]').onclick=()=>{if(!confirm('Delete this connection?'))return;data.externalConnections=data.externalConnections.filter(x=>x.id!==conn.id);save();closeModal();toast('Connection deleted');renderAdminTab()}}
}
function testConnection(id){
 const conn=(data.externalConnections||[]).find(c=>c.id===id); if(!conn)return;
 const calledAt=new Date().toISOString();
 const respPreview={simulated:true,status:200,note:'This is a simulated response — the online demo has no backend to make real outbound HTTP requests. Configuring a real integration works the same way; this preview just confirms the request shape.',request:{method:conn.method,url:conn.baseUrl,auth:conn.authType==='none'?'none':(conn.authType==='bearer'?'Bearer ***':'API key ***')}};
 conn.calls.unshift({id:uid(),calledAt,status:'success',responsePreview:respPreview});
 if(conn.calls.length>10)conn.calls.length=10;
 save();
 const authLine=conn.authType!=='none'?`\n${conn.authType==='bearer'?'Authorization: Bearer ***':'X-Api-Key: ***'}`:'';
 const body=`<div><strong>Request</strong><pre style="background:#0f172a;color:#e2e8f0;border-radius:10px;padding:12px;overflow:auto;font-size:12px">${conn.method} ${conn.baseUrl}${authLine}</pre></div><div style="margin-top:12px"><strong>Simulated response</strong> <span class="badge">200 OK</span><pre style="background:#0f172a;color:#e2e8f0;border-radius:10px;padding:12px;overflow:auto;font-size:12px;max-height:280px">${JSON.stringify(respPreview,null,2)}</pre></div><div class="modal-actions"><button class="btn btn-secondary" type="button" data-close>Close</button></div>`;
 modal(`Test request: ${conn.name}`,body);
 $('[data-close]').onclick=closeModal;
}
function connectionHistoryModal(id){
 const conn=(data.externalConnections||[]).find(c=>c.id===id); if(!conn)return;
 const body=`${conn.calls.length?`<div class="table-wrap"><table class="table"><thead><tr><th>When</th><th>Status</th><th>Details</th></tr></thead><tbody>${conn.calls.map(c=>`<tr><td>${new Date(c.calledAt).toLocaleString()}</td><td>${badgeMaybe('Completed')}</td><td style="font-size:12px" class="muted">${c.responsePreview.note}</td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No test requests yet — click Test request to simulate one.</div>'}<div class="modal-actions"><button class="btn btn-secondary" type="button" data-close>Close</button></div>`;
 modal(`Request history: ${conn.name}`,body);
 $('[data-close]').onclick=closeModal;
}
// A field's Phase 4 extras, summarized as small comma-separated notes for
// the list table - empty when none of default/unique/placeholder/help
// text are set, so a plain field still reads as just "—".
function fieldExtrasSummary(f){
 const notes=[];
 if(f.required)notes.push('Required');
 if(f.unique)notes.push('Unique');
 if(f.maxLength)notes.push(`Max length: ${f.maxLength}`);
 if(f.pattern)notes.push('Pattern set');
 if(f.minValue!==''&&f.minValue!==undefined&&f.minValue!==null)notes.push(`Min: ${f.minValue}`);
 if(f.maxValue!==''&&f.maxValue!==undefined&&f.maxValue!==null)notes.push(`Max: ${f.maxValue}`);
 if(f.searchable)notes.push('Searchable');
 if(f.filterable)notes.push('Filterable');
 if(f.reportable===false)notes.push('Not reportable');
 if(f.defaultValue)notes.push(`Default: ${f.defaultValue}`);
 if(f.placeholder)notes.push('Placeholder set');
 if(f.helpText)notes.push('Help text set');
 return notes.length?notes.join(', '):'—';
}
function fieldsTab(body){
 const keys=[...Object.keys(numberRules),...activeCustomObjectKeys()];
 const list=(data.customFields||[]).filter(f=>f.entity===cfEntity);
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Custom fields</h3><p class="muted">Add fields to any object. They appear automatically on that object's create/edit form, with an optional default value, uniqueness check, placeholder and help text.</p>
 ${entityPills(keys,cfEntity)}
 <div class="table-wrap"><table class="table"><thead><tr><th>Field</th><th>Type</th><th>Options</th><th>Settings</th><th>Active</th><th>Actions</th></tr></thead><tbody>${list.map(f=>`<tr><td>${f.label}</td><td>${f.type}</td><td>${f.type==='select'?f.options:'—'}</td><td class="muted">${fieldExtrasSummary(f)}</td><td>${badgeMaybe(f.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-edit-field="${f.id}">Edit</button><button class="icon-btn" data-del-field="${f.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No custom fields on '+entityLabel(cfEntity)+' yet</div>'}</div>
 <button class="btn btn-secondary" id="addField" style="margin-top:14px">+ New field</button>
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{cfEntity=b.dataset.entity;renderAdminTab()});
 $('#addField').onclick=()=>customFieldModal();
 body.querySelectorAll('[data-edit-field]').forEach(b=>b.onclick=()=>customFieldModal(data.customFields.find(f=>f.id===b.dataset.editField)));
 body.querySelectorAll('[data-del-field]').forEach(b=>b.onclick=()=>{if(confirm('Delete this custom field? Saved values remain on records but the field will no longer appear.')){data.customFields=data.customFields.filter(f=>f.id!==b.dataset.delField);save();toast('Field deleted');renderAdminTab()}});
}
function customFieldModal(field){
 const isEdit=!!field;
 const f=field||{entity:cfEntity,label:'',type:'text',options:'',active:true,defaultValue:'',unique:false,helpText:'',placeholder:'',required:false,maxLength:'',pattern:'',minValue:'',maxValue:'',searchable:false,filterable:false,reportable:true,hiddenByDefault:false};
 const isText=f.type==='text', isNum=f.type==='number';
 const body=`<form id="cfForm" class="form-grid">
 <div class="field"><label>Field label</label><input name="label" value="${f.label}" required></div>
 <div class="field"><label>Type</label><select name="type">${['text','number','date','boolean','select'].map(t=>`<option value="${t}" ${f.type===t?'selected':''}>${t}</option>`).join('')}</select></div>
 <div class="field full" id="cfOptionsWrap" ${f.type==='select'?'':'style="display:none"'}><label>Options (separate with |)</label><input name="options" value="${f.options||''}" placeholder="Referral|Website|Event"></div>
 <div class="field"><label>Default value (optional)</label><input name="defaultValue" value="${f.defaultValue||''}" placeholder="Applied when a save leaves this empty"></div>
 <div class="field"><label>Placeholder text (optional)</label><input name="placeholder" value="${f.placeholder||''}"></div>
 <div class="field full"><label>Help text (optional)</label><input name="helpText" value="${f.helpText||''}" placeholder="Shown under the field on the record form"></div>
 <div class="field"><label>Active</label><select name="active"><option value="true" ${f.active?'selected':''}>Active</option><option value="false" ${!f.active?'selected':''}>Inactive</option></select></div>
 <div class="field" id="cfMaxLenWrap" ${isText?'':'style="display:none"'}><label>Max length (optional)</label><input name="maxLength" type="number" min="1" value="${f.maxLength||''}" placeholder="e.g. 255"></div>
 <div class="field" id="cfPatternWrap" ${isText?'':'style="display:none"'}><label>Pattern / regex (optional)</label><input name="pattern" value="${f.pattern||''}" placeholder="e.g. ^[A-Z]{2}\\d{4}$"></div>
 <div class="field" id="cfMinWrap" ${isNum?'':'style="display:none"'}><label>Min value (optional)</label><input name="minValue" type="number" value="${f.minValue??''}"></div>
 <div class="field" id="cfMaxWrap" ${isNum?'':'style="display:none"'}><label>Max value (optional)</label><input name="maxValue" type="number" value="${f.maxValue??''}"></div>
 <div class="field full">
  <label class="checkbox-row" style="padding:0"><input type="checkbox" name="unique" value="true" id="cfUnique" ${f.unique?'checked':''} ${f.type==='boolean'?'disabled':''}> Require a unique value</label>
  <label class="checkbox-row" style="padding:0"><input type="checkbox" name="required" value="true" id="cfRequired" ${f.required?'checked':''}> Required</label>
  <label class="checkbox-row" style="padding:0"><input type="checkbox" name="searchable" value="true" ${f.searchable?'checked':''}> Searchable</label>
  <label class="checkbox-row" style="padding:0"><input type="checkbox" name="filterable" value="true" ${f.filterable?'checked':''}> Filterable</label>
  <label class="checkbox-row" style="padding:0"><input type="checkbox" name="reportable" value="true" ${f.reportable!==false?'checked':''}> Reportable</label>
  <label class="checkbox-row" style="padding:0" title="Left off every create/edit form unless a business rule's Show action targets it"><input type="checkbox" name="hiddenByDefault" value="true" ${f.hiddenByDefault?'checked':''}> Hide by default</label>
 </div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">${isEdit?'Save field':'Add field'}</button></div>
 </form>`;
 modal(isEdit?'Edit custom field':`New custom field on ${entityLabel(cfEntity)}`,body);
 $('[data-close]').onclick=closeModal;
 const cfForm=$('#cfForm'), typeSelect=cfForm.elements.type, optWrap=$('#cfOptionsWrap'), uniqueBox=$('#cfUnique');
 const maxLenWrap=$('#cfMaxLenWrap'), patternWrap=$('#cfPatternWrap'), minWrap=$('#cfMinWrap'), maxWrap=$('#cfMaxWrap');
 // A boolean field only has two possible values, so "unique" can never
 // hold more than one record - reject it the same way the desktop
 // edition does at definition time, by disabling the checkbox. Max
 // length/pattern only make sense for text, and min/max only for number.
 typeSelect.onchange=()=>{
  const t=typeSelect.value;
  optWrap.style.display=t==='select'?'':'none';
  maxLenWrap.style.display=t==='text'?'':'none';
  patternWrap.style.display=t==='text'?'':'none';
  minWrap.style.display=t==='number'?'':'none';
  maxWrap.style.display=t==='number'?'':'none';
  const isBool=t==='boolean';uniqueBox.disabled=isBool;if(isBool)uniqueBox.checked=false;
 };
 cfForm.onsubmit=e=>{
  e.preventDefault();
  const fd=Object.fromEntries(new FormData(e.target).entries());
  if(fd.type==='select'&&!fd.options.trim())return alert('Add at least one option, separated by |.');
  if(fd.type==='boolean'&&fd.unique==='true')return alert('A yes/no field only has two possible values and cannot require a unique value.');
  const shared={label:fd.label,type:fd.type,options:fd.type==='select'?fd.options:'',active:fd.active==='true',defaultValue:fd.defaultValue||'',placeholder:fd.placeholder||'',helpText:fd.helpText||'',unique:fd.unique==='true',
   required:fd.required==='true',
   maxLength:fd.type==='text'&&fd.maxLength?Number(fd.maxLength):null,
   pattern:fd.type==='text'?(fd.pattern||''):'',
   minValue:fd.type==='number'&&fd.minValue!==''?Number(fd.minValue):'',
   maxValue:fd.type==='number'&&fd.maxValue!==''?Number(fd.maxValue):'',
   searchable:fd.searchable==='true',filterable:fd.filterable==='true',reportable:fd.reportable==='true',
   hiddenByDefault:fd.hiddenByDefault==='true',
  };
  if(isEdit){Object.assign(field,shared)}
  else{data.customFields.push({id:uid(),entity:cfEntity,key:slugify(fd.label),...shared})}
  save();closeModal();toast('Custom field saved');renderView();
 };
}
// Phase 3 test mode: fill in hypothetical values for an entity's real
// condition fields and see which active rules/workflows would match -
// nothing is read from or written to actual records. Shared by the
// Business rules and Workflow automation tabs since both dry-run the same
// way (mirrors business_rule_service::test_rules / workflow test_workflows).
function testPanelHtml(entityKey,title){
 const fields=conditionFieldsFor(entityKey);
 return `<div class="panel" style="margin-top:14px;background:#f9fafb"><h4 style="margin-top:0">${title}</h4><p class="muted">Fill in hypothetical values for a ${entityLabel(entityKey).replace(/s$/,'').toLowerCase()} and see what would happen — nothing is created, changed or sent.</p><form id="testForm" class="form-grid">${fields.map(f=>`<div class="field"><label>${f[1]}</label>${fieldValueHtml(f[0],f,'')}</div>`).join('')}<div class="field full"><button class="btn btn-primary" type="submit">Run test</button></div></form><div id="testResults" style="margin-top:14px"></div></div>`;
}
function wireRuleTestPanel(entityKey){
 const form=$('#testForm'); if(!form)return;
 form.onsubmit=e=>{e.preventDefault();const hyp=Object.fromEntries(new FormData(form).entries());
  const matches=(data.fieldRules||[]).filter(r=>r.entity===entityKey&&r.active).filter(r=>conditionsMatch(r.matchType||'all',r.conditions,hyp));
  $('#testResults').innerHTML=matches.length?`<strong>${matches.length} matching rule(s):</strong>${matches.map(r=>`<div class="deal" style="margin-top:8px">${(r.actions||[]).map(a=>describeRuleAction(entityKey,a)).join('; ')}</div>`).join('')}`:'<div class="empty">No active rule matches these values.</div>';
 };
}
function wireWorkflowTestPanel(entityKey){
 const form=$('#testForm'); if(!form)return;
 form.onsubmit=e=>{e.preventDefault();const hyp=Object.fromEntries(new FormData(form).entries());
  const matches=(data.workflowRules||[]).filter(r=>r.entity===entityKey&&r.active).filter(r=>r.conditions&&r.conditions.length&&conditionsMatch(r.matchType||'all',r.conditions,hyp));
  $('#testResults').innerHTML=matches.length?`<strong>${matches.length} matching workflow(s):</strong>${matches.map(r=>`<div class="deal" style="margin-top:8px">Would ${(r.actions||[]).map(a=>describeWorkflowAction(a,entityKey)).join('; ')}${r.notify?' and notify admins':''}</div>`).join('')}`:'<div class="empty">No active workflow matches these values.</div>';
 };
}
function rulesTab(body){
 if(ruleBuilderMode){renderRuleBuilder(body);return}
 const keys=[...Object.keys(numberRules),...activeCustomObjectKeys()];
 const actionFields=actionableFieldsFor(ruleEntity);
 const list=(data.fieldRules||[]).filter(r=>r.entity===ruleEntity);
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Business rules</h3><p class="muted">Build an IF (AND/OR conditions, with one level of OR-groups) / THEN (any number of actions) rule against any built-in or custom field.</p>
 ${entityPills(keys,ruleEntity)}
 ${actionFields.length?`<div class="table-wrap"><table class="table"><thead><tr><th>If</th><th>Then</th><th>Active</th><th>Actions</th></tr></thead><tbody>${list.map(r=>`<tr><td>${describeConditions(r.entity,r.conditions,r.matchType||'all')}</td><td>${(r.actions||[]).map(a=>describeRuleAction(r.entity,a)).join('; ')}</td><td>${badgeMaybe(r.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-edit-rule="${r.id}">Edit</button><button class="icon-btn" data-del-rule="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No business rules on '+entityLabel(ruleEntity)+' yet</div>'}</div><button class="btn btn-secondary" id="addRule" style="margin-top:14px">+ New rule</button>`:`<div class="empty">${entityLabel(ruleEntity)} has no field a rule can act on yet.</div>`}
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{ruleEntity=b.dataset.entity;renderAdminTab()});
 $('#addRule')?.addEventListener('click',()=>{ruleBuilderMode='create';renderAdminTab()});
 body.querySelectorAll('[data-edit-rule]').forEach(b=>b.onclick=()=>{ruleBuilderMode=b.dataset.editRule;renderAdminTab()});
 body.querySelectorAll('[data-del-rule]').forEach(b=>b.onclick=()=>{data.fieldRules=data.fieldRules.filter(r=>r.id!==b.dataset.delRule);save();toast('Rule deleted');renderAdminTab()});
}
// Rule-builder page (Phase-3-and-visual-redesign parity with the desktop
// edition's RuleForm, extended by the second Admin Automation &
// Customization addendum round): a numbered Conditions/Actions layout with
// a live summary panel - any number of conditions (AND/OR, plus one level
// of nested OR-groups via mountConditionsEditor) and any number of actions
// (the full action palette via mountActionsEditor), with Test rule and
// Activate/Deactivate in the header.
function renderRuleBuilder(body){
 const isEdit=ruleBuilderMode!=='create';
 const existing=isEdit?data.fieldRules.find(r=>r.id===ruleBuilderMode):null;
 if(isEdit&&!existing){ruleBuilderMode=null;renderAdminTab();return}
 const entityKey=existing?existing.entity:ruleEntity;
 const condFields=conditionFieldsFor(entityKey);
 const actionFields=actionableFieldsFor(entityKey);
 const initialConditions=existing?.conditions?.length?existing.conditions:[{fieldKey:transitionFieldFor(entityKey),operator:'equals',value:'',compareField:null,groupId:null}];
 const initialActions=existing?.actions?.length?existing.actions:[{type:'require',targetField:actionFields[0]?.[0]||'',value:'',message:''}];
 body.innerHTML=`<div class="builder-header">
  <div>
   <div class="builder-breadcrumb">Business Rules / ${isEdit?'Edit rule':'New rule'}</div>
   <div class="builder-title-row"><h2>${isEdit?'Edit business rule':'New business rule'}</h2>${isEdit?`<span class="badge" style="${existing.active?'background:#dcfce7;color:#166534':''}">${existing.active?'Active':'Inactive'}</span>`:''}</div>
   <p class="builder-subtitle">Applies to ${entityLabel(entityKey)}.</p>
  </div>
  <div class="builder-header-actions">
   <button class="btn btn-secondary" type="button" id="ruleBuilderTest">${testingRules?'Hide test':'Test rule'}</button>
   ${isEdit?`<button class="btn btn-secondary" type="button" id="ruleBuilderToggleActive">${existing.active?'Deactivate':'Activate'}</button>`:''}
   <button class="btn btn-primary" type="submit" form="ruleBuilderForm">${isEdit?'Save':'Add rule'}</button>
  </div>
 </div>
 ${testingRules?testPanelHtml(entityKey,'Test business rules'):''}
 <form id="ruleBuilderForm">
 <div class="builder-layout">
  <div>
   <div class="builder-section">
    <div class="builder-section-title"><span class="step-badge">1</span> Conditions</div>
    <div id="rbConditions"></div>
   </div>
   <div class="builder-section">
    <div class="builder-section-title"><span class="step-badge">2</span> Actions</div>
    <p class="muted" style="margin-top:-4px">Choose what should happen when the conditions above are met - one rule can have several actions.</p>
    <div id="rbActions"></div>
   </div>
  </div>
  <div class="builder-summary-panel">
   <h4>Rule summary</h4>
   <div class="summary-row"><span class="label">Applies to</span><span class="value">${entityLabel(entityKey)}</span></div>
   <div class="summary-row"><span class="label">Execute on</span><span class="value">Create and edit</span></div>
   <div class="summary-row"><span class="label">Conditions</span><span class="value" id="summaryConditions"></span></div>
   <div class="summary-row"><span class="label">Field dependency</span><span class="value" id="summaryFieldDep"></span></div>
   <div class="summary-row"><span class="label">Stop processing</span><span class="value" id="summaryStop"></span></div>
  </div>
 </div>
 <div style="margin-top:4px;display:flex;gap:8px">
  <button type="button" class="btn btn-secondary" id="ruleBuilderCancel">Cancel</button>
  ${isEdit?`<button type="button" class="btn btn-secondary" id="ruleBuilderToggleActive2">${existing.active?'Deactivate':'Activate'}</button>`:''}
  <button class="btn btn-primary" type="submit" form="ruleBuilderForm">${isEdit?'Save':'Add rule'}</button>
 </div>
 </form>`;
 const form=$('#ruleBuilderForm');
 const condEditor=mountConditionsEditor($('#rbConditions',form),condFields,initialConditions,existing?.matchType||'all');
 const actEditor=mountActionsEditor($('#rbActions',form),actionFields,initialActions,actionFields[0]?.[0]||'');
 // Live-updating summary panel, mirroring the desktop edition's - re-derived
 // from the editors' own state (not the form's raw FormData, since row
 // count changes dynamically) on every change/input inside the form.
 function updateSummary(){
  const conditions=condEditor.getConditions(), matchType=condEditor.getMatchType(), actions=actEditor.getActions();
  const targets=[...new Set(actions.map(a=>a.targetField).filter(Boolean))].map(f=>fieldLabelFor(entityKey,f));
  $('#summaryConditions').innerHTML=describeConditions(entityKey,conditions,matchType);
  $('#summaryFieldDep').textContent=targets.length?targets.join(', '):'—';
  $('#summaryStop').textContent=actions.some(a=>a.type==='block_save')?'Yes':'No';
 }
 form.addEventListener('change',updateSummary);
 form.addEventListener('input',updateSummary);
 // Adding/removing a condition or action row re-renders its container via
 // innerHTML - a structural change, not a form 'change'/'input' event - so
 // a MutationObserver is what actually catches it (this is exactly the
 // kind of edit that left the old summary panel stale).
 const summaryObserver=new MutationObserver(updateSummary);
 summaryObserver.observe($('#rbConditions',form),{childList:true,subtree:true});
 summaryObserver.observe($('#rbActions',form),{childList:true,subtree:true});
 updateSummary();
 $('#ruleBuilderTest').onclick=()=>{testingRules=!testingRules;renderAdminTab()};
 if(testingRules)wireRuleTestPanel(entityKey);
 const toggleActive=()=>{existing.active=!existing.active;save();toast(existing.active?'Rule activated':'Rule deactivated');renderAdminTab()};
 $('#ruleBuilderToggleActive')?.addEventListener('click',toggleActive);
 $('#ruleBuilderToggleActive2')?.addEventListener('click',toggleActive);
 $('#ruleBuilderCancel').onclick=()=>{ruleBuilderMode=null;testingRules=false;renderAdminTab()};
 form.onsubmit=e=>{e.preventDefault();
  const conditions=condEditor.getConditions(), matchType=condEditor.getMatchType(), actions=actEditor.getActions();
  if(!conditions.length)return alert('Add at least one condition.');
  if(!actions.length)return alert('Add at least one action.');
  const payload={entity:entityKey,matchType,conditions,actions};
  if(isEdit){Object.assign(existing,payload)}else{data.fieldRules.push({id:uid(),active:true,...payload})}
  save();toast(isEdit?'Business rule saved':'Business rule added');ruleBuilderMode=null;testingRules=false;renderView()};
}
// Renders the operator + value pair for whichever condition field is
// currently selected - re-invoked on every "Field" change so the operator
// choices and the value widget (select/number/date/text) always match the
// chosen field's real type, the same as the desktop edition's condition row.
// Renders Operator + (fixed value / another field) + the value or
// compare-field widget for whichever condition field is currently
// selected - re-invoked on every field/operator/compare-mode change so
// each piece always matches the current selection, the same live-updating
// pattern the desktop edition's ConditionRow uses.
function conditionDynamicHtml(condFields,fieldKey,operator,value,compareField){
 const field=condFields.find(f=>f[0]===fieldKey);
 const ops=operatorsForType(field?field[2]:'text');
 const op=ops.includes(operator)?operator:ops[0];
 const needsValue=operatorNeedsValue(op);
 // in_list/not_in_list always take a plain "A|B|C" text list, regardless
 // of the field's own type, and (like desktop) can't compare to another
 // field - a single field's value isn't itself a list to match against.
 const isListOp=op==='in_list'||op==='not_in_list';
 const comparesToField=needsValue&&!isListOp&&!!compareField;
 const modeHtml=(needsValue&&!isListOp)?`<div class="field"><label>Compare against</label><select name="compareMode">
  <option value="literal" ${!comparesToField?'selected':''}>a fixed value</option>
  <option value="field" ${comparesToField?'selected':''}>another field</option>
 </select></div>`:'';
 const valueHtml=isListOp
  ?`<input name="triggerValue" type="text" value="${value}" placeholder="Option A|Option B|Option C">`
  :comparesToField
   ?`<select name="compareField">${condFields.map(f=>`<option value="${f[0]}" ${f[0]===compareField?'selected':''}>${f[1]}</option>`).join('')}</select>`
   :fieldValueHtml('triggerValue',field,value);
 return `<div class="field"><label>Operator</label><select name="operator">${ops.map(o=>`<option value="${o}" ${o===op?'selected':''}>${OPERATOR_LABELS[o]}</option>`).join('')}</select></div>
 ${modeHtml}
 <div class="field" id="condValueWrap" style="${needsValue?'':'display:none'}"><label>Value</label>${valueHtml}</div>`;
}
// Wires a condition's Field select plus its dynamic operator/value block
// (event-delegated, since the block's own selects get replaced on every
// re-render) - shared by ruleModal and workflowModal. `valueFieldName` is
// "triggerValue" for a business rule, "toValue" for a workflow - the
// dynamic block always renders "triggerValue" internally and this swaps
// the attribute after the fact, same as the old .replace() shortcut did.
function wireConditionPicker(form,fieldSelect,dynamicWrap,condFields,valueFieldName){
 function render(fieldKey,operator,value,compareField){
  let html=conditionDynamicHtml(condFields,fieldKey,operator,value,compareField);
  if(valueFieldName!=='triggerValue')html=html.replaceAll('name="triggerValue"',`name="${valueFieldName}"`);
  dynamicWrap.innerHTML=html;
 }
 fieldSelect.onchange=()=>render(fieldSelect.value,'equals','',null);
 dynamicWrap.addEventListener('change',(e)=>{
  if(e.target.name==='operator'){
   render(fieldSelect.value,e.target.value,'',null);
  }else if(e.target.name==='compareMode'){
   const operator=dynamicWrap.querySelector('[name=operator]').value;
   render(fieldSelect.value,operator,'',e.target.value==='field'?(condFields[0]?.[0]??''):null);
  }
 });
}

// ---- Multi-condition (+ OR group) editor, shared by the Business Rules
// and Workflow Automation builders (second Admin Automation & Customization
// addendum round). Mounted into a container element already inside a
// <form>; reads/writes plain JS state, only touching the DOM to re-render
// its own container - the rest of the enclosing form is untouched. Field
// names are index-prefixed ("cond0_field", "cond1_field", ...) reusing
// conditionDynamicHtml/wireConditionPicker's single-row rendering, the
// same name-swap trick wireConditionPicker already uses for the workflow
// trigger's "toValue".
function indexedConditionHtml(condFields,c,idx){
 let dyn=conditionDynamicHtml(condFields,c.fieldKey,c.operator,c.value,c.compareField);
 dyn=dyn.replaceAll('name="operator"',`name="cond${idx}_operator"`)
        .replaceAll('name="compareMode"',`name="cond${idx}_compareMode"`)
        .replaceAll('name="triggerValue"',`name="cond${idx}_value"`)
        .replaceAll('name="compareField"',`name="cond${idx}_compareField"`);
 return `<div class="builder-row-card">
  <select name="cond${idx}_field" data-cond-field="${idx}">${condFields.map(f=>`<option value="${f[0]}" ${f[0]===c.fieldKey?'selected':''}>${f[1]}</option>`).join('')}</select>
  <span data-cond-dynamic="${idx}" style="display:contents">${dyn}</span>
  <button type="button" class="builder-row-remove" data-cond-remove="${idx}" title="Remove condition">✕</button>
 </div>`;
}
function wireIndexedConditionPicker(form,idx,condFields){
 const fieldSelect=form.elements[`cond${idx}_field`];
 const dynamicWrap=form.querySelector(`[data-cond-dynamic="${idx}"]`);
 if(!fieldSelect||!dynamicWrap)return;
 function render(fieldKey,operator,value,compareField){
  let html=conditionDynamicHtml(condFields,fieldKey,operator,value,compareField);
  html=html.replaceAll('name="operator"',`name="cond${idx}_operator"`)
           .replaceAll('name="compareMode"',`name="cond${idx}_compareMode"`)
           .replaceAll('name="triggerValue"',`name="cond${idx}_value"`)
           .replaceAll('name="compareField"',`name="cond${idx}_compareField"`);
  dynamicWrap.innerHTML=html;
  wireDynamic();
 }
 function wireDynamic(){
  dynamicWrap.querySelector(`[name="cond${idx}_operator"]`)?.addEventListener('change',e=>render(fieldSelect.value,e.target.value,'',null));
  dynamicWrap.querySelector(`[name="cond${idx}_compareMode"]`)?.addEventListener('change',e=>{
   const operator=dynamicWrap.querySelector(`[name="cond${idx}_operator"]`).value;
   render(fieldSelect.value,operator,'',e.target.value==='field'?(condFields[0]?.[0]??''):null);
  });
 }
 fieldSelect.onchange=()=>render(fieldSelect.value,'equals','',null);
 wireDynamic();
}
/** Mounts a self-contained conditions editor (add/remove condition, "+ OR
 * group") into `container`, which must already be inside a <form>. Returns
 * `{getConditions,getMatchType}` for the enclosing form's submit handler to
 * read live state - conditions/matchType aren't otherwise part of the
 * form's own FormData, since row count changes dynamically. */
function mountConditionsEditor(container,condFields,initialConditions,initialMatchType){
 let conditions=(initialConditions||[]).map(c=>({...c}));
 let matchType=initialMatchType||'all';
 function syncFromDom(){
  const form=container.closest('form');
  conditions=conditions.map((c,idx)=>{
   const fieldEl=form.elements[`cond${idx}_field`]; if(!fieldEl)return c;
   return {
    fieldKey:fieldEl.value,
    operator:form.elements[`cond${idx}_operator`]?.value||'equals',
    value:form.elements[`cond${idx}_value`]?.value||'',
    compareField:form.elements[`cond${idx}_compareField`]?.value||null,
    groupId:c.groupId||null,
   };
  });
  const mt=form.elements['condMatchType']; if(mt)matchType=mt.value;
 }
 function emptyCondition(groupId){return {fieldKey:condFields[0]?.[0]||'',operator:'equals',value:'',compareField:null,groupId:groupId||null}}
 function render(){
  const units=groupConditionUnits(conditions);
  const rowsHtml=units.map((u,ui)=>{
   const divider=ui>0?`<div class="builder-and-divider">${matchType==='all'?'AND':'OR'}</div>`:'';
   if(u.kind==='single')return divider+indexedConditionHtml(condFields,conditions[u.index],u.index);
   const rows=u.indices.map((idx,mi)=>(mi>0?'<div class="builder-and-divider">OR</div>':'')+indexedConditionHtml(condFields,conditions[idx],idx)).join('');
   return divider+`<div class="builder-or-group"><div class="builder-or-group-label">OR group - any one condition below satisfies this unit</div>${rows}<button type="button" class="btn" data-add-to-group="${u.groupId}" style="margin-bottom:8px">+ Add to OR group</button></div>`;
  }).join('');
  container.innerHTML=`<div class="field" style="max-width:260px;margin-bottom:10px"><label>Match</label><select name="condMatchType">${MATCH_TYPES.map(m=>`<option value="${m}" ${m===matchType?'selected':''}>${m==='all'?'All conditions (AND)':'Any condition (OR)'}</option>`).join('')}</select></div>
   ${conditions.length?rowsHtml:'<p class="muted">No conditions yet.</p>'}
   <div style="display:flex;gap:8px;align-items:center">
    <button type="button" class="btn" data-cond-add>+ Add condition</button>
    <button type="button" class="btn" data-cond-add-group title="Add two conditions that are OR'd together into one unit before Match applies">+ OR group</button>
   </div>`;
  wire();
 }
 function wire(){
  const form=container.closest('form');
  conditions.forEach((c,idx)=>wireIndexedConditionPicker(form,idx,condFields));
  form.elements['condMatchType'].onchange=()=>{syncFromDom();render()};
  container.querySelector('[data-cond-add]').onclick=()=>{syncFromDom();conditions.push(emptyCondition());render()};
  container.querySelector('[data-cond-add-group]').onclick=()=>{syncFromDom();const gid=newGroupId();conditions.push(emptyCondition(gid));conditions.push(emptyCondition(gid));render()};
  container.querySelectorAll('[data-cond-remove]').forEach(b=>b.onclick=()=>{
   syncFromDom();
   const idx=Number(b.dataset.condRemove), groupId=conditions[idx].groupId;
   conditions.splice(idx,1);
   if(groupId&&conditions.filter(c=>c.groupId===groupId).length===1)conditions.forEach(c=>{if(c.groupId===groupId)c.groupId=null});
   render();
  });
  container.querySelectorAll('[data-add-to-group]').forEach(b=>b.onclick=()=>{syncFromDom();conditions.push(emptyCondition(b.dataset.addToGroup));render()});
 }
 render();
 return {getConditions:()=>{syncFromDom();return conditions},getMatchType:()=>matchType};
}

// ---- Multi-action editor for business rules (second addendum round) -
// the full action palette from the design mockup, minus Trigger approval
// (deferred). Same index-prefixed-FormData-name pattern as the conditions
// editor above.
const RULE_ACTION_TYPES=['require','hide','show','lock','editable','set_default','set_value','clear_value','restrict_choices','block_save','show_error','show_warning'];
const RULE_ACTION_LABELS={require:'Require field',hide:'Hide field',show:'Show field',lock:'Make read-only',editable:'Make editable',set_default:'Set default value',set_value:'Set field value',clear_value:'Clear field value',restrict_choices:'Restrict choices',block_save:'Block save',show_error:'Show error',show_warning:'Show warning'};
const RULE_ACTION_ICONS={require:'✅',hide:'🙈',show:'👁️',lock:'🔒',editable:'🔓',set_default:'🔧',set_value:'✏️',clear_value:'🧹',restrict_choices:'🎯',block_save:'🚫',show_error:'❗',show_warning:'⚠️'};
const FIELD_TARGETED_RULE_ACTIONS=['require','hide','show','lock','editable','set_default','set_value','clear_value','restrict_choices'];
const VALUE_REQUIRED_RULE_ACTIONS=['set_default','set_value'];
const MESSAGE_RULE_ACTIONS=['block_save','show_error','show_warning'];
function describeRuleAction(entityKey,a){
 const target=a.targetField?fieldLabelFor(entityKey,a.targetField):'';
 switch(a.type){
  case 'require':return `require ${target}`;
  case 'hide':return `hide ${target}`;
  case 'show':return `show ${target}`;
  case 'lock':return `lock ${target}`;
  case 'editable':return `unlock ${target}`;
  case 'set_default':return `default ${target} to "${a.value||''}"`;
  case 'set_value':return `set ${target} to "${a.value||''}"`;
  case 'clear_value':return `clear ${target}`;
  case 'restrict_choices':return `restrict ${target} to ${(a.value||'').split(LIST_SEPARATOR).filter(Boolean).join(', ')||'no options'}`;
  case 'block_save':return `block save: "${a.message||''}"`;
  case 'show_error':return `show error: "${a.message||''}"`;
  case 'show_warning':return `show warning: "${a.message||''}"`;
  default:return a.type;
 }
}
function actionRowHtml(actionFields,a,idx){
 const isRestrict=a.type==='restrict_choices';
 const targetChoices=isRestrict?actionFields.filter(f=>f[2]==='select'):actionFields;
 const isFieldTargeted=FIELD_TARGETED_RULE_ACTIONS.includes(a.type);
 const needsValue=VALUE_REQUIRED_RULE_ACTIONS.includes(a.type);
 const isMessage=MESSAGE_RULE_ACTIONS.includes(a.type);
 const targetField=targetChoices.find(f=>f[0]===a.targetField)||targetChoices[0];
 let restrictHtml='';
 if(isRestrict){
  const opts=targetField&&targetField[3]?targetField[3].split('|').filter(Boolean):[];
  const chosen=(a.value||'').split(LIST_SEPARATOR).filter(Boolean);
  restrictHtml=opts.length?`<span style="display:flex;gap:10px;flex-wrap:wrap;align-items:center">${opts.map(o=>`<label style="display:flex;gap:4px;align-items:center;font-size:12px"><input type="checkbox" name="act${idx}_choice_${o}" ${chosen.includes(o)?'checked':''}> ${o}</label>`).join('')}</span>`:'<span class="muted" style="font-size:12px">Pick a select field first</span>';
 }
 return `<div class="builder-row-card">
  <span style="font-size:15px">${RULE_ACTION_ICONS[a.type]||''}</span>
  <select name="act${idx}_type" data-act-type="${idx}">${RULE_ACTION_TYPES.map(t=>`<option value="${t}" ${t===a.type?'selected':''}>${RULE_ACTION_LABELS[t]}</option>`).join('')}</select>
  ${isFieldTargeted?`<select name="act${idx}_target" data-act-target="${idx}">${targetChoices.map(f=>`<option value="${f[0]}" ${f[0]===targetField?.[0]?'selected':''}>${f[1]}</option>`).join('')}</select>`:''}
  ${needsValue?`<input name="act${idx}_value" value="${a.value||''}" placeholder="Value" style="width:140px">`:''}
  ${isRestrict?restrictHtml:''}
  ${isMessage?`<input name="act${idx}_message" value="${a.message||''}" placeholder="Message shown to the user" style="width:260px">`:''}
  <button type="button" class="builder-row-remove" data-act-remove="${idx}" title="Remove action">✕</button>
 </div>`;
}
function mountActionsEditor(container,actionFields,initialActions,defaultTargetKey){
 let actions=(initialActions&&initialActions.length?initialActions:[{type:'require',targetField:defaultTargetKey,value:'',message:''}]).map(a=>({...a}));
 function syncFromDom(){
  const form=container.closest('form');
  actions=actions.map((a,idx)=>{
   const typeEl=form.elements[`act${idx}_type`]; if(!typeEl)return a;
   const type=typeEl.value;
   const targetEl=form.elements[`act${idx}_target`];
   const targetKey=targetEl?.value??a.targetField;
   let value=a.value||'';
   if(type==='restrict_choices'){
    const tf=actionFields.find(f=>f[0]===targetKey);
    const opts=tf&&tf[3]?tf[3].split('|').filter(Boolean):[];
    value=opts.filter(o=>form.elements[`act${idx}_choice_${o}`]?.checked).join(LIST_SEPARATOR);
   }else{
    value=form.elements[`act${idx}_value`]?.value??'';
   }
   return {type,targetField:FIELD_TARGETED_RULE_ACTIONS.includes(type)?targetKey:null,value,message:form.elements[`act${idx}_message`]?.value??''};
  });
 }
 function render(){
  container.innerHTML=actions.map((a,idx)=>actionRowHtml(actionFields,a,idx)).join('')+`<button type="button" class="btn" data-act-add>+ Add action</button>`;
  wire();
 }
 function wire(){
  const form=container.closest('form');
  actions.forEach((a,idx)=>{
   form.elements[`act${idx}_type`].onchange=()=>{
    syncFromDom();
    const t=form.elements[`act${idx}_type`].value;
    actions[idx]={type:t,targetField:FIELD_TARGETED_RULE_ACTIONS.includes(t)?(actions[idx].targetField||defaultTargetKey):null,value:'',message:''};
    render();
   };
   form.elements[`act${idx}_target`]?.addEventListener('change',()=>{syncFromDom();render()});
  });
  container.querySelector('[data-act-add]').onclick=()=>{syncFromDom();actions.push({type:'require',targetField:defaultTargetKey,value:'',message:''});render()};
  container.querySelectorAll('[data-act-remove]').forEach(b=>b.onclick=()=>{syncFromDom();actions.splice(Number(b.dataset.actRemove),1);render()});
 }
 render();
 return {getActions:()=>{syncFromDom();return actions}};
}
// Phase 3 action expansion, extended by the second Admin Automation &
// Customization addendum round: a workflow's "Then" is one or more
// actions, each one of create a task, create a new record, update a
// field on a record related to the trigger via the demo's fixed
// foreign-key graph (REVERSE_RELATIONS), update a field on this record,
// set a default value on this record (only if currently empty), or clear
// a field on this record.
function describeWorkflowAction(a,entityKey){
 if(a.type==='create_record')return UNNAMED_RECORD_TYPES.includes(a.recordTargetEntity)?`create a new ${ENTITY_SINGULAR[a.recordTargetEntity]||a.recordTargetEntity}`:`create ${ENTITY_SINGULAR[a.recordTargetEntity]||a.recordTargetEntity} "${a.recordNameTemplate||''}"`;
 if(a.type==='update_related_record')return `set ${fieldLabelFor(a.relTargetEntity,a.relTargetField)} = "${a.relValue||''}" on related ${entityLabel(a.relTargetEntity)}`;
 if(a.type==='update_field'||a.type==='set_default_field'||a.type==='clear_field'){
  if(!a.updateFieldKey)return 'set a field on this record';
  if(a.type==='clear_field')return `clear ${fieldLabelFor(entityKey,a.updateFieldKey)}`;
  const prefix=a.type==='set_default_field'?'default ':'set ';
  const suffix=a.type==='set_default_field'?' (only if currently empty)':'';
  return a.updateCopyFrom?`${prefix}${fieldLabelFor(entityKey,a.updateFieldKey)} = value copied from ${fieldLabelFor(entityKey,a.updateCopyFrom)}${suffix}`:`${prefix}${fieldLabelFor(entityKey,a.updateFieldKey)} = "${a.updateValue||''}"${suffix}`;
 }
 return `create task "${a.taskTitle||''}" (${a.daysOffset?`due ${a.daysOffset} day(s) later`:'due same day'})`;
}
function relTargetsFor(entityKey){return [...new Set((RELATIONS[entityKey]||[]).map(x=>x.target))]}
// Executes one workflow action against the record that just triggered its
// rule. Returns a short human description of what happened, for the
// notification message - or null when the action is a legitimate no-op
// (e.g. update_related_record with nothing linked yet, or set_default_field
// on a field that already has a value - same "left unchanged" semantics
// the desktop edition's apply_action has).
function executeWorkflowAction(a,key,record){
 if(a.type==='create_record'){
  const target=a.recordTargetEntity, name=a.recordNameTemplate;
  if(!UNNAMED_RECORD_TYPES.includes(target)&&!name)return null;
  const companyId=key==='companies'?record.id:record.companyId;
  if(COMPANY_DEPENDENT_TYPES.includes(target)&&!companyId)return null;
  const today=new Date().toISOString().slice(0,10);
  if(target==='companies'){
   data.companies.unshift({id:uid(),customerNumber:nextNumber('companies'),name,industry:'',city:'',owner:record.owner||'Unassigned',status:'Lead'});
   return `created company "${name}"`;
  }
  if(target==='contacts'){
   data.contacts.unshift({id:uid(),contactNumber:nextNumber('contacts'),name,companyId,role:'',email:'',phone:'',status:'Active'});
   return `created contact "${name}"`;
  }
  if(target==='opportunities'){
   data.opportunities.unshift({id:uid(),opportunityNumber:nextNumber('opportunities'),title:name,companyId,contactId:'',value:0,stage:'Lead',probability:10,close:'',owner:record.owner||'Unassigned',status:'Open'});
   return `created opportunity "${name}"`;
  }
  if(target==='products'){
   data.products.unshift({id:uid(),productNumber:nextNumber('products'),name,sku:'',type:'Product',category:'',price:0,tax:0,status:'Active'});
   return `created product "${name}"`;
  }
  if(target==='quotes'){
   data.quotes.unshift({id:uid(),number:nextNumber('quotes'),companyId,contactId:'',opportunityId:'',status:'Draft',date:today,valid:''});
   return `created quote for ${companyName(companyId)}`;
  }
  if(target==='orders'){
   data.orders.unshift({id:uid(),number:nextNumber('orders'),companyId,contactId:'',quoteId:'',status:'Draft',date:today});
   return `created order for ${companyName(companyId)}`;
  }
  if(target==='invoices'){
   data.invoices.unshift({id:uid(),number:nextNumber('invoices'),companyId,orderId:'',status:'Draft',due:''});
   return `created invoice for ${companyName(companyId)}`;
  }
  if(target==='contracts'){
   data.contracts.unshift({id:uid(),number:nextNumber('contracts'),companyId,contactId:'',title:name,value:0,status:'Draft',start:'',end:''});
   return `created contract "${name}"`;
  }
  if(target==='tasks'){
   data.tasks.unshift({id:uid(),taskNumber:nextNumber('tasks'),title:name,relatedType:'General',relatedId:'',owner:record.owner||'Unassigned',due:today,priority:'Medium',status:'Open'});
   return `created task "${name}"`;
  }
  return null;
 }
 if(a.type==='update_related_record'){
  const rel=(RELATIONS[key]||[]).find(x=>x.target===a.relTargetEntity);
  if(!rel||!a.relTargetField)return null;
  const {target:targetEntity,fk,direction}=rel;
  let linked;
  if(direction==='down'){
   // Target rows carry a foreign key pointing at this record.
   linked=(data[targetEntity]||[]).filter(x=>x[fk]===record.id);
  }else if(direction==='up'){
   // This record's own foreign key points at a single parent row.
   const parent=byId(targetEntity,record[fk]);
   linked=parent?[parent]:[];
  }else if(direction==='taskBack'){
   // Tasks linked to this record via the polymorphic relatedType/relatedId pair.
   linked=data.tasks.filter(t=>t.relatedType===relatedTypeFor[key]&&t.relatedId===record.id);
  }else{ // 'taskLink': the single record this task itself points at
   if(record.relatedType!==relatedTypeFor[targetEntity])linked=[];
   else{const parent=byId(targetEntity,record.relatedId);linked=parent?[parent]:[]}
  }
  if(!linked.length)return null;
  linked.forEach(x=>{x[a.relTargetField]=a.relValue});
  return `set ${fieldLabelFor(targetEntity,a.relTargetField)} = "${a.relValue}" on ${linked.length} related ${entityLabel(targetEntity).toLowerCase()}`;
 }
 // update_field/set_default_field/clear_field: the companion to
 // update_related_record for the common case of "when this record's
 // status changes, also update another field on this same record" (e.g.
 // Company status -> Customer also sets Industry). set_default_field only
 // writes when the target is currently empty; clear_field always writes
 // empty, ignoring updateValue/updateCopyFrom - same split as the desktop
 // edition's update_field/set_default_field/clear_field workflow actions.
 // record is the same object reference already mutated onto data[key] by
 // the caller, so writing to it here updates the live record directly -
 // same pattern update_related_record already uses for its linked records.
 if(a.type==='update_field'||a.type==='set_default_field'||a.type==='clear_field'){
  if(!a.updateFieldKey)return null;
  if(a.type==='clear_field'){
   record[a.updateFieldKey]='';
   return `cleared ${fieldLabelFor(key,a.updateFieldKey)}`;
  }
  if(a.type==='set_default_field'&&record[a.updateFieldKey])return null; // already has a value - left unchanged
  const value=a.updateCopyFrom?(record[a.updateCopyFrom]??''):(a.updateValue??'');
  record[a.updateFieldKey]=value;
  return a.type==='set_default_field'?`set default ${fieldLabelFor(key,a.updateFieldKey)} = "${value}"`:`set ${fieldLabelFor(key,a.updateFieldKey)} = "${value}" on this record`;
 }
 // create_task (default, and the only action type older saved data has)
 if(!a.taskTitle)return null;
 const due=new Date();due.setDate(due.getDate()+Number(a.daysOffset||0));
 // Custom objects aren't in relatedTypeFor (Tasks' relatedType dropdown is
 // a fixed built-ins-only list, matching desktop) - fall back to 'General'
 // rather than writing an unrecognized relatedType onto the created task.
 data.tasks.unshift({id:uid(),taskNumber:nextNumber('tasks'),title:a.taskTitle,relatedType:relatedTypeFor[key]||'General',relatedId:relatedTypeFor[key]?record.id:'',owner:record.owner||'Unassigned',due:due.toISOString().slice(0,10),priority:'Medium',status:'Open'});
 return `created task "${a.taskTitle}"`;
}
function workflowTab(body){
 if(wfBuilderMode){renderWorkflowBuilder(body);return}
 const keys=[...Object.keys(relatedTypeFor),...activeCustomObjectKeys()];
 const list=(data.workflowRules||[]).filter(r=>r.entity===wfEntity);
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Workflow automation</h3><p class="muted">Trigger any number of actions - create a task, create a new record, update a related record, or update/default/clear a field on this record - when a saved record's changed fields match a set of AND/OR conditions (with one level of OR-groups).</p>
 ${entityPills(keys,wfEntity)}
 <div class="table-wrap"><table class="table"><thead><tr><th>When</th><th>Then</th><th>Notifies admins</th><th>Active</th><th>Actions</th></tr></thead><tbody>${list.map(r=>`<tr><td>${describeConditions(r.entity,r.conditions,r.matchType||'all')}</td><td>${(r.actions||[]).map(a=>describeWorkflowAction(a,r.entity)).join('; ')}</td><td>${r.notify?'Yes':'No'}</td><td>${badgeMaybe(r.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-edit-wf="${r.id}">Edit</button><button class="icon-btn" data-del-wf="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No workflow rules on '+entityLabel(wfEntity)+' yet</div>'}</div>
 <button class="btn btn-secondary" id="addWf" style="margin-top:14px">+ New workflow rule</button>
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{wfEntity=b.dataset.entity;renderAdminTab()});
 $('#addWf').onclick=()=>{wfBuilderMode='create';renderAdminTab()};
 body.querySelectorAll('[data-edit-wf]').forEach(b=>b.onclick=()=>{wfBuilderMode=b.dataset.editWf;renderAdminTab()});
 body.querySelectorAll('[data-del-wf]').forEach(b=>b.onclick=()=>{data.workflowRules=data.workflowRules.filter(r=>r.id!==b.dataset.delWf);save();toast('Workflow rule deleted');renderAdminTab()});
}
const WORKFLOW_ACTION_TYPES=['create_task','create_record','update_related_record','update_field','set_default_field','clear_field'];
const WORKFLOW_ACTION_LABELS={create_task:'Create a task',create_record:'Create a new record',update_related_record:'Update a related record',update_field:'Update this record',set_default_field:'Set default value',clear_field:'Clear a field'};
const WORKFLOW_ACTION_ICONS={create_task:'📋',create_record:'➕',update_related_record:'🔗',update_field:'✏️',set_default_field:'🔧',clear_field:'🧹'};
// One workflow action row - create_task/create_record/update_related_record
// are unchanged from Phase 3's action expansion; update_field/
// set_default_field/clear_field (the last two new in the second addendum
// round) share this same target-field-plus-value sub-form, since all three
// write to a field on the triggering record itself (see
// executeWorkflowAction's shared branch).
function workflowActionRowHtml(entityKey,a,idx,recordTargets,relTargets){
 const actionableFields=actionableFieldsFor(entityKey);
 const type=WORKFLOW_ACTION_TYPES.includes(a.type)?a.type:'create_task';
 let bodyHtml='';
 if(type==='create_task'){
  bodyHtml=`<input name="wact${idx}_taskTitle" value="${a.taskTitle||''}" placeholder="e.g. Kick off onboarding" style="min-width:180px">
   <input name="wact${idx}_daysOffset" type="number" min="0" value="${a.daysOffset??0}" style="width:130px" placeholder="Due in days">`;
 }else if(type==='create_record'){
  bodyHtml=`<select name="wact${idx}_recordTargetEntity">${recordTargets.map(t=>`<option value="${t}" ${t===a.recordTargetEntity?'selected':''}>${entityLabel(t)}</option>`).join('')}</select>
   ${UNNAMED_RECORD_TYPES.includes(a.recordTargetEntity)?'':`<input name="wact${idx}_recordNameTemplate" value="${a.recordNameTemplate||''}" placeholder="Name/title" style="min-width:200px">`}`;
 }else if(type==='update_related_record'){
  const relOtherFields=a.relTargetEntity?actionableFieldsFor(a.relTargetEntity):[];
  bodyHtml=`<select name="wact${idx}_relTargetEntity">${relTargets.map(t=>`<option value="${t}" ${t===a.relTargetEntity?'selected':''}>${entityLabel(t)}</option>`).join('')}</select>
   <select name="wact${idx}_relTargetField">${relOtherFields.map(f=>`<option value="${f[0]}" ${f[0]===a.relTargetField?'selected':''}>${f[1]}</option>`).join('')}</select>
   <input name="wact${idx}_relValue" value="${a.relValue||''}" placeholder="New value" style="width:160px">`;
 }else{ // update_field / set_default_field / clear_field
  const isClear=type==='clear_field';
  bodyHtml=`<select name="wact${idx}_updateFieldKey">${actionableFields.map(f=>`<option value="${f[0]}" ${f[0]===a.updateFieldKey?'selected':''}>${f[1]}</option>`).join('')}</select>
   ${isClear?'':`<input name="wact${idx}_updateValue" value="${a.updateCopyFrom?'':(a.updateValue||'')}" placeholder="Set to value" style="width:160px" ${a.updateCopyFrom?'disabled':''}>
   <span class="muted" style="font-size:12px">or copy from</span>
   <select name="wact${idx}_updateCopyFrom"><option value="">— none —</option>${actionableFields.map(f=>`<option value="${f[0]}" ${f[0]===a.updateCopyFrom?'selected':''}>${f[1]}</option>`).join('')}</select>`}
   ${type==='set_default_field'?'<span class="muted" style="font-size:11px">Only fills the field if it\'s currently empty</span>':''}`;
 }
 return `<div class="builder-row-card" style="align-items:flex-start">
  <span style="font-size:15px">${WORKFLOW_ACTION_ICONS[type]||''}</span>
  <select name="wact${idx}_type" data-wact-type="${idx}">${WORKFLOW_ACTION_TYPES.filter(t=>t!=='update_related_record'||relTargets.length).map(t=>`<option value="${t}" ${t===type?'selected':''}>${WORKFLOW_ACTION_LABELS[t]}</option>`).join('')}</select>
  <span style="display:flex;gap:6px;flex-wrap:wrap;align-items:center">${bodyHtml}</span>
  <button type="button" class="builder-row-remove" data-wact-remove="${idx}" title="Remove action">✕</button>
 </div>`;
}
function emptyWorkflowAction(type,entityKey,recordTargets,relTargets){
 if(type==='create_record')return {type,recordTargetEntity:recordTargets[0]||'',recordNameTemplate:''};
 if(type==='update_related_record')return {type,relTargetEntity:relTargets[0]||'',relTargetField:'',relValue:''};
 if(type==='update_field'||type==='set_default_field'||type==='clear_field')return {type,updateFieldKey:actionableFieldsFor(entityKey)[0]?.[0]||'',updateValue:'',updateCopyFrom:''};
 return {type:'create_task',taskTitle:'',daysOffset:0};
}
/** Mounts a self-contained multi-action editor into `container` (must
 * already be inside a <form>) - "+ Add action" plus per-row type/remove,
 * mirroring mountActionsEditor's pattern for business rules. */
function mountWorkflowActionsEditor(container,entityKey,initialActions){
 const recordTargets=createRecordTargetsFor(entityKey), relTargets=relTargetsFor(entityKey);
 let actions=(initialActions&&initialActions.length?initialActions:[emptyWorkflowAction('create_task',entityKey,recordTargets,relTargets)]).map(a=>({...a}));
 function syncFromDom(){
  const form=container.closest('form');
  actions=actions.map((a,idx)=>{
   const typeEl=form.elements[`wact${idx}_type`]; if(!typeEl)return a;
   const type=typeEl.value, g=name=>form.elements[`wact${idx}_${name}`]?.value;
   if(type==='create_task')return {type,taskTitle:g('taskTitle')||'',daysOffset:Number(g('daysOffset')||0)};
   if(type==='create_record')return {type,recordTargetEntity:g('recordTargetEntity')||'',recordNameTemplate:g('recordNameTemplate')||''};
   if(type==='update_related_record')return {type,relTargetEntity:g('relTargetEntity')||'',relTargetField:g('relTargetField')||'',relValue:g('relValue')||''};
   const copyFrom=g('updateCopyFrom')||'';
   return {type,updateFieldKey:g('updateFieldKey')||'',updateValue:copyFrom?'':(g('updateValue')||''),updateCopyFrom:copyFrom};
  });
 }
 function render(){
  container.innerHTML=actions.map((a,idx)=>workflowActionRowHtml(entityKey,a,idx,recordTargets,relTargets)).join('')+`<button type="button" class="btn" data-wact-add>+ Add action</button>`;
  wire();
 }
 function wire(){
  const form=container.closest('form');
  actions.forEach((a,idx)=>{
   form.elements[`wact${idx}_type`].onchange=()=>{syncFromDom();actions[idx]=emptyWorkflowAction(form.elements[`wact${idx}_type`].value,entityKey,recordTargets,relTargets);render()};
   form.elements[`wact${idx}_recordTargetEntity`]?.addEventListener('change',()=>{syncFromDom();render()});
   form.elements[`wact${idx}_relTargetEntity`]?.addEventListener('change',()=>{syncFromDom();actions[idx].relTargetField='';render()});
   form.elements[`wact${idx}_updateCopyFrom`]?.addEventListener('change',()=>{syncFromDom();render()});
  });
  container.querySelector('[data-wact-add]').onclick=()=>{syncFromDom();actions.push(emptyWorkflowAction('create_task',entityKey,recordTargets,relTargets));render()};
  container.querySelectorAll('[data-wact-remove]').forEach(b=>b.onclick=()=>{syncFromDom();actions.splice(Number(b.dataset.wactRemove),1);render()});
 }
 render();
 return {getActions:()=>{syncFromDom();return actions}};
}
// Workflow-builder page: a Conditions/Actions left column paired with a
// live visual canvas on the right (Trigger -> Conditions -> Actions ->
// End). v0.25 bug-report round: the old separate "Trigger" (one mandatory
// field/operator/value) and "Extra conditions" (optional AND/OR) sections
// are merged into one unified Conditions section - the Salesforce/Dynamics
// "entry criteria" pattern - reusing mountConditionsEditor exactly as the
// business rule builder does, instead of a bespoke single-condition widget
// plus a second multi-condition editor bolted on next to it.
function renderWorkflowBuilder(body){
 const isEdit=wfBuilderMode!=='create';
 const existing=isEdit?data.workflowRules.find(r=>r.id===wfBuilderMode):null;
 if(isEdit&&!existing){wfBuilderMode=null;renderAdminTab();return}
 const entityKey=existing?existing.entity:wfEntity;
 const condFields=conditionFieldsFor(entityKey);
 const recordTargets=createRecordTargetsFor(entityKey);
 const relTargets=relTargetsFor(entityKey);
 const initialConditions=existing?.conditions?.length?existing.conditions:[{fieldKey:transitionFieldFor(entityKey),operator:'equals',value:'',compareField:null,groupId:null}];
 const initialActions=existing?.actions?.length?existing.actions:[emptyWorkflowAction('create_task',entityKey,recordTargets,relTargets)];
 body.innerHTML=`<div class="builder-header">
  <div>
   <div class="builder-breadcrumb">Workflow Automation / ${isEdit?'Edit workflow':'New workflow'}</div>
   <div class="builder-title-row"><h2>${isEdit?'Edit workflow rule':'New workflow rule'}</h2>${isEdit?`<span class="badge" style="${existing.active?'background:#dcfce7;color:#166534':''}">${existing.active?'Active':'Inactive'}</span>`:''}</div>
   <p class="builder-subtitle">Applies to ${entityLabel(entityKey)}.</p>
  </div>
  <div class="builder-header-actions">
   <button class="btn btn-secondary" type="button" id="wfBuilderTest">${testingWorkflow?'Hide test':'Test workflow'}</button>
   ${isEdit?`<button class="btn btn-secondary" type="button" id="wfBuilderToggleActive">${existing.active?'Deactivate':'Activate'}</button>`:''}
   <button class="btn btn-primary" type="submit" form="wfBuilderForm">${isEdit?'Save':'Add rule'}</button>
  </div>
 </div>
 ${testingWorkflow?testPanelHtml(entityKey,'Test workflow automation'):''}
 <form id="wfBuilderForm">
 <div class="workflow-builder-layout">
  <div>
   <div class="builder-section">
    <div class="builder-section-title"><span class="step-badge">1</span> Conditions</div>
    <p class="muted" style="margin-top:-4px">Runs whenever a saved ${entityLabel(entityKey).toLowerCase().replace(/s$/,'')} matches the conditions below - add more (AND/OR, with one level of OR-groups) to narrow it down.</p>
    <div id="wfConditions"></div>
   </div>
   <div class="builder-section">
    <div class="builder-section-title"><span class="step-badge">2</span> Actions</div>
    <div id="wfActions"></div>
    <div class="field" style="max-width:220px;margin-top:10px"><label>Also notify admins?</label><select name="notify"><option value="false" ${!existing?.notify?'selected':''}>No</option><option value="true" ${existing?.notify?'selected':''}>Yes</option></select></div>
   </div>
  </div>
  <div class="workflow-canvas-wrap">
   <div class="workflow-node workflow-node-trigger"><div class="workflow-node-head">Trigger</div><div class="workflow-node-body"><strong>${entityLabel(entityKey)}</strong><small>Record created or edited</small></div></div>
   <div class="workflow-connector">▼</div>
   <div class="workflow-node workflow-node-conditions"><div class="workflow-node-head">Conditions</div><div class="workflow-node-body" id="canvasConditions"></div></div>
   <div class="workflow-connector">▼</div>
   <div class="workflow-node workflow-node-actions"><div class="workflow-node-head">Actions</div><div class="workflow-node-body" id="canvasAction"></div></div>
   <div class="workflow-connector">▼</div>
   <div class="workflow-end-node">END</div>
  </div>
 </div>
 <div style="margin-top:4px;display:flex;gap:8px">
  <button type="button" class="btn btn-secondary" id="wfBuilderCancel">Cancel</button>
  ${isEdit?`<button type="button" class="btn btn-secondary" id="wfBuilderToggleActive2">${existing.active?'Deactivate':'Activate'}</button>`:''}
  <button class="btn btn-primary" type="submit" form="wfBuilderForm">${isEdit?'Save':'Add rule'}</button>
 </div>
 </form>`;
 const form=$('#wfBuilderForm');
 const condEditor=mountConditionsEditor($('#wfConditions',form),condFields,initialConditions,existing?.matchType||'all');
 const actEditor=mountWorkflowActionsEditor($('#wfActions',form),entityKey,initialActions);
 // The live canvas mirrors whatever the form currently says - re-derived
 // from the editors' own state on every change, the same "what will
 // actually happen" summary the desktop canvas gives at a glance. The
 // Conditions node honors OR-groups exactly like the builder rows above it.
 function updateCanvas(){
  const conditions=condEditor.getConditions(), matchType=condEditor.getMatchType();
  const units=groupConditionUnits(conditions);
  $('#canvasConditions').innerHTML=units.length?units.map((u,ui)=>{
   const divider=ui>0?`<span class="workflow-match-chip">${matchType==='all'?'AND':'OR'}</span>`:'';
   if(u.kind==='single')return divider+`<span class="workflow-condition-chip">${describeCondition(entityKey,conditions[u.index])}</span>`;
   const rows=u.indices.map((idx,mi)=>(mi>0?'<span class="workflow-match-chip">OR</span>':'')+`<span class="workflow-condition-chip">${describeCondition(entityKey,conditions[idx])}</span>`).join('');
   return divider+`<div class="workflow-or-group"><div class="workflow-or-group-label">OR group</div>${rows}</div>`;
  }).join(''):'<p class="empty" style="padding:8px">No conditions yet</p>';
  const actions=actEditor.getActions();
  $('#canvasAction').innerHTML=actions.length?actions.map(a=>`<div><strong>${WORKFLOW_ACTION_LABELS[a.type]||'Action'}</strong><small>${describeWorkflowAction(a,entityKey)}</small></div>`).join(''):'<p class="empty" style="padding:8px">No actions yet</p>';
 }
 form.addEventListener('change',updateCanvas);
 form.addEventListener('input',updateCanvas);
 // Same MutationObserver fallback as the rule builder's summary panel - a
 // pure add/remove-row re-render doesn't fire 'change'/'input' on the form.
 const canvasObserver=new MutationObserver(updateCanvas);
 canvasObserver.observe($('#wfConditions',form),{childList:true,subtree:true});
 canvasObserver.observe($('#wfActions',form),{childList:true,subtree:true});
 updateCanvas();
 $('#wfBuilderTest').onclick=()=>{testingWorkflow=!testingWorkflow;renderAdminTab()};
 if(testingWorkflow)wireWorkflowTestPanel(entityKey);
 const toggleActive=()=>{existing.active=!existing.active;save();toast(existing.active?'Rule activated':'Rule deactivated');renderAdminTab()};
 $('#wfBuilderToggleActive')?.addEventListener('click',toggleActive);
 $('#wfBuilderToggleActive2')?.addEventListener('click',toggleActive);
 $('#wfBuilderCancel').onclick=()=>{wfBuilderMode=null;testingWorkflow=false;renderAdminTab()};
 form.onsubmit=e=>{e.preventDefault();const fd=Object.fromEntries(new FormData(form).entries());
  const conditions=condEditor.getConditions(), matchType=condEditor.getMatchType(), actions=actEditor.getActions();
  if(!conditions.length)return alert('Add at least one condition.');
  if(!actions.length)return alert('Add at least one action.');
  for(const a of actions){
   if(a.type==='create_task'&&!a.taskTitle)return alert('Enter a task title.');
   if(a.type==='create_record'&&!UNNAMED_RECORD_TYPES.includes(a.recordTargetEntity)&&!a.recordNameTemplate)return alert('Enter a name/title for the new record.');
   if(a.type==='update_related_record'&&!a.relValue)return alert('Enter the value to write on the related record.');
   if((a.type==='update_field'||a.type==='set_default_field')&&!a.updateCopyFrom&&!a.updateValue)return alert('Enter a value to write, or a field to copy from.');
  }
  const payload={entity:entityKey,notify:fd.notify==='true',conditions,matchType,actions,conditionsMerged:true};
  if(isEdit){Object.assign(existing,payload)}else{data.workflowRules.push({id:uid(),active:true,...payload})}
  save();toast(isEdit?'Workflow rule saved':'Workflow rule added');wfBuilderMode=null;testingWorkflow=false;renderView()};
}

// ---- Status Transition Editor (Phase 2) -----------------------------------
function transitionsTab(body){
 const keys=[...Object.keys(numberRules),...activeCustomObjectKeys()];
 const list=(data.statusTransitionRules||[]).filter(r=>r.entity===trEntity);
 const tf=transitionFieldFor(trEntity);
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Status transitions</h3><p class="muted">Restrict which ${fieldLabelFor(trEntity,tf).toLowerCase()} changes are allowed on ${entityLabel(trEntity)}. With no active rules the field stays fully unrestricted; once at least one is active, only the listed moves are allowed.</p>
 ${entityPills(keys,trEntity)}
 <div class="table-wrap"><table class="table"><thead><tr><th>From</th><th>To</th><th>Active</th><th>Actions</th></tr></thead><tbody>${list.map(r=>`<tr><td>${r.from?badgeMaybe(r.from):'<em class="muted">Any status</em>'}</td><td>${badgeMaybe(r.to)}</td><td>${badgeMaybe(r.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-toggle-tr="${r.id}">${r.active?'Deactivate':'Activate'}</button><button class="icon-btn" data-del-tr="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No transition rules on '+entityLabel(trEntity)+' yet — every change is currently allowed.</div>'}</div>
 <button class="btn btn-secondary" id="addTransition" style="margin-top:14px">+ New transition rule</button>
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{trEntity=b.dataset.entity;renderAdminTab()});
 $('#addTransition').onclick=()=>transitionModal(trEntity);
 body.querySelectorAll('[data-toggle-tr]').forEach(b=>b.onclick=()=>{const r=data.statusTransitionRules.find(x=>x.id===b.dataset.toggleTr);r.active=!r.active;save();toast(r.active?'Rule activated':'Rule deactivated');renderAdminTab()});
 body.querySelectorAll('[data-del-tr]').forEach(b=>b.onclick=()=>{data.statusTransitionRules=data.statusTransitionRules.filter(r=>r.id!==b.dataset.delTr);save();toast('Transition rule deleted');renderAdminTab()});
}
function transitionModal(entityKey){
 const options=transitionOptionsFor(entityKey);
 const tf=transitionFieldFor(entityKey);
 const body=`<form id="trForm" class="form-grid">
 <div class="field"><label>From (leave as "Any status" for a wildcard rule)</label><select name="from"><option value="">Any status</option>${options.map(o=>`<option value="${o}">${o}</option>`).join('')}</select></div>
 <div class="field"><label>To</label><select name="to" required>${options.map(o=>`<option value="${o}">${o}</option>`).join('')}</select></div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Add rule</button></div>
 </form>`;
 modal(`New ${fieldLabelFor(entityKey,tf).toLowerCase()} transition — ${entityLabel(entityKey)}`,body);
 $('[data-close]').onclick=closeModal;
 $('#trForm').onsubmit=e=>{e.preventDefault();const fd=Object.fromEntries(new FormData(e.target).entries());
  data.statusTransitionRules.push({id:uid(),entity:entityKey,active:true,from:fd.from||'',to:fd.to});
  save();closeModal();toast('Transition rule added');renderView()};
}
function numberingTab(body){
 const keys=Object.keys(numberRules);
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Numbering</h3><p class="muted">Control the prefix and digit width used for each object's auto-generated ID. Existing numbers are not renumbered.</p>
 <div class="table-wrap"><table class="table"><thead><tr><th>Object</th><th>Prefix</th><th>Digits</th><th>Example</th><th>Actions</th></tr></thead><tbody>${keys.map(k=>{
  const base=numberRules[k];const o=data.numberingOverrides[k];
  const prefix=o?o.prefix:(base.year?`${base.prefix}-${year()}-`:`${base.prefix}-`);
  const width=o?o.width||base.width:base.width;
  const example=prefix+pad(1,width);
  const shownPrefix=o?o.prefix:base.prefix+(base.year?'-YYYY-':'-');
  return `<tr><td>${entityLabel(k)}</td><td>${shownPrefix}</td><td>${width}</td><td>${example}${o?' <span class="badge">Custom</span>':''}</td><td><div class="actions"><button class="icon-btn" data-edit-num="${k}">Edit</button>${o?`<button class="icon-btn" data-reset-num="${k}">Reset</button>`:''}</div></td></tr>`;
 }).join('')}</tbody></table></div>
 </div>`;
 body.querySelectorAll('[data-edit-num]').forEach(b=>b.onclick=()=>numberingModal(b.dataset.editNum));
 body.querySelectorAll('[data-reset-num]').forEach(b=>b.onclick=()=>{delete data.numberingOverrides[b.dataset.resetNum];save();toast('Numbering format reset to default');renderAdminTab()});
}
function numberingModal(key){
 const base=numberRules[key];const o=data.numberingOverrides[key];
 const body=`<form id="numForm" class="form-grid">
 <div class="field full"><label>Prefix (include any punctuation, e.g. "ACC-" or "ACC-ab")</label><input name="prefix" value="${o?o.prefix:base.prefix+(base.year?'-'+year()+'-':'-')}" required></div>
 <div class="field"><label>Digits</label><input name="width" type="number" min="1" max="10" value="${o?o.width||base.width:base.width}"></div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Save format</button></div>
 </form>`;
 modal(`Numbering format — ${entityLabel(key)}`,body);
 $('[data-close]').onclick=closeModal;
 $('#numForm').onsubmit=e=>{e.preventDefault();const fd=Object.fromEntries(new FormData(e.target).entries());if(!fd.prefix.trim())return alert('Enter a prefix.');data.numberingOverrides[key]={prefix:fd.prefix.trim(),width:Math.min(10,Math.max(1,Number(fd.width||base.width)))};save();closeModal();toast('Numbering format updated');renderView()};
}
function kpisTab(body){
 const prefs=data.kpiPrefs||[];
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Dashboard KPIs</h3><p class="muted">Choose which tiles show on the dashboard. Leave all unchecked to show every tile.</p>
 <form id="kpiForm">${KPI_DEFS.map(k=>`<label class="checkbox-row"><input type="checkbox" name="kpi" value="${k.key}" ${prefs.includes(k.key)?'checked':''}> ${k.label}</label>`).join('')}
 <div class="modal-actions" style="justify-content:flex-start;margin-top:16px"><button class="btn btn-primary" type="submit">Save selection</button> <button type="button" class="btn btn-secondary" id="showAllKpis">Show all</button></div>
 </form></div>`;
 $('#kpiForm').onsubmit=e=>{e.preventDefault();const checked=[...e.target.querySelectorAll('input[name="kpi"]:checked')].map(i=>i.value);data.kpiPrefs=checked;save();toast('Dashboard KPI selection saved');renderView()};
 $('#showAllKpis').onclick=()=>{data.kpiPrefs=[];save();toast('Dashboard now shows every KPI');renderAdminTab()};
}

function publicNav(){return `<nav class="landing-nav"><div class="container nav-inner"><a class="brand" href="/"><span class="brand-mark">L</span>Lanesra OS</a><div class="nav-links"><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a></div><div class="nav-actions"><a class="btn btn-primary mobile-try" href="/demo">Try Online →</a><button class="menu-toggle" aria-label="Open navigation" aria-expanded="false">☰</button></div></div><div class="mobile-drawer" hidden><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a><hr><a href="/roadmap">Roadmap & Backlog</a><a href="/releases">Releases</a><a href="https://vikramgrover.com">Built by Vikram Grover</a></div></nav>`}
function publicFooter(){return `<footer class="footer"><div class="container footer-grid"><div><a class="brand footer-brand" href="/"><span class="brand-mark">L</span>Lanesra OS</a><span class="muted">Modern, open-source business software for small businesses.</span></div><div><strong>Product</strong><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a></div><div><strong>Development</strong><a href="/roadmap">Roadmap & Backlog</a><a href="/releases">Releases</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a></div><div><strong>Creator</strong><a href="https://vikramgrover.com">VikramGrover.com</a></div></div><div class="container footer-bottom"><span>© 2026 Lanesra OS</span><span>Created by Vikram Grover</span></div></footer>`}
function roadmapPage(){
 document.title='Roadmap & Backlog — Lanesra OS';
 const shipped=[
  ['Core CRM & sales lifecycle','Companies, Contacts, Products, Opportunities, Quotes, Orders, Invoices, Contracts, Tasks — full CRUD, the flexible Company → Opportunity → Quote → Order → Invoice path plus direct-entry shortcuts, gap-free document numbering, integer-cent money math, duplicate-name/email warnings, and dashboard KPIs.'],
  ['Team Workspace (multi-user over LAN)','An axum HTTP server sharing the same business logic as the desktop app, cookie sessions, Docker packaging — a small team runs one server, everyone else uses a browser tab.'],
  ['Data safety & account','Whole-workspace backup/restore as a .lanesra file, self-service password change, admin-managed users with a last-Administrator lockout guard.'],
  ['Document output','PDF-quality print preview for quotes/orders/invoices via the browser\'s native print dialog; CSV export on every list screen and CSV import for Companies and Contacts, both routed through the same create commands the manual forms use.'],
  ['Admin panel & configurability','Branding & print customization; reports beyond the dashboard plus a simple custom report builder; custom fields, conditional business rules and workflow automation, generalized from Companies/Contacts to every major object; admin-configurable numbering per object; a dashboard KPI picker.'],
  ['Custom Objects — extensibility platform, Phase A','An Administrator defines a whole new business object at runtime with its own icon and ID format, no code change — and it works through the exact same custom fields, business rules and report builder every built-in entity uses.'],
  ['Custom Relationships — Phase B','Admins define relationships between any two record types (built-in or custom) — one-to-one / many-to-one / many-to-many cardinality, a restrict-or-archive delete policy, and a related-records list on record detail pages.'],
  ['Richer Business Rules engine — Phase C, extended','Multi-condition AND/OR matching with one level of nested OR-groups, 10 comparison operators, and 12 action types (require, show, hide, lock, make editable, set default, set/clear value, restrict choices, block save, show error, show warning), plus rule priority, optional effective-date windows, and a "hide by default" flag on custom fields.'],
  ['Richer Workflow Automation engine — Phase D, extended','7 trigger types (created/updated, status/field changed, date reached, due/overdue, scheduled), optional extra AND/OR conditions with OR-groups, and 8 action types (create task, update/default/clear a field, assign owner, create related record, add notification, create reminder), plus an in-app notification center.'],
  ['Field validation, task reminders, session lock — Phase E','Custom field validation (min/max, max length, regex) at both definition and save time; Windows task reminder toasts through the standard Web Notification API; a 15-minute session inactivity auto-lock.'],
  ['Condition engine v2','Four more comparison operators — starts with, ends with, is one of, is not one of — plus field-to-field comparison, so a condition can match against another field\'s live value instead of only a fixed one. Shared by business rules and workflow triggers, on desktop and in the online demo.'],
  ['Status Transition Editor','Restrict which status/stage changes are allowed on any object, with a wildcard "from any status" starting point and a per-rule active toggle. No active rules leaves the field fully unrestricted; resaving the same status is never blocked.'],
  ['Workflow action & test-mode expansion','Workflow actions reaching beyond the triggering record: create a new record (optionally linked) or update a field on already-linked records. A Test rule / Test workflow dry-run mode shows what active rules and workflows would do against hypothetical values, without touching real data.'],
  ['Custom field extensibility','Four more settings on any custom field: a default value applied when a save leaves it empty, a "require a unique value" check (rejected at definition time for yes/no fields), placeholder text, and help text shown under the field on the record form.'],
  ['Customer 360 / Contact 360','A dedicated detail page for every company and contact — full field overview plus every linked record (contacts, opportunities, quotes, orders, invoices, contracts, tasks) one click away, replacing edit-modal-only access.'],
  ['Business Rules & Workflow Automation redesign','Both builders rebuilt as a numbered Condition/Effect (or Trigger/Action) layout with a live rule-summary panel; Workflow Automation gained a connected visual canvas (Trigger → Conditions → Actions → End) with zoom. Test and Activate/Deactivate moved into the builder header, alongside full editing, not just create.'],
  ['Online demo: full interactive parity','The browser demo at /demo mirrors everything above as real interactive features, not just changelog copy — its own Status Transitions tab, expanded workflow actions, Test rule/Test workflow panels, the redesigned rule-builder layout with a visual canvas, custom field extensibility, and Customer 360/Contact 360 detail pages.'],
  ['Online demo: workflow-action & custom-field parity','Workflow "Create a new record" now offers all 9 built-in entities, not just 3, and "Update a related record" walks the relationship graph in both directions so every trigger entity gets its actual related-record options. Custom fields in the demo gained the same Required/Max length/Pattern/Min/Max/Searchable/Filterable/Reportable settings the desktop edition already had.'],
  ['Online demo: Custom Objects','An Administrator can define a whole new business object at runtime from Admin → Custom Objects - its own icon, sidebar entry and ID format, no code change - and it works through the demo\'s existing Custom Fields, Business Rules, Status Transitions and Workflow Automation tabs exactly like a built-in entity. Delete is blocked while records exist; deactivate is always safe and reversible.'],
  ['Online demo: Custom Relationships','An Administrator can connect any two object types - built-in or custom - from Admin → Relationships, with a cardinality (many-to-one/one-to-one/many-to-many), forward/reverse labels, and a delete behavior (Restrict or Archive). Every record\'s edit form gets a "Related records" panel showing every link from either direction, with inline Link/Unlink.'],
  ['Online demo: Reports','A new Reports section in the browser demo with the desktop edition\'s full fixed report gallery (Revenue by month, Win rate by owner, Lost reasons, AR aging, Sales by owner) plus a Custom Reports builder that can group any built-in or custom object by its status/stage or a reportable custom field, count or sum, with CSV export on every report - closing the online demo\'s last desktop-parity gap.'],
  ['Online demo: Screen layouts (no-code UI designer)','A new capability that doesn\'t exist on desktop either: from Admin → Screen layouts, an admin drag-orders any built-in or custom object\'s create/edit fields into named sections. Editing only ever touches a draft - the live form keeps its default order until Publish, and Preview shows the draft rendered before that. A scoped, demo-first version of the "No-code Screen/UI Designer" item still proposed below for the full desktop admin extensibility spec (which also covers detail-page and tab/column layouts).'],
  ['Online demo: Integrations (UI-only simulation)','A new Admin → Integrations section, also new to the product rather than a desktop port: scheduled data Export/Import/Sync jobs against any object with a Run now simulation and per-job history, defined-and-exposed API endpoints with a Test call that returns real demo data as JSON, and configured external API connections with a Test request and call history. Everything is a local simulation against this browser\'s data - the static demo has no server, so it\'s built and labeled that way rather than faking a real backend.'],
  ['Online demo: workflow self-updates + custom-object workflow fix','Workflow automation gained the desktop edition\'s update_field action - set another field on the same record a workflow just triggered on (e.g. Company status becomes Customer, so Industry gets set to Active), with the new value either fixed or copied live from another field. Also fixed a bug where workflow rules on admin-defined Custom Objects were creatable but silently never fired.'],
  ['Record detail pages, ID hyperlinks & new fields — Products, Quotes, Orders, Invoices, Contracts, Tasks','The click-an-ID-to-open-a-record-you-can-view/edit/see-related-records-from pattern Companies and Contacts already had is now every object\'s behavior, on both the online demo and the desktop app. Every list\'s ID column is a hyperlink; each of the 6 newly-covered entities gets a detail page with an Overview panel, line items + totals for the three document types, and related-record links (a Product shows every Quote/Order/Invoice referencing it; documents show downstream documents and Tasks). Also added a batch of relevant out-of-the-box fields across 9 entities - Company phone/email/website/annual revenue/employee count/preferred contact method, Contact mobile/department/LinkedIn, and equivalents for Quotes/Orders/Invoices/Contracts/Products/Tasks.'],
  ['Online demo: mobile layout fix + stale favicon fix','The new record detail pages above overflowed horizontally on a phone - a nested overflow:auto table wrapper without min-width:0 on its containing grid, the same class of bug across Order/Invoice/Quote/Product/Contract detail; fixed to match Company/Contact 360\'s existing correct mobile behavior. Also fixed the browser tab favicon, which still drew a "B" glyph left over from the product\'s original BusinessOS name.'],
 ];
 const planned=[
  ['Admin UX polish','Duplicate/copy for a rule or workflow, version history, and a dependency warning before deactivating something another rule or field relies on — the last scoped item in the Admin Automation & Customization addendum.','spec §10','S'],
  ['Global search & list-view filtering','The desktop app has neither today — a pre-existing gap, more visible now that Phase E added is_searchable/is_filterable capability flags to custom fields that currently do nothing. Building this feature gives both flags their first real use. (The online demo already has its own simple ⌘K search, unaffected.)','spec §5.3/§9.3','M'],
  ['Self-hosted internet deployment','The Docker-packaged Team Workspace server already binds 0.0.0.0 and runs multi-user over a network — today that\'s scoped and documented as a LAN team server. This hardens the exact same server for an organization to expose it on the open internet on its own domain/infrastructure (still self-hosted, never a Lanesra-run SaaS): mark the session cookie Secure when served over HTTPS, a documented reverse-proxy/TLS recipe (nginx or Caddy in front of the existing container), and a pass over CORS and security headers for a public origin.','ops/infra','M'],
  ['Admin landing page redesign','Replace the current flat Admin tab row with a proper landing page: categorized cards (e.g. "Customization" → Custom Objects/Fields/Relationships, "Automation" → Business Rules/Workflow/Status Transitions, "Data" → Numbering/Import-Export/Backup, "Access" → Users/Roles) that route straight into the relevant builder on click, instead of one long tab strip.','ADM-UX','S–M'],
 ];
 const proposed=[
  ['Full-fledged Screen/App Builder','A drag-and-drop layout builder for create/edit/detail screens on any object, built-in or custom — sections, tabs, columns, field placement, related-list placement — with Draft → Preview → Publish, closer to Salesforce Lightning App Builder than the demo\'s existing scoped Screen layouts (field-order-only, no tabs/columns/related lists). The single largest remaining piece of the admin extensibility spec.','Substantial standalone UI project spanning both the demo and desktop\'s React frontend, and the desktop edition doesn\'t have any layout designer yet, scoped or otherwise — this would be the first. Most valuable now that Custom Objects/Relationships exist to build layouts for.','ADM-UI','L'],
  ['Full drag-and-drop report builder','The shipped report builder covers pick-an-object → group-by-field (including custom fields) → count or sum. A richer builder — multiple group-bys, filters, joins across objects, a visual canvas — was scoped down to that simpler version by explicit choice.','Worth revisiting once real usage shows the count/sum + single group-by shape is genuinely too narrow.',null,'M–L'],
  ['Full dashboard customization','Beyond the shipped KPI picker (choose which fixed tiles show): drag-and-drop widget placement, multiple named dashboards, and a real widget catalog (charts, lists, custom-report tiles) assignable per user or role — closer to Salesforce/D365\'s dashboard builder than a tile checklist.','A large, open-ended surface — needs a decision on the widget catalog\'s scope and whether layouts are admin-defined-for-everyone, per-role, or per-user before it can be scoped down to a buildable size.',null,'L'],
  ['Optional Google sign-in','Let a user log in with their Google/Gmail identity alongside the existing local username/password, as an optional per-workspace toggle.','Needs a decision first: since every workspace is self-hosted (not a shared Lanesra SaaS), each organization would have to register its own Google OAuth client — is that acceptable setup friction, or does this need a generic OIDC option instead of Google specifically? Also unclear whether it applies to the Team Workspace server only, or the Tauri desktop app too (an OAuth redirect flow is awkward inside a native webview).',null,'M'],
  ['Code-signed Windows installer','The published installer is unsigned, so Windows SmartScreen flags it as an unknown publisher.','Mostly not a coding task: buy a certificate, add a signtool step to the release workflow. The real cost is procurement — identity verification lead time, a recurring fee — an ops/budget decision, not an engineering one.',null,'S (code) / ops-heavy'],
 ];
 const futureIdeas=['Projects and milestones','Inventory and suppliers','Recurring invoices','Customer portal','Plugin architecture'];
 const sequence=[
  ['1 · Admin UX polish','The last scoped item in the Admin Automation & Customization addendum — small and well-defined, a clean next build.',true],
  ['2 · Global search & list-view filtering','Gives the existing is_searchable/is_filterable flags their first real use.',false],
  ['3 · Decide: Screen Designer, richer report builder, code signing','Three independent scope/budget calls, worth a deliberate conversation each rather than defaulting into months of work.',false],
 ];
 $('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Built from the working codebase</div><h1>Roadmap & backlog.</h1><p>Where Lanesra OS stands today, what's being built next, and everything queued up after that — compiled from the actual code (core/server/src-tauri/frontend) plus the online demo, not a wishlist. Every "Shipped" line below is running code with tests.</p><div class="status-row"><span class="status-chip">Early Access v0.24.1</span><span class="muted">Last updated August 2026</span></div>
 <div class="backlog-callout" id="desktop"><h3>Release status</h3><p><b>desktop-v0.10.0 is the latest tagged release</b> (installers attached, Early Access/prerelease as intended). Everything below is merged to <code>main</code>. Full desktop feature list → <a href="/download">/download</a>.</p><p class="muted">Repo hygiene: a real MIT <code>LICENSE</code>, <code>CONTRIBUTING.md</code>, <code>CODE_OF_CONDUCT.md</code>, <code>SECURITY.md</code>, issue/PR templates, and a root README written for someone landing on the repo, not a deploy runbook.</p></div>
 </div></section>
 <section class="section"><div class="container narrow">

 <div class="backlog-legend"><span class="backlog-pill shipped">shipped</span><span class="backlog-pill planned">planned — scoped, ready to build</span><span class="backlog-pill proposed">proposed — needs a decision</span></div>

 <div class="backlog-stats"><div class="backlog-stat"><div class="n">${shipped.length}</div><div class="l">shipped epics</div></div><div class="backlog-stat"><div class="n">${planned.length}</div><div class="l">planned, scoped items</div></div><div class="backlog-stat"><div class="n">${proposed.length}</div><div class="l">proposed, awaiting a decision</div></div><div class="backlog-stat"><div class="n">1</div><div class="l">up next</div></div></div>

 <section class="backlog-group"><div class="backlog-group-head"><h2>Shipped</h2><span class="backlog-group-note">running code, tested — all merged to main</span></div><div class="backlog-shipped-list">${shipped.map(s=>`<div class="backlog-shipped-item"><div class="mark">✓</div><div><div class="t">${s[0]}</div><div class="d">${s[1]}</div></div></div>`).join('')}</div></section>

 <section class="backlog-group"><div class="backlog-group-head"><h2>Up next</h2><span class="backlog-group-note">actively next in line, per the recommended sequencing below</span></div><p class="roadmap-row active">↻ Admin UX polish — ${planned[0][1]}</p></section>

 <section class="backlog-group"><div class="backlog-group-head"><h2>Near-term backlog</h2><span class="backlog-group-note">scoped, not yet started</span></div>${planned.map(p=>`<div class="backlog-card"><div class="backlog-card-head"><h3>${p[0]}</h3><div class="backlog-card-tags"><span class="backlog-tag planned-tag">planned</span><span class="backlog-tag">${p[2]}</span><span class="backlog-tag">size: ${p[3]}</span></div></div><p class="ask">${p[1]}</p></div>`).join('')}</section>

 <section class="backlog-group"><div class="backlog-group-head"><h2>Proposed — awaiting a decision</h2><span class="backlog-group-note">explicitly deferred, not forgotten</span></div>${proposed.map(p=>`<div class="backlog-card"><div class="backlog-card-head"><h3>${p[0]}</h3><div class="backlog-card-tags"><span class="backlog-tag proposed-tag">proposed</span>${p[3]?`<span class="backlog-tag">${p[3]}</span>`:''}<span class="backlog-tag">size: ${p[4]}</span></div></div><p class="ask">${p[1]}</p><div class="backlog-solution"><div class="sol-label">Why it's still just proposed</div><ul><li>${p[2]}</li></ul></div></div>`).join('')}</section>

 <section class="backlog-group"><div class="backlog-group-head"><h2>Future ideas</h2><span class="backlog-group-note">not yet scoped — added once there's real signal they're needed</span></div>${futureIdeas.map(x=>`<p class="roadmap-row">○ ${x}</p>`).join('')}</section>

 <section class="backlog-group"><div class="backlog-group-head"><h2>Recommended sequencing</h2><span class="backlog-group-note">one reasonable order, not the only one</span></div><div class="timeline">${sequence.map(s=>`<div class="timeline-item ${s[2]?'current':''}"><div class="timeline-date">${s[0].split(' · ')[0]}</div><div class="timeline-dot"></div><div class="timeline-content" style="padding:16px 20px"><div class="rt">${s[0].split(' · ')[1]}</div><div class="rd">${s[1]}</div></div></div>`).join('')}</div></section>

 </div></section>
 </main>${publicFooter()}`;
 bindPublicNav();
}
function releasesPage(){document.title='Releases — Lanesra OS';const releases=[['v0.24.1','August 2026','Mobile layout fix for record detail pages, and a stale favicon fix',['The new Products/Quotes/Orders/Invoices/Contracts/Tasks record detail pages (shipped in v0.24.0) overflowed horizontally on a phone - reported directly from mobile testing against the live demo. The line-items table wasn\'t wrapped in the same `.table-wrap` (overflow:auto) pattern every other data table in the app already uses, and the surrounding grid/header layout sized to that table\'s intrinsic minimum width instead of the actual viewport - a classic CSS grid/flex "min-width:0" gap. Fixed both; Company/Contact 360 (unaffected by the bug) and every other detail page now match viewport width exactly on mobile, with no change to desktop-width layout','Fixed the browser tab favicon, which still drew a "B" glyph left over from when the product was called BusinessOS (renamed to Lanesra OS in v0.5.0) - it now draws an "L" matching the in-app sidebar mark']],['v0.24.0','August 2026','Multi-condition OR-groups and a full action palette for Business Rules & Workflow Automation, on desktop and in the online demo',['Both engines\' conditions gained one level of nested OR-grouping on top of the existing AND/OR: a rule can now express "A AND (B OR C)", not just a flat AND or a flat OR - the builders show a dashed "OR group" box with a "+ OR group" control alongside "+ Add condition"','Business rule actions expanded from require/hide/lock/set-default/set-value/block-save/show-message to the full palette: Show and Make editable (explicit counterparts to Hide/Lock, most useful together with a new "Hide by default" flag on a custom field), Clear field value, Restrict choices (narrows a select field\'s options while the rule matches), and a severity split of the old generic message into Show error/Show warning - "Effects" is renamed "Action" throughout both builders','A custom field can now be flagged "Hide by default" - left off every create/edit form unless a business rule\'s Show action currently targets it, enforced server-side on desktop (a hidden-by-default+required field can never block a save) and mirrored client-side in the demo','Workflow automation gained two new field-behavior actions - Set default value (only fills a field if currently empty) and Clear a field - and, in the online demo specifically, an optional "extra conditions" section (AND/OR, same OR-groups) evaluated once the trigger itself already fired, plus true multi-action workflows (previously one action per rule)','"Trigger approval" from the design mockup is deliberately not included in this round','Desktop: Rust core (migration 0020), 8 new tests, full workspace suite green; React admin screens for both builders rebuilt to match. Online demo: business rules and workflow automation rebuilt from single-condition/single-action to fully array-based, with an in-place upgrade of any rule saved before this release - existing rules keep evaluating exactly as before until edited through the new builder']],['v0.23.1','August 2026','Online demo parity fix: workflow self-updates and custom-object workflows',['Workflow automation gained the desktop edition\'s update_field action (the companion to update_related_record) - "when this record\'s field changes, also set another field on this same record" - e.g. "when Status becomes Customer, set Industry to Active." The new value can be a fixed value or copied live from another field on the same record','Fixed a bug where a workflow rule defined on an admin-defined Custom Object could be created in Admin → Workflow Automation but would silently never fire - execution was gated on a built-ins-only lookup table left over from before Custom Objects existed as a workflow-eligible entity; workflows on custom objects now run exactly like on any built-in entity','Both fixes verified against the exact reported scenario (Company status change auto-updating a second field on the same Company) and against a custom object end to end']],['v0.23.0','August 2026','Integrations admin section in the online demo',['A new Admin → Integrations section - another capability that doesn\'t exist on the desktop edition, built for the demo first, as a UI-only simulation: the static demo has no server, so nothing here makes a real network call or runs on a real schedule','Scheduled jobs: define a data Export/Import/Sync job against any built-in or custom object with a schedule (manual, hourly, daily, weekly) and format (CSV/JSON); Run now simulates it immediately, with a per-job history log of simulated runs and record counts','API endpoints: define and "expose" a GET or POST endpoint backed by any object, with API-key or public auth; Test call shows the exact request and a realistic JSON response built from your own current demo data, entirely local','External API connections: configure an outbound connection (base URL, method, none/API-key/bearer auth) this workspace would call; Test request logs a simulated response with a call history, clearly labeled as simulated since the demo can\'t make real outbound requests','With this, the online demo has closed every gap identified in this round of parity work: Custom Objects, Custom Relationships, Reports, a no-code Screen layouts designer, and now Integrations - all four built for the demo, two catching up to desktop and two (layouts, integrations) new to the product entirely']],['v0.22.0','August 2026','No-code Screen layouts in the online demo',['A new Admin → Screen layouts tab lets an admin drag-order any built-in or custom object\'s create/edit fields into named sections - a capability that doesn\'t exist on the desktop edition either, built for the demo first','A layout has a draft and a published copy: dragging fields between/within sections, renaming a section, or adding one only ever edits the draft - the live create/edit form keeps using the plain default field order until Publish copies the draft over, and Unpublish clears it straight back to that default','A Preview button renders the draft exactly as the live form would show it, without saving anything or touching the live workspace','A published layout never drops a field it doesn\'t recognize - any field missing from the layout (a new custom field added after publishing, or a stale key) is automatically appended to a trailing "Other fields" section so nothing can go missing from a form because of a layout edit','With this, the online demo\'s only remaining gap against the desktop edition\'s admin extensibility spec is the Integrations admin section - up next, also a demo-first UI-only build']],['v0.21.0','August 2026','Reports in the online demo',['The online demo has a new top-level Reports section with the same fixed report gallery the desktop edition ships: Revenue by month, Win rate by owner, Lost reasons, AR aging and Sales by owner, each with a date-range (or "as of") filter and a bar-chart table matching desktop\'s layout','Added a Custom Reports builder to the demo, mirroring desktop\'s admin report builder: pick any object - built-in or a Custom Object - group by its status/stage or an active, reportable custom field, and count records or sum a numeric custom field; only fields an admin flagged Reportable are offered, same as desktop','Added CSV export to every report in the demo (a new capability for the demo generally, self-contained via a Blob download - no server involved)','Two disclosed substitutions where this demo\'s simpler data model doesn\'t match desktop\'s: Revenue by month/Sales by owner group by each invoice\'s due date since the demo has no separate issue date, and AR aging uses each invoice\'s full total as its balance since the demo doesn\'t track partial payments - both called out in the report\'s own subtitle rather than silently faked','Added a Lost reason field to Opportunities so the Lost Reasons report has something real to report on','With this, the online demo\'s only remaining gaps against the desktop edition are two capabilities that don\'t exist on desktop either: a no-code UI layout designer and a UI-only Integrations admin section - both underway next']],['v0.20.0','August 2026','Custom Relationships in the online demo',['An Administrator can now connect any two object types - built-in or custom - from Admin → Relationships: a cardinality (many-to-one, one-to-one or many-to-many), a forward/reverse label pair, and a choice of what happens to a link when a linked record is deleted (Restrict blocks the delete, Archive drops the link and keeps both records)','Every record\'s edit form now shows a "Related records" panel listing every linked record across every applicable relationship, from either direction, with inline Link/Unlink - the same place desktop puts its related-records card, since most objects in this demo have an edit form but no separate detail page','Cardinality is enforced on link (a many-to-one or one-to-one side can\'t be linked twice), and a relationship can\'t connect an object type to itself - both match the desktop edition\'s validation exactly','Reports beyond the dashboard remain the one desktop-only capability left in the online demo\'s parity work']],['v0.19.0','August 2026','Custom Objects in the online demo',['The online demo caught up with one of the desktop edition\'s biggest capabilities: an Administrator can now define a whole new business object at runtime - Vendors, Assets, Projects - from Admin → Custom Objects, with its own icon, sidebar entry and record-number prefix/digit width, no code change','A custom object is a full citizen of the demo\'s admin subsystems exactly like a built-in entity: it gets its own tab in Custom Fields, Business Rules, Status Transitions and Workflow Automation, and its records go through the same create/edit/list screens, auto-numbering and delete-dependency checks as Companies or Contacts','A custom object can\'t be named the same as a built-in entity (rejected at creation, matching desktop); deleting its definition is blocked while any record still exists, while deactivating is always safe and reversible since it only hides the object from navigation and new-record creation','Reports beyond the dashboard and Custom Relationships between record types remain desktop-only for now - next up in the online demo\'s parity work']],['v0.18.1','August 2026','Online demo parity fixes',['Workflow "Create a new record" now offers all 9 built-in record types in the online demo (companies, contacts, opportunities, products, quotes, orders, invoices, contracts, tasks), not just 3 - matching the desktop edition\'s full creatable set, with company-dependent types offered only when the trigger record actually carries a company','Workflow "Update a related record" now walks the demo\'s foreign-key graph in both directions, not just downward: a Contact can update its own parent Company the same way a Company can update its Contacts, and every entity gets its linked Tasks (and vice versa) through the existing relatedType/relatedId link','Custom fields in the online demo gained the validation and capability settings the desktop edition already had: Required, Max length and Pattern/regex (text fields), Min/Max value (number fields), and Searchable/Filterable/Reportable flags - enforced with native HTML5 form validation and shown in the custom fields list']],['v0.18.0','August 2026','Status transitions, richer workflow actions, test mode, a rule-builder redesign, and Customer 360',['Added a Status Transition Editor: restrict which status/stage changes are allowed on any object with a fixed-schema field (companies, contacts, opportunities, products, quotes, orders, invoices, contracts, tasks) - each rule is one from → to move, with a wildcard "any status" starting point and its own active toggle; with no active rules a field stays fully unrestricted, and resaving the same status is never blocked','Workflow automation actions expanded beyond "create a task": a workflow can now create a new record (a company, opportunity or task) or update a field on a record related to the trigger through the demo\'s existing company/contact/opportunity/quote/order relationships','Added a Test rule / Test workflow dry-run mode to both Business Rules and Workflow Automation: fill in hypothetical values for an object and see exactly which active rules or workflows would match and what they would do, without creating, changing or sending anything','Redesigned the Business Rules and Workflow Automation builders to match the desktop edition\'s rule-builder layout: numbered Condition/Effect (or Trigger/Action) sections, a live-updating rule summary panel, and - for workflows - a visual Trigger → Action → End canvas that mirrors the form as you edit it; both builders gained full editing (not just create) and header-level Test/Activate-Deactivate/Save controls','Custom fields gained four more settings: an optional default value applied whenever a save leaves the field empty, a "require a unique value" check (rejected at definition time for yes/no fields, since they only have two possible values), placeholder text, and help text shown under the field on every record form','Added Customer 360 and Contact 360: clicking a company or contact name anywhere in the app now opens a dedicated detail page with its full field overview and every linked record - contacts, opportunities, quotes, orders, invoices, contracts and tasks - each one click away, replacing edit-modal-only access','Fixed a pre-existing bug surfaced while building the above: the admin panel\'s tab row no longer freezes on whichever tab was open first - switching tabs now correctly highlights the one you\'re on']],['v0.17.0','August 2026','More operators & field-to-field comparison',['Business rules and workflows gained four more comparison operators - starts with, ends with, is one of, is not one of - on top of is/is not/contains/is empty/is not empty/greater than/less than','A condition can now compare a field against another field\'s live value instead of only a fixed value - e.g. "require Flag when Notes equals Expected Notes" - with the same live-updating preview on the record form that a fixed-value condition already had','Windows desktop edition: the shared condition engine gained the same operators and field-to-field comparison, for both business rule conditions and workflow triggers']],['v0.16.0','August 2026','Business rules & workflows now work on any field, not just status',['Business rules can now condition on any built-in field - name, industry, value, close date, whatever the object has - not only the status/stage field, with a real comparison operator (is/is not, contains, is empty, greater than, less than) chosen per field, and their require/hide action can now target a built-in field too, not just a custom one','Workflow automation\'s field-changed trigger can now watch any built-in field the same way, so "when Industry changes to X" or "when Due date is set" can create a task and notify admins, not only "when status/stage reaches a value"','Windows desktop edition: the underlying business rules and workflow engines gained the same any-built-in-field support for both conditions/triggers and actions (require, hide, lock, set default, force value, and the workflow update-field action), writing through each entity\'s own validation so nothing bypasses existing rules']],['v0.15.0','August 2026','Custom relationships, richer business rules & workflow automation',['Added admin-defined custom relationships between any two record types (companies, contacts, custom objects, and more), with one-to-one/many-to-one/many-to-many cardinality and a choice of what happens to linked records on delete','Added a related-records view on record detail pages showing every linked record through those relationships','Replaced the business rules engine: rules can now combine multiple conditions with AND/OR, use 10 comparison operators (not just equals), and lock a field, set a default or exact value, block saving entirely, or show a message — not just require or hide','Replaced the workflow automation engine: triggers now include field changes and dates reached/overdue in addition to status changes, and actions include assigning the record\'s owner, creating a related record, and posting an in-app notification, on top of creating a task','Added an in-app notification center (bell icon with unread count) for workflow-triggered notifications','Added optional validation for custom fields — a min/max range for number fields, a max length and regex pattern for text fields — plus searchable/filterable/reportable capability flags','Added Windows task reminder notifications (native toast notifications via the desktop app\'s webview)','Added a session inactivity auto-lock (15 minutes idle) requiring the current user\'s password to resume','Updated the online demo: business rules now support an "is / is not" operator, and workflow rules can optionally post an admin notification, shown in a new notification bell']],['v0.14.0','August 2026','Admin-defined Custom Objects',['Added Custom Objects: an Administrator can define an entirely new record type (its own label, fields and ID/numbering format) without any code changes','Custom Objects automatically get their own navigation section, and are full citizens of the existing custom fields, business rules and custom report builder — no per-object code was needed for any of the three','A custom object can\'t be named the same as a built-in entity, and deleting its definition is blocked while records exist (deactivating it is always safe and non-destructive)']],['v0.13.0','August 2026','Admin panel: users, roles & flexible configuration everywhere',['Added an Admin panel with user & role management, moved out of the main navigation into one dedicated section','Added an editable business profile (name, phone, address, city, logo) shown across the workspace','Generalized custom fields from Companies/Contacts to every major object: Opportunities, Quotes, Orders, Invoices, Contracts, Products and Tasks','Generalized conditional business rules and workflow automation the same way, so any object with custom fields can use them','Added admin-configurable numbering: choose the prefix and digit width used for each object\'s auto-generated ID (e.g. "ACC-000001" or "ACC-ab0001")','Added a simple custom report builder: pick an object, group by any field including custom fields, and count or sum','Added a dashboard KPI picker so admins choose which tiles show, in what selection, for the whole workspace','Updated the online demo with a full working Admin panel — mirrors every feature above in the browser']],['v0.12.0','August 2026','Branding, reports, custom fields, business rules & workflow automation',['Added business branding (logo, editable business profile) shown on the print letterhead for quotes, orders and invoices','Added reports beyond the dashboard: revenue by month, win rate by owner, lost reasons, AR aging and sales by owner','Added admin-defined custom fields on Companies and Contacts (text, number, date, yes/no, select), enforced both client- and server-side','Added conditional business rules that require or hide a custom field based on a record\'s status','Added Phase 1 workflow automation: auto-create a follow-up task when an Opportunity\'s stage or an Invoice\'s status changes']],['v0.11.0','August 2026','PDF printing & CSV import/export',['Added a browser-native "Print / Save as PDF" preview for quotes, orders and invoices, with business letterhead, line items and totals','Added CSV export on every list screen','Added CSV import for Companies and Contacts, validated row by row through the same rules as the manual forms']],['v0.10.0','August 2026','Team Workspace, backup & restore',['Added Team Workspace mode — a small team shares one server over the local network from browser tabs, with per-user sessions','Added whole-workspace backup and restore as a single file, safe to run against a live database','Added self-service password change from a "My account" screen']],['v0.9.0','August 2026','Desktop edition foundation published',['Published the Windows desktop edition source: Tauri v2 + Rust + SQLite','Implemented the full sales lifecycle on desktop — Companies, Contacts, Products, Opportunities, Quotes, Orders and Invoices','Added quote-to-order and order-to-invoice conversion, atomic document numbering and local user authentication','No packaged installer yet — desktop is available to build and run from source']],['v0.8.0','August 2026','Interactive navigation & public pages',['Made dashboard KPIs clickable with filtered drill-downs','Added a global Quick Create menu','Added mobile navigation while keeping Try Online prominent','Replaced Journey with Principles and added Compare and Download pages','Marked desktop downloads as Coming Soon','Fixed desktop sidebar navigation']],['v0.7.0','August 2026','Trust & product transparency',['Added Roadmap, Changelog and creator attribution','Added Person JSON-LD and updated discovery files']],['v0.6.0','August 2026','Record numbering & search',['Added automatically generated identifiers','Rebuilt global search as one stable result panel','Added keyboard shortcuts and wider search coverage']],['v0.5.0','July 2026','Lanesra OS rebrand',['Renamed BusinessOS to Lanesra OS','Updated product branding, metadata and documentation']],['v0.4.0','July 2026','Relationship integrity',['Added opportunity-to-contact relationship','Removed opportunity-to-contract relationship','Added company-filtered relationship dropdowns']],['v0.3.0','July 2026','Flexible sales flow',['Made opportunities optional for quotes','Made quotes optional for orders','Added products, services and line-item quantities']],['v0.2.0','June 2026','Connected sales MVP',['Added quotes, orders, invoices, contracts and dashboards','Connected core entities using clean relationships']],['v0.1.0','May 2026','First working prototype',['Launched the first browser-based MVP with sample data']]];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Release history</div><h1>Releases</h1><p>Every meaningful improvement to Lanesra OS, with detailed per-version release notes — documented publicly.</p><div class="status-row"><span class="status-chip">Latest: v0.24.1</span><span class="muted">Early Access</span></div></div></section><section class="section"><div class="container changelog-list">${releases.map(r=>`<article class="release" id="${r[0].replaceAll('.','-')}"><div class="release-meta"><span class="status-chip">${r[0]}</span><span>${r[1]}</span></div><div><h2>${r[2]}</h2><ul>${r[3].map(x=>`<li>${x}</li>`).join('')}</ul></div></article>`).join('')}</div></section></main>${publicFooter()}`;bindPublicNav()}
function principlesPage(){document.title='Principles — Lanesra OS';const principles=[['Own your data','Your customer and sales information should remain under your control—not trapped behind a subscription or vendor lock-in.'],['Offline first','Core work should continue even when the internet does not. The Windows desktop edition runs entirely on local SQLite storage, with no server or account required.'],['Relationships over spreadsheets','Customers, contacts, opportunities, quotes, orders and invoices stay linked so data remains clean and useful — and that same connected model extends to any custom record type you define.'],['Simple before powerful','Every feature must reduce effort. Complexity is added only when it clearly improves the work.'],['Configurable, not hardcoded','A business shouldn\'t need a developer to add a field, a record type, a rule or an automation. Admins reshape Lanesra from a settings screen — the software adapts to the business, not the other way around.'],['Open by default','The product roadmap, backlog, release notes and source code are all public so users can inspect how Lanesra evolves.'],['Business software deserves good design','Small businesses should not have to accept dated interfaces or confusing navigation to access serious capabilities.']];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">How Lanesra is designed</div><h1>Principles before features.</h1><p>The decisions behind Lanesra OS are guided by a small set of practical beliefs about ownership, simplicity and product quality.</p></div></section><section class="section"><div class="container principles-page-grid">${principles.map((p,i)=>`<article class="principle-card"><span>0${i+1}</span><h2>${p[0]}</h2><p>${p[1]}</p></article>`).join('')}</div></section><section class="section maintenance"><div class="container narrow"><div class="eyebrow">The business flow</div><h2>Connected by design.</h2><div class="flow-map"><strong>Customer</strong><span>→</span><div>Contacts<br>Opportunities <em>optional</em><br>Quotes <em>optional</em><br>Orders<br>Invoices<br>Contracts<br>Tasks</div></div><p class="muted" style="margin-top:18px">That same connected model isn't fixed to these nine record types — admins can add their own (Vendors, Assets, Projects…) and link them into this graph with custom relationships, so the "no dangling free text" principle holds for whatever your business actually looks like.</p></div></section></main>${publicFooter()}`;bindPublicNav()}
function comparePage(){document.title='Compare — Lanesra OS';const rows=[['Runs without internet','Partial','No','No','Yes'],['Open source','No','No','No','Yes'],['Local database','No','No','No','Yes (desktop)'],['Mandatory subscription','No','Yes','Yes','No'],['Connected sales workflow','Manual','Limited','Advanced','Yes'],['Custom record types, no code','No','Limited, paid tiers','Yes, complex/paid','Yes, built in'],['Custom business rules & workflow automation','No','Paid tiers','Yes, needs admin training','Yes, built in'],['Designed for small business','General','Yes','Enterprise','Yes'],['Self-owned business data','File-based','Cloud-hosted','Cloud-hosted','Yes']];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Choose with context</div><h1>Where Lanesra fits.</h1><p>A factual comparison for small businesses deciding between spreadsheets, cloud CRMs and a local-first open-source system.</p></div></section><section class="section"><div class="container compare-wrap"><table class="compare-table"><thead><tr><th>Capability</th><th>Excel</th><th>HubSpot</th><th>Salesforce</th><th class="lanesra-col">Lanesra OS</th></tr></thead><tbody>${rows.map(r=>`<tr>${r.map((x,i)=>`<td class="${i===4?'lanesra-col':''}">${x}</td>`).join('')}</tr>`).join('')}</tbody></table><p class="compare-note">Comparisons are intentionally high-level. Product capabilities and commercial terms can change; review each vendor's current documentation before making a purchase decision.</p></div></section></main>${publicFooter()}`;bindPublicNav()}
function downloadPage(){document.title='Download — Lanesra OS';$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Local-first desktop edition</div><h1>Download Lanesra OS.</h1><p>The independent desktop edition runs locally with no cloud account or mandatory internet connection. It is in active early development, with an Early Access Windows installer now available.</p></div></section><section class="section"><div class="container download-grid"><article class="download-card featured"><span class="status-chip">Early access — installer available</span><h2>Windows</h2><p>Tauri + Rust + SQLite desktop app with the full sales lifecycle, Contracts, Tasks and user management working: Companies, Contacts, Products, Opportunities, Quotes, Orders, Invoices, Contracts and Tasks — plus Team Workspace mode for small teams, backup and restore, PDF printing, CSV import/export, admin-defined Custom Objects and relationships between any record types, and an Admin panel covering branding, user roles, custom fields, richer conditional business rules, richer workflow automation with in-app notifications, configurable ID formats and a dashboard KPI picker. Unsigned .exe and .msi installers are on GitHub Releases (Windows will warn on first run since they aren't code-signed yet).</p><a class="btn btn-primary" href="https://github.com/vikram2409-eng/Lanesra-OS/releases" target="_blank" rel="noopener">Download for Windows</a><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS/tree/main/desktop" target="_blank" rel="noopener">View desktop source on GitHub</a></article><article class="download-card"><span class="status-chip">Planned</span><h2>macOS</h2><p>Apple silicon and Intel packaging will follow the Windows early-access release.</p><button class="btn btn-secondary" disabled>Planned</button></article><article class="download-card"><span class="status-chip">Planned</span><h2>Linux</h2><p>AppImage or Debian packaging is planned after the initial desktop release stabilizes.</p><button class="btn btn-secondary" disabled>Planned</button></article></div></section><section class="section maintenance"><div class="container narrow"><h2>What the desktop edition includes today</h2><div class="download-checks"><span>✓ No licence key</span><span>✓ No cloud account</span><span>✓ Standard SQLite database</span><span>✓ Offline from first launch</span><span>✓ Full sales lifecycle (quotes → orders → invoices)</span><span>✓ Contracts and tasks</span><span>✓ User management</span><span>✓ Team Workspace mode for small teams (Docker)</span><span>✓ Windows installer (unsigned, Early Access)</span><span>✓ Backup and restore</span><span>✓ Self-service password change</span><span>✓ PDF generation and printing</span><span>✓ CSV import and export</span><span>✓ Branding and print customization</span><span>✓ Reports, plus a custom report builder</span><span>✓ Custom fields & business rules on every object</span><span>✓ Workflow automation with in-app notifications</span><span>✓ Admin-defined Custom Objects</span><span>✓ Custom relationships between record types</span><span>✓ Windows task reminder notifications</span><span>✓ Session inactivity auto-lock</span><span>✓ Admin panel: user roles & configurable numbering</span><span>✓ Open-source code</span><span>○ Code-signed installer — planned</span></div><div class="hero-actions"><a class="btn btn-secondary" href="/roadmap#desktop">View desktop roadmap</a><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS/tree/main/desktop" target="_blank" rel="noopener">View source on GitHub</a></div></div></section></main>${publicFooter()}`;bindPublicNav()}
function bindPublicNav(){document.querySelectorAll('.menu-toggle').forEach(btn=>{btn.onclick=()=>{const nav=btn.closest('.landing-nav');const drawer=nav.querySelector('.mobile-drawer');const open=drawer.hasAttribute('hidden');if(open)drawer.removeAttribute('hidden');else drawer.setAttribute('hidden','');btn.setAttribute('aria-expanded',String(open));btn.textContent=open?'×':'☰'}});document.querySelectorAll('.mobile-drawer a').forEach(a=>a.addEventListener('click',()=>{const drawer=a.closest('.mobile-drawer');drawer.setAttribute('hidden','');const btn=drawer.closest('.landing-nav').querySelector('.menu-toggle');btn.textContent='☰';btn.setAttribute('aria-expanded','false')}))}
const path=location.pathname.replace(/\/$/,'')||'/';
// /backlog and /changelog are retired URLs - Roadmap absorbed the backlog
// content and Changelog was renamed Releases. Netlify 301s these at the
// edge (see _redirects/netlify.toml); this is a client-side fallback for
// anyone who lands on the SPA directly (e.g. a stale bookmark hitting a
// preview deploy without the redirect rules), so old links still work.
if(path==='/backlog'){history.replaceState(null,'','/roadmap');roadmapPage()}
else if(path==='/changelog'){history.replaceState(null,'','/releases');releasesPage()}
else if(path==='/demo')appShell();else if(path==='/roadmap')roadmapPage();else if(path==='/releases')releasesPage();else if(path==='/principles'||path==='/journey'||path==='/our-story'||path==='/about')principlesPage();else if(path==='/compare')comparePage();else if(path==='/download')downloadPage();else landing();
