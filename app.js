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
  {id:'fr1',entity:'opportunities',triggerField:'stage',fieldKey:'leadSource',triggerValue:'Won',effect:'require',operator:'equals',active:true}
 ],
 workflowRules:[
  {id:'wf1',entity:'opportunities',triggerField:'stage',toValue:'Won',taskTitle:'Kick off onboarding',daysOffset:2,notify:true,operator:'equals',actionType:'create_task',active:true},
  {id:'wf2',entity:'invoices',triggerField:'status',toValue:'Overdue',taskTitle:'Follow up on overdue invoice',daysOffset:0,notify:false,operator:'equals',actionType:'create_task',active:true}
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
 (data.fieldRules||[]).forEach(r=>{if(!r.operator)r.operator='equals';if(!r.triggerField)r.triggerField=transitionFieldFor(r.entity);if(r.active===undefined)r.active=true});
 (data.workflowRules||[]).forEach(r=>{if(!r.operator)r.operator='equals';if(!r.triggerField)r.triggerField=transitionFieldFor(r.entity);if(!r.actionType)r.actionType='create_task';if(r.active===undefined)r.active=true});
 (data.customFields||[]).forEach(f=>{if(f.defaultValue===undefined)f.defaultValue='';if(f.unique===undefined)f.unique=false;if(f.helpText===undefined)f.helpText='';if(f.placeholder===undefined)f.placeholder='';if(f.required===undefined)f.required=false;if(f.maxLength===undefined)f.maxLength=null;if(f.pattern===undefined)f.pattern='';if(f.minValue===undefined)f.minValue='';if(f.maxValue===undefined)f.maxValue='';if(f.searchable===undefined)f.searchable=false;if(f.filterable===undefined)f.filterable=false;if(f.reportable===undefined)f.reportable=true});
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
function applyFieldRules(entityKey,form){
 const rules=(data.fieldRules||[]).filter(r=>r.entity===entityKey&&r.active);
 if(!rules.length)return;
 function apply(){
  rules.forEach(r=>{
   const triggerField=r.triggerField||transitionFieldFor(entityKey);
   const trigger=form.elements[triggerField]; const input=form.elements[r.fieldKey];
   if(!trigger||!input)return;
   const wrap=input.closest('.field'); const label=wrap?.querySelector('label');
   // Field-to-field comparison: compare against the other field's live
   // value on this same form instead of the rule's fixed triggerValue.
   const compareValue=r.compareField?(form.elements[r.compareField]?.value??''):r.triggerValue;
   const match=operatorMatch(r.operator,trigger.value,compareValue);
   if(r.effect==='hide'){
    if(wrap)wrap.style.display=match?'none':'';
    input.required=false;
   }else if(r.effect==='require'){
    if(wrap)wrap.style.display='';
    input.required=match;
    if(label){const base=label.textContent.replace(/\s*\*$/,'');label.textContent=match?base+' *':base}
   }
  });
 }
 const watchFields=[...new Set(rules.flatMap(r=>[r.triggerField||transitionFieldFor(entityKey),r.compareField].filter(Boolean)))];
 watchFields.forEach(fk=>{const el=form.elements[fk]; if(el){el.addEventListener('change',apply);el.addEventListener('input',apply)}});
 apply();
}
const KPI_DEFS=[
 {key:'openPipeline',label:'Open pipeline',nav:'pipeline',filter:'open',value:()=>money(data.opportunities.filter(o=>!['Won','Lost'].includes(o.stage)).reduce((s,o)=>s+Number(o.value||0),0))},
 {key:'wonRevenue',label:'Won revenue',nav:'pipeline',filter:'won',value:()=>money(data.opportunities.filter(o=>o.stage==='Won').reduce((s,o)=>s+Number(o.value||0),0))},
 {key:'outstandingInvoices',label:'Outstanding invoices',nav:'invoices',filter:'outstanding',value:()=>money(data.invoices.filter(i=>!['Paid','Cancelled'].includes(i.status)).reduce((s,i)=>s+docTotal(i),0))},
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
 ['⬡','Custom Objects','Define an entirely new record type — Vendors, Assets, Projects, anything — with its own fields, ID format and navigation section.'],['⇄','Custom Relationships','Connect any two record types with one-to-one, many-to-one or many-to-many links, and a related-records list that appears automatically.'],['◈','Business Rules','Require, hide, lock or set a field\'s value with multi-condition AND/OR logic and 10 comparison operators — or block a save entirely with a custom message.'],['⚙','Workflow Automation','Trigger on a status/field change, a due date, or a schedule; create a task, assign an owner, create a related record, or post a notification.'],['▥','Custom Reports & Fields','Add validated custom fields to any object, then build reports that group and sum on them — no separate reporting tool.'],['🔔','Notifications & Admin Panel','An in-app notification center, user roles, branding, numbering formats and dashboard KPIs — one place to configure the whole workspace.']].map(x=>`<article class="feature-card"><div class="feature-icon">${x[0]}</div><h3>${x[1]}</h3><p class="muted">${x[2]}</p></article>`).join('')}</div></div></section>
 <section id="desktop" class="section"><div class="container split"><div class="choice-card"><div class="eyebrow">Try online</div><h2>Explore a working business</h2><p class="muted">Open the live demo with realistic sample customers, opportunities, quotes, invoices and contracts. No registration required.</p><ul><li>Sample company included</li><li>Create and edit records</li><li>Reset demo anytime</li></ul><a class="btn btn-primary" href="/demo">Open live demo</a></div><div class="choice-card dark"><div class="eyebrow" style="color:#a5b4fc">Desktop edition</div><h2>Your software. Your computer. Your data.</h2><p style="color:#cbd5e1">A private desktop edition is available now for Windows (Early Access, unsigned installer), with macOS and Linux to follow. The source is public on GitHub today.</p><ul><li>No cloud account required</li><li>Works without internet</li><li>No activation or subscription</li></ul><a class="btn btn-secondary" href="/download">Desktop status — Windows installer available</a></div></div></section>
 <section id="open-source" class="section"><div class="container cta"><div class="eyebrow" style="color:#a5b4fc">Open source by design</div><h2>Inspect it. Run it. Improve it.</h2><p style="color:#cbd5e1;max-width:700px;margin:0 auto 24px">Lanesra OS is designed to be transparent, community-driven and free from licence keys or mandatory telemetry.</p><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank" rel="noopener">View GitHub repository</a></div></section>
 </main>${publicFooter()}`;
 bindPublicNav();
}


function appShell(){
 document.title='Lanesra OS Demo';
 $('#app').innerHTML=`<div class="demo-banner">You are exploring the sample workspace. Changes stay in this browser. <button class="link-btn" id="resetDemo">Reset demo</button><a class="link-btn" href="/">Product website</a></div><div class="app-shell"><aside class="sidebar"><div class="side-brand"><span class="brand-mark">L</span><span>Lanesra OS</span><span class="demo-pill">DEMO</span></div><nav class="side-nav" id="sideNav">${Object.keys(labels).map(k=>`<button data-nav="${k}"><b>${icons[k]}</b><span>${labels[k]}</span></button>`).join('')}<button data-nav="admin" class="admin-nav-btn"><b>⚙</b><span>Admin</span></button></nav><div class="side-bottom"><div class="side-meta"><strong>Early Access v0.22.0</strong><div class="side-product-links"><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/roadmap">Roadmap</a><a href="/releases">Releases</a></div><span>Created by <a href="https://vikramgrover.com">Vikram Grover</a></span></div><button class="btn btn-secondary" style="width:100%" onclick="location.href='/'">← Website</button></div></aside><main class="app-main"><header class="topbar"><div class="search"><input id="globalSearch" autocomplete="off" placeholder="Search companies, contacts, deals…  ⌘K"><div id="searchResults" class="search-results" hidden></div></div><div class="top-actions"><div class="notif-wrap"><button class="icon-btn" id="notifButton" aria-label="Notifications">🔔<span id="notifBadge" class="notif-badge" hidden></span></button><div id="notifPanel" class="notif-panel" hidden></div></div><button class="icon-btn" id="helpButton" aria-label="Help">?</button><div class="avatar">MC</div></div></header><div class="content" id="view"></div></main></div>`;
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
const docTotal=r=>(r.items||[]).reduce((s,i)=>s+lineTotal(i),0);
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
 if(detailRecord&&detailRecord.type===current)return detailRecord.type==='companies'?companyDetail(detailRecord.id):contactDetail(detailRecord.id);
 // Admin-defined Custom Objects reuse the exact same generic tablePage +
 // recordModal flow every built-in entity uses - only the columns/fields
 // are built from the object's definition instead of being hardcoded.
 const co=customObjectByKey(current);
 if(co)return tablePage(current,{cols:[['number','ID'],['name','Name'],['status','Status'],['owner','Owner']],fields:()=>fieldsFor(current,customObjectFields)});
 const configs={
 companies:{cols:[['customerNumber','Customer ID'],['name','Company','companyLink'],['industry','Industry'],['city','City'],['owner','Owner'],['status','Status']],fields:()=>fieldsFor('companies',companyFields)},
 contacts:{cols:[['contactNumber','Contact ID'],['name','Contact','contactLink'],['companyId','Company','company'],['role','Role'],['email','Email'],['status','Status']],fields:()=>fieldsFor('contacts',contactFields)},
 products:{cols:[['productNumber','Product ID'],['name','Product / Service'],['type','Type'],['sku','SKU'],['price','Price','money'],['status','Status']],fields:()=>fieldsFor('products',productFields)},
 quotes:{cols:[['number','Quote'],['companyId','Customer','company'],['opportunityId','Opportunity','opportunity'],['amount','Amount','docmoney'],['status','Status']],fields:()=>fieldsFor('quotes',quoteFields),document:true},
 orders:{cols:[['number','Order'],['companyId','Customer','company'],['quoteId','Quote','quote'],['amount','Amount','docmoney'],['status','Status']],fields:()=>fieldsFor('orders',orderFields),document:true},
 invoices:{cols:[['number','Invoice'],['companyId','Customer','company'],['orderId','Order','order'],['amount','Amount','docmoney'],['status','Status']],fields:()=>fieldsFor('invoices',invoiceFields),document:true},
 contracts:{cols:[['number','Contract'],['companyId','Customer','company'],['title','Title'],['value','Value','money'],['status','Status'],['end','End date']],fields:()=>fieldsFor('contracts',contractFields)},
 tasks:{cols:[['title','Task'],['relatedId','Related to','related'],['owner','Owner'],['due','Due'],['priority','Priority'],['status','Status']],fields:()=>fieldsFor('tasks',taskFields)}
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
function companyFields(){return [['customerNumber','Customer ID','auto'],['name','Company name'],['industry','Industry'],['city','City'],['owner','Owner'],['status','Status','select','Lead|Prospect|Customer|Inactive']]}
function contactFields(){return [['contactNumber','Contact ID','auto'],['name','Full name'],['companyId','Company','relation','companies'],['role','Role'],['email','Email'],['phone','Phone'],['status','Status','select','Active|Inactive']]}
function opportunityFields(){return [['opportunityNumber','Opportunity ID','auto'],['title','Opportunity title'],['companyId','Customer','relation','companies'],['contactId','Primary contact (optional)','filteredContact'],['value','Value','number'],['stage','Stage','select','Lead|Qualified|Discovery|Proposal|Negotiation|Won|Lost'],['probability','Probability %','number'],['close','Expected close','date'],['owner','Owner'],['status','Status','select','Open|On Hold|Won|Lost'],['lostReason','Lost reason (optional)']]}
function productFields(){return [['productNumber','Product ID','auto'],['name','Name'],['sku','SKU'],['type','Type','select','Product|Service'],['category','Category'],['price','Unit price','number'],['tax','Tax %','number'],['status','Status','select','Active|Inactive']]}
function quoteFields(){return [['number','Quote number','auto'],['companyId','Customer','relation','companies'],['contactId','Contact (optional)','filteredContact'],['opportunityId','Opportunity (optional)','filteredOpportunity'],['status','Status','select','Draft|Sent|Accepted|Rejected|Expired'],['date','Quote date','date'],['valid','Valid until','date']]}
function orderFields(){return [['number','Order number','auto'],['companyId','Customer','relation','companies'],['contactId','Contact (optional)','filteredContact'],['quoteId','Source quote (optional)','filteredQuote'],['status','Status','select','Draft|Confirmed|In Progress|Completed|Cancelled'],['date','Order date','date']]}
function invoiceFields(){return [['number','Invoice number','auto'],['companyId','Customer','relation','companies'],['orderId','Source order (optional)','filteredOrder'],['status','Status','select','Draft|Sent|Partially Paid|Paid|Overdue|Cancelled'],['due','Due date','date']]}
function contractFields(){return [['number','Contract number','auto'],['companyId','Customer','relation','companies'],['contactId','Contact (optional)','filteredContact'],['title','Title'],['value','Value','number'],['status','Status','select','Draft|Active|Renewal Due|Expired|Terminated'],['start','Start','date'],['end','End','date']]}
function taskFields(){return [['taskNumber','Task ID','auto'],['title','Task title'],['relatedType','Related record type','select','General|Company|Contact|Opportunity|Quote|Order|Invoice|Contract'],['relatedId','Related record','dynamicRelation'],['owner','Owner'],['due','Due date','date'],['priority','Priority','select','Low|Medium|High|Urgent'],['status','Status','select','Open|In Progress|Completed|Cancelled']]}
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
 // No balance_cents field in this demo - stand in with each invoice's full
 // total, and treat Paid the same as "no balance left" since there's no
 // partial-payment tracking to check instead.
 const rows=data.invoices.filter(i=>!['Draft','Cancelled','Paid'].includes(i.status));
 const order=['Not yet due','1-30 days overdue','31-60 days overdue','61-90 days overdue','90+ days overdue','No due date'];
 const buckets=Object.fromEntries(order.map(b=>[b,{bucket:b,count:0,balance:0}]));
 rows.forEach(i=>{
  const bal=docTotal(i);
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
// Phase 5 Customer/Contact 360: a company/contact reference anywhere in a
// list becomes a clickable link into its 360 page, not just plain text.
function cellValue(r,c){const [key,,type]=c;if(type==='money')return money(r[key]);if(type==='docmoney')return money(docTotal(r));if(type==='company')return r[key]?`<a class="cell-link" data-open-company="${r[key]}">${companyName(r[key])}</a>`:'—';if(type==='companyLink')return `<a class="cell-link" data-open-company="${r.id}">${r[key]}</a>`;if(type==='contactLink')return `<a class="cell-link" data-open-contact="${r.id}">${r[key]}</a>`;if(type==='opportunity')return opportunityName(r[key]);if(type==='quote')return quoteName(r[key]);if(type==='order')return orderName(r[key]);if(type==='related')return relatedLabel(r);return badgeMaybe(r[key])}
function wireCellLinks(scope){
 scope.querySelectorAll('[data-open-company]').forEach(a=>a.onclick=(e)=>{e.stopPropagation();openCompanyDetail(a.dataset.openCompany)});
 scope.querySelectorAll('[data-open-contact]').forEach(a=>a.onclick=(e)=>{e.stopPropagation();openContactDetail(a.dataset.openContact)});
}
function tablePage(key,cfg){
 let arr=data[key];
 if(key==='tasks'&&viewFilter==='open')arr=arr.filter(x=>!['Completed','Cancelled'].includes(x.status));
 if(key==='invoices'&&viewFilter==='outstanding')arr=arr.filter(x=>!['Paid','Cancelled'].includes(x.status));
 $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><span>${viewFilter?viewFilter.charAt(0).toUpperCase()+viewFilter.slice(1):labels[key]}</span></div><h1>${viewFilter==='open'&&key==='tasks'?'Open Tasks':viewFilter==='outstanding'?'Outstanding Invoices':labels[key]}</h1><p class="muted">${arr.length} connected records in the sample workspace</p></div><button class="btn btn-primary" id="addRecord">+ New ${labels[key].replace(/s$/,'')}</button></div><div class="table-wrap"><table class="table"><thead><tr>${cfg.cols.map(c=>`<th>${c[1]}</th>`).join('')}<th>Actions</th></tr></thead><tbody>${arr.map(r=>`<tr>${cfg.cols.map(c=>`<td>${cellValue(r,c)}</td>`).join('')}<td><div class="actions"><button class="icon-btn" data-edit="${r.id}">Edit</button><button class="icon-btn" data-del="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${arr.length?'':'<div class="empty">No records yet</div>'}</div>`;
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
// ---- Customer 360 / Contact 360 (Phase 5) ---------------------------------
function openCompanyDetail(id){current='companies';detailRecord={type:'companies',id};renderView()}
function openContactDetail(id){current='contacts';detailRecord={type:'contacts',id};renderView()}
// A card of related records for the 360 page's right column - each row
// navigates to that record's own list (or its own 360 page, for another
// company/contact), the same click-through pattern as a cell-link.
function relatedCardHtml(title,items,navKey,labelFn,metaFn){
 return `<div class="rule360-related-card"><h4>${title} (${items.length})</h4>${items.length?items.map(x=>`<div class="rule360-related-row"><a class="cell-link" data-nav-related="${navKey}:${x.id}">${labelFn(x)}</a><span class="muted">${metaFn?metaFn(x):''}</span></div>`).join(''):'<div class="muted" style="padding:6px 0">None yet</div>'}</div>`;
}
function wireRelatedRows(scope){
 scope.querySelectorAll('[data-nav-related]').forEach(a=>a.onclick=()=>{
  const [navKey,id]=a.dataset.navRelated.split(':');
  if(navKey==='companies')return openCompanyDetail(id);
  if(navKey==='contacts')return openContactDetail(id);
  current=navKey==='opportunities'?'pipeline':navKey; viewFilter=null; detailRecord=null; renderView();
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
 if(wasEdit&&relatedTypeFor[key]&&before){
  // Each workflow rule watches its own field (not just status/stage) - fire
  // only when that field actually changed, the rule is active, and the
  // rule's operator matches.
  (data.workflowRules||[]).filter(r=>r.entity===key&&r.active).forEach(r=>{
   const wf=r.triggerField||transitionFieldFor(key);
   if(obj[wf]===undefined||obj[wf]===before[wf])return;
   // Field-to-field comparison: compare against the other field's value
   // on the just-saved record instead of the rule's fixed toValue.
   const compareValue=r.compareField?(obj[r.compareField]??''):r.toValue;
   if(!operatorMatch(r.operator||'equals',obj[wf],compareValue))return;
   const actionDescription=executeWorkflowAction(r,key,record);
   if(!actionDescription)return; // e.g. update_related_record with nothing linked yet - a silent no-op, same as desktop
   if(r.notify){
    const label=obj.name||obj.title||obj.number||entityLabel(key);
    data.notifications.unshift({id:uid(),message:`${entityLabel(key).replace(/s$/,'')} "${label}" — ${fieldLabelFor(key,wf)} ${OPERATOR_LABELS[r.operator||'equals']}${operatorNeedsValue(r.operator||'equals')?` ${describeComparand(key,r.compareField,r.toValue)}`:''} — ${actionDescription}`,createdAt:new Date().toISOString(),read:false});
   }
  });
 }
 save();closeModal();toast('Record saved');refreshNotifBadge();renderView()};
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
 const tabs=[['profile','Business profile'],['users','Users & roles'],['objects','Custom Objects'],['relationships','Relationships'],['fields','Custom fields'],['rules','Business rules'],['workflow','Workflow automation'],['transitions','Status transitions'],['layouts','Screen layouts'],['numbering','Numbering'],['kpis','Dashboard KPIs']];
 $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><span>Admin</span></div><h1>Admin panel</h1><p class="muted">Configure your workspace, users and automation. Changes save immediately in this browser.</p></div></div><div class="tabs">${tabs.map(t=>`<button class="tab ${adminTab===t[0]?'active':''}" data-admin-tab="${t[0]}">${t[1]}</button>`).join('')}</div><div id="adminBody" class="admin-body"></div>`;
 $('[data-clear-filter]').onclick=()=>{current='dashboard';viewFilter=null;renderView()};
 document.querySelectorAll('[data-admin-tab]').forEach(b=>b.onclick=()=>{adminTab=b.dataset.adminTab;renderAdminTab()});
 renderAdminTab();
}
function renderAdminTab(){
 document.querySelectorAll('[data-admin-tab]').forEach(b=>b.classList.toggle('active',b.dataset.adminTab===adminTab));
 const body=$('#adminBody');
 ({profile:profileTab,users:usersTab,objects:objectsTab,relationships:relationshipsTab,fields:fieldsTab,rules:rulesTab,workflow:workflowTab,transitions:transitionsTab,layouts:layoutsTab,numbering:numberingTab,kpis:kpisTab}[adminTab])(body);
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
 const f=field||{entity:cfEntity,label:'',type:'text',options:'',active:true,defaultValue:'',unique:false,helpText:'',placeholder:'',required:false,maxLength:'',pattern:'',minValue:'',maxValue:'',searchable:false,filterable:false,reportable:true};
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
  const matches=(data.fieldRules||[]).filter(r=>r.entity===entityKey&&r.active).filter(r=>{
   const tf=r.triggerField||transitionFieldFor(entityKey);
   const compareValue=r.compareField?(hyp[r.compareField]??''):r.triggerValue;
   return operatorMatch(r.operator||'equals',hyp[tf],compareValue);
  });
  $('#testResults').innerHTML=matches.length?`<strong>${matches.length} matching rule(s):</strong>${matches.map(r=>`<div class="deal" style="margin-top:8px">${r.effect==='require'?'Require':'Hide'} ${fieldLabelFor(entityKey,r.fieldKey)}</div>`).join('')}`:'<div class="empty">No active rule matches these values.</div>';
 };
}
function wireWorkflowTestPanel(entityKey){
 const form=$('#testForm'); if(!form)return;
 form.onsubmit=e=>{e.preventDefault();const hyp=Object.fromEntries(new FormData(form).entries());
  const matches=(data.workflowRules||[]).filter(r=>r.entity===entityKey&&r.active).filter(r=>{
   const tf=r.triggerField||transitionFieldFor(entityKey);
   const compareValue=r.compareField?(hyp[r.compareField]??''):r.toValue;
   return operatorMatch(r.operator||'equals',hyp[tf],compareValue);
  });
  $('#testResults').innerHTML=matches.length?`<strong>${matches.length} matching workflow(s):</strong>${matches.map(r=>`<div class="deal" style="margin-top:8px">Would ${describeWorkflowAction(r)}${r.notify?' and notify admins':''}</div>`).join('')}`:'<div class="empty">No active workflow matches these values.</div>';
 };
}
function rulesTab(body){
 if(ruleBuilderMode){renderRuleBuilder(body);return}
 const keys=[...Object.keys(numberRules),...activeCustomObjectKeys()];
 const actionFields=actionableFieldsFor(ruleEntity);
 const list=(data.fieldRules||[]).filter(r=>r.entity===ruleEntity);
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Business rules</h3><p class="muted">Require or hide a field - built-in or custom - based on any other field's value, with a real comparison operator, not just the status/stage field.</p>
 ${entityPills(keys,ruleEntity)}
 ${actionFields.length?`<div class="table-wrap"><table class="table"><thead><tr><th>When</th><th>Then</th><th>Field</th><th>Active</th><th>Actions</th></tr></thead><tbody>${list.map(r=>`<tr><td>${fieldLabelFor(r.entity,r.triggerField||transitionFieldFor(r.entity))} ${OPERATOR_LABELS[r.operator]||'is'}${operatorNeedsValue(r.operator)?' '+describeComparand(r.entity,r.compareField,r.triggerValue):''}</td><td>${r.effect==='require'?'Require':'Hide'}</td><td>${fieldLabelFor(r.entity,r.fieldKey)}</td><td>${badgeMaybe(r.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-edit-rule="${r.id}">Edit</button><button class="icon-btn" data-del-rule="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No business rules on '+entityLabel(ruleEntity)+' yet</div>'}</div><button class="btn btn-secondary" id="addRule" style="margin-top:14px">+ New rule</button>`:`<div class="empty">${entityLabel(ruleEntity)} has no field a rule can require/hide yet.</div>`}
 <p class="muted" style="margin-top:14px">This demo shows the core require/hide rule. The Windows desktop edition also supports multi-condition AND/OR rules and lock/set-value/block-save/show-message actions — see the <a href="/roadmap">roadmap</a>.</p>
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{ruleEntity=b.dataset.entity;renderAdminTab()});
 $('#addRule')?.addEventListener('click',()=>{ruleBuilderMode='create';renderAdminTab()});
 body.querySelectorAll('[data-edit-rule]').forEach(b=>b.onclick=()=>{ruleBuilderMode=b.dataset.editRule;renderAdminTab()});
 body.querySelectorAll('[data-del-rule]').forEach(b=>b.onclick=()=>{data.fieldRules=data.fieldRules.filter(r=>r.id!==b.dataset.delRule);save();toast('Rule deleted');renderAdminTab()});
}
// Rule-builder page (Phase-3-and-visual-redesign parity with the desktop
// edition's RuleForm): a numbered Condition/Effect layout with a live
// summary panel, replacing the old create-only modal - now also supports
// editing an existing rule, with Test rule and Activate/Deactivate moved
// into the header the same way the desktop redesign moved them.
function renderRuleBuilder(body){
 const isEdit=ruleBuilderMode!=='create';
 const existing=isEdit?data.fieldRules.find(r=>r.id===ruleBuilderMode):null;
 if(isEdit&&!existing){ruleBuilderMode=null;renderAdminTab();return}
 const entityKey=existing?existing.entity:ruleEntity;
 const condFields=conditionFieldsFor(entityKey);
 const actionFields=actionableFieldsFor(entityKey);
 const defaultField=existing?.triggerField||transitionFieldFor(entityKey);
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
    <div class="builder-section-title"><span class="step-badge">1</span> Condition</div>
    <div class="form-grid">
     <div class="field"><label>Field</label><select name="triggerField" id="condField">${condFields.map(f=>`<option value="${f[0]}" ${f[0]===defaultField?'selected':''}>${f[1]}</option>`).join('')}</select></div>
     <div id="condDynamic" style="display:contents">${conditionDynamicHtml(condFields,defaultField,existing?.operator||'equals',existing?.triggerValue||'',existing?.compareField||null)}</div>
    </div>
   </div>
   <div class="builder-section">
    <div class="builder-section-title"><span class="step-badge">2</span> Effect</div>
    <div class="form-grid">
     <div class="field"><label>Then</label><select name="targetField" id="ruleTargetField">${actionFields.map(f=>`<option value="${f[0]}" ${f[0]===existing?.fieldKey?'selected':''}>${f[1]}</option>`).join('')}</select></div>
     <div class="field"><label>Effect</label><select name="effect" id="ruleEffect"><option value="require" ${existing&&existing.effect==='hide'?'':'selected'}>Require the field</option><option value="hide" ${existing?.effect==='hide'?'selected':''}>Hide the field</option></select></div>
    </div>
   </div>
  </div>
  <div class="builder-summary-panel">
   <h4>Rule summary</h4>
   <div class="summary-row"><span class="label">Applies to</span><span class="value">${entityLabel(entityKey)}</span></div>
   <div class="summary-row"><span class="label">Watches</span><span class="value" id="summaryWatch">${fieldLabelFor(entityKey,defaultField)}</span></div>
   <div class="summary-row"><span class="label">Effect</span><span class="value" id="summaryEffect">${existing?.effect==='hide'?'Hide':'Require'}</span></div>
   <div class="summary-row"><span class="label">Field</span><span class="value" id="summaryField">${fieldLabelFor(entityKey,existing?.fieldKey||actionFields[0]?.[0]||'')}</span></div>
  </div>
 </div>
 <div style="margin-top:4px"><button type="button" class="btn btn-secondary" id="ruleBuilderCancel">Cancel</button></div>
 </form>`;
 const form=$('#ruleBuilderForm');
 wireConditionPicker(form,form.elements.triggerField,$('#condDynamic',form),condFields,'triggerValue');
 function updateSummary(){
  $('#summaryWatch').textContent=fieldLabelFor(entityKey,form.elements.triggerField.value);
  $('#summaryEffect').textContent=form.elements.effect.value==='hide'?'Hide':'Require';
  $('#summaryField').textContent=fieldLabelFor(entityKey,form.elements.targetField.value);
 }
 form.addEventListener('change',updateSummary);
 $('#ruleBuilderTest').onclick=()=>{testingRules=!testingRules;renderAdminTab()};
 if(testingRules)wireRuleTestPanel(entityKey);
 $('#ruleBuilderToggleActive')?.addEventListener('click',()=>{existing.active=!existing.active;save();toast(existing.active?'Rule activated':'Rule deactivated');renderAdminTab()});
 $('#ruleBuilderCancel').onclick=()=>{ruleBuilderMode=null;testingRules=false;renderAdminTab()};
 form.onsubmit=e=>{e.preventDefault();const fd=Object.fromEntries(new FormData(form).entries());
  const payload={entity:entityKey,triggerField:fd.triggerField,operator:fd.operator,triggerValue:fd.triggerValue||'',compareField:fd.compareField||null,fieldKey:fd.targetField,effect:fd.effect};
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
// Phase 3 action expansion: a workflow's "Then" is one of three action
// types - create a task (the original behavior), create a new record of
// a safely-constructible type, or update a field on a record related to
// the trigger via the demo's fixed foreign-key graph (REVERSE_RELATIONS).
function describeWorkflowAction(r){
 if(r.actionType==='create_record')return UNNAMED_RECORD_TYPES.includes(r.recordTargetEntity)?`create a new ${ENTITY_SINGULAR[r.recordTargetEntity]||r.recordTargetEntity}`:`create ${ENTITY_SINGULAR[r.recordTargetEntity]||r.recordTargetEntity} "${r.recordNameTemplate||''}"`;
 if(r.actionType==='update_related_record')return `set ${fieldLabelFor(r.relTargetEntity,r.relTargetField)} = "${r.relValue||''}" on related ${entityLabel(r.relTargetEntity)}`;
 return `create task "${r.taskTitle||''}" (${r.daysOffset?`due ${r.daysOffset} day(s) later`:'due same day'})`;
}
function relTargetsFor(entityKey){return [...new Set((RELATIONS[entityKey]||[]).map(x=>x.target))]}
// Executes a workflow rule's action against the record that just triggered
// it. Returns a short human description of what happened, for the
// notification message - or null when the action is a legitimate no-op
// (e.g. update_related_record with nothing linked yet, same as the
// desktop edition's "no-op when nothing is linked yet" semantics).
function executeWorkflowAction(r,key,record){
 if(r.actionType==='create_record'){
  const target=r.recordTargetEntity, name=r.recordNameTemplate;
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
 if(r.actionType==='update_related_record'){
  const rel=(RELATIONS[key]||[]).find(x=>x.target===r.relTargetEntity);
  if(!rel||!r.relTargetField)return null;
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
  linked.forEach(x=>{x[r.relTargetField]=r.relValue});
  return `set ${fieldLabelFor(targetEntity,r.relTargetField)} = "${r.relValue}" on ${linked.length} related ${entityLabel(targetEntity).toLowerCase()}`;
 }
 // create_task (default, and the only action type older saved data has)
 if(!r.taskTitle)return null;
 const due=new Date();due.setDate(due.getDate()+Number(r.daysOffset||0));
 // Custom objects aren't in relatedTypeFor (Tasks' relatedType dropdown is
 // a fixed built-ins-only list, matching desktop) - fall back to 'General'
 // rather than writing an unrecognized relatedType onto the created task.
 data.tasks.unshift({id:uid(),taskNumber:nextNumber('tasks'),title:r.taskTitle,relatedType:relatedTypeFor[key]||'General',relatedId:relatedTypeFor[key]?record.id:'',owner:record.owner||'Unassigned',due:due.toISOString().slice(0,10),priority:'Medium',status:'Open'});
 return `created task "${r.taskTitle}"`;
}
function workflowTab(body){
 if(wfBuilderMode){renderWorkflowBuilder(body);return}
 const keys=[...Object.keys(relatedTypeFor),...activeCustomObjectKeys()];
 const list=(data.workflowRules||[]).filter(r=>r.entity===wfEntity);
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Workflow automation</h3><p class="muted">Trigger an action - create a task, create a new record, or update a related record - when any built-in or custom field changes and matches a comparison you choose, and optionally notify admins.</p>
 ${entityPills(keys,wfEntity)}
 <div class="table-wrap"><table class="table"><thead><tr><th>When</th><th>Then</th><th>Notifies admins</th><th>Active</th><th>Actions</th></tr></thead><tbody>${list.map(r=>`<tr><td>${fieldLabelFor(r.entity,r.triggerField||transitionFieldFor(r.entity))} ${OPERATOR_LABELS[r.operator||'equals']}${operatorNeedsValue(r.operator||'equals')?' '+describeComparand(r.entity,r.compareField,r.toValue):''}</td><td>${describeWorkflowAction(r)}</td><td>${r.notify?'Yes':'No'}</td><td>${badgeMaybe(r.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-edit-wf="${r.id}">Edit</button><button class="icon-btn" data-del-wf="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No workflow rules on '+entityLabel(wfEntity)+' yet</div>'}</div>
 <button class="btn btn-secondary" id="addWf" style="margin-top:14px">+ New workflow rule</button>
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{wfEntity=b.dataset.entity;renderAdminTab()});
 $('#addWf').onclick=()=>{wfBuilderMode='create';renderAdminTab()};
 body.querySelectorAll('[data-edit-wf]').forEach(b=>b.onclick=()=>{wfBuilderMode=b.dataset.editWf;renderAdminTab()});
 body.querySelectorAll('[data-del-wf]').forEach(b=>b.onclick=()=>{data.workflowRules=data.workflowRules.filter(r=>r.id!==b.dataset.delWf);save();toast('Workflow rule deleted');renderAdminTab()});
}
const WORKFLOW_ACTION_TITLES={create_task:'Create task',create_record:'Create record',update_related_record:'Update related record'};
// Workflow-builder page: a Trigger/Action left column paired with a live
// visual canvas on the right (Trigger -> Action -> End), matching the
// desktop redesign's visual-flow language at the scale this demo's
// single-trigger/single-action model actually supports. Replaces the old
// create-only modal and adds editing, Test workflow and Activate/Deactivate
// in the header, same as the rule builder above.
function renderWorkflowBuilder(body){
 const isEdit=wfBuilderMode!=='create';
 const existing=isEdit?data.workflowRules.find(r=>r.id===wfBuilderMode):null;
 if(isEdit&&!existing){wfBuilderMode=null;renderAdminTab();return}
 const entityKey=existing?existing.entity:wfEntity;
 const condFields=conditionFieldsFor(entityKey);
 const defaultField=existing?.triggerField||transitionFieldFor(entityKey);
 const recordTargets=createRecordTargetsFor(entityKey);
 const relTargets=relTargetsFor(entityKey);
 const actionType=existing?.actionType||'create_task';
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
    <div class="builder-section-title"><span class="step-badge">1</span> Trigger</div>
    <div class="form-grid">
     <div class="field"><label>Watch field</label><select name="triggerField" id="wfField">${condFields.map(f=>`<option value="${f[0]}" ${f[0]===defaultField?'selected':''}>${f[1]}</option>`).join('')}</select></div>
     <div id="wfDynamic" style="display:contents">${conditionDynamicHtml(condFields,defaultField,existing?.operator||'equals',existing?.toValue||'',existing?.compareField||null).replaceAll('name="triggerValue"','name="toValue"')}</div>
    </div>
   </div>
   <div class="builder-section">
    <div class="builder-section-title"><span class="step-badge">2</span> Action</div>
    <div class="form-grid">
     <div class="field full"><label>Action</label><select name="actionType" id="wfActionType">
      <option value="create_task" ${actionType==='create_task'?'selected':''}>Create a task</option>
      <option value="create_record" ${actionType==='create_record'?'selected':''}>Create a new record</option>
      ${relTargets.length?`<option value="update_related_record" ${actionType==='update_related_record'?'selected':''}>Update a related record</option>`:''}
     </select></div>
     <div id="actionCreateTask" style="display:contents">
      <div class="field full"><label>Task title</label><input name="taskTitle" value="${existing?.taskTitle||''}" placeholder="e.g. Kick off onboarding"></div>
      <div class="field"><label>Due (days after change)</label><input name="daysOffset" type="number" min="0" value="${existing?.daysOffset??0}"></div>
     </div>
     <div id="actionCreateRecord" style="display:none">
      <div class="field"><label>Record type</label><select name="recordTargetEntity" id="wfRecordTargetEntity">${recordTargets.map(t=>`<option value="${t}" ${t===existing?.recordTargetEntity?'selected':''}>${entityLabel(t)}</option>`).join('')}</select></div>
      <div class="field full" id="wfRecordNameWrap"><label>Name / title</label><input name="recordNameTemplate" value="${existing?.recordNameTemplate||''}" placeholder="e.g. Renewal follow-up"></div>
     </div>
     <div id="actionUpdateRelated" style="display:none">
      <div class="field"><label>Related record type</label><select name="relTargetEntity" id="wfRelEntity">${relTargets.map(t=>`<option value="${t}" ${t===existing?.relTargetEntity?'selected':''}>${entityLabel(t)}</option>`).join('')}</select></div>
      <div class="field"><label>Field to set</label><select name="relTargetField" id="wfRelField"></select></div>
      <div class="field"><label>New value</label><input name="relValue" value="${existing?.relValue||''}" placeholder="New value to write"></div>
     </div>
     <div class="field"><label>Also notify admins?</label><select name="notify"><option value="false" ${!existing?.notify?'selected':''}>No</option><option value="true" ${existing?.notify?'selected':''}>Yes</option></select></div>
    </div>
   </div>
  </div>
  <div class="workflow-canvas-wrap">
   <div class="workflow-node workflow-node-trigger"><div class="workflow-node-head">Trigger</div><div class="workflow-node-body"><strong>${entityLabel(entityKey)}</strong><small id="canvasTrigger"></small></div></div>
   <div class="workflow-connector">▼</div>
   <div class="workflow-node workflow-node-actions"><div class="workflow-node-head">Action</div><div class="workflow-node-body" id="canvasAction"></div></div>
   <div class="workflow-connector">▼</div>
   <div class="workflow-end-node">END</div>
  </div>
 </div>
 <div style="margin-top:4px"><button type="button" class="btn btn-secondary" id="wfBuilderCancel">Cancel</button></div>
 </form>`;
 const form=$('#wfBuilderForm');
 wireConditionPicker(form,form.elements.triggerField,$('#wfDynamic',form),condFields,'toValue');
 const actionSelect=$('#wfActionType',form);
 function updateActionVisibility(){
  const v=actionSelect.value;
  $('#actionCreateTask',form).style.display=v==='create_task'?'contents':'none';
  $('#actionCreateRecord',form).style.display=v==='create_record'?'contents':'none';
  $('#actionUpdateRelated',form).style.display=v==='update_related_record'?'contents':'none';
 }
 const relEntitySelect=form.elements.relTargetEntity;
 function populateRelField(){if(!relEntitySelect)return;const fields=actionableFieldsFor(relEntitySelect.value);$('#wfRelField',form).innerHTML=fields.map(f=>`<option value="${f[0]}" ${f[0]===existing?.relTargetField?'selected':''}>${f[1]}</option>`).join('')}
 // Quotes/orders/invoices have no name/title field to fill in from a
 // template (see UNNAMED_RECORD_TYPES) - hide that input for them so the
 // builder never asks for something the created record can't store.
 const recordTargetSelect=$('#wfRecordTargetEntity',form), recordNameWrap=$('#wfRecordNameWrap',form);
 function updateRecordNameVisibility(){if(!recordTargetSelect)return;recordNameWrap.style.display=UNNAMED_RECORD_TYPES.includes(recordTargetSelect.value)?'none':''}
 if(recordTargetSelect){updateRecordNameVisibility();recordTargetSelect.onchange=()=>{updateRecordNameVisibility();updateCanvas()}}
 // The live canvas mirrors whatever the form currently says - re-derived
 // from the form's own values on every change, the same "what will
 // actually happen" summary the desktop canvas gives at a glance.
 function updateCanvas(){
  const tf=form.elements.triggerField.value, op=form.elements.operator?.value||'equals';
  const needsValue=operatorNeedsValue(op);
  const val=form.elements.toValue?.value||(form.elements.compareField?fieldLabelFor(entityKey,form.elements.compareField.value):'');
  $('#canvasTrigger').textContent=`When ${fieldLabelFor(entityKey,tf)} ${OPERATOR_LABELS[op]||'is'}${needsValue?' '+(val||'…'):''}`;
  const fd=Object.fromEntries(new FormData(form).entries());
  fd.daysOffset=Number(fd.daysOffset||0); // FormData gives strings - "0" is truthy, so coerce before describeWorkflowAction's truthy check
  $('#canvasAction').innerHTML=`<strong>${WORKFLOW_ACTION_TITLES[fd.actionType]||'Action'}</strong><small>${describeWorkflowAction(fd)}</small>`;
 }
 actionSelect.onchange=()=>{updateActionVisibility();updateCanvas()};
 updateActionVisibility();
 if(relEntitySelect){populateRelField();relEntitySelect.onchange=()=>{populateRelField();updateCanvas()}}
 form.addEventListener('change',updateCanvas);
 form.addEventListener('input',updateCanvas);
 updateCanvas();
 $('#wfBuilderTest').onclick=()=>{testingWorkflow=!testingWorkflow;renderAdminTab()};
 if(testingWorkflow)wireWorkflowTestPanel(entityKey);
 $('#wfBuilderToggleActive')?.addEventListener('click',()=>{existing.active=!existing.active;save();toast(existing.active?'Rule activated':'Rule deactivated');renderAdminTab()});
 $('#wfBuilderCancel').onclick=()=>{wfBuilderMode=null;testingWorkflow=false;renderAdminTab()};
 form.onsubmit=e=>{e.preventDefault();const fd=Object.fromEntries(new FormData(form).entries());
  if(fd.actionType==='create_task'&&!fd.taskTitle)return alert('Enter a task title.');
  if(fd.actionType==='create_record'&&!UNNAMED_RECORD_TYPES.includes(fd.recordTargetEntity)&&!fd.recordNameTemplate)return alert('Enter a name/title for the new record.');
  if(fd.actionType==='update_related_record'&&!fd.relValue)return alert('Enter the value to write.');
  const payload={
   entity:entityKey,triggerField:fd.triggerField,operator:fd.operator,toValue:fd.toValue||'',
   compareField:fd.compareField||null,notify:fd.notify==='true',actionType:fd.actionType,
   taskTitle:fd.taskTitle||'',daysOffset:Number(fd.daysOffset||0),
   recordTargetEntity:fd.recordTargetEntity||'',recordNameTemplate:fd.recordNameTemplate||'',
   relTargetEntity:fd.relTargetEntity||'',relTargetField:fd.relTargetField||'',relValue:fd.relValue||'',
  };
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
  ['Richer Business Rules engine — Phase C','Multi-condition AND/OR matching, 10 comparison operators, and 7 action types (require, hide, lock, set default, set exact value, block save, show a message), plus rule priority and optional effective-date windows.'],
  ['Richer Workflow Automation engine — Phase D','7 trigger types (created/updated, status/field changed, date reached, due/overdue, scheduled) and 6 action types (create task, update field, assign owner, create related record, add notification, create reminder), plus an in-app notification center.'],
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
 ];
 const planned=[
  ['Admin UX polish','Duplicate/copy for a rule or workflow, version history, and a dependency warning before deactivating something another rule or field relies on — the last scoped item in the Admin Automation & Customization addendum.','spec §10','S'],
  ['Global search & list-view filtering','The desktop app has neither today — a pre-existing gap, more visible now that Phase E added is_searchable/is_filterable capability flags to custom fields that currently do nothing. Building this feature gives both flags their first real use. (The online demo already has its own simple ⌘K search, unaffected.)','spec §5.3/§9.3','M'],
 ];
 const proposed=[
  ['No-code Screen/UI Designer','A drag-ordered layout builder for create/edit/detail screens (sections, tabs, columns, field placement) with Draft → Preview → Publish, so a layout change never breaks the workspace mid-edit. The single largest remaining piece of the admin extensibility spec.','It\'s a substantial standalone UI project, not an extension of an existing screen — and it\'s most valuable once there are Custom Objects/Relationships to build layouts for, which now exist. Least urgent since today\'s auto-generated forms already work.','ADM-UI','L'],
  ['Full drag-and-drop report builder','The shipped report builder covers pick-an-object → group-by-field (including custom fields) → count or sum. A richer builder — multiple group-bys, filters, joins across objects, a visual canvas — was scoped down to that simpler version by explicit choice.','Worth revisiting once real usage shows the count/sum + single group-by shape is genuinely too narrow.',null,'M–L'],
  ['Code-signed Windows installer','The published installer is unsigned, so Windows SmartScreen flags it as an unknown publisher.','Mostly not a coding task: buy a certificate, add a signtool step to the release workflow. The real cost is procurement — identity verification lead time, a recurring fee — an ops/budget decision, not an engineering one.',null,'S (code) / ops-heavy'],
 ];
 const futureIdeas=['Projects and milestones','Inventory and suppliers','Recurring invoices','Customer portal','Plugin architecture'];
 const sequence=[
  ['1 · Admin UX polish','The last scoped item in the Admin Automation & Customization addendum — small and well-defined, a clean next build.',true],
  ['2 · Global search & list-view filtering','Gives the existing is_searchable/is_filterable flags their first real use.',false],
  ['3 · Decide: Screen Designer, richer report builder, code signing','Three independent scope/budget calls, worth a deliberate conversation each rather than defaulting into months of work.',false],
 ];
 $('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Built from the working codebase</div><h1>Roadmap & backlog.</h1><p>Where Lanesra OS stands today, what's being built next, and everything queued up after that — compiled from the actual code (core/server/src-tauri/frontend) plus the online demo, not a wishlist. Every "Shipped" line below is running code with tests.</p><div class="status-row"><span class="status-chip">Early Access v0.22.0</span><span class="muted">Last updated August 2026</span></div>
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
function releasesPage(){document.title='Releases — Lanesra OS';const releases=[['v0.22.0','August 2026','No-code Screen layouts in the online demo',['A new Admin → Screen layouts tab lets an admin drag-order any built-in or custom object\'s create/edit fields into named sections - a capability that doesn\'t exist on the desktop edition either, built for the demo first','A layout has a draft and a published copy: dragging fields between/within sections, renaming a section, or adding one only ever edits the draft - the live create/edit form keeps using the plain default field order until Publish copies the draft over, and Unpublish clears it straight back to that default','A Preview button renders the draft exactly as the live form would show it, without saving anything or touching the live workspace','A published layout never drops a field it doesn\'t recognize - any field missing from the layout (a new custom field added after publishing, or a stale key) is automatically appended to a trailing "Other fields" section so nothing can go missing from a form because of a layout edit','With this, the online demo\'s only remaining gap against the desktop edition\'s admin extensibility spec is the Integrations admin section - up next, also a demo-first UI-only build']],['v0.21.0','August 2026','Reports in the online demo',['The online demo has a new top-level Reports section with the same fixed report gallery the desktop edition ships: Revenue by month, Win rate by owner, Lost reasons, AR aging and Sales by owner, each with a date-range (or "as of") filter and a bar-chart table matching desktop\'s layout','Added a Custom Reports builder to the demo, mirroring desktop\'s admin report builder: pick any object - built-in or a Custom Object - group by its status/stage or an active, reportable custom field, and count records or sum a numeric custom field; only fields an admin flagged Reportable are offered, same as desktop','Added CSV export to every report in the demo (a new capability for the demo generally, self-contained via a Blob download - no server involved)','Two disclosed substitutions where this demo\'s simpler data model doesn\'t match desktop\'s: Revenue by month/Sales by owner group by each invoice\'s due date since the demo has no separate issue date, and AR aging uses each invoice\'s full total as its balance since the demo doesn\'t track partial payments - both called out in the report\'s own subtitle rather than silently faked','Added a Lost reason field to Opportunities so the Lost Reasons report has something real to report on','With this, the online demo\'s only remaining gaps against the desktop edition are two capabilities that don\'t exist on desktop either: a no-code UI layout designer and a UI-only Integrations admin section - both underway next']],['v0.20.0','August 2026','Custom Relationships in the online demo',['An Administrator can now connect any two object types - built-in or custom - from Admin → Relationships: a cardinality (many-to-one, one-to-one or many-to-many), a forward/reverse label pair, and a choice of what happens to a link when a linked record is deleted (Restrict blocks the delete, Archive drops the link and keeps both records)','Every record\'s edit form now shows a "Related records" panel listing every linked record across every applicable relationship, from either direction, with inline Link/Unlink - the same place desktop puts its related-records card, since most objects in this demo have an edit form but no separate detail page','Cardinality is enforced on link (a many-to-one or one-to-one side can\'t be linked twice), and a relationship can\'t connect an object type to itself - both match the desktop edition\'s validation exactly','Reports beyond the dashboard remain the one desktop-only capability left in the online demo\'s parity work']],['v0.19.0','August 2026','Custom Objects in the online demo',['The online demo caught up with one of the desktop edition\'s biggest capabilities: an Administrator can now define a whole new business object at runtime - Vendors, Assets, Projects - from Admin → Custom Objects, with its own icon, sidebar entry and record-number prefix/digit width, no code change','A custom object is a full citizen of the demo\'s admin subsystems exactly like a built-in entity: it gets its own tab in Custom Fields, Business Rules, Status Transitions and Workflow Automation, and its records go through the same create/edit/list screens, auto-numbering and delete-dependency checks as Companies or Contacts','A custom object can\'t be named the same as a built-in entity (rejected at creation, matching desktop); deleting its definition is blocked while any record still exists, while deactivating is always safe and reversible since it only hides the object from navigation and new-record creation','Reports beyond the dashboard and Custom Relationships between record types remain desktop-only for now - next up in the online demo\'s parity work']],['v0.18.1','August 2026','Online demo parity fixes',['Workflow "Create a new record" now offers all 9 built-in record types in the online demo (companies, contacts, opportunities, products, quotes, orders, invoices, contracts, tasks), not just 3 - matching the desktop edition\'s full creatable set, with company-dependent types offered only when the trigger record actually carries a company','Workflow "Update a related record" now walks the demo\'s foreign-key graph in both directions, not just downward: a Contact can update its own parent Company the same way a Company can update its Contacts, and every entity gets its linked Tasks (and vice versa) through the existing relatedType/relatedId link','Custom fields in the online demo gained the validation and capability settings the desktop edition already had: Required, Max length and Pattern/regex (text fields), Min/Max value (number fields), and Searchable/Filterable/Reportable flags - enforced with native HTML5 form validation and shown in the custom fields list']],['v0.18.0','August 2026','Status transitions, richer workflow actions, test mode, a rule-builder redesign, and Customer 360',['Added a Status Transition Editor: restrict which status/stage changes are allowed on any object with a fixed-schema field (companies, contacts, opportunities, products, quotes, orders, invoices, contracts, tasks) - each rule is one from → to move, with a wildcard "any status" starting point and its own active toggle; with no active rules a field stays fully unrestricted, and resaving the same status is never blocked','Workflow automation actions expanded beyond "create a task": a workflow can now create a new record (a company, opportunity or task) or update a field on a record related to the trigger through the demo\'s existing company/contact/opportunity/quote/order relationships','Added a Test rule / Test workflow dry-run mode to both Business Rules and Workflow Automation: fill in hypothetical values for an object and see exactly which active rules or workflows would match and what they would do, without creating, changing or sending anything','Redesigned the Business Rules and Workflow Automation builders to match the desktop edition\'s rule-builder layout: numbered Condition/Effect (or Trigger/Action) sections, a live-updating rule summary panel, and - for workflows - a visual Trigger → Action → End canvas that mirrors the form as you edit it; both builders gained full editing (not just create) and header-level Test/Activate-Deactivate/Save controls','Custom fields gained four more settings: an optional default value applied whenever a save leaves the field empty, a "require a unique value" check (rejected at definition time for yes/no fields, since they only have two possible values), placeholder text, and help text shown under the field on every record form','Added Customer 360 and Contact 360: clicking a company or contact name anywhere in the app now opens a dedicated detail page with its full field overview and every linked record - contacts, opportunities, quotes, orders, invoices, contracts and tasks - each one click away, replacing edit-modal-only access','Fixed a pre-existing bug surfaced while building the above: the admin panel\'s tab row no longer freezes on whichever tab was open first - switching tabs now correctly highlights the one you\'re on']],['v0.17.0','August 2026','More operators & field-to-field comparison',['Business rules and workflows gained four more comparison operators - starts with, ends with, is one of, is not one of - on top of is/is not/contains/is empty/is not empty/greater than/less than','A condition can now compare a field against another field\'s live value instead of only a fixed value - e.g. "require Flag when Notes equals Expected Notes" - with the same live-updating preview on the record form that a fixed-value condition already had','Windows desktop edition: the shared condition engine gained the same operators and field-to-field comparison, for both business rule conditions and workflow triggers']],['v0.16.0','August 2026','Business rules & workflows now work on any field, not just status',['Business rules can now condition on any built-in field - name, industry, value, close date, whatever the object has - not only the status/stage field, with a real comparison operator (is/is not, contains, is empty, greater than, less than) chosen per field, and their require/hide action can now target a built-in field too, not just a custom one','Workflow automation\'s field-changed trigger can now watch any built-in field the same way, so "when Industry changes to X" or "when Due date is set" can create a task and notify admins, not only "when status/stage reaches a value"','Windows desktop edition: the underlying business rules and workflow engines gained the same any-built-in-field support for both conditions/triggers and actions (require, hide, lock, set default, force value, and the workflow update-field action), writing through each entity\'s own validation so nothing bypasses existing rules']],['v0.15.0','August 2026','Custom relationships, richer business rules & workflow automation',['Added admin-defined custom relationships between any two record types (companies, contacts, custom objects, and more), with one-to-one/many-to-one/many-to-many cardinality and a choice of what happens to linked records on delete','Added a related-records view on record detail pages showing every linked record through those relationships','Replaced the business rules engine: rules can now combine multiple conditions with AND/OR, use 10 comparison operators (not just equals), and lock a field, set a default or exact value, block saving entirely, or show a message — not just require or hide','Replaced the workflow automation engine: triggers now include field changes and dates reached/overdue in addition to status changes, and actions include assigning the record\'s owner, creating a related record, and posting an in-app notification, on top of creating a task','Added an in-app notification center (bell icon with unread count) for workflow-triggered notifications','Added optional validation for custom fields — a min/max range for number fields, a max length and regex pattern for text fields — plus searchable/filterable/reportable capability flags','Added Windows task reminder notifications (native toast notifications via the desktop app\'s webview)','Added a session inactivity auto-lock (15 minutes idle) requiring the current user\'s password to resume','Updated the online demo: business rules now support an "is / is not" operator, and workflow rules can optionally post an admin notification, shown in a new notification bell']],['v0.14.0','August 2026','Admin-defined Custom Objects',['Added Custom Objects: an Administrator can define an entirely new record type (its own label, fields and ID/numbering format) without any code changes','Custom Objects automatically get their own navigation section, and are full citizens of the existing custom fields, business rules and custom report builder — no per-object code was needed for any of the three','A custom object can\'t be named the same as a built-in entity, and deleting its definition is blocked while records exist (deactivating it is always safe and non-destructive)']],['v0.13.0','August 2026','Admin panel: users, roles & flexible configuration everywhere',['Added an Admin panel with user & role management, moved out of the main navigation into one dedicated section','Added an editable business profile (name, phone, address, city, logo) shown across the workspace','Generalized custom fields from Companies/Contacts to every major object: Opportunities, Quotes, Orders, Invoices, Contracts, Products and Tasks','Generalized conditional business rules and workflow automation the same way, so any object with custom fields can use them','Added admin-configurable numbering: choose the prefix and digit width used for each object\'s auto-generated ID (e.g. "ACC-000001" or "ACC-ab0001")','Added a simple custom report builder: pick an object, group by any field including custom fields, and count or sum','Added a dashboard KPI picker so admins choose which tiles show, in what selection, for the whole workspace','Updated the online demo with a full working Admin panel — mirrors every feature above in the browser']],['v0.12.0','August 2026','Branding, reports, custom fields, business rules & workflow automation',['Added business branding (logo, editable business profile) shown on the print letterhead for quotes, orders and invoices','Added reports beyond the dashboard: revenue by month, win rate by owner, lost reasons, AR aging and sales by owner','Added admin-defined custom fields on Companies and Contacts (text, number, date, yes/no, select), enforced both client- and server-side','Added conditional business rules that require or hide a custom field based on a record\'s status','Added Phase 1 workflow automation: auto-create a follow-up task when an Opportunity\'s stage or an Invoice\'s status changes']],['v0.11.0','August 2026','PDF printing & CSV import/export',['Added a browser-native "Print / Save as PDF" preview for quotes, orders and invoices, with business letterhead, line items and totals','Added CSV export on every list screen','Added CSV import for Companies and Contacts, validated row by row through the same rules as the manual forms']],['v0.10.0','August 2026','Team Workspace, backup & restore',['Added Team Workspace mode — a small team shares one server over the local network from browser tabs, with per-user sessions','Added whole-workspace backup and restore as a single file, safe to run against a live database','Added self-service password change from a "My account" screen']],['v0.9.0','August 2026','Desktop edition foundation published',['Published the Windows desktop edition source: Tauri v2 + Rust + SQLite','Implemented the full sales lifecycle on desktop — Companies, Contacts, Products, Opportunities, Quotes, Orders and Invoices','Added quote-to-order and order-to-invoice conversion, atomic document numbering and local user authentication','No packaged installer yet — desktop is available to build and run from source']],['v0.8.0','August 2026','Interactive navigation & public pages',['Made dashboard KPIs clickable with filtered drill-downs','Added a global Quick Create menu','Added mobile navigation while keeping Try Online prominent','Replaced Journey with Principles and added Compare and Download pages','Marked desktop downloads as Coming Soon','Fixed desktop sidebar navigation']],['v0.7.0','August 2026','Trust & product transparency',['Added Roadmap, Changelog and creator attribution','Added Person JSON-LD and updated discovery files']],['v0.6.0','August 2026','Record numbering & search',['Added automatically generated identifiers','Rebuilt global search as one stable result panel','Added keyboard shortcuts and wider search coverage']],['v0.5.0','July 2026','Lanesra OS rebrand',['Renamed BusinessOS to Lanesra OS','Updated product branding, metadata and documentation']],['v0.4.0','July 2026','Relationship integrity',['Added opportunity-to-contact relationship','Removed opportunity-to-contract relationship','Added company-filtered relationship dropdowns']],['v0.3.0','July 2026','Flexible sales flow',['Made opportunities optional for quotes','Made quotes optional for orders','Added products, services and line-item quantities']],['v0.2.0','June 2026','Connected sales MVP',['Added quotes, orders, invoices, contracts and dashboards','Connected core entities using clean relationships']],['v0.1.0','May 2026','First working prototype',['Launched the first browser-based MVP with sample data']]];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Release history</div><h1>Releases</h1><p>Every meaningful improvement to Lanesra OS, with detailed per-version release notes — documented publicly.</p><div class="status-row"><span class="status-chip">Latest: v0.22.0</span><span class="muted">Early Access</span></div></div></section><section class="section"><div class="container changelog-list">${releases.map(r=>`<article class="release" id="${r[0].replaceAll('.','-')}"><div class="release-meta"><span class="status-chip">${r[0]}</span><span>${r[1]}</span></div><div><h2>${r[2]}</h2><ul>${r[3].map(x=>`<li>${x}</li>`).join('')}</ul></div></article>`).join('')}</div></section></main>${publicFooter()}`;bindPublicNav()}
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
