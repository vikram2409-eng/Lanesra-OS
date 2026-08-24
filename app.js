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
// App Builder: which app (data.apps entry, if any) the sidebar App
// Switcher currently has selected - `null` is "All", the pre-App-Builder
// sidebar with every section visible. See navSections()/activeApp() below.
let activeAppId=null;
const save=()=>localStorage.setItem(storeKey,JSON.stringify(data));
const uid=()=>Math.random().toString(36).slice(2,10);
const pad=(n,w=4)=>String(n).padStart(w,'0');
const year=()=>new Date().getFullYear();
// Audit trail: mirrors the desktop app's created_at/created_by/updated_at/
// updated_by on every entity, built-in or custom-defined. The demo has no
// real sign-in, so CURRENT_USER_ID stands in for "whoever has this browser
// tab open" - the same identity the sidebar's "MC" avatar already implies
// (Maya Chen, the workspace's seed Administrator, u1).
const CURRENT_USER_ID='u1';
function userName(id){return (data.users||[]).find(u=>u.id===id)?.name||'System'}
// Stamps a freshly created record with created_at/created_by AND
// updated_at/updated_by set equal, exactly like every desktop repository's
// INSERT statement (see e.g. custom_record_repo::create).
function stampCreate(obj){const now=new Date().toISOString();obj.createdAt=now;obj.createdBy=CURRENT_USER_ID;obj.updatedAt=now;obj.updatedBy=CURRENT_USER_ID;return obj}
// Refreshes only updated_at/updated_by on a save - created_at/created_by
// are never touched again after a record's first save (mirrors every
// desktop UPDATE statement, including numbering_override_repo::upsert's ON
// CONFLICT branch, which is the one place this demo also upserts rather
// than always inserting).
function stampUpdate(obj){obj.updatedAt=new Date().toISOString();obj.updatedBy=CURRENT_USER_ID;return obj}
// "Created by X on ... · Last updated by Y on ..." - same convention as
// the desktop app's AuditByline component. Renders nothing for anything
// that predates this feature (or was never stamped, like a User) rather
// than showing a misleading blank line.
function auditByline(r){
 if(!r||!r.createdAt)return '';
 const created=new Date(r.createdAt).toLocaleString();
 let html=`Created by ${userName(r.createdBy)} on ${created}`;
 if(r.updatedAt&&r.updatedAt!==r.createdAt)html+=` · Last updated by ${userName(r.updatedBy)} on ${new Date(r.updatedAt).toLocaleString()}`;
 return `<div class="muted" style="font-size:12px;margin-top:4px">${html}</div>`;
}
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
 if(!data.savedViews)data.savedViews=[];
 if(!data.relationshipDefinitions)data.relationshipDefinitions=[];
 if(!data.relationshipInstances)data.relationshipInstances=[];
 (data.relationshipDefinitions||[]).forEach(d=>{if(d.active===undefined)d.active=true;if(d.showRelatedList===undefined)d.showRelatedList=true;if(d.required===undefined)d.required=false});
 if(!data.customReports)data.customReports=[];
 if(!data.uiLayouts)data.uiLayouts={};
 if(!data.dashboards||!data.dashboards.length)data.dashboards=[freshDashboard('Default',true)];
 if(!data.dashboards.some(d=>d.isDefault))data.dashboards[0].isDefault=true;
 if(!data.apps)data.apps=[];
 (data.apps||[]).forEach(a=>{if(!a.permissions)a.permissions=[];if(!a.objectKeys)a.objectKeys=[];if(a.description===undefined)a.description=''});
 // Industry Data Model / App Catalog - see REFERENCE_PACKAGES below.
 // appPackages is the "imported, not yet installed" catalog;
 // installedApps records what installReferencePackage actually created, so
 // it can be deactivated/reactivated as one unit without re-parsing the
 // package definition.
 if(!data.appPackages)data.appPackages=[];
 if(!data.installedApps)data.installedApps=[];
 (data.installedApps||[]).forEach(a=>{if(a.status===undefined)a.status='active';if(!a.artifacts)a.artifacts=[]});
 // Solution Management Phase 3 mirror: component-tagging's registry -
 // see tagLocalComponent/retagComponent's own comment.
 if(!data.components)data.components=[];
 // Solution Management Phase 4 mirror: named, scoped Solutions - see
 // createSolution's own comment.
 if(!data.solutions)data.solutions=[];
 // Solution Management Phase 2 mirror: a real Publisher registry, same
 // as the desktop edition's migration 0029 - see ensureDefaultPublishers.
 // Idempotent and called here (every load, including "Reset demo") so a
 // save from before this feature existed self-heals with no separate
 // migration step.
 ensureDefaultPublishers();
 if(!data.integrationJobs)data.integrationJobs=[];
 if(!data.apiEndpoints)data.apiEndpoints=[];
 if(!data.externalConnections)data.externalConnections=[];
 // Integration Hub demo, Webhooks & Events - see webhooksSubTab's own comment.
 if(!data.webhooks)data.webhooks=[];
 (data.integrationJobs||[]).forEach(j=>{if(j.active===undefined)j.active=true;if(!j.runs)j.runs=[]});
 (data.apiEndpoints||[]).forEach(e=>{if(e.active===undefined)e.active=true});
 (data.externalConnections||[]).forEach(c=>{if(c.active===undefined)c.active=true;if(!c.calls)c.calls=[]});
 (data.webhooks||[]).forEach(w=>{if(w.active===undefined)w.active=true;if(!w.deliveries)w.deliveries=[]});
 (data.fieldRules||[]).forEach(migrateFieldRule);
 (data.workflowRules||[]).forEach(migrateWorkflowRule);
 (data.customFields||[]).forEach(f=>{if(f.defaultValue===undefined)f.defaultValue='';if(f.unique===undefined)f.unique=false;if(f.helpText===undefined)f.helpText='';if(f.placeholder===undefined)f.placeholder='';if(f.required===undefined)f.required=false;if(f.maxLength===undefined)f.maxLength=null;if(f.pattern===undefined)f.pattern='';if(f.minValue===undefined)f.minValue='';if(f.maxValue===undefined)f.maxValue='';if(f.searchable===undefined)f.searchable=false;if(f.filterable===undefined)f.filterable=false;if(f.reportable===undefined)f.reportable=true;if(f.hiddenByDefault===undefined)f.hiddenByDefault=false});
 // Audit trail: backfill created_at/created_by/updated_at/updated_by on
 // anything saved (or seeded) before this feature existed - additive and
 // idempotent, same as every other migration step in this function. Not
 // hardcoded against numberRules (that const isn't defined yet the first
 // time this runs, at module load) - the built-in keys are spelled out
 // directly instead.
 const AUDITED_BUILTIN_KEYS=['companies','contacts','opportunities','products','quotes','orders','invoices','contracts','tasks'];
 [...AUDITED_BUILTIN_KEYS.map(k=>data[k]),...((data.customObjects||[]).map(o=>data[o.key]))].forEach(arr=>(arr||[]).forEach(r=>{if(!r.createdAt)stampCreate(r)}));
 (data.customFields||[]).forEach(r=>{if(!r.createdAt)stampCreate(r)});
 (data.fieldRules||[]).forEach(r=>{if(!r.createdAt)stampCreate(r)});
 (data.workflowRules||[]).forEach(r=>{if(!r.createdAt)stampCreate(r)});
 (data.statusTransitionRules||[]).forEach(r=>{if(!r.createdAt)stampCreate(r)});
 (data.customReports||[]).forEach(r=>{if(!r.createdAt)stampCreate(r)});
 (data.relationshipDefinitions||[]).forEach(r=>{if(!r.createdAt)stampCreate(r)});
 // Relationship instances only ever get created_by (see the desktop
 // RelationshipInstance model's own comment - they're never updated, only
 // created or deleted).
 (data.relationshipInstances||[]).forEach(r=>{if(!r.createdAt){r.createdAt=new Date().toISOString();r.createdBy=CURRENT_USER_ID}});
 Object.values(data.numberingOverrides||{}).forEach(o=>{if(!o.createdAt)stampCreate(o)});
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
// 'landing' shows the categorized Admin home; 'tool' shows one builder
// (whichever adminTab points at) with a breadcrumb back to that home.
// Clicking the sidebar Admin icon always resets to 'landing' - same as
// Setup Home in Salesforce - a deep link into a specific tool sets 'tool'
// directly instead (see adminCategoryItemClick).
let adminView='landing';
const ADMIN_TAB_DEFS=[['profile','Business profile'],['users','Users & roles'],['objects','Custom Objects'],['relationships','Relationships'],['fields','Custom fields'],['rules','Business rules'],['workflow','Workflow automation'],['transitions','Status transitions'],['layouts','Screen layouts'],['apps','Apps'],['packages','App Catalog'],['solutions','Solution Management'],['integrations','Integrations'],['numbering','Numbering'],['kpis','Dashboard KPIs'],['dashboards','Dashboards']];
// Regrouped along the same lines as the desktop edition's Admin IA
// reshuffle (Settings.tsx ADMIN_CATEGORIES) - Data Model/Experience split
// out of the old flat "Customization", Analytics split out of
// "Workspace", and a new Solution Management category. Integrations
// stays its own category (demo-only, no desktop equivalent) rather than
// folding into anything else.
const ADMIN_CATEGORIES=[
 {key:'workspace',label:'Workspace',icon:'⚙',note:'How the workspace looks and is identified',items:['profile','numbering']},
 {key:'access',label:'Access',icon:'👤',note:'Who can sign in and what they can do',items:['users']},
 {key:'data-model',label:'Data Model',icon:'🧩',note:'Objects, relationships and fields',items:['objects','relationships','fields']},
 {key:'experience',label:'Experience',icon:'▦',note:'How records look on screen',items:['layouts']},
 {key:'automation',label:'Automation',icon:'⚡',note:'Rules and workflows that run themselves',items:['rules','workflow','transitions']},
 {key:'apps',label:'Apps',icon:'⬡',note:'Package objects into a focused app, or install one ready-made',items:['apps','packages']},
 {key:'analytics',label:'Analytics',icon:'📊',note:'What shows on the dashboard',items:['kpis','dashboards']},
 {key:'solutions',label:'Solution Management',icon:'🗂',note:"What's installed, what it created, and who published it",items:['solutions']},
 {key:'integrations',label:'Integrations',icon:'🔌',note:'Connect this workspace to other systems',items:['integrations']},
];
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
// Per-app scoped automation: which app's rules/workflows/dashboards the
// list view is filtered to - 'all' | 'none' (workspace-wide only) | an
// app id. Mirrors the desktop edition's AppScopeFilter state.
let ruleAppFilter='all';
let wfAppFilter='all';
let dashAppFilter='all';
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

// Dashboard customization (mirrors the desktop edition's dashboard_layout_
// service/dashboard_widget_service, all 3 phases at once): named dashboard
// layouts, each an ordered list of widgets, assigned to roles with a
// required Default fallback, and Draft -> Publish - same shape as Screen/
// App Builder's own layouts above, just at the workspace level instead of
// per entity_type. No dashboard published yet (the common case) leaves the
// live Dashboard rendering exactly as it did before this feature existed,
// driven by the older workspace-wide kpiPrefs selection above.
function freshDashboardWidget(kind,config){return {id:uid(),kind,config}}
function freshDashboard(name,isDefault,appId){return {id:uid(),name,isDefault,roles:[],draftWidgets:[],publishedWidgets:null,appId:appId||null}}
function ensureDashboards(){
 if(!data.dashboards||!data.dashboards.length)data.dashboards=[freshDashboard('Default',true)];
 if(!data.dashboards.some(d=>d.isDefault))data.dashboards[0].isDefault=true;
 return data.dashboards;
}
function defaultDashboard(){const arr=ensureDashboards();return arr.find(d=>d.isDefault)||arr[0]}
function dashboardById(id){return ensureDashboards().find(d=>d.id===id)}
// There's no signed-in user in this browser demo (same note layoutsTab's
// own doc comment makes) - the live Dashboard always uses the Default
// dashboard's published widgets, if any; role assignment on non-default
// dashboards here is illustrative only, same as Users & roles.
function effectiveDashboardWidgets(){return defaultDashboard().publishedWidgets}

// Phase 3's per-entity "title" field, mirroring entity_registry's
// per-entity match arms in the Rust core - the field a record-list widget
// row shows as its title. Falls back to 'name' for custom objects, whose
// records all share that same primary field (see customObjectFields).
const RECORD_TITLE_FIELD={companies:'name',contacts:'name',opportunities:'title',products:'name',quotes:'number',orders:'number',invoices:'number',contracts:'title',tasks:'title'};
function recordTitle(entityKey,r){return r[RECORD_TITLE_FIELD[entityKey]||'name']||r.id}
// Only Tasks and Invoices carry a real due date to sort "due soon" by -
// every other entity type's due_soon request falls back to "recent",
// mirroring dashboard_widget_service::run's identical scoping.
const DUE_SOON_ENTITY_KEYS=['tasks','invoices'];
function dashboardRecordListRows(entityKey,mode,limit){
 limit=Math.min(10,Math.max(1,Number(limit)||5));
 const arr=data[entityKey]||[];
 if(mode==='due_soon'&&entityKey==='tasks'){
  return arr.filter(t=>t.due&&!['Completed','Cancelled'].includes(t.status))
   .slice().sort((a,b)=>(a.due||'').localeCompare(b.due||''))
   .slice(0,limit).map(t=>({id:t.id,title:t.title,subtitle:`Due ${t.due}`}));
 }
 if(mode==='due_soon'&&entityKey==='invoices'){
  return arr.filter(i=>i.due&&!['Paid','Cancelled'].includes(i.status))
   .slice().sort((a,b)=>(a.due||'').localeCompare(b.due||''))
   .slice(0,limit).map(i=>({id:i.id,title:i.number||i.id,subtitle:`Due ${i.due}`}));
 }
 // "recent" (or due_soon requested for a type with no due date to sort
 // by): newest first. This demo has no created-at timestamp on every
 // record, so insertion order (new records are always appended) stands
 // in for it - a "close enough for a browser demo" approximation, the
 // same kind the fixed reports above already disclose for a few of their
 // own date fields.
 return arr.slice(-limit).reverse().map(r=>({id:r.id,title:recordTitle(entityKey,r),subtitle:null}));
}

function landing(){
 document.title='Lanesra OS — Open-source CRM, app platform & integration hub';
 $('#app').innerHTML=`
 ${publicNav()}
 <main>
 <section class="hero"><div class="container hero-grid"><div><div class="eyebrow">An open-source business application platform</div><h1>A complete CRM out of the box. A platform to build, package and connect whatever's next.</h1><p>Lanesra OS gives you one modern workspace for customers, opportunities, products, quotes, orders, invoices, contracts and daily follow-ups — built from the same admin-configurable platform available to you: custom objects, relationships, screens, business rules, workflows and dashboards, no code required. Package what you build into a named Solution and promote it to another workspace, or install one of 10 ready-made industry apps instead. Connect it all to the rest of your stack with a REST API, webhooks and scheduled sync. <a href="/platform" style="color:inherit;text-decoration:underline">See what else you can build with it →</a></p><div class="hero-actions"><a class="btn btn-primary" href="/demo">Try the live demo →</a><a class="btn btn-secondary" href="/download">Desktop edition — Windows installer available</a></div><div class="trust-row"><span>✓ Free to use</span><span>✓ No licence key</span><span>✓ No-code customization</span><span>✓ Own your data</span></div></div><div class="mock"><div class="mock-top"><span class="dot"></span><span class="dot"></span><span class="dot"></span></div><div class="mock-body"><div class="mock-grid"><div class="mock-card"><small class="muted">Pipeline</small><br><strong>$192K</strong></div><div class="mock-card"><small class="muted">Revenue</small><br><strong>$84K</strong></div><div class="mock-card mock-chart">${[40,70,52,88,62,100].map(h=>`<div class="bar" style="height:${h}%"></div>`).join('')}</div></div></div></div></div></section>
 <section id="features" class="section"><div class="container"><div class="section-head"><div class="eyebrow">Complete sales journey</div><h2>Everything connected from first conversation to invoice.</h2><p class="muted">No maze of modules. No enterprise setup project. Just the essentials your team uses every day.</p></div><div class="feature-grid">${[
 ['◎','Companies & Contacts','Keep customer profiles, people, notes and activities together.'],['⌁','Sales Pipeline','Move opportunities visually from lead to won.'],['◇','Products & Services','Maintain reusable pricing, categories and tax settings.'],['▤','Quotes','Create professional commercial proposals and track acceptance.'],['▣','Orders','Convert approved quotes into trackable sales orders.'],['$','Invoices','Issue invoices and monitor paid, open and overdue balances.'],['▧','Contracts','Track agreement values, dates, files and renewals.'],['✓','Tasks & Activities','Manage calls, meetings, follow-ups and priorities.'],['▦','Sales Dashboard','See pipeline, revenue, customers and next actions instantly.']].map(x=>`<article class="feature-card"><div class="feature-icon">${x[0]}</div><h3>${x[1]}</h3><p class="muted">${x[2]}</p></article>`).join('')}</div></div></section>
 <section id="extensibility" class="section" style="background:var(--surface-alt,#f7f8fc)"><div class="container"><div class="section-head"><div class="eyebrow">Make it yours — no code required</div><h2>Every business outgrows a fixed data model. Lanesra doesn't have one.</h2><p class="muted">An Administrator can reshape the workspace itself from a settings screen — not a developer, not a support ticket.</p></div><div class="feature-grid">${[
 ['⬡','Custom Objects','Define an entirely new record type — Vendors, Assets, Projects, anything — with its own fields, ID format and navigation section.'],['⇄','Custom Relationships','Connect any two record types with one-to-one, many-to-one or many-to-many links, and a related-records list that appears automatically.'],['▦','Screen/App Builder','Design the create/edit form and detail view for any object — tabs, multi-column sections, field placement and related-list placement — with named layouts assigned by role and Draft → Publish.'],['◈','Business Rules','Require, show, hide, lock, unlock, set or clear a field\'s value with multi-condition AND/OR logic (plus nested OR-groups) and 10 comparison operators — restrict a select field\'s choices, or block a save with an error or warning message.'],['⚙','Workflow Automation','Trigger on a status/field change, a due date, or a schedule; create a task, assign an owner, create a related record, or post a notification.'],['▥','Custom Reports & Fields','Add validated custom fields to any object, then build reports that group and sum on them — no separate reporting tool.'],['◫','Custom Dashboards','Build named dashboard layouts, assign them by role, and mix KPI tiles, chart widgets and record-list widgets on each one — with a required Default fallback for everyone else.'],['🔔','Notifications & Admin Panel','An in-app notification center, user roles, branding, numbering formats and role-based access — one place to configure the whole workspace.'],['⊞','App Builder','Group a set of objects, their screens and a dashboard into one named, publishable app with its own icon — then grant it to roles or users as Viewer or Editor, enforced everywhere a record is written, not just in the UI.'],['⬢','App Catalog','Don\'t want to build one from scratch? Install a ready-made industry app — Field Service, Property Management, Legal Practice and 7 more — into your workspace in a few clicks, reusing your existing customers and contacts instead of creating a parallel database.'],['🗂','Solution Management','Publish your own customizations as a versioned Solution and promote it from a test workspace to production, the same build-in-test / export / import pattern enterprise platforms charge for — plus a Publisher registry so package identifiers never collide.'],['🔌','Integration Hub','Connect Lanesra to everything else you run: encrypted Connections, OpenAPI-imported Connectors, a generic REST API with scoped keys, HMAC-signed webhooks, a CSV data-exchange wizard, and scheduled sync jobs.']].map(x=>`<article class="feature-card"><div class="feature-icon">${x[0]}</div><h3>${x[1]}</h3><p class="muted">${x[2]}</p></article>`).join('')}</div></div></section>
 <section class="section"><div class="container cta"><div class="eyebrow" style="color:#a5b4fc">Not just a CRM</div><h2>Build the app your business actually runs on — or install one.</h2><p style="color:#cbd5e1;max-width:700px;margin:0 auto 24px">Field Service. Property Management. Construction. Professional Services. Practice Administration. Recruitment. Real Estate. Legal Practice. Nonprofit & Association Management. Auto Repair & Service Garage. All 10 are real, installable apps in the App Catalog today, built from the exact same Custom Objects, Relationships, Screens, Business Rules, Workflows and Dashboards you just saw above — not hypothetical examples of what's possible.</p><a class="btn btn-secondary" href="/platform">Explore the platform →</a></div></section>
 <section class="section" style="background:var(--surface-alt,#f7f8fc)"><div class="container split"><div class="choice-card"><div class="eyebrow">Package it, promote it</div><h2>Solution Management</h2><p class="muted">Curate a named, versioned Solution from anything you've built — objects, fields, rules, workflows, screens — export it, and import it into another workspace. The build-in-test, promote-to-production pattern enterprise platforms sell as a premium feature, running here as ordinary export/import between two self-hosted instances you already own.</p><ul><li>Publisher registry, no namespace collisions</li><li>Component-tagged, so nothing's a mystery</li><li>Update-with-diff before you apply a new version</li></ul><a class="btn btn-secondary" href="/platform#examples">See the platform →</a></div><div class="choice-card"><div class="eyebrow">Connect it</div><h2>Integration Hub</h2><p class="muted">Encrypted Connections to REST, SFTP, PostgreSQL, OData and SMTP systems. OpenAPI-imported Connectors your workflows can call. A generic REST API secured by scoped, hashed keys. HMAC-signed webhooks with retry. A CSV wizard that reuses the same validated write path as the API. Scheduled Integration Jobs for recurring sync.</p><ul><li>No integration platform subscription required</li><li>Every delivery signed and retried, not fire-and-forget</li><li>A unified log across every API call and delivery</li></ul><a class="btn btn-secondary" href="/roadmap#shipped">See what's shipped →</a></div></div></section>
 <section id="desktop" class="section"><div class="container split"><div class="choice-card"><div class="eyebrow">Try online</div><h2>Explore a working business</h2><p class="muted">Open the live demo with realistic sample customers, opportunities, quotes, invoices and contracts. No registration required.</p><ul><li>Sample company included</li><li>Create and edit records</li><li>Reset demo anytime</li></ul><a class="btn btn-primary" href="/demo">Open live demo</a></div><div class="choice-card dark"><div class="eyebrow" style="color:#a5b4fc">Desktop edition</div><h2>Your software. Your computer. Your data.</h2><p style="color:#cbd5e1">A private desktop edition is available now for Windows (Early Access, unsigned installer), with macOS and Linux to follow. The source is public on GitHub today.</p><ul><li>No cloud account required</li><li>Works without internet</li><li>No activation or subscription</li></ul><a class="btn btn-secondary" href="/download">Desktop status — Windows installer available</a></div></div></section>
 <section id="open-source" class="section"><div class="container cta"><div class="eyebrow" style="color:#a5b4fc">Open source by design</div><h2>Inspect it. Run it. Improve it.</h2><p style="color:#cbd5e1;max-width:700px;margin:0 auto 24px">Lanesra OS is designed to be transparent, community-driven and free from licence keys or mandatory telemetry.</p><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank" rel="noopener">View GitHub repository</a></div></section>
 </main>${publicFooter()}`;
 bindPublicNav();
}


function appShell(){
 document.title='Lanesra OS Demo';
 $('#app').innerHTML=`<div class="demo-banner">You are exploring the sample workspace. Changes stay in this browser. <button class="link-btn" id="resetDemo">Reset demo</button><a class="link-btn" href="/">Product website</a></div><div class="app-shell"><aside class="sidebar"><div class="side-brand"><span class="brand-mark">L</span><span>Lanesra OS</span><span class="demo-pill">DEMO</span></div><nav class="side-nav" id="sideNav"></nav><div class="side-bottom"><div class="side-meta"><strong>Early Access v0.36.0</strong><div class="side-product-links"><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/roadmap">Roadmap</a><a href="/releases">Releases</a></div><span>Created by <a href="https://vikramgrover.com">Vikram Grover</a></span></div><button class="btn btn-secondary" style="width:100%" onclick="location.href='/'">← Website</button></div></aside><main class="app-main"><header class="topbar"><div class="search"><input id="globalSearch" autocomplete="off" placeholder="Search companies, contacts, deals…  ⌘K"><div id="searchResults" class="search-results" hidden></div></div><div class="top-actions"><div class="notif-wrap"><button class="icon-btn" id="notifButton" aria-label="Notifications">🔔<span id="notifBadge" class="notif-badge" hidden></span></button><div id="notifPanel" class="notif-panel" hidden></div></div><button class="icon-btn" id="helpButton" aria-label="Help">?</button><div class="avatar">MC</div></div></header><div class="content" id="view"></div></main></div>`;
 renderSidebarNav();
 $('#resetDemo').onclick=()=>{data=structuredClone(seed);ensureAdminData();syncCustomObjectRegistry();activeAppId=null;renderSidebarNav();current='dashboard';detailRecord=null;save();toast('Demo data restored');refreshNotifBadge();renderView()};
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
function selectHtml(name,label,items,value,required=true,cls='field'){return `<div class="${cls}"><label>${label}</label><select name="${name}" ${required?'required':''}>${options(items,value)}</select></div>`}
// The sidebar's <nav> is only built once, inside appShell()'s initial
// innerHTML - unlike #view/#adminBody it's never re-rendered by
// renderView()/renderAdminTab(), so a Custom Objects create/edit/delete
// (or Reset demo) needs to explicitly rebuild it or its entry would never
// appear/disappear from navigation. App Builder: also owns the App
// Switcher <select> sitting above the nav buttons, so switching apps -
// which changes which sections are visible - only ever needs this one
// function called again, same as any other sidebar-affecting change.
function renderSidebarNav(){
 const nav=$('#sideNav'); if(!nav)return;
 const apps=ensureApps().filter(a=>a.isPublished);
 const switcherHtml=apps.length?`<div class="side-app-switcher"><select id="appSwitcher" aria-label="Switch app"><option value="">All</option>${apps.map(a=>`<option value="${a.id}" ${a.id===activeAppId?'selected':''}>${a.icon} ${a.name}</option>`).join('')}</select></div>`:'';
 nav.innerHTML=`${switcherHtml}${navSections().map(k=>`<button data-nav="${k}"><b>${icons[k]}</b><span>${labels[k]}</span></button>`).join('')}<button data-nav="admin" class="admin-nav-btn"><b>⚙</b><span>Admin</span></button>`;
 document.querySelectorAll('[data-nav]').forEach(b=>{b.onclick=()=>{current=b.dataset.nav;viewFilter=null;detailRecord=null;if(current==='admin')adminView='landing';renderView()};b.classList.toggle('active',b.dataset.nav===current)});
 const switcher=$('#appSwitcher');
 if(switcher)switcher.onchange=e=>{
  activeAppId=e.target.value||null;
  // A section the just-left app exposed may not exist in the new scope
  // (or "All") - land back on Dashboard rather than risk stranding on a
  // now-hidden nav item, same reasoning App.tsx's switchApp uses on desktop.
  current='dashboard';viewFilter=null;detailRecord=null;
  renderSidebarNav();renderView();
 };
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
// A chart widget's reportId may point at a report that's since been
// deleted - filtered out by the caller before this ever runs, the same
// "stale key, no cleanup needed" choice a stale kpiKey already gets.
function dashboardChartWidgetHtml(w){
 const report=(data.customReports||[]).find(r=>r.id===w.config.reportId);
 const rows=runCustomReport(report);
 const max=Math.max(0,...rows.map(r=>r.value));
 return `<section class="panel"><div class="panel-head"><h3>${report.name}</h3></div>${rows.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Group</th><th></th><th>Value</th></tr></thead><tbody>${rows.map(r=>`<tr><td>${r.group}</td><td>${reportBarHtml(r.value,max)}</td><td>${report.aggregate==='sum'?r.value.toLocaleString():r.value}</td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No data yet.</div>'}</section>`;
}
function dashboardRecordListWidgetHtml(w){
 const {entityKey,mode,limit}=w.config;
 const rows=dashboardRecordListRows(entityKey,mode,limit);
 return `<section class="panel"><div class="panel-head"><h3>${entityLabel(entityKey)} — ${mode==='due_soon'?'due soon':'recent'}</h3></div>${rows.length?rows.map(r=>`<div class="deal" style="cursor:pointer" data-open-record="${entityKey}:${r.id}"><strong>${r.title}</strong>${r.subtitle?`<small class="muted"> · ${r.subtitle}</small>`:''}</div>`).join(''):'<div class="empty">Nothing here yet.</div>'}</section>`;
}
function dashboard(){
 // Dashboard customization: a published Default dashboard's widgets
 // override the fixed KPI row and add chart/record-list widget rows -
 // `null` (the common case until an admin builds one) falls back to the
 // pre-this-feature behavior below, unchanged. See effectiveDashboardWidgets.
 // App Builder: when the sidebar App Switcher has an app selected and that
 // app names a dashboard, its *published* widgets win instead - the same
 // "content only ever comes from published, never draft" rule the Default
 // dashboard above already follows. A picked-but-unpublished dashboard (or
 // one that's since been deleted) falls back exactly like "no app selected"
 // would, not a broken/empty state.
 const appForDash=activeApp();
 const appDashboard=appForDash&&appForDash.dashboardId?dashboardById(appForDash.dashboardId):null;
 const effective=appDashboard?appDashboard.publishedWidgets:effectiveDashboardWidgets();
 const kpis=effective
  ?effective.filter(w=>w.kind==='kpi').map(w=>KPI_DEFS.find(k=>k.key===w.config.kpiKey)).filter(Boolean)
  :visibleKpis();
 const chartWidgets=effective?effective.filter(w=>w.kind==='chart'&&(data.customReports||[]).some(r=>r.id===w.config.reportId)):[];
 const listWidgets=effective?effective.filter(w=>w.kind==='record_list'):[];
 $('#view').innerHTML=`<div class="page-head"><div><div class="eyebrow">${data.workspace.name}</div><h1>Good afternoon, Maya</h1><p class="muted">Here is what needs your attention today.</p></div><div class="quick-create"><button class="btn btn-primary" id="quickNew">+ New</button><div class="quick-menu" id="quickMenu" hidden>${[['companies','Company'],['contacts','Contact'],['opportunities','Opportunity'],['quotes','Quote'],['orders','Order'],['invoices','Invoice'],['contracts','Contract'],['tasks','Task']].map(x=>`<button data-create="${x[0]}">${x[1]}</button>`).join('')}</div></div></div><div class="kpi-grid">${kpis.map(k=>`<button class="kpi kpi-link" data-kpi-nav="${k.nav}" data-kpi-filter="${k.filter}"><div class="kpi-label">${k.label}</div><div class="kpi-value">${k.value()}</div><span>View ${k.label.toLowerCase()} →</span></button>`).join('')}</div>${(chartWidgets.length||listWidgets.length)?`<div class="grid-2">${chartWidgets.map(dashboardChartWidgetHtml).join('')}${listWidgets.map(dashboardRecordListWidgetHtml).join('')}</div>`:''}<div class="grid-2"><section class="panel"><div class="panel-head"><h3>Pipeline snapshot</h3><button class="link-btn" data-nav2="pipeline" data-filter2="open">Open pipeline</button></div>${data.opportunities.filter(o=>!['Won','Lost'].includes(o.stage)).slice(0,5).map(o=>`<div class="deal"><div style="display:flex;justify-content:space-between"><strong>${o.title}</strong><strong>${money(o.value)}</strong></div><small class="muted">${companyName(o.companyId)} · ${o.stage}</small></div>`).join('')}</section><section class="panel"><div class="panel-head"><h3>Tasks requiring attention</h3><button class="link-btn" data-nav2="tasks" data-filter2="open">View tasks</button></div>${data.tasks.filter(t=>!['Completed','Cancelled'].includes(t.status)).map(t=>`<div class="deal"><strong>${t.title}</strong><small class="muted">${relatedLabel(t)} · ${t.due}</small></div>`).join('')}</section></div>`;
 document.querySelectorAll('[data-kpi-nav]').forEach(b=>b.onclick=()=>{current=b.dataset.kpiNav;viewFilter=b.dataset.kpiFilter;detailRecord=null;renderView()});
 document.querySelectorAll('[data-nav2]').forEach(b=>b.onclick=()=>{current=b.dataset.nav2;viewFilter=b.dataset.filter2||null;detailRecord=null;renderView()});
 wireCellLinks($('#view'));
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
  const report=stampCreate({id:uid(),name:fd.name,entityKey:fd.entityKey,groupBySource:fd.groupBy==='__builtin__'?'builtin':'custom',groupByField:fd.groupBy==='__builtin__'?transitionFieldFor(fd.entityKey):fd.groupBy,aggregate:fd.aggregate,sumFieldKey:fd.aggregate==='sum'?fd.sumFieldKey:''});
  data.customReports.push(report);
  tagLocalComponent('customReport',report.id);
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
// ---- Saved Views & Bulk Actions -------------------------------------------
// A saved view persists a {status filter, sort, group-by} combination for
// one object key, reusable and settable as that object's default - the
// online demo's own scoped mirror of the same backlog item shipped on
// desktop (core::services::saved_view_service / bulk_action_service). Two
// honest scope differences from the desktop edition, both because of gaps
// already established elsewhere in this demo, not new ones invented here:
// no Private/Shared distinction (the demo has no signed-in-user concept at
// all - see App Builder Phase 1's own note on this) and no per-custom-field
// filter (the demo never got desktop's per-field list filtering - see the
// Roadmap's "Desktop: Global search & list-view filtering" item, which
// states plainly that the demo's own search stays unaffected) - a saved
// view here filters only by the object's own status field, which every
// wired object already has. Wired into Companies, Contacts, Tasks and every
// Custom Object's record list - the same generic tablePage they already
// share. Opportunities is kanban-only in this demo (no flat list to select
// rows from), so it's out of scope here exactly as it is for the pipeline
// board everywhere else.
const SAVED_VIEW_KEYS=['companies','contacts','tasks'];
function savedViewCapable(key){return SAVED_VIEW_KEYS.includes(key)||!!customObjectByKey(key)}
function savedViewsFor(key){return (data.savedViews||[]).filter(v=>v.objectKey===key)}
function defaultSavedView(key){return savedViewsFor(key).find(v=>v.isDefault)}
let viewState={};
function ensureViewState(key){
 if(!viewState[key]){
  const def=defaultSavedView(key);
  viewState[key]=def
   ?{activeId:def.id,statusFilter:def.statusFilter||'',sortField:def.sortField||'',sortDirection:def.sortDirection||'asc',groupByField:def.groupByField||''}
   :{activeId:'',statusFilter:'',sortField:'',sortDirection:'asc',groupByField:''};
 }
 return viewState[key];
}
function applySavedView(key,arr,statusFieldKey){
 const vs=ensureViewState(key);
 let out=arr;
 if(vs.statusFilter&&statusFieldKey)out=out.filter(r=>r[statusFieldKey]===vs.statusFilter);
 if(vs.sortField){
  out=[...out].sort((a,b)=>{
   const av=a[vs.sortField]??'',bv=b[vs.sortField]??'';
   const cmp=(typeof av==='number'&&typeof bv==='number')?av-bv:String(av).localeCompare(String(bv));
   return vs.sortDirection==='desc'?-cmp:cmp;
  });
 }
 return out;
}
function groupRows(arr,groupByField){
 if(!groupByField)return [{label:null,rows:arr}];
 const groups=new Map();
 arr.forEach(r=>{
  const label=(r[groupByField]===undefined||r[groupByField]===null||r[groupByField]==='')?'(none)':String(r[groupByField]);
  if(!groups.has(label))groups.set(label,[]);
  groups.get(label).push(r);
 });
 return [...groups.entries()].map(([label,rows])=>({label,rows}));
}
// Which bulk operations make sense for an object - mirrors
// bulk_action_service's own per-entity allowlists: only Companies/Tasks/
// Custom Objects carry a demo `owner` field (Contacts doesn't); tagging
// isn't offered at all since this demo has no tags concept on any entity
// yet, stated here rather than faked.
function bulkOwnerCapable(key){return key==='companies'||key==='tasks'||!!customObjectByKey(key)}
function selectionStore(){if(!window.__bulkSelection)window.__bulkSelection={};return window.__bulkSelection}
function selectionFor(key){const s=selectionStore();if(!s[key])s[key]=new Set();return s[key]}
function savedViewBarHtml(key,cfg,statusField){
 const vs=ensureViewState(key);
 const views=savedViewsFor(key);
 const sortableFields=cfg.cols.filter(c=>c[0]!=='id');
 const isDirty=(()=>{
  const active=views.find(v=>v.id===vs.activeId);
  if(!active)return !!(vs.statusFilter||vs.sortField||vs.groupByField);
  return active.statusFilter!==vs.statusFilter||active.sortField!==vs.sortField||active.sortDirection!==vs.sortDirection||active.groupByField!==vs.groupByField;
 })();
 const activeView=views.find(v=>v.id===vs.activeId);
 return `<div class="view-bar" style="display:flex;flex-wrap:wrap;gap:10px;align-items:center;margin:14px 0">
  <select id="viewSelect" style="min-width:160px">
   <option value="">All records (no view)</option>
   ${views.map(v=>`<option value="${v.id}" ${v.id===vs.activeId?'selected':''}>${v.isDefault?'★ ':''}${v.name}</option>`).join('')}
  </select>
  ${statusField?`<label style="display:flex;align-items:center;gap:6px;font-size:13px">Status<select id="viewStatusFilter"><option value="">Any</option>${statusField[3].split('|').map(o=>`<option value="${o}" ${o===vs.statusFilter?'selected':''}>${o}</option>`).join('')}</select></label>`:''}
  <label style="display:flex;align-items:center;gap:6px;font-size:13px">Sort by
   <select id="viewSort"><option value="">—</option>${sortableFields.map(c=>`<option value="${c[0]}" ${c[0]===vs.sortField?'selected':''}>${c[1]}</option>`).join('')}</select>
   ${vs.sortField?`<button type="button" class="icon-btn" id="viewSortDir" title="Toggle direction">${vs.sortDirection==='desc'?'↓':'↑'}</button>`:''}
  </label>
  <label style="display:flex;align-items:center;gap:6px;font-size:13px">Group by
   <select id="viewGroup"><option value="">—</option>${sortableFields.map(c=>`<option value="${c[0]}" ${c[0]===vs.groupByField?'selected':''}>${c[1]}</option>`).join('')}</select>
  </label>
  ${isDirty?`<button type="button" class="link-btn" id="viewSaveNew">Save as view…</button>`:''}
  ${activeView&&isDirty?`<button type="button" class="link-btn" id="viewUpdate">Update "${activeView.name}"</button>`:''}
  ${activeView?`${!activeView.isDefault?`<button type="button" class="link-btn" id="viewSetDefault">Set as default</button>`:''}<button type="button" class="link-btn" id="viewDelete" style="color:var(--danger,#c0392b)">Delete view</button>`:''}
 </div>`;
}
function bulkActionBarHtml(key,statusField){
 const ids=[...selectionFor(key)];
 if(!ids.length)return '';
 return `<div class="bulk-bar" style="display:flex;flex-wrap:wrap;gap:10px;align-items:center;margin:10px 0;padding:10px;border:1px solid var(--border,#e5e7eb);border-radius:8px">
  <strong>${ids.length} selected</strong>
  ${statusField?`<button type="button" class="btn btn-secondary" id="bulkStatus">Change status…</button>`:''}
  ${bulkOwnerCapable(key)?`<button type="button" class="btn btn-secondary" id="bulkOwner">Reassign owner…</button>`:''}
  <button type="button" class="btn btn-secondary" id="bulkExport">Export selected (CSV)</button>
  <button type="button" class="btn btn-secondary" id="bulkDelete" style="color:var(--danger,#c0392b)">Delete selected</button>
  <button type="button" class="link-btn" id="bulkClear">Clear selection</button>
 </div>`;
}
function bulkFieldModal(title,inputHtml,onSubmit){
 modal(title,`<form id="bulkForm">${inputHtml}<div class="modal-actions"><button class="btn btn-primary" type="submit">Apply</button> <button type="button" class="btn btn-secondary" onclick="closeModal()">Cancel</button></div></form>`);
 $('#bulkForm').onsubmit=e=>{e.preventDefault();onSubmit(new FormData(e.target));closeModal()};
}
function wireSavedViewsAndBulkActions(key,cfg,statusField){
 const vs=ensureViewState(key);
 const sel=selectionFor(key);
 $('#viewSelect')?.addEventListener('change',e=>{
  const v=savedViewsFor(key).find(x=>x.id===e.target.value);
  viewState[key]=v?{activeId:v.id,statusFilter:v.statusFilter||'',sortField:v.sortField||'',sortDirection:v.sortDirection||'asc',groupByField:v.groupByField||''}:{activeId:'',statusFilter:'',sortField:'',sortDirection:'asc',groupByField:''};
  renderView();
 });
 $('#viewStatusFilter')?.addEventListener('change',e=>{vs.statusFilter=e.target.value;renderView()});
 $('#viewSort')?.addEventListener('change',e=>{vs.sortField=e.target.value||'';vs.sortDirection='asc';renderView()});
 $('#viewSortDir')?.addEventListener('click',()=>{vs.sortDirection=vs.sortDirection==='desc'?'asc':'desc';renderView()});
 $('#viewGroup')?.addEventListener('change',e=>{vs.groupByField=e.target.value||'';renderView()});
 $('#viewSaveNew')?.addEventListener('click',()=>{
  bulkFieldModal('Save as view','<div class="field full"><label>View name</label><input name="name" required autofocus></div>',fd=>{
   const name=fd.get('name').trim();if(!name)return;
   const view={id:uid(),objectKey:key,name,statusFilter:vs.statusFilter,sortField:vs.sortField,sortDirection:vs.sortDirection,groupByField:vs.groupByField,isDefault:false};
   data.savedViews.push(view);save();vs.activeId=view.id;toast(`View "${name}" saved`);renderView();
  });
 });
 $('#viewUpdate')?.addEventListener('click',()=>{
  const view=savedViewsFor(key).find(v=>v.id===vs.activeId);if(!view)return;
  Object.assign(view,{statusFilter:vs.statusFilter,sortField:vs.sortField,sortDirection:vs.sortDirection,groupByField:vs.groupByField});
  save();toast(`View "${view.name}" updated`);renderView();
 });
 $('#viewSetDefault')?.addEventListener('click',()=>{
  savedViewsFor(key).forEach(v=>v.isDefault=(v.id===vs.activeId));save();toast('Default view set');renderView();
 });
 $('#viewDelete')?.addEventListener('click',()=>{
  const view=savedViewsFor(key).find(v=>v.id===vs.activeId);if(!view)return;
  if(!confirm(`Delete the view "${view.name}"?`))return;
  data.savedViews=data.savedViews.filter(v=>v.id!==view.id);save();
  viewState[key]={activeId:'',statusFilter:'',sortField:'',sortDirection:'asc',groupByField:''};
  toast('View deleted');renderView();
 });
 document.querySelectorAll('[data-bulk-id]').forEach(cb=>cb.addEventListener('change',()=>{
  const id=cb.dataset.bulkId;
  if(cb.checked)sel.add(id);else sel.delete(id);
  renderView();
 }));
 $('#bulkClear')?.addEventListener('click',()=>{sel.clear();renderView()});
 $('#bulkDelete')?.addEventListener('click',()=>{
  const ids=[...sel];if(!ids.length)return;
  if(!confirm(`Delete ${ids.length} selected record${ids.length===1?'':'s'}?`))return;
  let blocked=0;
  ids.forEach(id=>{
   const refs=dependencies(key,id);const relBlock=relationshipDeleteCheck(key,id);
   if(refs.length||relBlock){blocked++;return}
   clearArchivableRelationshipInstances(key,id);data[key]=data[key].filter(x=>x.id!==id);sel.delete(id);
  });
  save();
  toast(blocked?`Deleted ${ids.length-blocked} record(s); ${blocked} skipped (still connected to other records)`:`Deleted ${ids.length} record(s)`);
  renderView();
 });
 $('#bulkStatus')?.addEventListener('click',()=>{
  const ids=[...sel];if(!ids.length||!statusField)return;
  bulkFieldModal('Change status',`<div class="field full"><label>New status</label><select name="status">${statusField[3].split('|').map(o=>`<option value="${o}">${o}</option>`).join('')}</select></div>`,fd=>{
   const status=fd.get('status');
   data[key].filter(r=>ids.includes(r.id)).forEach(r=>{r[statusField[0]]=status});
   save();toast(`Updated status for ${ids.length} record(s)`);renderView();
  });
 });
 $('#bulkOwner')?.addEventListener('click',()=>{
  const ids=[...sel];if(!ids.length)return;
  bulkFieldModal('Reassign owner','<div class="field full"><label>Owner</label><input name="owner" placeholder="Leave blank to unassign"></div>',fd=>{
   const owner=fd.get('owner')||'';
   data[key].filter(r=>ids.includes(r.id)).forEach(r=>{r.owner=owner});
   save();toast(`Reassigned owner for ${ids.length} record(s)`);renderView();
  });
 });
 $('#bulkExport')?.addEventListener('click',()=>{
  const ids=[...sel];if(!ids.length)return;
  const rows=data[key].filter(r=>ids.includes(r.id));
  downloadCsv(`${key}-selected.csv`,cfg.cols.map(c=>c[1]),rows.map(r=>cfg.cols.map(c=>r[c[0]]??'')));
 });
}
function tablePage(key,cfg){
 let arr=data[key];
 if(key==='tasks'&&viewFilter==='open')arr=arr.filter(x=>!['Completed','Cancelled'].includes(x.status));
 if(key==='invoices'&&viewFilter==='outstanding')arr=arr.filter(x=>!['Paid','Cancelled'].includes(x.status));
 const svCapable=savedViewCapable(key)&&!viewFilter;
 const statusField=svCapable?cfg.fields().find(f=>f[0]==='status'):null;
 if(svCapable)arr=applySavedView(key,arr,statusField?statusField[0]:null);
 const groups=svCapable?groupRows(arr,ensureViewState(key).groupByField):[{label:null,rows:arr}];
 const sel=svCapable?selectionFor(key):new Set();
 const extraCols=svCapable?1:0;
 const rowHtml=r=>`<tr>${svCapable?`<td><input type="checkbox" class="bulk-check" data-bulk-id="${r.id}" ${sel.has(r.id)?'checked':''}></td>`:''}${cfg.cols.map(c=>`<td>${cellValue(r,c,key)}</td>`).join('')}<td><div class="actions"><button class="icon-btn" data-edit="${r.id}">Edit</button><button class="icon-btn" data-del="${r.id}">Delete</button></div></td></tr>`;
 const bodyHtml=groups.map(g=>`${g.label!==null?`<tr class="group-row"><td colspan="${cfg.cols.length+1+extraCols}" style="font-weight:600;background:var(--surface-2,#f8f9fb)">${g.label}</td></tr>`:''}${g.rows.map(rowHtml).join('')}`).join('');
 $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><span>${viewFilter?viewFilter.charAt(0).toUpperCase()+viewFilter.slice(1):labels[key]}</span></div><h1>${viewFilter==='open'&&key==='tasks'?'Open Tasks':viewFilter==='outstanding'?'Outstanding Invoices':labels[key]}</h1><p class="muted">${arr.length} connected records in the sample workspace</p></div><button class="btn btn-primary" id="addRecord">+ New ${labels[key].replace(/s$/,'')}</button></div>${svCapable?savedViewBarHtml(key,cfg,statusField):''}<div class="table-wrap"><table class="table"><thead><tr>${svCapable?'<th></th>':''}${cfg.cols.map(c=>`<th>${c[1]}</th>`).join('')}<th>Actions</th></tr></thead><tbody>${bodyHtml}</tbody></table>${arr.length?'':'<div class="empty">No records yet</div>'}</div>${svCapable?bulkActionBarHtml(key,statusField):''}`;
 document.querySelector('[data-clear-filter]')?.addEventListener('click',()=>{current='dashboard';viewFilter=null;detailRecord=null;renderView()});
 wireCellLinks($('#view'));
 $('#addRecord').onclick=()=>recordModal(key,cfg.fields());
 document.querySelectorAll('[data-edit]').forEach(b=>b.onclick=()=>recordModal(key,cfg.fields(),byId(key,b.dataset.edit)));
 document.querySelectorAll('[data-del]').forEach(b=>b.onclick=()=>remove(key,b.dataset.del));
 if(svCapable)wireSavedViewsAndBulkActions(key,cfg,statusField);
}
function badgeMaybe(v){const vals=['Active','Inactive','Customer','Prospect','Lead','Sent','Accepted','Draft','Paid','Overdue','Open','Completed','High','Medium','Low','Urgent','Renewal Due','In Progress','Won','Lost','Confirmed','Cancelled'];return vals.includes(String(v))?`<span class="badge">${v}</span>`:(v??'—')}
// `full` (Screen/App Builder Phase 2): explicit true/false forces this
// field's width, overriding the field's own default - how a layout
// section's admin-set full_width choice takes effect. Left undefined
// (every call site outside the layout system), the pre-Phase-2 default
// applies: only the "title" field is full-width.
function fieldHtml(f,record,full){const [name,label,type,opts]=f;const extra=f[4];const val=record[name]??(!record.id&&extra?.defaultValue?extra.defaultValue:'');const help=extra?.helpText?`<small class="field-help">${extra.helpText}</small>`:'';const req=extra?.required?'required':(['name','title','number'].includes(name)?'required':'');const cls=`field${(full??name==='title')?' full':''}`;if(type==='auto')return `<div class="${cls}"><label>${label}</label><input name="${name}" value="${val}" readonly placeholder="Generated automatically"><small class="field-help">Generated when the record is saved</small></div>`;if(type==='select')return `<div class="${cls}"><label>${label}</label><select name="${name}" ${req}>${opts.split('|').map(o=>`<option value="${o}" ${val===o?'selected':''}>${o}</option>`).join('')}</select>${help}</div>`;if(type==='relation')return selectHtml(name,label,data[opts],val,true,cls);if(type==='filteredContact')return `<div class="${cls}"><label>${label}</label><select name="${name}" data-filter="contact">${optionalOptions(data.contacts.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No contact')}</select></div>`;if(type==='filteredOpportunity')return `<div class="${cls}"><label>${label}</label><select name="${name}" data-filter="opportunity">${optionalOptions(data.opportunities.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No opportunity',x=>x.title)}</select></div>`;if(type==='filteredQuote')return `<div class="${cls}"><label>${label}</label><select name="${name}" data-filter="quote">${optionalOptions(data.quotes.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No source quote',x=>x.number+' · '+money(docTotal(x)))}</select></div>`;if(type==='filteredOrder')return `<div class="${cls}"><label>${label}</label><select name="${name}" data-filter="order">${optionalOptions(data.orders.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No source order',x=>x.number+' · '+money(docTotal(x)))}</select></div>`;if(type==='dynamicRelation')return `<div class="${cls}"><label>${label}</label><select name="${name}" data-dynamic-related></select></div>`;
 // Custom fields carry their own HTML5-native validation constraints - a
 // maxlength/pattern for text, min/max for number - so the browser blocks
 // an invalid save the same way desktop's server-side validation does,
 // without a separate client-side validator to keep in sync.
 const extraAttrs=type==='number'
  ?`${extra?.minValue!==''&&extra?.minValue!==undefined?`min="${extra.minValue}"`:''} ${extra?.maxValue!==''&&extra?.maxValue!==undefined?`max="${extra.maxValue}"`:''}`
  :`${extra?.maxLength?`maxlength="${extra.maxLength}"`:''} ${extra?.pattern?`pattern="${extra.pattern}"`:''}`;
 return `<div class="${cls}"><label>${label}</label><input name="${name}" type="${type||'text'}" value="${val}" placeholder="${extra?.placeholder||''}" ${req} ${extraAttrs}>${help}</div>`}
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
function detail360Header(breadcrumbLabel,title,eyebrow,metaHtml,auditHtml){
 return `<div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><button data-back-list>${breadcrumbLabel}</button><span>›</span><span>${title}</span></div>
 <div class="rule360-header">
  <div><div class="eyebrow">${eyebrow||''}</div><h1>${title}</h1><div class="rule360-meta">${metaHtml}</div>${auditHtml||''}</div>
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
 $('#view').innerHTML=`${detail360Header('Companies',c.name,c.customerNumber,`${badgeMaybe(c.status)}<span>Owner: ${c.owner||'Unassigned'}</span>`,auditByline(c))}
 <div class="rule360-grid">
  <div><div class="panel"><h3 style="margin-top:0">Overview</h3>${overviewGroupsHtml('companies',overviewGroupsFor('companies',overviewFields),c)}</div></div>
  <div>
   ${relatedCardHtml('Contacts',contacts,'contacts',x=>x.name,x=>x.role||'')}
   ${relatedCardHtml('Sales Pipeline',opportunities,'opportunities',x=>x.title,x=>money(x.value))}
   ${relatedCardHtml('Quotes',quotes,'quotes',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Orders',orders,'orders',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Invoices',invoices,'invoices',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Contracts',contracts,'contracts',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Tasks',tasks,'tasks',x=>x.title,x=>x.status)}
   ${customRelatedCardsHtml('companies',id)}
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
 $('#view').innerHTML=`${detail360Header('Contacts',c.name,c.contactNumber,`${badgeMaybe(c.status)}<span>${c.role||'—'}</span><a class="cell-link" data-nav-related="companies:${c.companyId}">${companyName(c.companyId)}</a>`,auditByline(c))}
 <div class="rule360-grid">
  <div><div class="panel"><h3 style="margin-top:0">Overview</h3>${overviewGroupsHtml('contacts',overviewGroupsFor('contacts',overviewFields),c)}</div></div>
  <div>
   ${relatedCardHtml('Sales Pipeline',opportunities,'opportunities',x=>x.title,x=>money(x.value))}
   ${relatedCardHtml('Quotes',quotes,'quotes',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Orders',orders,'orders',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Contracts',contracts,'contracts',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Tasks',tasks,'tasks',x=>x.title,x=>x.status)}
   ${customRelatedCardsHtml('contacts',id)}
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
// Screen/App Builder Phase 4: groups a detail page's Overview fields per
// the object's published layout, same order/sections/columns the live
// edit form already uses (`orderedTabsFor`) - flattened across tabs
// since a detail page is one glanceable summary, not a form filled out
// in stages, so (unlike the edit form) the layout's *tab* boundaries
// don't carry over here, only its section grouping and field placement.
// No published layout at all falls back to one untitled group in the
// fields' plain order, exactly the pre-Phase-4 behavior.
function overviewGroupsFor(key,fields){
 return orderedTabsFor(fields,defaultLayoutFor(key).publishedTabs).flatMap(t=>t.groups);
}
// Read-only counterpart to groupsHtml, for overviewGroupsFor's output.
function overviewGroupsHtml(key,groups,record){
 return groups.map(g=>`${g.title?`<h4 style="margin:0 0 8px">${g.title}</h4>`:''}<div class="form-grid" style="grid-template-columns:repeat(${g.columns},1fr);margin-bottom:14px">${g.items.map(it=>`<div class="field${it.full?' full':''}"><label>${it.field[1]}</label><div>${overviewValueHtml(key,it.field,record)}</div></div>`).join('')}</div>`).join('');
}
// Screen/App Builder Phase 4: the custom-relationship "Related records"
// this record participates in (Admin > Relationships), reusing the same
// relatedCardHtml every other related list on a detail page already
// renders with - previously shown nowhere on any detail page, only in
// the create/edit modal (Phase 3). Read-only here (click through to the
// linked record, same as every other card on this page) - link/unlink
// stays a create/edit-modal action, consistent with how this page's
// other related cards work.
function customRelatedCardsHtml(entityType,entityId){
 return relatedGroupsFor(entityType,entityId).map(g=>relatedCardHtml(g.label,g.rows.map(r=>({...r,id:r.entityId})),g.otherType,x=>x.displayName,x=>x.status)).join('');
}
function genericRecordDetail(key,id){
 const r=byId(key,id);
 if(!r){current=key;detailRecord=null;return renderView()}
 const fieldsFn=DETAIL_FIELDS_FN[key];
 const relationTypes=['auto','relation','filteredContact','filteredOpportunity','filteredQuote','filteredOrder','dynamicRelation'];
 const overviewFields=fieldsFor(key,fieldsFn).filter(f=>!relationTypes.includes(f[2])&&f[0]!==DETAIL_TITLE_FIELD[key]);
 const isDoc=['quotes','orders','invoices'].includes(key);
 const linesHtml=isDoc?`<div class="panel" style="margin-bottom:16px"><h3 style="margin-top:0">Products & services</h3><div class="table-wrap"><table class="table"><thead><tr><th>Product / service</th><th>Quantity</th><th>Unit price</th><th>Line total</th></tr></thead><tbody>${(r.items||[]).map(i=>`<tr><td>${productName(i.productId)}</td><td>${i.quantity}</td><td>${money(i.unitPrice)}</td><td>${money(lineTotal(i))}</td></tr>`).join('')}</tbody></table></div><div class="line-total">Total <strong>${money(docTotal(r))}</strong>${key==='invoices'?` <span class="muted" style="font-size:13px;font-weight:400">· Balance ${money(docBalance(r))}</span>`:''}</div></div>`:'';
 $('#view').innerHTML=`${detail360Header(DETAIL_BREADCRUMB[key],r[DETAIL_TITLE_FIELD[key]]||'—',DETAIL_EYEBROW(key,r),recordEyebrowMeta(key,r),auditByline(r))}
 <div class="rule360-grid">
  <div>
   ${linesHtml}
   <div class="panel"><h3 style="margin-top:0">Overview</h3>${overviewGroupsHtml(key,overviewGroupsFor(key,overviewFields),r)}</div>
  </div>
  <div>${recordRelatedDefs(key,r).map(([title,items,navKey,labelFn,metaFn])=>relatedCardHtml(title,items,navKey,labelFn,metaFn)).join('')}${customRelatedCardsHtml(key,id)}</div>
 </div>`;
 wireDetail360Nav(()=>recordModal(key,fieldsFor(key,fieldsFn),r));
}
function recordModal(key,fields,record={}){
 const isDoc=['quotes','orders','invoices'].includes(key);
 if(!record.id){const r0=effectiveRule(key);if(r0)record={...record,[r0.field]:nextNumber(key)}}
 // Screen/App Builder Phase 1: the live form always renders the object's
 // Default layout's published tabs/sections (see layoutsTab's own comment
 // on why there's no per-role resolution here - this demo has no
 // signed-in user). No published layout at all renders the plain default
 // field order in one untitled tab, exactly as before this feature
 // existed. Every tab's fields stay present in the DOM (just hidden) so
 // switching tabs never loses values already typed into another one.
 const tabsOut=orderedTabsFor(fields,defaultLayoutFor(key).publishedTabs);
 const tabsHtml=tabsOut.length>1?`<div class="layout-tabs" style="display:flex;gap:8px;margin-bottom:12px">${tabsOut.map((t,i)=>`<button type="button" class="tab ${i===0?'active':''}" data-form-tab="${i}">${t.title||'Details'}</button>`).join('')}</div>`:'';
 // Screen/App Builder Phase 3: each tab gets a `data-related-slot` for
 // whichever relationships it claims, filled in below (only meaningful
 // once record.id exists - a create form has nothing to link yet).
 const panelsHtml=tabsOut.map((t,i)=>`<div data-form-panel="${i}" style="${i===0?'':'display:none'}">${groupsHtml(t.groups,record)}<div data-related-slot="${i}"></div></div>`).join('');
 // Audit trail: entities with their own detail page (companies, contacts,
 // and the six DETAIL_PAGE_ENTITIES generic-360 types) show the byline
 // there instead - only show it here, in the shared edit form, for
 // entities that have no separate detail view (opportunities, custom
 // object records).
 const auditHtml=(record.id&&!DETAIL_PAGE_ENTITIES.has(key))?auditByline(record):'';
 const form=`<form id="recordForm">${auditHtml}${tabsHtml}${panelsHtml}${isDoc?lineItemsHtml(record.items||[]):''}<div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Save record</button></div></form>${record.id?'<div id="relatedRecordsPanel"></div>':''}`;
 modal(record.id?'Edit record':'Create record',form); $('[data-close]').onclick=closeModal;
 document.querySelectorAll('[data-form-tab]').forEach(b=>b.onclick=()=>{
  document.querySelectorAll('[data-form-panel]').forEach(p=>p.style.display=p.dataset.formPanel===b.dataset.formTab?'':'none');
  document.querySelectorAll('[data-form-tab]').forEach(x=>x.classList.toggle('active',x===b));
 });
 wireRelations(record); if(isDoc)wireLines();
 applyFieldRules(key,$('#recordForm'));
 // Custom Relationships (admin extensibility, Phase B) + Screen/App
 // Builder Phase 3: a record being edited shows every linked record
 // across every applicable relationship, with inline link/unlink -
 // placed per-tab where a layout claims one, with anything unclaimed in
 // an always-visible panel below the tabs. Mirrors desktop's
 // RelatedRecordsCard (`only` prop), just re-rendering itself in place
 // on every change instead of a query-client refetch.
 if(record.id){relLinkingKey=null;refreshRelatedPanels(key,record.id,tabsOut)}
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
 // Audit trail: every entity through this shared save path gets stamped
 // except users (the desktop User model carries no created_by/updated_by
 // either - see audit_trail.rs's own scope).
 if(wasEdit){const target=byId(key,record.id);Object.assign(target,obj);if(key!=='users')stampUpdate(target)}
 else{const rule=effectiveRule(key);if(rule&&!obj[rule.field])obj[rule.field]=nextNumber(key);const created={id:uid(),...obj};if(key!=='users')stampCreate(created);data[key].unshift(created)}
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
// The related-record groups for entityType/entityId, optionally
// restricted to `onlyKeys` (Screen/App Builder Phase 3 - a tab's
// `related` list, or the leftover keys no tab claims) - mirrors
// desktop's RelatedRecordsCard's `only` prop.
function relatedGroupsFor(entityType,entityId,onlyKeys){
 const defs=relationshipDefsFor(entityType).filter(d=>!onlyKeys||onlyKeys.includes(d.key));
 const related=relatedRecordsFor(entityType,entityId);
 return defs.map(def=>{
  const isSource=def.sourceEntity===entityType;
  return {def,label:isSource?def.forwardLabel:def.reverseLabel,otherType:isSource?def.targetEntity:def.sourceEntity,isSource,rows:related.filter(r=>r.defKey===def.key)};
 });
}
function relatedGroupsHtml(groups){
 if(!groups.length)return '';
 return `<div class="panel" style="margin-top:16px"><h3 style="margin-top:0">Related records</h3>${groups.map(g=>`<div style="margin-bottom:14px"><div class="panel-head" style="margin-bottom:4px"><strong>${g.label}</strong><button type="button" class="btn btn-secondary" data-link-group="${g.def.key}">+ Link</button></div>${g.rows.length?g.rows.map(r=>`<div class="deal" style="display:flex;justify-content:space-between;align-items:center"><span>${r.displayName} ${badgeMaybe(r.status)}</span><button type="button" class="icon-btn" data-unlink="${r.instanceId}">Unlink</button></div>`).join(''):'<div class="empty">None linked</div>'}${relLinkingKey===g.def.key?linkPickerHtml(g):''}</div>`).join('')}</div>`;
}
// Fills in every related-records slot for a record being edited - each
// tab's `data-related-slot` (Screen/App Builder Phase 3 placement) plus
// the always-visible `#relatedRecordsPanel` for anything no tab claims -
// and (re)wires every link/unlink control across all of them. Called
// once when the form opens and again after every link/unlink/cancel, the
// same self-re-rendering pattern the old single-panel version used.
function refreshRelatedPanels(entityType,entityId,tabsOut){
 const claimed=tabsOut.flatMap(t=>t.related);
 tabsOut.forEach((t,i)=>{
  const slot=document.querySelector(`[data-related-slot="${i}"]`);
  if(slot)slot.innerHTML=relatedGroupsHtml(relatedGroupsFor(entityType,entityId,t.related));
 });
 const leftoverKeys=relationshipDefsFor(entityType).map(d=>d.key).filter(k=>!claimed.includes(k));
 const leftoverPanel=$('#relatedRecordsPanel');
 if(leftoverPanel)leftoverPanel.innerHTML=relatedGroupsHtml(relatedGroupsFor(entityType,entityId,leftoverKeys));

 document.querySelectorAll('[data-link-group]').forEach(btn=>{
  btn.onclick=()=>{relLinkingKey=relLinkingKey===btn.dataset.linkGroup?null:btn.dataset.linkGroup;refreshRelatedPanels(entityType,entityId,tabsOut)};
 });
 document.querySelectorAll('[data-unlink]').forEach(b=>b.onclick=()=>{
  data.relationshipInstances=(data.relationshipInstances||[]).filter(i=>i.id!==b.dataset.unlink);
  save();toast('Unlinked');refreshRelatedPanels(entityType,entityId,tabsOut);
 });
 // relLinkingKey is a single value, so at most one group across every
 // slot + the leftover panel ever renders a link picker - looking it up
 // unscoped (rather than per-container) is safe.
 const allGroups=[...tabsOut.flatMap(t=>relatedGroupsFor(entityType,entityId,t.related)),...relatedGroupsFor(entityType,entityId,leftoverKeys)];
 const linkGroup=allGroups.find(g=>g.def.key===relLinkingKey);
 if(linkGroup){
  const select=document.querySelector('[data-link-select]'), linkBtn=document.querySelector('[data-link-submit]'), cancelBtn=document.querySelector('[data-link-cancel]');
  if(cancelBtn)cancelBtn.onclick=()=>{relLinkingKey=null;refreshRelatedPanels(entityType,entityId,tabsOut)};
  if(linkBtn)linkBtn.onclick=()=>{
   const otherId=select.value; if(!otherId)return;
   const sourceEntity=linkGroup.isSource?entityType:linkGroup.otherType, sourceId=linkGroup.isSource?entityId:otherId;
   const targetEntity=linkGroup.isSource?linkGroup.otherType:entityType, targetId=linkGroup.isSource?otherId:entityId;
   const err=relationshipLinkError(linkGroup.def,sourceId,targetId);
   if(err)return alert(err);
   // Relationship instances only ever get created_by (see the desktop
   // RelationshipInstance model's own comment - never updated, only
   // created/deleted).
   data.relationshipInstances.push({id:uid(),definitionId:linkGroup.def.id,sourceEntity,sourceId,targetEntity,targetId,createdAt:new Date().toISOString(),createdBy:CURRENT_USER_ID});
   save();toast('Linked');relLinkingKey=null;refreshRelatedPanels(entityType,entityId,tabsOut);
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
function tabLabel(key){return (ADMIN_TAB_DEFS.find(t=>t[0]===key)||[])[1]||key}
function adminPage(){
 document.title='Admin — Lanesra OS Demo';
 if(adminView==='landing')return adminLanding();
 return adminToolView();
}
function adminLanding(){
 $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><span>Admin</span></div><h1>Admin</h1><p class="muted">Configure your workspace, users and automation. Changes save immediately in this browser.</p></div></div>
 <div class="admin-landing-grid">${ADMIN_CATEGORIES.map(c=>`<div class="admin-cat-card"><div class="admin-cat-head"><span class="admin-cat-icon">${c.icon}</span><div><h3>${c.label}</h3><p class="muted">${c.note}</p></div></div><div class="admin-cat-items">${c.items.map(k=>`<button class="admin-cat-item" data-admin-open="${k}">${tabLabel(k)}<span class="admin-cat-arrow">→</span></button>`).join('')}</div></div>`).join('')}</div>`;
 $('[data-clear-filter]').onclick=()=>{current='dashboard';viewFilter=null;renderView()};
 document.querySelectorAll('[data-admin-open]').forEach(b=>b.onclick=()=>{adminTab=b.dataset.adminOpen;adminView='tool';renderView()});
}
function adminToolView(){
 $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><button data-admin-home>Admin</button><span>›</span><span>${tabLabel(adminTab)}</span></div><h1>${tabLabel(adminTab)}</h1></div></div><div id="adminBody" class="admin-body"></div>`;
 $('[data-clear-filter]').onclick=()=>{current='dashboard';viewFilter=null;renderView()};
 $('[data-admin-home]').onclick=()=>{adminView='landing';renderView()};
 renderAdminTab();
}
function renderAdminTab(){
 document.querySelectorAll('[data-admin-tab]').forEach(b=>b.classList.toggle('active',b.dataset.adminTab===adminTab));
 const body=$('#adminBody');
 ({profile:profileTab,users:usersTab,objects:objectsTab,relationships:relationshipsTab,fields:fieldsTab,rules:rulesTab,workflow:workflowTab,transitions:transitionsTab,layouts:layoutsTab,apps:appsTab,packages:packagesTab,solutions:solutionsTab,integrations:integrationsTab,numbering:numberingTab,kpis:kpisTab,dashboards:dashboardsTab}[adminTab])(body);
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
   const newId=uid();
   data.customObjects.push({id:newId,key,label:fd.singular,labelPlural:fd.plural,icon:fd.icon,prefix,digits,active:true});
   data[key]=[];
   tagLocalComponent('customObject',newId);
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
 ${isEdit?`<div class="field full">${auditByline(def)}</div>`:''}
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
   stampUpdate(def);
  }else{
   if(fd.source===fd.target)return alert('A relationship must connect two different object types.');
   const base=`${fd.source}_${fd.target}`;
   let key=base,suffix=2;
   while((data.relationshipDefinitions||[]).some(d=>d.key===key))key=`${base}_${suffix++}`;
   const newRel=stampCreate({id:uid(),key,sourceEntity:fd.source,targetEntity:fd.target,relType:fd.relType,forwardLabel:fd.forwardLabel,reverseLabel:fd.reverseLabel,deleteBehavior:fd.deleteBehavior,showRelatedList:fd.showRelatedList==='true',required:fd.required==='true',active:true,protected:false});
   data.relationshipDefinitions.push(newRel);
   tagLocalComponent('relationship',newRel.id);
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
// ---- Screen layouts (Screen/App Builder Phase 1: multi-layout, tabs, roles)
// A new capability, not a desktop port (desktop had no layout designer
// either until this same round). An object can have several named
// layouts, not just one - each with its own tabs of drag-ordered field
// sections - and an admin assigns roles to a layout so different roles
// see a different arrangement; exactly one layout is the Default, the
// fallback for any role no other layout claims.
//
// This demo has no signed-in user/session (see usersTab's own "roles are
// illustrative" note), so the live create/edit form always renders the
// object's *Default* layout - there's no real viewer identity to resolve
// role assignment against. Role assignment is still fully editable here
// (illustrative, same honesty level as Users & roles) and Preview always
// shows the layout you're actively editing regardless of role, so the
// tabs/sections mechanics themselves are fully exercisable.
//
// Editing only ever touches a layout's *draft* - Publish copies draftTabs
// to publishedTabs, Unpublish clears the published copy back to null. A
// published layout never hides a field it doesn't know about: any field
// missing from it (a new custom field added after publishing, a stale key
// from a deleted one) is auto-appended to a trailing "Other fields"
// section, so a layout can never silently drop something off the form.
const DEMO_LAYOUT_ROLES=['Administrator','Sales Rep','Viewer'];
const SECTION_COLUMN_CHOICES=[1,2,3];
let layoutsEntityKey=null;
let layoutsSelectedLayoutId=null;
let layoutsActiveTabIdx=0;
let dashboardsSelectedId=null;
function allFieldsFor(entityKey){return fieldsFor(entityKey,fieldsFnFor(entityKey))}
// Screen/App Builder Phase 2: a section field is `{key,full}` - `full`
// spans the section's full width instead of one column. Accepts a plain
// key string too (the Phase 1 shape, `full` defaulting to false), so a
// layout saved before Phase 2 still normalizes cleanly - see
// `normalizeSection` below, the same backward-compat approach the Rust
// core's `deserialize_fields` takes for the desktop edition.
function freshSectionField(key,full=false){return {key,full}}
function normalizeFieldEntry(sf){return typeof sf==='string'?freshSectionField(sf):{key:sf.key,full:!!sf.full}}
function normalizeSection(s){return {...s,columns:s.columns||2,fields:(s.fields||[]).map(normalizeFieldEntry)}}
function freshTab(entityKey){return {id:uid(),title:'Details',sections:[{id:uid(),title:'Details',columns:2,fields:allFieldsFor(entityKey).map(f=>freshSectionField(f[0]))}],related:[]}}
function freshLayout(entityKey,name,isDefault){return {id:uid(),name,isDefault,roles:[],draftTabs:[freshTab(entityKey)],publishedTabs:null,updatedAt:null}}
// Migrates the old single-layout-per-entity shape (a bare
// {draftSections,publishedSections}) into an array of named layouts, and
// auto-provisions a Default layout for any entity with none yet. Also
// normalizes every section to the Phase 2 shape (a `columns` count, each
// field a `{key,full}` pair) and every tab to carry a `related` array
// (Phase 3 - relationship-definition keys whose related-records list
// shows on that tab) - in memory only, the same as the layouts-array
// migration below it; the next actual edit's `save()` persists it.
function ensureLayouts(entityKey){
 if(!data.uiLayouts)data.uiLayouts={};
 let arr=data.uiLayouts[entityKey];
 if(arr&&!Array.isArray(arr)){
  arr=[{id:uid(),name:'Default',isDefault:true,roles:[],
   draftTabs:[{id:uid(),title:'Details',sections:arr.draftSections||[]}],
   publishedTabs:arr.publishedSections?[{id:uid(),title:'Details',sections:arr.publishedSections}]:null,
   updatedAt:arr.updatedAt||null}];
 }
 if(!arr||!arr.length)arr=[freshLayout(entityKey,'Default',true)];
 if(!arr.some(l=>l.isDefault))arr[0].isDefault=true;
 arr.forEach(l=>{
  l.draftTabs.forEach(t=>{t.sections=t.sections.map(normalizeSection);t.related=t.related||[]});
  if(l.publishedTabs)l.publishedTabs.forEach(t=>{t.sections=t.sections.map(normalizeSection);t.related=t.related||[]});
 });
 data.uiLayouts[entityKey]=arr;
 return arr;
}
function defaultLayoutFor(entityKey){const arr=ensureLayouts(entityKey);return arr.find(l=>l.isDefault)||arr[0]}
function layoutById(entityKey,id){return ensureLayouts(entityKey).find(l=>l.id===id)}
// Groups fields into the tab/section structure `tabs` describes (a
// layout's draftTabs or publishedTabs); `tabs` null/undefined means "no
// layout in effect" - a single untitled tab/section in the fields' plain
// default order, exactly the pre-layout-builder behavior. Each group
// carries its own `columns` (Phase 2) and each resolved field its `full`
// flag; each output tab also carries its `related` list (Phase 3 -
// relationship-definition keys), threaded straight through unchanged. A
// tab survives even with zero field groups as long as it claims a
// related list - a tab can be purely a related-records tab.
function orderedTabsFor(fields,tabs){
 const plainGroup=()=>({title:null,columns:2,items:fields.map(f=>({field:f,full:undefined}))});
 if(!tabs||!tabs.length)return [{title:null,groups:[plainGroup()],related:[]}];
 const byKey=Object.fromEntries(fields.map(f=>[f[0],f]));
 const used=new Set();
 const out=tabs.map(t=>{
  const groups=t.sections.map(s=>{
   const items=(s.fields||[]).map(normalizeFieldEntry).map(sf=>{
    const field=byKey[sf.key];
    if(!field)return null;
    used.add(sf.key);
    return {field,full:sf.full};
   }).filter(Boolean);
   return {title:s.title,columns:s.columns||2,items};
  }).filter(g=>g.items.length);
  return {title:t.title,groups,related:t.related||[]};
 }).filter(t=>t.groups.length||t.related.length);
 const rest=fields.filter(f=>!used.has(f[0]));
 if(rest.length){
  const restGroup={title:'Other fields',columns:2,items:rest.map(f=>({field:f,full:undefined}))};
  if(out.length)out[out.length-1].groups.push(restGroup);
  else out.push({title:null,groups:[restGroup],related:[]});
 }
 return out.length?out:[{title:null,groups:[plainGroup()],related:[]}];
}
// Renders `orderedTabsFor`'s groups for one tab panel - each group is its
// own `.form-grid` (Phase 2: a section's own `columns` count, not one
// grid shared by every section in the tab), with the section title (if
// any) as a plain heading above it rather than a grid cell.
function groupsHtml(groups,record){
 return groups.map(g=>`${g.title?`<h4 style="margin:0 0 8px">${g.title}</h4>`:''}<div class="form-grid" style="grid-template-columns:repeat(${g.columns},1fr);margin-bottom:14px">${g.items.map(it=>fieldHtml(it.field,record,it.full)).join('')}</div>`).join('');
}
function layoutsTab(body){
 const keys=allEntityTypeKeys();
 if(!layoutsEntityKey||!keys.includes(layoutsEntityKey))layoutsEntityKey=keys[0];
 const entityKey=layoutsEntityKey;
 const layouts=ensureLayouts(entityKey);
 if(!layoutsSelectedLayoutId||!layouts.some(l=>l.id===layoutsSelectedLayoutId))layoutsSelectedLayoutId=defaultLayoutFor(entityKey).id;
 const layout=layoutById(entityKey,layoutsSelectedLayoutId);
 if(layoutsActiveTabIdx>=layout.draftTabs.length)layoutsActiveTabIdx=0;
 const isPublished=!!layout.publishedTabs;
 const hasDraftChanges=JSON.stringify(layout.draftTabs)!==JSON.stringify(layout.publishedTabs);
 body.innerHTML=`<div class="panel">
 <div class="panel-head"><h3>Screen layouts</h3><select id="layoutsEntitySelect">${keys.map(k=>`<option value="${k}" ${k===entityKey?'selected':''}>${entityLabel(k)}</option>`).join('')}</select></div>
 <p class="muted" style="font-size:13px">Build multiple named layouts for ${entityLabel(entityKey)}'s create/edit form, each with its own tabs of drag-ordered field sections, and assign roles to each. There's no signed-in user in this browser demo, so the live form always uses the <b>Default</b> layout — role assignment here is illustrative, the same as Users & roles; the desktop edition resolves it against the real signed-in user.</p>
 <div class="panel-head" style="margin-top:4px">
  <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap">${layouts.map(l=>`<button type="button" class="tab ${l.id===layout.id?'active':''}" data-select-layout="${l.id}">${l.name}${l.isDefault?' · Default':''}</button>`).join('')}</div>
  <button class="btn btn-secondary" id="addLayout" type="button">+ New layout</button>
 </div>
 <div class="layout-meta" style="display:flex;gap:12px;align-items:flex-end;flex-wrap:wrap;margin:12px 0">
  <div class="field" style="margin:0"><label>Layout name</label><input id="layoutName" value="${layout.name}" style="border:1px solid var(--line);border-radius:8px;padding:6px 9px"></div>
  <div class="field" style="margin:0"><label>Visible to roles</label><div style="display:flex;gap:10px;flex-wrap:wrap;padding-top:6px">${DEMO_LAYOUT_ROLES.map(r=>`<label style="font-size:13px;display:flex;gap:5px;align-items:center"><input type="checkbox" data-layout-role="${r}" ${layout.roles.includes(r)?'checked':''}> ${r}</label>`).join('')}</div></div>
  ${layout.isDefault?'<span class="badge">Default layout — fallback for any unassigned role</span>':'<button class="btn btn-secondary" id="makeDefaultLayout" type="button">Make default</button>'}
  <button class="btn btn-secondary" id="deleteLayout" type="button" ${layouts.length<=1||layout.isDefault?'disabled':''}>Delete layout</button>
 </div>
 <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin-bottom:10px">${layout.draftTabs.map((t,i)=>`<button type="button" class="tab ${i===layoutsActiveTabIdx?'active':''}" data-select-tab="${i}">${t.title}</button>`).join('')}<button class="icon-btn" id="addTab" type="button">+ Add tab</button></div>
 <div style="margin-bottom:14px"><span class="badge">${isPublished?(hasDraftChanges?'Published — unpublished draft changes':'Published'):'Not published — using default field order'}</span></div>
 <div id="layoutTabMeta"></div>
 <div id="layoutSections"></div>
 <div id="layoutRelated"></div>
 <div class="actions" style="margin-top:16px;flex-wrap:wrap">
  <button class="btn btn-secondary" id="addSection" type="button">+ Add section</button>
  <button class="btn btn-secondary" id="previewLayout" type="button">Preview draft</button>
  <button class="btn btn-secondary" id="revertLayout" type="button" ${isPublished?'':'disabled'}>Revert draft to published</button>
  <button class="btn btn-primary" id="publishLayout" type="button">Publish</button>
  ${isPublished?'<button class="btn btn-secondary" id="unpublishLayout" type="button">Unpublish</button>':''}
 </div>
 </div>`;
 $('#layoutsEntitySelect').onchange=e=>{layoutsEntityKey=e.target.value;layoutsSelectedLayoutId=null;layoutsActiveTabIdx=0;layoutsTab(body)};
 body.querySelectorAll('[data-select-layout]').forEach(b=>b.onclick=()=>{layoutsSelectedLayoutId=b.dataset.selectLayout;layoutsActiveTabIdx=0;layoutsTab(body)});
 $('#addLayout').onclick=()=>{const name=prompt('New layout name?','New layout');if(!name)return;const l=freshLayout(entityKey,name.trim(),false);layouts.push(l);tagLocalComponent('screenLayout',l.id);save();layoutsSelectedLayoutId=l.id;layoutsActiveTabIdx=0;layoutsTab(body)};
 // setTimeout defers the re-render past this same change event's own
 // dispatch/blur handling - re-rendering (tearing down #layoutName) while
 // the browser is still processing the event that fired on it throws
 // "node is no longer a child of this node".
 $('#layoutName').onchange=e=>{layout.name=e.target.value.trim()||layout.name;save();setTimeout(()=>layoutsTab(body),0)};
 body.querySelectorAll('[data-layout-role]').forEach(cb=>cb.onchange=()=>{
  const role=cb.dataset.layoutRole;
  layout.roles=cb.checked?[...layout.roles,role]:layout.roles.filter(r=>r!==role);
  save();
 });
 const makeDefault=$('#makeDefaultLayout'); if(makeDefault)makeDefault.onclick=()=>{layouts.forEach(l=>l.isDefault=(l.id===layout.id));save();toast(`${layout.name} is now the default layout`);layoutsTab(body)};
 $('#deleteLayout').onclick=()=>{
  if(layouts.length<=1||layout.isDefault)return;
  if(!confirm(`Delete the "${layout.name}" layout? This can't be undone.`))return;
  data.uiLayouts[entityKey]=layouts.filter(l=>l.id!==layout.id);
  save();layoutsSelectedLayoutId=null;layoutsActiveTabIdx=0;toast('Layout deleted');layoutsTab(body);
 };
 body.querySelectorAll('[data-select-tab]').forEach(b=>b.onclick=()=>{layoutsActiveTabIdx=Number(b.dataset.selectTab);layoutsTab(body)});
 $('#addTab').onclick=()=>{layout.draftTabs.push(freshTabEmpty());save();layoutsActiveTabIdx=layout.draftTabs.length-1;layoutsTab(body)};
 renderLayoutTabMeta(entityKey);
 renderLayoutSections(entityKey);
 renderLayoutRelated(entityKey);
 $('#addSection').onclick=()=>{layout.draftTabs[layoutsActiveTabIdx].sections.push({id:uid(),title:'New section',columns:2,fields:[]});save();renderLayoutSections(entityKey)};
 $('#previewLayout').onclick=()=>layoutPreviewModal(entityKey);
 $('#publishLayout').onclick=()=>{layout.publishedTabs=structuredClone(layout.draftTabs);layout.updatedAt=new Date().toISOString();save();toast('Layout published');layoutsTab(body)};
 $('#revertLayout').onclick=()=>{if(!layout.publishedTabs)return;layout.draftTabs=structuredClone(layout.publishedTabs);save();toast('Draft reverted to the published layout');layoutsTab(body)};
 const unpub=$('#unpublishLayout'); if(unpub)unpub.onclick=()=>{if(!confirm('Unpublish this layout? Any role assigned to it falls back to the Default layout until you publish again.'))return;layout.publishedTabs=null;save();toast('Layout unpublished');layoutsTab(body)};
}
function freshTabEmpty(){return {id:uid(),title:'New tab',sections:[{id:uid(),title:'Section',columns:2,fields:[]}]}}
function renderLayoutTabMeta(entityKey){
 const layout=layoutById(entityKey,layoutsSelectedLayoutId);
 const tab=layout.draftTabs[layoutsActiveTabIdx];
 const box=$('#layoutTabMeta'); if(!box)return;
 box.innerHTML=`<div style="display:flex;gap:8px;align-items:center;margin-bottom:10px">
  <input id="tabTitle" value="${tab.title}" style="border:1px solid var(--line);border-radius:8px;padding:6px 9px;font-weight:700">
  <button class="icon-btn" id="deleteTab" type="button" ${layout.draftTabs.length<=1?'disabled':''}>Delete tab</button>
 </div>`;
 // Same deferred-re-render reasoning as #layoutName above.
 $('#tabTitle').onchange=e=>{tab.title=e.target.value.trim()||'Tab';save();setTimeout(()=>layoutsTab($('#adminBody')),0)};
 $('#deleteTab').onclick=()=>{if(layout.draftTabs.length<=1)return;layout.draftTabs.splice(layoutsActiveTabIdx,1);layoutsActiveTabIdx=0;save();layoutsTab($('#adminBody'))};
}
function renderLayoutSections(entityKey){
 const layout=layoutById(entityKey,layoutsSelectedLayoutId);
 const tab=layout.draftTabs[layoutsActiveTabIdx];
 const allFields=allFieldsFor(entityKey);
 const fieldLabel=k=>{const f=allFields.find(x=>x[0]===k);return f?f[1]:k};
 // "Available" is scoped to *this* tab - not just fields placed nowhere
 // in the whole layout, but also anything currently sitting on a
 // different tab, so dragging it into a section here moves it onto this
 // tab (see moveField's own comment on why that's a move, not a copy).
 const placedInThisTab=new Set(tab.sections.flatMap(s=>s.fields.map(f=>f.key)));
 const available=allFields.map(f=>f[0]).filter(k=>!placedInThisTab.has(k));
 // `full`/`onCols` are only meaningful for a chip actually sitting in a
 // section (idx>=0) - the "Available" bucket isn't part of any grid yet.
 const chip=(key,idx,full,onCols)=>`<span class="layout-field-chip" draggable="true" data-field-key="${key}" data-section-idx="${idx}" style="border:1px solid var(--line);border-radius:8px;padding:6px 10px;background:${idx>=0?'#f9fafb':'#fff'};cursor:grab;font-size:13px;display:inline-flex;align-items:center;gap:6px">⠿ ${fieldLabel(key)}${idx>=0?`<button type="button" class="link-btn" draggable="false" data-toggle-full data-section-idx="${idx}" data-field-key="${key}" style="font-size:11px" ${onCols===1?'disabled':''} title="${full?'Full width - click to shrink to one column':'One column - click to span the full section'}">${full?'⭤ full':'⭤ 1 col'}</button>`:''}</span>`;
 const listHtml=(entries,idx,columns)=>`<div class="layout-field-list" data-section-idx="${idx}" style="display:flex;flex-wrap:wrap;gap:8px;min-height:34px">${entries.map(e=>chip(e.key,idx,e.full,columns)).join('')||'<span class="muted" style="font-size:12px">Drag fields here</span>'}</div>`;
 const columnsPicker=(idx,current)=>`<div style="display:flex;align-items:center;gap:6px;font-size:12px;color:#6b7280">Columns${SECTION_COLUMN_CHOICES.map(n=>`<button type="button" class="tab ${current===n?'active':''}" data-set-columns="${idx}" data-columns="${n}" style="padding:2px 10px">${n}</button>`).join('')}</div>`;
 const box=$('#layoutSections'); if(!box)return;
 box.innerHTML=`${tab.sections.map((s,idx)=>`<div class="layout-section" style="border:1px solid var(--line);border-radius:12px;padding:12px;margin-bottom:12px">
  <div style="display:flex;justify-content:space-between;align-items:center;gap:8px;margin-bottom:8px;flex-wrap:wrap">
   <input class="layout-section-title" data-section-idx="${idx}" value="${s.title}" style="border:1px solid var(--line);border-radius:8px;padding:6px 9px;font-weight:700;flex:1;min-width:120px">
   ${columnsPicker(idx,s.columns)}
   <button class="icon-btn" data-remove-section="${idx}" type="button" ${tab.sections.length<=1?'disabled':''}>Delete section</button>
  </div>
  ${listHtml(s.fields,idx,s.columns)}
 </div>`).join('')}
 <div class="layout-section" style="border:1px dashed var(--line);border-radius:12px;padding:12px">
  <div class="muted" style="font-weight:700;margin-bottom:8px">Available fields — not on this tab (drag into a section above to place here)</div>
  ${listHtml(available.map(k=>({key:k,full:undefined})),-1,2)}
 </div>`;
 box.querySelectorAll('.layout-section-title').forEach(inp=>inp.onchange=e=>{const idx=Number(e.target.dataset.sectionIdx);tab.sections[idx].title=e.target.value.trim()||'Section';save()});
 box.querySelectorAll('[data-remove-section]').forEach(b=>b.onclick=()=>{const idx=Number(b.dataset.removeSection);if(tab.sections.length<=1)return;tab.sections.splice(idx,1);save();renderLayoutSections(entityKey)});
 box.querySelectorAll('[data-set-columns]').forEach(b=>b.onclick=()=>{const idx=Number(b.dataset.setColumns);tab.sections[idx].columns=Number(b.dataset.columns);save();renderLayoutSections(entityKey)});
 box.querySelectorAll('[data-toggle-full]').forEach(b=>b.onclick=()=>{
  const idx=Number(b.dataset.sectionIdx),key=b.dataset.fieldKey;
  const f=tab.sections[idx].fields.find(x=>x.key===key);
  if(f){f.full=!f.full;save();renderLayoutSections(entityKey)}
 });
 wireLayoutDragDrop(entityKey);
}
function wireLayoutDragDrop(entityKey){
 const layout=layoutById(entityKey,layoutsSelectedLayoutId);
 const tab=layout.draftTabs[layoutsActiveTabIdx];
 let dragKey=null;
 function moveField(toIdx,beforeKey){
  if(dragKey===null)return;
  // A field lives in at most one section across the *whole* layout, not
  // just this tab - strip it from every tab's sections first, so dragging
  // a field that's currently on a different tab into this tab's section
  // moves it here instead of placing a second copy. Its `full` choice
  // travels with it (a re-drag is a move, not a fresh placement); a field
  // coming from the "Available" bucket has no prior entry, so defaults to
  // one column, same as picking a brand new field on desktop.
  let full=false;
  layout.draftTabs.forEach(t=>t.sections.forEach(s=>{
   const existing=s.fields.find(f=>f.key===dragKey);
   if(existing)full=existing.full;
   s.fields=s.fields.filter(f=>f.key!==dragKey);
  }));
  if(toIdx>=0){
   const s=tab.sections[toIdx];
   const insertAt=beforeKey?s.fields.findIndex(f=>f.key===beforeKey):-1;
   s.fields.splice(insertAt<0?s.fields.length:insertAt,0,{key:dragKey,full});
  }
  save();dragKey=null;
  renderLayoutSections(entityKey);
 }
 document.querySelectorAll('.layout-field-chip').forEach(el=>{
  el.ondragstart=e=>{dragKey=el.dataset.fieldKey;e.dataTransfer.effectAllowed='move'};
  el.ondragover=e=>{e.preventDefault();e.stopPropagation()};
  el.ondrop=e=>{e.preventDefault();e.stopPropagation();moveField(Number(el.dataset.sectionIdx),el.dataset.fieldKey)};
 });
 document.querySelectorAll('.layout-field-list').forEach(list=>{
  list.ondragover=e=>{e.preventDefault();e.dataTransfer.dropEffect='move'};
  list.ondrop=e=>{e.preventDefault();moveField(Number(list.dataset.sectionIdx),null)};
 });
}
// Screen/App Builder Phase 3: lets an admin pick which of this object's
// relationships show their related-records list on the active tab. A
// relationship shows on at most one tab, same "only lives in one place"
// rule fields follow - checking it here strips it from every other tab
// first. Nothing to place at all (no applicable relationships) renders
// nothing rather than an empty box.
function renderLayoutRelated(entityKey){
 const layout=layoutById(entityKey,layoutsSelectedLayoutId);
 const tab=layout.draftTabs[layoutsActiveTabIdx];
 const defs=relationshipDefsFor(entityKey);
 const box=$('#layoutRelated'); if(!box)return;
 if(!defs.length){box.innerHTML='';return}
 box.innerHTML=`<div class="layout-section" style="border:1px solid var(--line);border-radius:12px;padding:12px;margin-top:12px">
  <div style="font-weight:700;margin-bottom:6px">Related lists</div>
  <p class="muted" style="font-size:12px;margin:0 0 8px">Shows once a record exists to link against — not on the create form. A relationship left unchecked everywhere still shows, in an always-visible spot below the tabs.</p>
  <div style="display:flex;gap:12px;flex-wrap:wrap">${defs.map(def=>{
   const isSource=def.sourceEntity===entityKey;
   const label=isSource?def.forwardLabel:def.reverseLabel;
   return `<label style="font-size:13px;display:flex;gap:6px;align-items:center"><input type="checkbox" data-related-key="${def.key}" ${tab.related.includes(def.key)?'checked':''}> ${label}</label>`;
  }).join('')}</div>
 </div>`;
 box.querySelectorAll('[data-related-key]').forEach(cb=>cb.onchange=()=>{
  const key=cb.dataset.relatedKey;
  layout.draftTabs.forEach(t=>{t.related=t.related.filter(r=>r!==key)});
  if(cb.checked)tab.related=[...tab.related,key];
  save();
 });
}
function layoutPreviewModal(entityKey){
 const fields=allFieldsFor(entityKey);
 const layout=layoutById(entityKey,layoutsSelectedLayoutId);
 const sample={};
 const tabsOut=orderedTabsFor(fields,layout.draftTabs);
 const defs=relationshipDefsFor(entityKey);
 const relatedLabel=key=>{const def=defs.find(d=>d.key===key);if(!def)return key;return def.sourceEntity===entityKey?def.forwardLabel:def.reverseLabel};
 const tabsHtml=tabsOut.length>1?`<div class="layout-tabs" style="display:flex;gap:8px;margin-bottom:12px">${tabsOut.map((t,i)=>`<button type="button" class="tab ${i===0?'active':''}" data-preview-tab="${i}">${t.title||'Details'}</button>`).join('')}</div>`:'';
 const panelsHtml=tabsOut.map((t,i)=>`<div data-preview-panel="${i}" style="${i===0?'':'display:none'}">${groupsHtml(t.groups,sample)}${t.related.length?`<div class="panel" style="margin-top:12px"><strong style="font-size:13px">Related records</strong><p class="muted" style="font-size:12px;margin:4px 0 0">${t.related.map(relatedLabel).join(', ')} — shown here once a record exists to link against.</p></div>`:''}</div>`).join('');
 const body=`${tabsHtml}${panelsHtml}<p class="muted" style="font-size:12px;margin-top:12px">Preview of "${layout.name}"'s draft only — nothing here is saved, and the live form is unaffected until you Publish.</p><div class="modal-actions"><button class="btn btn-secondary" type="button" data-close>Close</button></div>`;
 modal(`Preview: ${entityLabel(entityKey)} — ${layout.name}`,body);
 $('[data-close]').onclick=closeModal;
 document.querySelectorAll('[data-preview-tab]').forEach(b=>b.onclick=()=>{
  document.querySelectorAll('[data-preview-panel]').forEach(p=>p.style.display=p.dataset.previewPanel===b.dataset.previewTab?'':'none');
  document.querySelectorAll('[data-preview-tab]').forEach(x=>x.classList.toggle('active',x===b));
 });
}
// ---- Dashboards (mirrors the desktop edition's dashboard_layout_service/
// dashboard_widget_service, all 3 phases at once) --------------------------
// Structurally the same draft/publish/role-assignment feature as Screen
// layouts above, just at the workspace level: one dashboard per layout,
// not one per object, so there's no entity selector here. Widgets: 'kpi'
// (a KPI_DEFS tile), 'chart' (an existing saved custom report, run fresh -
// see runCustomReport above), 'record_list' (a short list of records for
// one entity type - see dashboardRecordListRows above).
function dashboardWidgetLabel(w,reports){
 if(w.kind==='kpi'){const k=KPI_DEFS.find(k=>k.key===w.config.kpiKey);return k?k.label:'(KPI removed)'}
 if(w.kind==='chart'){const r=reports.find(r=>r.id===w.config.reportId);return r?`📊 ${r.name}`:'📊 (report deleted)'}
 if(w.kind==='record_list')return `📋 ${entityLabel(w.config.entityKey)} — ${w.config.mode==='due_soon'?'due soon':'recent'}`;
 return w.kind;
}
function dashboardsTab(body){
 const dashboards=ensureDashboards();
 if(!dashboardsSelectedId||!dashboards.some(d=>d.id===dashboardsSelectedId))dashboardsSelectedId=defaultDashboard().id;
 const dash=dashboardById(dashboardsSelectedId);
 const visibleDashboards=dashboards.filter(d=>matchesAppFilter(d.appId,dashAppFilter));
 const isPublished=!!dash.publishedWidgets;
 const hasDraftChanges=JSON.stringify(dash.draftWidgets)!==JSON.stringify(dash.publishedWidgets);
 const reports=data.customReports||[];
 const usedKpiKeys=new Set(dash.draftWidgets.filter(w=>w.kind==='kpi').map(w=>w.config.kpiKey));
 const availableKpis=KPI_DEFS.filter(k=>!usedKpiKeys.has(k.key));
 const usedReportIds=new Set(dash.draftWidgets.filter(w=>w.kind==='chart').map(w=>w.config.reportId));
 const availableReports=reports.filter(r=>!usedReportIds.has(r.id));
 const entityKeys=allEntityTypeKeys();
 body.innerHTML=`<div class="panel">
 <div class="panel-head"><h3>Dashboards</h3></div>
 <p class="muted" style="font-size:13px">Build multiple named dashboard layouts — an ordered list of widgets — and assign roles to each. There's no signed-in user in this browser demo, so the live Dashboard always uses the <b>Default</b> dashboard — role assignment here is illustrative, the same as Screen layouts and Users & roles.</p>
 ${appFilterPills(dashAppFilter)}
 <div class="panel-head" style="margin-top:4px">
  <div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap">${visibleDashboards.map(d=>`<button type="button" class="tab ${d.id===dash.id?'active':''}" data-select-dashboard="${d.id}">${d.name}${d.isDefault?' · Default':''}</button>`).join('')}${visibleDashboards.length?'':'<span class="muted" style="font-size:13px">No dashboards match this app filter.</span>'}</div>
  <button class="btn btn-secondary" id="addDashboard" type="button">+ New dashboard</button>
 </div>
 <div class="layout-meta" style="display:flex;gap:12px;align-items:flex-end;flex-wrap:wrap;margin:12px 0">
  <div class="field" style="margin:0"><label>Dashboard name</label><input id="dashboardName" value="${dash.name}" style="border:1px solid var(--line);border-radius:8px;padding:6px 9px"></div>
  <div class="field" style="margin:0"><label>Visible to roles</label><div style="display:flex;gap:10px;flex-wrap:wrap;padding-top:6px">${DEMO_LAYOUT_ROLES.map(r=>`<label style="font-size:13px;display:flex;gap:5px;align-items:center"><input type="checkbox" data-dashboard-role="${r}" ${dash.roles.includes(r)?'checked':''}> ${r}</label>`).join('')}</div></div>
  ${appSelectHtml('dashboardApp',dash.appId||null)}
  ${dash.isDefault?'<span class="badge">Default dashboard — fallback for any unassigned role</span>':'<button class="btn btn-secondary" id="makeDefaultDashboard" type="button">Make default</button>'}
  <button class="btn btn-secondary" id="deleteDashboard" type="button" ${dashboards.length<=1||dash.isDefault?'disabled':''}>Delete dashboard</button>
 </div>
 <div style="margin-bottom:14px"><span class="badge">${isPublished?(hasDraftChanges?'Published — unpublished draft changes':'Published'):'Not published — Dashboard shows the fixed KPI picker selection'}</span></div>
 <div style="font-weight:700;margin-bottom:8px">Widgets</div>
 <div style="display:flex;flex-wrap:wrap;gap:8px;margin:8px 0">${dash.draftWidgets.length?dash.draftWidgets.map((w,i)=>`<span class="badge" style="display:inline-flex;align-items:center;gap:6px">${dashboardWidgetLabel(w,reports)}<button class="icon-btn" data-move-widget="${w.id}" data-dir="-1" ${i===0?'disabled':''} type="button" title="Move earlier">↑</button><button class="icon-btn" data-move-widget="${w.id}" data-dir="1" ${i===dash.draftWidgets.length-1?'disabled':''} type="button" title="Move later">↓</button><button class="icon-btn" data-remove-widget="${w.id}" type="button" title="Remove">×</button></span>`).join(''):'<span class="muted">No widgets yet.</span>'}</div>
 <div style="display:flex;gap:8px;flex-wrap:wrap;align-items:center">
  ${availableKpis.length?`<select id="addKpiWidget"><option value="">+ Add KPI tile…</option>${availableKpis.map(k=>`<option value="${k.key}">${k.label}</option>`).join('')}</select>`:''}
  ${reports.length===0?'<span class="muted" style="font-size:12px">No custom reports yet — build one in Reports → Custom reports to add it as a chart here.</span>':(availableReports.length?`<select id="addChartWidget"><option value="">+ Add chart…</option>${availableReports.map(r=>`<option value="${r.id}">${r.name}</option>`).join('')}</select>`:'')}
  <span style="display:inline-flex;gap:6px;align-items:center">
   <select id="recordListEntity">${entityKeys.map(k=>`<option value="${k}">${entityLabel(k)}</option>`).join('')}</select>
   <select id="recordListMode"></select>
   <button class="btn btn-secondary" id="addRecordListWidget" type="button">+ Add record list</button>
  </span>
 </div>
 <div class="actions" style="margin-top:16px;flex-wrap:wrap">
  <button class="btn btn-secondary" id="previewDashboard" type="button">Preview draft</button>
  <button class="btn btn-secondary" id="revertDashboard" type="button" ${isPublished?'':'disabled'}>Revert draft to published</button>
  <button class="btn btn-primary" id="publishDashboard" type="button">Publish</button>
  ${isPublished?'<button class="btn btn-secondary" id="unpublishDashboard" type="button">Unpublish</button>':''}
 </div>
 </div>`;
 $('#addDashboard').onclick=()=>{const name=prompt('New dashboard name?','New dashboard');if(!name)return;const d=freshDashboard(name.trim(),false,defaultAppIdFor(dashAppFilter));dashboards.push(d);save();dashboardsSelectedId=d.id;dashboardsTab(body)};
 body.querySelectorAll('[data-select-dashboard]').forEach(b=>b.onclick=()=>{dashboardsSelectedId=b.dataset.selectDashboard;dashboardsTab(body)});
 wireAppFilterPills(body,()=>dashAppFilter,v=>dashAppFilter=v,()=>dashboardsTab(body));
 const dashboardAppSelect=$('#dashboardApp'); if(dashboardAppSelect)dashboardAppSelect.onchange=e=>{dash.appId=e.target.value||null;save();dashboardsTab(body)};
 // Same deferred-re-render reasoning as layoutsTab's #layoutName.
 $('#dashboardName').onchange=e=>{dash.name=e.target.value.trim()||dash.name;save();setTimeout(()=>dashboardsTab(body),0)};
 body.querySelectorAll('[data-dashboard-role]').forEach(cb=>cb.onchange=()=>{
  const role=cb.dataset.dashboardRole;
  dash.roles=cb.checked?[...dash.roles,role]:dash.roles.filter(r=>r!==role);
  save();
 });
 const makeDefault=$('#makeDefaultDashboard'); if(makeDefault)makeDefault.onclick=()=>{dashboards.forEach(d=>d.isDefault=(d.id===dash.id));save();toast(`${dash.name} is now the default dashboard`);dashboardsTab(body)};
 $('#deleteDashboard').onclick=()=>{
  if(dashboards.length<=1||dash.isDefault)return;
  if(!confirm(`Delete the "${dash.name}" dashboard? This can't be undone.`))return;
  data.dashboards=dashboards.filter(d=>d.id!==dash.id);
  save();dashboardsSelectedId=null;toast('Dashboard deleted');dashboardsTab(body);
 };
 const addKpi=$('#addKpiWidget'); if(addKpi)addKpi.onchange=e=>{if(!e.target.value)return;dash.draftWidgets.push(freshDashboardWidget('kpi',{kpiKey:e.target.value}));save();dashboardsTab(body)};
 const addChart=$('#addChartWidget'); if(addChart)addChart.onchange=e=>{if(!e.target.value)return;dash.draftWidgets.push(freshDashboardWidget('chart',{reportId:e.target.value}));save();dashboardsTab(body)};
 const recordListEntity=$('#recordListEntity'),recordListMode=$('#recordListMode');
 function refreshRecordListModeOptions(){
  const dueSoonAvailable=DUE_SOON_ENTITY_KEYS.includes(recordListEntity.value);
  recordListMode.innerHTML=dueSoonAvailable
   ?'<option value="due_soon">Due soon</option><option value="recent">Recently created</option>'
   :'<option value="recent">Recently created</option>';
 }
 refreshRecordListModeOptions();
 recordListEntity.onchange=refreshRecordListModeOptions;
 $('#addRecordListWidget').onclick=()=>{
  dash.draftWidgets.push(freshDashboardWidget('record_list',{entityKey:recordListEntity.value,mode:recordListMode.value,limit:5}));
  save();dashboardsTab(body);
 };
 body.querySelectorAll('[data-move-widget]').forEach(b=>b.onclick=()=>{
  const idx=dash.draftWidgets.findIndex(w=>w.id===b.dataset.moveWidget);
  const swapWith=idx+Number(b.dataset.dir);
  if(idx<0||swapWith<0||swapWith>=dash.draftWidgets.length)return;
  [dash.draftWidgets[idx],dash.draftWidgets[swapWith]]=[dash.draftWidgets[swapWith],dash.draftWidgets[idx]];
  save();dashboardsTab(body);
 });
 body.querySelectorAll('[data-remove-widget]').forEach(b=>b.onclick=()=>{dash.draftWidgets=dash.draftWidgets.filter(w=>w.id!==b.dataset.removeWidget);save();dashboardsTab(body)});
 $('#previewDashboard').onclick=()=>dashboardPreviewModal();
 $('#publishDashboard').onclick=()=>{dash.publishedWidgets=structuredClone(dash.draftWidgets);save();toast('Dashboard published');dashboardsTab(body)};
 $('#revertDashboard').onclick=()=>{if(!dash.publishedWidgets)return;dash.draftWidgets=structuredClone(dash.publishedWidgets);save();toast('Draft reverted to the published dashboard');dashboardsTab(body)};
 const unpub=$('#unpublishDashboard'); if(unpub)unpub.onclick=()=>{if(!confirm('Unpublish this dashboard? Any role assigned to it falls back to the Default dashboard until you publish again.'))return;dash.publishedWidgets=null;save();toast('Dashboard unpublished');dashboardsTab(body)};
}
function dashboardPreviewModal(){
 const dash=dashboardById(dashboardsSelectedId);
 const reports=data.customReports||[];
 const body=`<p class="muted" style="font-size:12px;margin-top:0">Shows "${dash.name}"'s draft, as it will appear once published. Values are illustrative here.</p><div style="display:flex;flex-wrap:wrap;gap:12px">${dash.draftWidgets.length?dash.draftWidgets.map(w=>`<div class="kpi" style="min-width:140px;cursor:default"><div class="kpi-value">—</div><div class="kpi-label">${dashboardWidgetLabel(w,reports)}</div></div>`).join(''):'<span class="muted">No widgets on this dashboard yet.</span>'}</div><div class="modal-actions"><button class="btn btn-secondary" type="button" data-close>Close</button></div>`;
 modal(`Preview: ${dash.name}`,body);
 $('[data-close]').onclick=closeModal;
}
// ---- App Builder (mirrors the desktop edition's app_service) --------------
// The packaging layer on top of everything above it: an Administrator
// groups a set of already-existing objects (built-in or custom), their
// screens and a dashboard into one named, publishable application - with
// its own icon and access grants - the same way Salesforce's AppDefinition
// scopes a Lightning app. No draft/publish content fork like Screen layouts
// or Dashboards above (an app's grouping doesn't need a preview-before-
// going-live step) - just a visibility boolean, `isPublished`.
//
// Access grants (role or one specific person, viewer or editor) are stored
// and editable here, the same genuinely-new-per-app permission model the
// desktop edition ships - but this browser demo has no signed-in user (the
// same limitation every other role assignment here already has, see
// usersTab's own note), so the sidebar App Switcher simply shows every
// *published* app to everyone; permission grants are illustrative only.
const APP_ICON_CHOICES=['⬡','🏠','👥','💼','🔧','📦','🏗️','📋','🚗','🏭'];
const APP_PERMISSION_LEVELS=['viewer','editor'];
// Built-in object types an app can group, using the same capitalized-
// singular vocabulary the desktop edition's object_keys use - a custom
// object contributes its own lowercase key instead (== its data[key]
// section already), exactly like everywhere else custom objects plug into
// a fixed vocabulary (business rules, workflow, custom fields).
const APP_OBJECT_TYPES=['Company','Contact','Opportunity','Quote','Order','Invoice','Contract','Task','Product'];
const APP_OBJECT_TYPE_LABELS={Company:'Companies',Contact:'Contacts',Opportunity:'Sales Pipeline',Quote:'Quotes',Order:'Orders',Invoice:'Invoices',Contract:'Contracts',Task:'Tasks',Product:'Products'};
// Maps an object_keys entry to the sidebar section it resolves to via
// sectionForAppObjectKey below - mirrors AppShell.tsx's own sectionFor.
const APP_OBJECT_TYPE_SECTION={Company:'companies',Contact:'contacts',Opportunity:'pipeline',Quote:'quotes',Order:'orders',Invoice:'invoices',Contract:'contracts',Task:'tasks',Product:'products'};
function sectionForAppObjectKey(k){return APP_OBJECT_TYPE_SECTION[k]||k}
function appObjectChoices(){return [...APP_OBJECT_TYPES.map(k=>({key:k,label:APP_OBJECT_TYPE_LABELS[k]})),...activeCustomObjects().map(o=>({key:o.key,label:o.labelPlural}))]}
function freshApp(name,icon){return {id:uid(),name,icon,description:'',objectKeys:[],dashboardId:null,isPublished:false,permissions:[]}}
function ensureApps(){if(!data.apps)data.apps=[];return data.apps}
function appLabel(a){return `${a.icon} ${a.name}`}
// Per-app scoped automation: business rules, workflow rules, and
// dashboards can each optionally carry an appId tagging them to one App
// Builder app instead of always being workspace-wide - mirrors the
// desktop edition's migration 0028 (`app_id`) and its AppScope.tsx
// components. Rules/workflows/dashboards keep evaluating exactly as they
// always have regardless of appId; it's purely which app's Admin screen
// shows them by default.
function matchesAppFilter(appId,filter){if(filter==='all')return true;if(filter==='none')return !appId;return appId===filter}
function appFilterPills(active){
 const apps=ensureApps();
 if(!apps.length)return '';
 return `<div class="entity-tabs" style="margin-bottom:8px">
  <button type="button" class="pill-tab ${active==='all'?'active':''}" data-app-filter="all">All apps</button>
  <button type="button" class="pill-tab ${active==='none'?'active':''}" data-app-filter="none">Workspace-wide</button>
  ${apps.map(a=>`<button type="button" class="pill-tab ${active===a.id?'active':''}" data-app-filter="${a.id}">${appLabel(a)}</button>`).join('')}
 </div>`;
}
function wireAppFilterPills(body,get,set,rerender){
 body.querySelectorAll('[data-app-filter]').forEach(b=>b.onclick=()=>{set(b.dataset.appFilter);rerender()});
}
function appSelectHtml(id,selectedAppId){
 const apps=ensureApps();
 if(!apps.length)return '';
 return `<div class="field" style="margin:0"><label>App</label><select id="${id}"><option value="">Workspace-wide</option>${apps.map(a=>`<option value="${a.id}" ${selectedAppId===a.id?'selected':''}>${appLabel(a)}</option>`).join('')}</select></div>`;
}
function appNameFor(appId){if(!appId)return null;const a=ensureApps().find(x=>x.id===appId);return a?appLabel(a):null}
/** Which real app id (if any) a "+ New" button should default a freshly
 * created rule/workflow/dashboard to: the currently selected app filter,
 * or null (workspace-wide) when the filter is 'all'/'none'. */
function defaultAppIdFor(filter){return (filter!=='all'&&filter!=='none')?filter:null}
function appPermPrincipalLabel(p){if(p.principalType==='role')return p.principalId;const u=(data.users||[]).find(u=>u.id===p.principalId);return u?u.name:'(user removed)'}
function appPermLevelLabel(l){return l==='editor'?'Editor':'Viewer'}
// The currently selected app, only if it's still published - an app
// unpublished or deleted out from under an active selection silently falls
// back to "All" everywhere this is consulted (navSections, the live
// Dashboard), never a broken/half-filtered state.
function activeApp(){const app=ensureApps().find(a=>a.id===activeAppId);return (app&&app.isPublished)?app:null}
// Structural sections an active app never hides - Dashboard and Reports
// aren't object-specific, and Admin is a fixed button outside this list
// entirely (see renderSidebarNav) so it's never filtered either.
const APP_STRUCTURAL_SECTIONS=new Set(['dashboard','reports']);
function navSections(){
 const app=activeApp();
 if(!app)return Object.keys(labels);
 const allowed=new Set(app.objectKeys.map(sectionForAppObjectKey));
 return Object.keys(labels).filter(k=>APP_STRUCTURAL_SECTIONS.has(k)||allowed.has(k));
}
let appsSelectedId=null;
function appsTab(body){
 const apps=ensureApps();
 if(!appsSelectedId||!apps.some(a=>a.id===appsSelectedId))appsSelectedId=apps[0]?.id||null;
 const app=apps.find(a=>a.id===appsSelectedId);
 const dashboards=ensureDashboards();
 body.innerHTML=`<div class="panel">
 <div class="panel-head"><h3>Apps</h3><button class="btn btn-primary" id="addApp" type="button">+ New app</button></div>
 <p class="muted" style="font-size:13px">Group a set of objects, their screens and a dashboard into one named, publishable application, with its own icon and access grants. Every primitive an app assembles — Custom Objects, Screen/App Builder, Dashboards — already ships and works elsewhere in Admin; this is the packaging layer on top.</p>
 ${apps.length?`<div style="display:flex;gap:8px;align-items:center;flex-wrap:wrap;margin:12px 0">${apps.map(a=>`<button type="button" class="tab ${a.id===appsSelectedId?'active':''}" data-select-app="${a.id}">${appLabel(a)}${a.isPublished?'':' · Draft'}</button>`).join('')}</div>`:'<div class="empty">No apps yet.</div>'}
 ${app?appEditorHtml(app,dashboards):''}
 </div>`;
 $('#addApp').onclick=()=>{const name=prompt('New app name?','Property Management');if(!name)return;const a=freshApp(name.trim(),APP_ICON_CHOICES[0]);apps.push(a);save();appsSelectedId=a.id;appsTab(body)};
 if(!app)return;
 body.querySelectorAll('[data-select-app]').forEach(b=>b.onclick=()=>{appsSelectedId=b.dataset.selectApp;appsTab(body)});
 wireAppEditor(body,app,()=>appsTab(body));
}
function appEditorHtml(app,dashboards){
 const objectChoices=appObjectChoices();
 return `
 <div class="layout-meta" style="display:flex;gap:12px;align-items:flex-end;flex-wrap:wrap;margin:16px 0">
  <div class="field" style="margin:0"><label>App name</label><input id="appName" value="${app.name}" style="border:1px solid var(--line);border-radius:8px;padding:6px 9px"></div>
  <div class="field" style="margin:0"><label>Icon</label><select id="appIcon">${APP_ICON_CHOICES.map(i=>`<option value="${i}" ${app.icon===i?'selected':''}>${i}</option>`).join('')}</select></div>
  <div class="field" style="margin:0;flex:1;min-width:220px"><label>Description (optional)</label><input id="appDescription" value="${app.description||''}" placeholder="What this app is for" style="width:100%;border:1px solid var(--line);border-radius:8px;padding:6px 9px"></div>
 </div>
 <div style="margin-bottom:14px;display:flex;gap:8px;align-items:center;flex-wrap:wrap">
  <span class="badge">${app.isPublished?'Published':'Draft'}</span>
  <button class="btn ${app.isPublished?'btn-secondary':'btn-primary'}" id="togglePublishApp" type="button">${app.isPublished?'Unpublish':'Publish'}</button>
  <button class="btn btn-secondary" id="deleteApp" type="button">Delete app</button>
 </div>
 <div style="font-weight:700;margin-bottom:8px">Objects in this app</div>
 <p class="muted" style="font-size:12px;margin-top:0">${app.objectKeys.length?`${app.objectKeys.length} object${app.objectKeys.length===1?'':'s'} selected.`:'Pick at least one object before publishing.'}</p>
 <div style="display:flex;flex-wrap:wrap;gap:12px;margin-bottom:14px">${objectChoices.map(c=>`<label style="display:flex;gap:6px;align-items:center;font-size:13px"><input type="checkbox" data-app-object="${c.key}" ${app.objectKeys.includes(c.key)?'checked':''}> ${c.label}</label>`).join('')}</div>
 <div class="field" style="max-width:320px;margin-bottom:20px"><label>Dashboard for this app (optional)</label><select id="appDashboard"><option value="">No dashboard</option>${dashboards.map(d=>`<option value="${d.id}" ${app.dashboardId===d.id?'selected':''}>${d.name}${d.isDefault?' · Default':''}</option>`).join('')}</select></div>
 ${appPermissionsHtml(app)}
 `;
}
function appPermissionsHtml(app){
 const perms=app.permissions||[];
 const grantedRoles=new Set(perms.filter(p=>p.principalType==='role').map(p=>p.principalId));
 const grantedUserIds=new Set(perms.filter(p=>p.principalType==='user').map(p=>p.principalId));
 const availableRoles=DEMO_LAYOUT_ROLES.filter(r=>!grantedRoles.has(r));
 const availableUsers=(data.users||[]).filter(u=>!grantedUserIds.has(u.id));
 return `<div class="panel" style="background:var(--surface-alt,#f7f8fc);margin-top:4px">
 <div style="font-weight:700;margin-bottom:8px">Access</div>
 <p class="muted" style="font-size:12px;margin-top:0">Administrators always see every published app. Everyone else would need a grant here — to a role, or to one specific person — on the desktop edition; this browser demo has no signed-in user, so the App Switcher shows every published app to everyone regardless of grants below (see this section's own doc comment).</p>
 ${perms.length?`<div style="display:flex;flex-direction:column;gap:6px;margin:8px 0">${perms.map(p=>`<span class="badge" style="display:inline-flex;align-items:center;gap:8px;justify-content:space-between;width:fit-content">${p.principalType==='role'?'Role':'Person'}: ${appPermPrincipalLabel(p)} — ${appPermLevelLabel(p.level)}<button class="icon-btn" data-revoke-perm="${p.id}" type="button" title="Remove this grant">×</button></span>`).join('')}</div>`:'<p class="empty">No grants yet.</p>'}
 <div style="display:flex;gap:16px;flex-wrap:wrap;margin-top:8px">
  ${availableRoles.length?`<span style="display:inline-flex;gap:6px;align-items:center"><select id="grantRoleSelect"><option value="">Choose a role…</option>${availableRoles.map(r=>`<option value="${r}">${r}</option>`).join('')}</select><select id="grantRoleLevel">${APP_PERMISSION_LEVELS.map(l=>`<option value="${l}">${appPermLevelLabel(l)}</option>`).join('')}</select><button class="btn btn-secondary" id="grantRoleBtn" type="button">+ Grant role</button></span>`:''}
  ${availableUsers.length?`<span style="display:inline-flex;gap:6px;align-items:center"><select id="grantUserSelect"><option value="">Choose a person…</option>${availableUsers.map(u=>`<option value="${u.id}">${u.name}</option>`).join('')}</select><select id="grantUserLevel">${APP_PERMISSION_LEVELS.map(l=>`<option value="${l}">${appPermLevelLabel(l)}</option>`).join('')}</select><button class="btn btn-secondary" id="grantUserBtn" type="button">+ Grant person</button></span>`:''}
 </div>
 </div>`;
}
function wireAppEditor(body,app,rerender){
 $('#appName').onchange=e=>{app.name=e.target.value.trim()||app.name;save();setTimeout(rerender,0)};
 $('#appIcon').onchange=e=>{app.icon=e.target.value;save();rerender()};
 $('#appDescription').onchange=e=>{app.description=e.target.value.trim();save()};
 body.querySelectorAll('[data-app-object]').forEach(cb=>cb.onchange=()=>{
  const key=cb.dataset.appObject;
  const adding=cb.checked;
  // cb's own <label> text is the object's display label - cheaper than
  // re-deriving it from appObjectChoices()/activeCustomObjects() here.
  const label=(cb.closest('label')?.textContent||key).trim();
  app.objectKeys=adding?[...app.objectKeys,key]:app.objectKeys.filter(k=>k!==key);
  // Bug fix: adding/removing an object on an *already-published* app saved
  // correctly but never refreshed the sidebar's own nav-section filter
  // (only togglePublishApp did), so a newly-added object silently never
  // showed up until something else forced a re-render - looked like the
  // change "didn't take" even though it was saved and no republish is
  // actually required.
  //
  // Second bug fix: even after that first fix, the toggle gave zero
  // *confirmation* of anything happening - no toast, and on a narrow/
  // mobile viewport the icon-only collapsed sidebar makes a newly-added
  // nav item easy to miss entirely. From a user's seat this reads
  // exactly like "there's no way to add an object" even though the save
  // already happened - so confirm it the same way every other admin
  // action in this app already does.
  save();renderSidebarNav();rerender();
  toast(`${adding?'Added':'Removed'} ${label}${adding?' to':' from'} ${app.name}`);
 });
 $('#appDashboard').onchange=e=>{app.dashboardId=e.target.value||null;save()};
 $('#togglePublishApp').onclick=()=>{
  if(!app.isPublished&&app.objectKeys.length===0){toast('Add at least one object before publishing this app');return}
  app.isPublished=!app.isPublished;
  if(!app.isPublished&&activeAppId===app.id)activeAppId=null;
  save();toast(app.isPublished?'App published':'App unpublished');renderSidebarNav();rerender();
 };
 $('#deleteApp').onclick=()=>{
  if(!confirm(`Delete the "${app.name}" app? This can't be undone.`))return;
  data.apps=ensureApps().filter(a=>a.id!==app.id);
  if(activeAppId===app.id)activeAppId=null;
  save();appsSelectedId=null;toast('App deleted');renderSidebarNav();appsTab(body);
 };
 const grantRoleBtn=$('#grantRoleBtn'); if(grantRoleBtn)grantRoleBtn.onclick=()=>{
  const role=$('#grantRoleSelect').value; if(!role)return;
  const level=$('#grantRoleLevel').value;
  app.permissions.push({id:uid(),principalType:'role',principalId:role,level});
  save();rerender();
 };
 const grantUserBtn=$('#grantUserBtn'); if(grantUserBtn)grantUserBtn.onclick=()=>{
  const userId=$('#grantUserSelect').value; if(!userId)return;
  const level=$('#grantUserLevel').value;
  app.permissions.push({id:uid(),principalType:'user',principalId:userId,level});
  save();rerender();
 };
 body.querySelectorAll('[data-revoke-perm]').forEach(b=>b.onclick=()=>{
  app.permissions=app.permissions.filter(p=>p.id!==b.dataset.revokePerm);
  save();rerender();
 });
}
// ---- Industry Data Model / App Catalog -------------------------------------
// Mirrors the desktop edition's industry_package_service: an Administrator
// imports a package manifest into a local catalog for review, then installs
// it, which creates real Custom Objects/Fields/Relationships/Business Rules/
// Workflow rules and a published App in one step - every artifact type this
// demo already supports elsewhere in Admin, just assembled from one bundle
// instead of by hand. There's no free-text manifest upload here (nothing in
// this browser-only demo parses arbitrary JSON against a server-side
// schema) - REFERENCE_PACKAGES holds the two bundled starters as plain JS,
// the same "compiled, not user-supplied" approach the desktop edition's own
// reference_packages.rs takes for its starter packages.
//
// Two real engine gaps surfaced while porting the desktop edition's Field
// Service and Property Management packages here, both left deliberately
// unfixed and worked around below rather than faked:
//  - Workflow rules only ever fire on an *edit* that changes a watched
//    field (see the wasEdit&&before guard around executeWorkflowAction's
//    call site) - there's no "on record created" trigger in this demo the
//    way desktop's record_created trigger type provides. Every workflow
//    below is written as a field-changed-on-edit rule instead (e.g. "stage
//    becomes Completed"), never "when first created" - the same fix desktop
//    had to make for its own "work order scheduled" workflow, just forced
//    on every workflow here rather than just one.
//  - The Lease date-validation rule desktop ships (block save when an end
//    date is on/before a start date) isn't reproduced here: this demo's
//    greater_than/less_than operators compare with `Number(v)`, which is
//    NaN for an ISO date string like "2026-06-01" - a pre-existing bug in
//    the condition engine, unrelated to this feature, and out of scope to
//    fix here. Both packages below only use operators (equals/is_empty/
//    in_list) that already work correctly on every field type.
// update_related_record itself needed one real fix, not a workaround: it
// only ever consulted the static built-in foreign-key graph (RELATIONS),
// so it silently no-op'd for any relationship between two Custom Objects -
// exactly what every relationship in both packages below is. See
// customRelationTargetsFor and executeWorkflowAction's update_related_record
// branch above for the fix (falls back to an admin-defined Custom
// Relationship when the built-in graph has nothing).
function freshFieldDef(entity,key,label,type,options){
 return {id:uid(),entity,key,label,type,options:options||'',active:true,defaultValue:'',unique:false,helpText:'',placeholder:'',required:false,maxLength:null,pattern:'',minValue:'',maxValue:'',searchable:false,filterable:false,reportable:true,hiddenByDefault:false};
}
const REFERENCE_PACKAGES={
 field_service:{
  packageId:'lanesra.field_service',name:'Field Service',industry:'Field Service',version:'1.0.0',
  description:'Sites, assets, work orders and appointments for a field service crew - dispatch work, track what was serviced, and close out completed jobs.',
  appIcon:'🔧',
  objects:[
   {key:'service_site',label:'Service Site',labelPlural:'Service Sites',icon:'📍',prefix:'SITE',digits:4},
   {key:'asset',label:'Asset',labelPlural:'Assets',icon:'🔩',prefix:'AST',digits:4},
   {key:'work_order',label:'Work Order',labelPlural:'Work Orders',icon:'🛠️',prefix:'WO',digits:4},
   {key:'service_appointment',label:'Appointment',labelPlural:'Appointments',icon:'📅',prefix:'APT',digits:4},
   {key:'warranty_claim',label:'Warranty Claim',labelPlural:'Warranty Claims',icon:'🧾',prefix:'WC',digits:5},
  ],
  fields:[
   ['service_site','address','Address','text'],['service_site','city','City','text'],
   ['asset','model','Model','text'],['asset','serial_number','Serial Number','text'],
   ['asset','last_service_status','Last Service Status','select','Pending|Serviced'],
   ['work_order','wo_stage','Stage','select','New|Scheduled|In Progress|Completed|Cancelled'],
   ['work_order','priority','Priority','select','Low|Medium|High|Urgent'],
   ['work_order','resolution','Resolution','text'],['work_order','completion_date','Completion Date','date'],
   ['service_appointment','appt_stage','Stage','select','Scheduled|En Route|In Progress|Completed|Cancelled'],
   ['service_appointment','actual_start','Actual Start','date'],['service_appointment','actual_end','Actual End','date'],
   ['service_appointment','outcome','Outcome','text'],
   ['warranty_claim','claim_status','Status','select','Draft|Submitted|Approved|Denied|Reimbursed'],
   ['warranty_claim','resolution_notes','Resolution Notes','text'],['warranty_claim','amount_approved','Amount Approved','number'],
  ],
  relationships:[
   {source:'work_order',target:'service_site',relType:'many_to_one',forwardLabel:'Service Site',reverseLabel:'Work Orders'},
   {source:'work_order',target:'asset',relType:'many_to_one',forwardLabel:'Asset',reverseLabel:'Work Orders'},
   {source:'service_appointment',target:'work_order',relType:'many_to_one',forwardLabel:'Work Order',reverseLabel:'Appointments'},
   {source:'warranty_claim',target:'asset',relType:'many_to_one',forwardLabel:'Asset',reverseLabel:'Warranty Claims'},
   {source:'work_order',target:'Company',relType:'many_to_one',forwardLabel:'Customer',reverseLabel:'Work Orders'},
  ],
  rules:[
   {entity:'work_order',matchType:'all',
    conditions:[{fieldKey:'wo_stage',operator:'equals',value:'Completed',compareField:null,groupId:null},{fieldKey:'resolution',operator:'is_empty',value:'',compareField:null,groupId:'g1'},{fieldKey:'completion_date',operator:'is_empty',value:'',compareField:null,groupId:'g1'}],
    actions:[{type:'block_save',message:'Enter a resolution and completion date before marking a work order Completed.'}]},
   {entity:'service_appointment',matchType:'all',
    conditions:[{fieldKey:'appt_stage',operator:'equals',value:'Completed',compareField:null,groupId:null},{fieldKey:'actual_start',operator:'is_empty',value:'',compareField:null,groupId:'g1'},{fieldKey:'actual_end',operator:'is_empty',value:'',compareField:null,groupId:'g1'},{fieldKey:'outcome',operator:'is_empty',value:'',compareField:null,groupId:'g1'}],
    actions:[{type:'block_save',message:'Enter actual start/end times and an outcome before marking an appointment Completed.'}]},
   {entity:'warranty_claim',matchType:'all',
    conditions:[{fieldKey:'claim_status',operator:'in_list',value:'Approved|Denied|Reimbursed',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'resolution_notes',value:'',message:''}]},
   {entity:'warranty_claim',matchType:'all',
    conditions:[{fieldKey:'claim_status',operator:'equals',value:'Reimbursed',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'amount_approved',value:'',message:''}]},
  ],
  workflows:[
   {entity:'work_order',matchType:'all',notify:true,
    conditions:[{fieldKey:'wo_stage',operator:'equals',value:'Completed',compareField:null,groupId:null}],
    actions:[{type:'update_related_record',relTargetEntity:'asset',relTargetField:'last_service_status',relValue:'Serviced'},{type:'create_task',taskTitle:'Review completed work order',daysOffset:1}]},
   {entity:'warranty_claim',matchType:'all',notify:true,
    conditions:[{fieldKey:'claim_status',operator:'equals',value:'Submitted',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Review warranty claim',daysOffset:2}]},
  ],
 },
 property_management:{
  packageId:'lanesra.property_management',name:'Property Management',industry:'Real Estate',version:'1.0.0',
  description:'Properties, units, leases and maintenance requests - activate or end a lease and its unit\'s occupancy follows automatically.',
  appIcon:'🏢',
  objects:[
   {key:'property',label:'Property',labelPlural:'Properties',icon:'🏢',prefix:'PROP',digits:4},
   {key:'unit',label:'Unit',labelPlural:'Units',icon:'🚪',prefix:'UNIT',digits:4},
   {key:'lease',label:'Lease',labelPlural:'Leases',icon:'📄',prefix:'LSE',digits:4},
   {key:'maintenance_request',label:'Maintenance Request',labelPlural:'Maintenance Requests',icon:'🔧',prefix:'MNT',digits:4},
   {key:'unit_showing',label:'Unit Showing',labelPlural:'Unit Showings',icon:'👁',prefix:'SHW',digits:5},
  ],
  fields:[
   ['property','address','Address','text'],['property','city','City','text'],
   ['unit','unit_stage','Occupancy','select','Vacant|Occupied|Under Maintenance'],['unit','bedrooms','Bedrooms','number'],
   ['lease','lease_stage','Stage','select','Draft|Active|Expired|Terminated|Renewed'],
   ['lease','start_date','Start Date','date'],['lease','end_date','End Date','date'],['lease','monthly_rent','Monthly Rent','number'],
   ['maintenance_request','mr_stage','Stage','select','New|Assigned|In Progress|Closed'],
   ['maintenance_request','description','Description','text'],['maintenance_request','resolution','Resolution','text'],['maintenance_request','completed_date','Completed Date','date'],
   ['unit_showing','showing_stage','Stage','select','Scheduled|Completed|Cancelled|No Show'],
   ['unit_showing','interest_level','Interest Level','select','Low|Medium|High'],
  ],
  relationships:[
   {source:'unit',target:'property',relType:'many_to_one',forwardLabel:'Property',reverseLabel:'Units'},
   {source:'lease',target:'unit',relType:'many_to_one',forwardLabel:'Unit',reverseLabel:'Leases'},
   {source:'maintenance_request',target:'unit',relType:'many_to_one',forwardLabel:'Unit',reverseLabel:'Maintenance Requests'},
   {source:'unit_showing',target:'unit',relType:'many_to_one',forwardLabel:'Unit',reverseLabel:'Showings'},
   {source:'property',target:'Company',relType:'many_to_one',forwardLabel:'Owner',reverseLabel:'Properties'},
  ],
  rules:[
   {entity:'maintenance_request',matchType:'all',
    conditions:[{fieldKey:'mr_stage',operator:'equals',value:'Closed',compareField:null,groupId:null},{fieldKey:'resolution',operator:'is_empty',value:'',compareField:null,groupId:'g1'},{fieldKey:'completed_date',operator:'is_empty',value:'',compareField:null,groupId:'g1'}],
    actions:[{type:'block_save',message:'Enter a resolution and completed date before closing a maintenance request.'}]},
   {entity:'unit_showing',matchType:'all',
    conditions:[{fieldKey:'showing_stage',operator:'equals',value:'Completed',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'interest_level',value:'',message:''}]},
  ],
  workflows:[
   {entity:'lease',matchType:'all',notify:false,
    conditions:[{fieldKey:'lease_stage',operator:'equals',value:'Active',compareField:null,groupId:null}],
    actions:[{type:'update_related_record',relTargetEntity:'unit',relTargetField:'unit_stage',relValue:'Occupied'}]},
   {entity:'lease',matchType:'all',notify:false,
    conditions:[{fieldKey:'lease_stage',operator:'in_list',value:'Expired|Terminated',compareField:null,groupId:null}],
    actions:[{type:'update_related_record',relTargetEntity:'unit',relTargetField:'unit_stage',relValue:'Vacant'}]},
   {entity:'maintenance_request',matchType:'all',notify:true,
    conditions:[{fieldKey:'mr_stage',operator:'equals',value:'Assigned',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Follow up on assigned maintenance request',daysOffset:1}]},
   {entity:'unit_showing',matchType:'all',notify:false,
    conditions:[{fieldKey:'showing_stage',operator:'equals',value:'Completed',compareField:null,groupId:null},{fieldKey:'interest_level',operator:'equals',value:'High',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Follow up with interested prospect',daysOffset:1}]},
  ],
 },
 // The remaining eight packages below are reduced mirrors of the desktop
 // edition's own reference_packages.rs manifests (same package_id,
 // industry, stage vocabulary and object/field/rule/workflow names) -
 // not full parity. Two structural limits of this demo's simpler engine
 // (neither new to this change, both pre-existing) shaped every one of
 // them the same way the two packages above were already shaped by:
 // - Workflow rules only ever fire on an *edit* where a watched field's
 //   value actually changed (see the `wasEdit&&before` guard around
 //   data.workflowRules execution) - there's no "on record created"
 //   firing at all, unlike desktop's `record_created` trigger type. Every
 //   desktop workflow that trigger_type:"record_created" (Real Estate's
 //   "Showing scheduled", Practice Administration's "Appointment
 //   confirmation", Nonprofit's "Program participation", Recruitment's
 //   "Interview scheduled") is left out here rather than shipped broken.
 // - create_record only knows how to construct the nine built-in record
 //   types (see executeWorkflowAction's target checks) - it silently
 //   no-ops for a custom-object target. Every desktop workflow that
 //   create_record's a custom object (Construction/Professional
 //   Services' "Opportunity won creates project/engagement", Real
 //   Estate's "Offer accepted" opening a Transaction, Recruitment's
 //   "Offer accepted" opening a Placement) keeps only its
 //   update_related_record half here, the half this demo can actually run.
 // Desktop's `on_or_after`/`on_or_before` date-comparison operators also
 // don't exist in this demo's OPERATOR_LABELS - Nonprofit's "Renewal
 // integrity" (a date-to-date field comparison) is left out for the same
 // reason, rather than approximated with a numeric operator that would
 // silently never fire on date strings.
 construction:{
  packageId:'lanesra.construction',name:'Construction & Contractors',industry:'Construction',version:'1.0.0',
  description:'Projects, work packages, change orders and inspections for general contractors and specialty trades.',
  appIcon:'🏗️',
  objects:[
   {key:'project',label:'Project',labelPlural:'Projects',icon:'🏗',prefix:'PROJ',digits:4},
   {key:'work_package',label:'Work Package',labelPlural:'Work Packages',icon:'📦',prefix:'WP',digits:4},
   {key:'change_order',label:'Change Order',labelPlural:'Change Orders',icon:'📝',prefix:'CO',digits:4},
   {key:'inspection',label:'Inspection',labelPlural:'Inspections',icon:'🔎',prefix:'INSP',digits:4},
   {key:'punch_list_item',label:'Punch List Item',labelPlural:'Punch List Items',icon:'📋',prefix:'PLI',digits:4},
  ],
  fields:[
   ['project','stage','Stage','select','Lead/Estimating|Awarded|Planning|Active|On Hold|Substantially Complete|Closed|Cancelled'],
   ['project','start_date','Start Date','date'],['project','actual_end_date','Actual End Date','date'],['project','contract_value','Contract Value','number'],
   ['work_package','stage','Stage','select','Planned|Ready|In Progress|Blocked|Complete'],
   ['work_package','trade_scope','Trade / Scope','text'],['work_package','completion_date','Completion Date','date'],
   ['change_order','stage','Stage','select','Draft|Submitted|Approved|Rejected|Cancelled'],
   ['change_order','amount','Requested Amount','number'],['change_order','approved_amount','Approved Amount','number'],
   ['inspection','stage','Stage','select','Planned|Passed|Failed|Reinspection Required'],['inspection','inspection_type','Type','text'],
   ['punch_list_item','stage','Stage','select','Open|In Progress|Resolved|Verified|Won\'t Fix'],
   ['punch_list_item','description','Description','text'],['punch_list_item','resolved_date','Resolved Date','date'],
  ],
  relationships:[
   {source:'work_package',target:'project',relType:'many_to_one',forwardLabel:'Project',reverseLabel:'Work Packages'},
   {source:'change_order',target:'project',relType:'many_to_one',forwardLabel:'Project',reverseLabel:'Change Orders'},
   {source:'inspection',target:'project',relType:'many_to_one',forwardLabel:'Project',reverseLabel:'Inspections'},
   {source:'punch_list_item',target:'project',relType:'many_to_one',forwardLabel:'Project',reverseLabel:'Punch List Items'},
   {source:'project',target:'Company',relType:'many_to_one',forwardLabel:'Customer',reverseLabel:'Projects'},
  ],
  rules:[
   {entity:'project',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Closed',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'actual_end_date',value:'',message:''}]},
   {entity:'change_order',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Approved',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'approved_amount',value:'',message:''}]},
   {entity:'punch_list_item',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'in_list',value:'Resolved|Verified',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'resolved_date',value:'',message:''}]},
  ],
  // Punch List Item's own "Punch item created" workflow desktop ships is
  // trigger_type:"record_created" - left out here, same reasoning as
  // every other record_created workflow in this file (see this const's
  // own doc comment above).
  workflows:[
   {entity:'inspection',matchType:'all',notify:true,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Failed',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Address failed inspection',daysOffset:2}]},
   {entity:'project',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Closed',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Final billing review',daysOffset:3}]},
  ],
 },
 professional_services:{
  packageId:'lanesra.professional_services',name:'Professional Services',industry:'Professional Services',version:'1.0.0',
  description:'Engagements, milestones, time entries and expenses for consultancies and services firms.',
  appIcon:'💼',
  objects:[
   {key:'engagement',label:'Engagement',labelPlural:'Engagements',icon:'💼',prefix:'ENG',digits:4},
   {key:'milestone',label:'Milestone',labelPlural:'Milestones',icon:'🚩',prefix:'MS',digits:4},
   {key:'time_entry',label:'Time Entry',labelPlural:'Time Entries',icon:'⏱',prefix:'TE',digits:5},
   {key:'expense',label:'Expense',labelPlural:'Expenses',icon:'💳',prefix:'EXP',digits:5},
   {key:'change_request',label:'Change Request',labelPlural:'Change Requests',icon:'🔄',prefix:'CR',digits:4},
  ],
  fields:[
   ['engagement','stage','Stage','select','Discovery|Proposed|Active|On Hold|Complete|Cancelled'],['engagement','actual_end_date','Actual End Date','date'],
   ['milestone','stage','Stage','select','Planned|In Progress|Complete|At Risk'],['milestone','completed_date','Completed Date','date'],
   ['time_entry','stage','Stage','select','Draft|Submitted|Approved|Rejected'],['time_entry','hours','Hours','number'],
   ['time_entry','billing_status','Billing Status','select','Not Eligible|Eligible'],
   ['expense','stage','Stage','select','Draft|Submitted|Approved|Reimbursed'],
   ['expense','category','Category','text'],['expense','amount','Amount','number'],['expense','date','Date','date'],
   ['change_request','stage','Stage','select','Draft|Submitted|Approved|Rejected|Implemented'],['change_request','approved_date','Approved Date','date'],
  ],
  relationships:[
   {source:'milestone',target:'engagement',relType:'many_to_one',forwardLabel:'Engagement',reverseLabel:'Milestones'},
   {source:'time_entry',target:'engagement',relType:'many_to_one',forwardLabel:'Engagement',reverseLabel:'Time Entries'},
   {source:'expense',target:'engagement',relType:'many_to_one',forwardLabel:'Engagement',reverseLabel:'Expenses'},
   {source:'change_request',target:'engagement',relType:'many_to_one',forwardLabel:'Engagement',reverseLabel:'Change Requests'},
   {source:'engagement',target:'Company',relType:'many_to_one',forwardLabel:'Customer',reverseLabel:'Engagements'},
  ],
  rules:[
   {entity:'engagement',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Complete',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'actual_end_date',value:'',message:''}]},
   {entity:'time_entry',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Submitted',compareField:null,groupId:null},{fieldKey:'hours',operator:'less_than',value:'0.01',compareField:null,groupId:null}],
    actions:[{type:'block_save',message:'Enter more than zero hours before submitting a time entry.'}]},
   {entity:'expense',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Submitted',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'category',value:'',message:''},{type:'require',targetField:'amount',value:'',message:''},{type:'require',targetField:'date',value:'',message:''}]},
   {entity:'milestone',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Complete',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'completed_date',value:'',message:''}]},
   {entity:'change_request',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'in_list',value:'Approved|Implemented',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'approved_date',value:'',message:''}]},
  ],
  workflows:[
   {entity:'time_entry',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Approved',compareField:null,groupId:null}],
    actions:[{type:'update_field',updateFieldKey:'billing_status',updateValue:'Eligible',updateCopyFrom:''}]},
   {entity:'engagement',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Complete',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Prepare final invoice',daysOffset:2}]},
   {entity:'change_request',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Submitted',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Review change request',daysOffset:2}]},
  ],
 },
 practice_admin:{
  packageId:'lanesra.practice_admin',name:'Dental / Clinic Practice Administration',industry:'Practice Administration',version:'1.0.0',
  description:'Patients, providers, appointments and treatment plans for a small dental or clinic practice.',
  appIcon:'🦷',
  objects:[
   {key:'patient_profile',label:'Patient Profile',labelPlural:'Patient Profiles',icon:'🧑',prefix:'PAT',digits:5},
   {key:'provider_profile',label:'Provider Profile',labelPlural:'Provider Profiles',icon:'🩺',prefix:'PROV',digits:3},
   {key:'appointment',label:'Appointment',labelPlural:'Appointments',icon:'📅',prefix:'APT',digits:5},
   {key:'treatment_plan',label:'Treatment Plan',labelPlural:'Treatment Plans',icon:'📋',prefix:'TX',digits:4},
   {key:'billing_claim',label:'Billing Claim',labelPlural:'Billing Claims',icon:'🧾',prefix:'CLM',digits:5},
  ],
  fields:[
   ['provider_profile','specialty','Specialty','text'],
   ['appointment','stage','Status','select','Requested|Confirmed|Checked In|Completed|No Show|Cancelled'],['appointment','completed_date','Completed Date','date'],
   ['treatment_plan','stage','Stage','select','Proposed|Accepted|In Progress|Complete|Declined'],
   ['billing_claim','claim_status','Status','select','Draft|Submitted|Paid|Denied'],
   ['billing_claim','paid_amount','Paid Amount','number'],['billing_claim','payment_date','Payment Date','date'],['billing_claim','denial_reason','Denial Reason','text'],
  ],
  relationships:[
   {source:'appointment',target:'patient_profile',relType:'many_to_one',forwardLabel:'Patient',reverseLabel:'Appointments'},
   {source:'appointment',target:'provider_profile',relType:'many_to_one',forwardLabel:'Provider',reverseLabel:'Appointments'},
   {source:'treatment_plan',target:'patient_profile',relType:'many_to_one',forwardLabel:'Patient',reverseLabel:'Treatment Plans'},
   {source:'billing_claim',target:'patient_profile',relType:'many_to_one',forwardLabel:'Patient',reverseLabel:'Billing Claims'},
   {source:'patient_profile',target:'Contact',relType:'many_to_one',forwardLabel:'Contact',reverseLabel:'Patient Profile'},
  ],
  rules:[
   {entity:'appointment',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Completed',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'completed_date',value:'',message:''}]},
   {entity:'billing_claim',matchType:'all',
    conditions:[{fieldKey:'claim_status',operator:'equals',value:'Paid',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'paid_amount',value:'',message:''},{type:'require',targetField:'payment_date',value:'',message:''}]},
   {entity:'billing_claim',matchType:'all',
    conditions:[{fieldKey:'claim_status',operator:'equals',value:'Denied',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'denial_reason',value:'',message:''}]},
  ],
  workflows:[
   {entity:'appointment',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'No Show',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Follow up on missed appointment',daysOffset:1}]},
   {entity:'treatment_plan',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Accepted',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Prepare billing/quote for accepted treatment plan',daysOffset:1}]},
   {entity:'billing_claim',matchType:'all',notify:false,
    conditions:[{fieldKey:'claim_status',operator:'equals',value:'Submitted',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Track submitted billing claim',daysOffset:14}]},
  ],
 },
 recruitment:{
  packageId:'lanesra.recruitment',name:'Recruitment & Staffing',industry:'Recruitment & Staffing',version:'1.0.0',
  description:'Job requisitions, candidates, applications and offers for a small staffing or recruiting desk.',
  appIcon:'🧑‍💼',
  objects:[
   {key:'job_requisition',label:'Job Requisition',labelPlural:'Job Requisitions',icon:'📋',prefix:'JOB',digits:4},
   {key:'candidate_profile',label:'Candidate Profile',labelPlural:'Candidate Profiles',icon:'🧑',prefix:'CAND',digits:5},
   {key:'application',label:'Application',labelPlural:'Applications',icon:'📨',prefix:'APP',digits:5},
   {key:'offer',label:'Offer',labelPlural:'Offers',icon:'📝',prefix:'OFR',digits:4},
   {key:'reference_check',label:'Reference Check',labelPlural:'Reference Checks',icon:'📞',prefix:'REF',digits:5},
  ],
  fields:[
   ['job_requisition','stage','Stage','select','Draft|Open|On Hold|Filled|Closed|Cancelled'],
   ['job_requisition','title','Title','text'],['job_requisition','openings','Openings','number'],
   ['candidate_profile','skills_summary','Skills Summary','text'],['candidate_profile','source','Source','text'],
   ['application','stage','Stage','select','Sourced|Screening|Submitted|Interview|Offer|Placed|Rejected|Withdrawn'],['application','score','Score','number'],
   ['offer','amount','Amount','number'],['offer','start_date','Start Date','date'],['offer','stage','Stage','select','Draft|Sent|Accepted|Rejected|Withdrawn'],
   ['reference_check','check_status','Status','select','Requested|Completed|Failed'],['reference_check','completed_date','Completed Date','date'],
  ],
  relationships:[
   {source:'application',target:'candidate_profile',relType:'many_to_one',forwardLabel:'Candidate',reverseLabel:'Applications'},
   {source:'application',target:'job_requisition',relType:'many_to_one',forwardLabel:'Job',reverseLabel:'Applications'},
   {source:'offer',target:'application',relType:'many_to_one',forwardLabel:'Application',reverseLabel:'Offers'},
   {source:'reference_check',target:'candidate_profile',relType:'many_to_one',forwardLabel:'Candidate',reverseLabel:'Reference Checks'},
   {source:'job_requisition',target:'Company',relType:'many_to_one',forwardLabel:'Customer',reverseLabel:'Job Requisitions'},
  ],
  rules:[
   {entity:'offer',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Accepted',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'start_date',value:'',message:''}]},
   {entity:'reference_check',matchType:'all',
    conditions:[{fieldKey:'check_status',operator:'in_list',value:'Completed|Failed',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'completed_date',value:'',message:''}]},
  ],
  // Reference Check's own "Reference check requested" workflow desktop
  // ships is trigger_type:"record_created" - left out here, same reasoning
  // as every other record_created workflow in this file.
  workflows:[
   {entity:'application',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Interview',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Schedule interview',daysOffset:1}]},
   {entity:'offer',matchType:'all',notify:true,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Accepted',compareField:null,groupId:null}],
    actions:[{type:'update_related_record',relTargetEntity:'application',relTargetField:'stage',relValue:'Placed'}]},
  ],
 },
 real_estate:{
  packageId:'lanesra.real_estate',name:'Real Estate Brokerage',industry:'Real Estate',version:'1.0.0',
  description:'Properties, listings, offers and transactions for a small real-estate brokerage.',
  appIcon:'🏠',
  objects:[
   {key:'listing_property',label:'Property',labelPlural:'Properties',icon:'🏠',prefix:'PROP',digits:4},
   {key:'listing',label:'Listing',labelPlural:'Listings',icon:'📣',prefix:'LST',digits:4},
   {key:'purchase_offer',label:'Offer',labelPlural:'Offers',icon:'📝',prefix:'OFR',digits:4},
   {key:'transaction',label:'Transaction',labelPlural:'Transactions',icon:'🤝',prefix:'TXN',digits:4},
   {key:'commission_disbursement',label:'Commission Disbursement',labelPlural:'Commission Disbursements',icon:'💰',prefix:'DSB',digits:5},
  ],
  fields:[
   ['listing_property','property_type','Property Type','text'],
   ['listing','stage','Stage','select','Draft|Active|Pending|Closed|Expired|Withdrawn'],
   ['listing','list_price','List Price','number'],['listing','list_date','List Date','date'],
   ['purchase_offer','stage','Stage','select','Draft|Submitted|Accepted|Rejected|Withdrawn'],
   ['purchase_offer','amount','Amount','number'],['purchase_offer','expiry_date','Expiry Date','date'],
   ['transaction','status','Status','select','Open|Pending|Closed|Cancelled'],
   ['transaction','closing_date','Closing Date','date'],['transaction','final_price','Final Price','number'],
   ['commission_disbursement','disbursement_status','Status','select','Pending|Paid'],['commission_disbursement','paid_date','Paid Date','date'],
  ],
  relationships:[
   {source:'listing',target:'listing_property',relType:'many_to_one',forwardLabel:'Property',reverseLabel:'Listings'},
   {source:'purchase_offer',target:'listing',relType:'many_to_one',forwardLabel:'Listing',reverseLabel:'Offers'},
   {source:'transaction',target:'purchase_offer',relType:'many_to_one',forwardLabel:'Accepted Offer',reverseLabel:'Transaction'},
   {source:'transaction',target:'listing',relType:'many_to_one',forwardLabel:'Listing',reverseLabel:'Transactions'},
   {source:'commission_disbursement',target:'transaction',relType:'many_to_one',forwardLabel:'Transaction',reverseLabel:'Commission Disbursements'},
   {source:'listing_property',target:'Contact',relType:'many_to_one',forwardLabel:'Owner/Seller',reverseLabel:'Properties'},
  ],
  rules:[
   {entity:'purchase_offer',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'not_equals',value:'Draft',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'amount',value:'',message:''},{type:'require',targetField:'expiry_date',value:'',message:''}]},
   {entity:'listing',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Active',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'list_price',value:'',message:''},{type:'require',targetField:'list_date',value:'',message:''}]},
   {entity:'transaction',matchType:'all',
    conditions:[{fieldKey:'status',operator:'equals',value:'Closed',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'closing_date',value:'',message:''},{type:'require',targetField:'final_price',value:'',message:''}]},
   {entity:'commission_disbursement',matchType:'all',
    conditions:[{fieldKey:'disbursement_status',operator:'equals',value:'Paid',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'paid_date',value:'',message:''}]},
  ],
  // Commission Disbursement's own "Commission disbursement created"
  // workflow desktop ships is trigger_type:"record_created" - left out
  // here, same reasoning as every other record_created workflow in this file.
  workflows:[
   {entity:'purchase_offer',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Accepted',compareField:null,groupId:null}],
    actions:[{type:'update_related_record',relTargetEntity:'listing',relTargetField:'stage',relValue:'Pending'}]},
   {entity:'transaction',matchType:'all',notify:false,
    conditions:[{fieldKey:'status',operator:'equals',value:'Closed',compareField:null,groupId:null}],
    actions:[{type:'update_related_record',relTargetEntity:'listing',relTargetField:'stage',relValue:'Closed'},{type:'create_task',taskTitle:'Post-closing checklist',daysOffset:2}]},
  ],
 },
 legal_practice:{
  packageId:'lanesra.legal_practice',name:'Legal Practice',industry:'Legal',version:'1.0.0',
  description:'Matters, deadlines and time entries for a small law firm.',
  appIcon:'⚖️',
  objects:[
   {key:'matter',label:'Matter',labelPlural:'Matters',icon:'⚖️',prefix:'MTR',digits:4},
   {key:'matter_time_entry',label:'Time Entry',labelPlural:'Time Entries',icon:'⏱',prefix:'TE',digits:5},
   {key:'matter_deadline',label:'Deadline',labelPlural:'Deadlines',icon:'⏰',prefix:'DL',digits:4},
   {key:'conflict_check',label:'Conflict Check',labelPlural:'Conflict Checks',icon:'🔍',prefix:'CFC',digits:5},
  ],
  fields:[
   ['matter','stage','Stage','select','Prospective|Open|On Hold|Closing|Closed|Archived'],['matter','closed_date','Closed Date','date'],
   ['matter_time_entry','hours','Hours','number'],['matter_time_entry','description','Description','text'],
   ['matter_time_entry','stage','Status','select','Draft|Submitted|Approved|Invoiced|Rejected'],
   ['matter_time_entry','billing_status','Billing Status','select','Not Billed|Eligible|Billed'],
   ['matter_deadline','stage','Status','select','Open|Completed|Cancelled'],['matter_deadline','completed_date','Completed Date','date'],
   ['conflict_check','check_status','Status','select','Pending|Cleared|Conflict Found'],
   ['conflict_check','cleared_date','Cleared Date','date'],['conflict_check','resolution_notes','Resolution Notes','text'],
  ],
  relationships:[
   {source:'matter_time_entry',target:'matter',relType:'many_to_one',forwardLabel:'Matter',reverseLabel:'Time Entries'},
   {source:'matter_deadline',target:'matter',relType:'many_to_one',forwardLabel:'Matter',reverseLabel:'Deadlines'},
   {source:'conflict_check',target:'matter',relType:'many_to_one',forwardLabel:'Matter',reverseLabel:'Conflict Checks'},
   {source:'matter',target:'Contact',relType:'many_to_one',forwardLabel:'Client',reverseLabel:'Matters'},
  ],
  rules:[
   {entity:'matter',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Closed',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'closed_date',value:'',message:''}]},
   {entity:'matter_time_entry',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Submitted',compareField:null,groupId:null},{fieldKey:'hours',operator:'less_than',value:'0.01',compareField:null,groupId:null}],
    actions:[{type:'block_save',message:'Enter more than zero hours before submitting a time entry.'}]},
   {entity:'matter_time_entry',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Submitted',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'description',value:'',message:''}]},
   {entity:'matter_deadline',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Completed',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'completed_date',value:'',message:''}]},
   {entity:'conflict_check',matchType:'all',
    conditions:[{fieldKey:'check_status',operator:'equals',value:'Cleared',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'cleared_date',value:'',message:''}]},
   {entity:'conflict_check',matchType:'all',
    conditions:[{fieldKey:'check_status',operator:'equals',value:'Conflict Found',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'resolution_notes',value:'',message:''}]},
  ],
  // Conflict Check's own "Conflict check requested" workflow desktop
  // ships is trigger_type:"record_created" - left out here, same
  // reasoning as every other record_created workflow in this file.
  workflows:[
   {entity:'matter',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Open',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Matter opening checklist',daysOffset:3}]},
   {entity:'matter_time_entry',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Approved',compareField:null,groupId:null}],
    actions:[{type:'update_field',updateFieldKey:'billing_status',updateValue:'Eligible',updateCopyFrom:''}]},
   {entity:'matter',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Closing',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Closing checklist',daysOffset:7}]},
   {entity:'matter',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Closed',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Final billing review',daysOffset:5}]},
  ],
 },
 nonprofit_association:{
  packageId:'lanesra.nonprofit_association',name:'Nonprofit & Association',industry:'Nonprofit',version:'1.0.0',
  description:'Constituents, memberships and donations for a small nonprofit or association.',
  appIcon:'🤝',
  objects:[
   {key:'constituent_profile',label:'Constituent Profile',labelPlural:'Constituent Profiles',icon:'👤',prefix:'CONST',digits:4},
   {key:'membership',label:'Membership',labelPlural:'Memberships',icon:'🪪',prefix:'MBR',digits:5},
   {key:'donation',label:'Donation',labelPlural:'Donations',icon:'💝',prefix:'DON',digits:5},
   {key:'grant',label:'Grant',labelPlural:'Grants',icon:'🏛',prefix:'GRT',digits:4},
  ],
  fields:[
   ['membership','stage','Stage','select','Pending|Active|Grace Period|Expired|Cancelled'],
   ['membership','start_date','Start Date','date'],['membership','end_date','End Date','date'],
   ['donation','amount','Amount','number'],['donation','stage','Status','select','Pending|Completed|Refunded'],
   ['grant','grant_status','Status','select','Applied|Awarded|Declined|Closed'],
   ['grant','awarded_amount','Awarded Amount','number'],['grant','award_date','Award Date','date'],
  ],
  relationships:[
   {source:'membership',target:'constituent_profile',relType:'many_to_one',forwardLabel:'Constituent',reverseLabel:'Memberships'},
   {source:'donation',target:'constituent_profile',relType:'many_to_one',forwardLabel:'Constituent',reverseLabel:'Donations'},
   {source:'grant',target:'constituent_profile',relType:'many_to_one',forwardLabel:'Funder',reverseLabel:'Grants'},
   {source:'constituent_profile',target:'Contact',relType:'many_to_one',forwardLabel:'Contact',reverseLabel:'Constituent Profile'},
  ],
  rules:[
   {entity:'membership',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Active',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'start_date',value:'',message:''},{type:'require',targetField:'end_date',value:'',message:''}]},
   {entity:'donation',matchType:'all',
    conditions:[{fieldKey:'amount',operator:'less_than',value:'0.01',compareField:null,groupId:null}],
    actions:[{type:'block_save',message:'Enter a donation amount greater than zero.'}]},
   {entity:'grant',matchType:'all',
    conditions:[{fieldKey:'grant_status',operator:'equals',value:'Awarded',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'awarded_amount',value:'',message:''},{type:'require',targetField:'award_date',value:'',message:''}]},
  ],
  // Grant's own "Grant application submitted" workflow desktop ships is
  // trigger_type:"record_created" - left out here, same reasoning as
  // every other record_created workflow in this file.
  workflows:[
   {entity:'donation',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Completed',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Send donation acknowledgement',daysOffset:3}]},
  ],
 },
 auto_service:{
  packageId:'lanesra.auto_service',name:'Auto Repair & Service Garage',industry:'Automotive',version:'1.0.0',
  description:'Vehicles, repair orders and appointments for an independent auto repair garage.',
  appIcon:'🚗',
  objects:[
   {key:'vehicle',label:'Vehicle',labelPlural:'Vehicles',icon:'🚗',prefix:'VEH',digits:5},
   {key:'repair_order',label:'Repair Order',labelPlural:'Repair Orders',icon:'🧾',prefix:'RO',digits:5},
   {key:'repair_line',label:'Repair Line',labelPlural:'Repair Lines',icon:'📄',prefix:'RL',digits:5},
   {key:'vehicle_appointment',label:'Appointment',labelPlural:'Appointments',icon:'📅',prefix:'APT',digits:5},
   {key:'parts_order',label:'Parts Order',labelPlural:'Parts Orders',icon:'📦',prefix:'PO',digits:5},
  ],
  fields:[
   ['vehicle','stage','Stage','select','Active|Sold or Transferred|Inactive'],['vehicle','make','Make','text'],['vehicle','model','Model','text'],
   ['repair_order','stage','Stage','select','Draft|Authorized|In Progress|Waiting Parts|Ready|Completed|Cancelled'],
   ['repair_order','odometer_in','Odometer In','number'],['repair_order','odometer_out','Odometer Out','number'],['repair_order','completion_date','Completion Date','date'],
   ['repair_line','stage','Stage','select','Proposed|Authorized|In Progress|Complete|Declined'],['repair_line','price','Price','number'],
   ['vehicle_appointment','stage','Status','select','Requested|Confirmed|Checked In|Completed|No Show|Cancelled'],
   ['parts_order','order_status','Status','select','Ordered|Backordered|Received|Cancelled'],
   ['parts_order','part_name','Part Name','text'],['parts_order','received_date','Received Date','date'],
  ],
  relationships:[
   {source:'repair_order',target:'vehicle',relType:'many_to_one',forwardLabel:'Vehicle',reverseLabel:'Repair Orders'},
   {source:'repair_line',target:'repair_order',relType:'many_to_one',forwardLabel:'Repair Order',reverseLabel:'Lines'},
   {source:'vehicle_appointment',target:'vehicle',relType:'many_to_one',forwardLabel:'Vehicle',reverseLabel:'Appointments'},
   {source:'parts_order',target:'repair_order',relType:'many_to_one',forwardLabel:'Repair Order',reverseLabel:'Parts Orders'},
   {source:'vehicle',target:'Contact',relType:'many_to_one',forwardLabel:'Owner',reverseLabel:'Vehicles'},
  ],
  rules:[
   {entity:'repair_order',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Completed',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'completion_date',value:'',message:''},{type:'require',targetField:'odometer_out',value:'',message:''}]},
   {entity:'repair_line',matchType:'all',
    conditions:[{fieldKey:'stage',operator:'equals',value:'Authorized',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'price',value:'',message:''}]},
   {entity:'repair_order',matchType:'all',
    conditions:[{fieldKey:'odometer_out',operator:'less_than',value:'',compareField:'odometer_in',groupId:null}],
    actions:[{type:'block_save',message:'Odometer out cannot be less than odometer in.'}]},
   {entity:'parts_order',matchType:'all',
    conditions:[{fieldKey:'order_status',operator:'equals',value:'Received',compareField:null,groupId:null}],
    actions:[{type:'require',targetField:'received_date',value:'',message:''}]},
  ],
  workflows:[
   {entity:'repair_order',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'Authorized',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Assign technician',daysOffset:1}]},
   {entity:'vehicle_appointment',matchType:'all',notify:false,
    conditions:[{fieldKey:'stage',operator:'equals',value:'No Show',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Reschedule appointment',daysOffset:1}]},
   {entity:'parts_order',matchType:'all',notify:false,
    conditions:[{fieldKey:'order_status',operator:'equals',value:'Backordered',compareField:null,groupId:null}],
    actions:[{type:'create_task',taskTitle:'Follow up on backordered part',daysOffset:3}]},
  ],
 },
};
// Every custom-object key a package would create - used to block importing/
// installing a package that collides with an object already in this
// workspace (matches desktop's own import-time collision check).
function packageObjectKeys(pkg){return pkg.objects.map(o=>o.key)}
function packageArtifactCount(pkg){return pkg.objects.length+pkg.fields.length+pkg.relationships.length+pkg.rules.length+pkg.workflows.length}
function packageCollision(pkg){return packageObjectKeys(pkg).find(k=>(data.customObjects||[]).some(o=>o.key===k)||RESERVED_ENTITY_KEYS.includes(k))}
// ---- Solution Management Phase 2 mirror: Publishers -----------------------
// Same reasoning as the desktop edition's publisher_service: a real
// registry, not a stub - see migration 0029's own comment there. Every
// workspace auto-seeds two publishers idempotently (called from
// migrateData on every load, so a save from before this feature existed
// self-heals with no separate migration step): "lanesra" (owns every
// bundled reference package, all of which already ship packageIds
// prefixed "lanesra.") and "local" (the implicit home for hand-built
// customizations - component-tagging, below, wires every hand-built and
// installed component to one of these two, mirroring desktop's
// solution_component_service exactly).
const RESERVED_PUBLISHER_KEYS=['lanesra','local'];
function ensureDefaultPublishers(){
 if(!data.publishers)data.publishers=[];
 const seed=(key,name,description,isOfficial,isLocal)=>{
  if(data.publishers.some(p=>p.key===key))return;
  data.publishers.push(stampCreate({id:uid(),key,name,description,isOfficial,isLocal}));
 };
 seed('lanesra','Lanesra OS','The official publisher of every bundled industry reference package.',true,false);
 seed('local','Local Workspace','The implicit home for whatever you build by hand in this workspace, rather than install from a package.',false,true);
}
// Lowercase ascii letters/digits/underscore, must start with a letter,
// 2-32 characters - mirrors publisher_service::validate_key exactly, since
// this becomes a literal dotted-namespace prefix on every package_id
// under it, the same "no surprises" shape a URL path segment needs.
function validatePublisherKey(rawKey){
 const key=(rawKey||'').trim().toLowerCase();
 if(key.length<2||key.length>32)return 'Publisher key must be 2-32 characters';
 if(!/^[a-z][a-z0-9_]*$/.test(key))return 'Publisher key must start with a lowercase letter and contain only lowercase letters, digits and underscores';
 return null;
}
function createPublisher(input){
 const key=(input.key||'').trim().toLowerCase();
 const keyError=validatePublisherKey(key);
 if(keyError)return {error:keyError};
 if(RESERVED_PUBLISHER_KEYS.includes(key))return {error:`'${key}' is a reserved publisher key`};
 if(!(input.name||'').trim())return {error:'Publisher name is required'};
 if(data.publishers.some(p=>p.key===key))return {error:`A publisher with key '${key}' already exists in this workspace`};
 const publisher=stampCreate({id:uid(),key,name:input.name.trim(),description:input.description?.trim()||null,isOfficial:false,isLocal:false});
 data.publishers.push(publisher);
 save();
 return {publisher};
}
function publisherKeyFromPackageId(packageId){return (packageId||'').split('.')[0]||null}
function publisherForPackageId(packageId){const key=publisherKeyFromPackageId(packageId);return key?data.publishers.find(p=>p.key===key):null}

// ---- Solution Management Phase 3 mirror: component-tagging, Local
// Workspace, export ---------------------------------------------------------
// Mirrors desktop's solution_component_service: a single workspace-wide
// `data.components` registry (id, type, metadataId, publisherKey,
// installedAppId) covering every hand-built AND package-installed
// customObject/customField/relationship/businessRule/workflow/screenLayout/
// customReport. Two ways it's populated, exactly like the desktop core:
//  - tagLocalComponent is called from every one of the 7 hand-built
//    creation sites (the Admin screens' own "save" handlers) the instant a
//    new one is made - the ordinary admin-UI path has no other publisher
//    context to tag with.
//  - retagComponent is called by installReferencePackage right after it
//    builds a package's artifacts array, overwriting the 'local' tag with
//    the installing publisher (desktop's run_install does the identical
//    retag step against the same solution_components rows).
function tagLocalComponent(type,id){
 if(!data.components)data.components=[];
 ensureDefaultPublishers();
 const existing=data.components.find(c=>c.type===type&&c.metadataId===id);
 if(existing){existing.publisherKey='local';existing.installedAppId=null}
 else data.components.push({id:uid(),type,metadataId:id,publisherKey:'local',installedAppId:null,createdAt:new Date().toISOString()});
}
function retagComponent(type,id,publisherKey,installedAppId){
 if(!data.components)data.components=[];
 const existing=data.components.find(c=>c.type===type&&c.metadataId===id);
 if(existing){existing.publisherKey=publisherKey;existing.installedAppId=installedAppId}
 else data.components.push({id:uid(),type,metadataId:id,publisherKey,installedAppId,createdAt:new Date().toISOString()});
}
// The Managed/Unmanaged distinction's Unmanaged half: everything still
// owned by 'local', broken down by type - the Solution Packages tab's
// synthetic "Local Workspace" row, without ever inventing a fake
// installedApps entry for it.
function localWorkspaceSummary(){
 const comps=(data.components||[]).filter(c=>c.publisherKey==='local');
 const byType={};
 comps.forEach(c=>{byType[c.type]=(byType[c.type]||0)+1});
 return {componentCount:comps.length,byType};
}
// Every component in the workspace, joined with its owning publisher and
// (if installed) which app installed it - the Components tab's data
// source, superseding the narrower installedApps-only artifact list.
function listSolutionComponents(){
 return (data.components||[]).map(c=>{
  const publisher=data.publishers.find(p=>p.key===c.publisherKey);
  const installedApp=c.installedAppId?(data.installedApps||[]).find(a=>a.id===c.installedAppId):null;
  return {...c,publisherName:publisher?publisher.name:c.publisherKey,isLocal:!!publisher?.isLocal,installedAppName:installedApp?installedApp.name:null};
 });
}
// Builds a downloadable JSON snapshot of everything the 'local' publisher
// owns - the same idea as desktop's export_local_workspace, in the shape
// of a manifest (package_id/objects/fields/relationships/business_rules/
// workflows/reports) so it reads the same way on both platforms. Unlike
// desktop, the online demo has no pathway to *import* a hand-authored
// manifest back in (installReferencePackage only ever installs from the
// fixed REFERENCE_PACKAGES catalog by key) - this is a real, complete
// export for backup/inspection/sharing, not a re-importable package
// within the demo itself. Documented here rather than silently
// overclaiming round-trip support the demo genuinely doesn't have.
// Shared by exportLocalWorkspace and exportSolution: everything about
// turning a set of component ids into a manifest object is identical
// between the two - only which ids go in, and what package_id/name/
// version get stamped on the result, differ. Mirrors desktop's
// build_export_manifest helper (industry_package_service.rs).
function buildExportManifest(idSet,packageId,name,version){
 const objects=(data.customObjects||[]).filter(o=>idSet.has(o.id)).map(o=>({key:o.key,singular_label:o.label,plural_label:o.labelPlural,icon:o.icon,prefix:o.prefix,digits:o.digits}));
 const fields=(data.customFields||[]).filter(f=>idSet.has(f.id)).map(f=>({key:f.key,entity_type:f.entity,label:f.label,field_type:f.type,options:f.options?f.options.split('|'):[],required:!!f.required}));
 const relationships=(data.relationshipDefinitions||[]).filter(r=>idSet.has(r.id)).map(r=>({key:r.key,source_entity_type:r.sourceEntity,target_entity_type:r.targetEntity,relationship_type:r.relType,forward_label:r.forwardLabel,reverse_label:r.reverseLabel}));
 const businessRules=(data.fieldRules||[]).filter(r=>idSet.has(r.id)).map(r=>({entity_type:r.entity,match_type:r.matchType,conditions:r.conditions,actions:r.actions}));
 const workflows=(data.workflowRules||[]).filter(w=>idSet.has(w.id)).map(w=>({entity_type:w.entity,match_type:w.matchType,conditions:w.conditions,actions:w.actions}));
 const reports=(data.customReports||[]).filter(r=>idSet.has(r.id)).map(r=>({name:r.name,entity_type:r.entityKey,group_by_source:r.groupBySource,group_by_field:r.groupByField,aggregate:r.aggregate,sum_field_key:r.sumFieldKey||null}));
 return {format_version:1,package_id:packageId,name,industry:'Custom',version,min_lanesra_version:'0.1.0',objects,fields,relationships,business_rules:businessRules,workflows,reports};
}
function exportLocalWorkspace(){
 const localIds=new Set((data.components||[]).filter(c=>c.publisherKey==='local').map(c=>c.metadataId));
 return JSON.stringify(buildExportManifest(localIds,'local.workspace_export','Local Workspace Export',`1.0.${Date.now()}`),null,2);
}
// ---- Solution Management Phase 4 mirror: named, scoped Solutions --------
// The Dynamics-365-style "build a solution in test, export it, import it
// in prod" workflow. Where exportLocalWorkspace is all-or-nothing across
// everything the 'local' publisher owns, a Solution is a named, versioned,
// admin-picked *subset* of components (data.solutions[].memberIds, each
// {type,metadataId} - the same identity pair data.components uses),
// exportable on its own into the identical manifest shape. Mirrors
// desktop's `solutions`/`solution_members` tables (migration 0031) and
// solution_service - see that migration's own comment for why
// "environment" needed no new modeling here either: this browser tab's
// localStorage is one workspace, same as one desktop install is one
// workspace: promoting a Solution to "prod" means downloading its export
// and importing it into a *different* workspace (a different browser
// profile, or - for a real round trip - the desktop edition, whose Admin →
// App Catalog can actually import a hand-authored manifest back in, unlike
// this demo's fixed reference-package catalog).
function createSolution(input){
 const name=(input.name||'').trim();
 if(!name)return {error:'Solution name is required'};
 if(data.solutions.some(s=>s.name===name))return {error:`A solution named '${name}' already exists in this workspace`};
 ensureDefaultPublishers();
 const solution=stampCreate({id:uid(),name,description:(input.description||'').trim()||null,version:'1.0.0.0',publisherKey:'local',memberIds:[]});
 data.solutions.push(solution);
 save();
 return {solution};
}
function updateSolution(id,input){
 const solution=data.solutions.find(s=>s.id===id); if(!solution)return {error:'Solution not found'};
 const name=(input.name||'').trim();
 if(!name)return {error:'Solution name is required'};
 if(data.solutions.some(s=>s.id!==id&&s.name===name))return {error:`A solution named '${name}' already exists in this workspace`};
 solution.name=name;
 solution.description=(input.description||'').trim()||null;
 solution.version=(input.version||'').trim()||solution.version;
 solution.updatedAt=new Date().toISOString();solution.updatedBy=CURRENT_USER_ID;
 save();
 return {solution};
}
function deleteSolution(id){
 data.solutions=data.solutions.filter(s=>s.id!==id);
 save();
}
// Curating a component only ever records the (type,metadataId) pair
// alongside the solution - the component itself, and its data.components
// ownership tag, are untouched (removing it from a solution never deletes
// it), mirroring desktop's solution_members table.
function addSolutionMember(solutionId,type,metadataId){
 const solution=data.solutions.find(s=>s.id===solutionId); if(!solution)return;
 if(!solution.memberIds.some(m=>m.type===type&&m.metadataId===metadataId))solution.memberIds.push({type,metadataId});
 save();
}
function removeSolutionMember(solutionId,type,metadataId){
 const solution=data.solutions.find(s=>s.id===solutionId); if(!solution)return;
 solution.memberIds=solution.memberIds.filter(m=>!(m.type===type&&m.metadataId===metadataId));
 save();
}
// A Solution's members resolved to the same display shape
// listSolutionComponents() already gives the Components tab.
function solutionMembersResolved(solution){
 const keys=new Set(solution.memberIds.map(m=>`${m.type}:${m.metadataId}`));
 return listSolutionComponents().filter(c=>keys.has(`${c.type}:${c.metadataId}`));
}
function exportSolution(solutionId){
 const solution=data.solutions.find(s=>s.id===solutionId); if(!solution)return null;
 const ids=new Set(solution.memberIds.map(m=>m.metadataId));
 return JSON.stringify(buildExportManifest(ids,`local.solution.${solution.id}`,solution.name,solution.version),null,2);
}
function solutionFilename(name){
 const slug=(name||'solution').toLowerCase().replace(/[^a-z0-9]+/g,'-').replace(/(^-+|-+$)/g,'');
 return `${slug||'solution'}.lanesra.json`;
}
function downloadJson(filename,content){
 const blob=new Blob([content],{type:'application/json;charset=utf-8;'});
 const url=URL.createObjectURL(blob);
 const a=document.createElement('a');
 a.href=url;a.download=filename;document.body.appendChild(a);a.click();document.body.removeChild(a);
 URL.revokeObjectURL(url);
}

function importPackage(key){
 const pkg=REFERENCE_PACKAGES[key]; if(!pkg)return;
 if((data.appPackages||[]).some(p=>p.key===key))return toast('Already imported');
 // Publisher/namespace scope, enforced the same way import_package does
 // on desktop: a package_id's prefix must resolve to a registered
 // publisher. Every bundled starter is "lanesra.<name>" and "lanesra"
 // always auto-seeds, so this never actually blocks the Import button
 // above - it exists so the concept is real, not just documented.
 const publisher=publisherForPackageId(pkg.packageId);
 if(!publisher){
  const wantKey=publisherKeyFromPackageId(pkg.packageId);
  return alert(`'${wantKey}' isn't a registered publisher in this workspace yet - register it under Admin → Solution Management → Publishers before importing a package under that namespace.`);
 }
 data.appPackages.push({id:uid(),key,packageId:pkg.packageId,name:pkg.name,industry:pkg.industry,version:pkg.version,importedAt:new Date().toISOString()});
 save();toast(`${pkg.name} imported - review it below, then Install`);
}
// Creates every artifact a package defines - Custom Objects, their fields,
// the relationships between them, business rules, workflow rules, and a
// published App grouping the lot - in one step, then records what it did in
// data.installedApps so Deactivate/Reactivate can act on the whole bundle
// without re-reading the package definition. Mirrors desktop's transactional
// industry_package_service::install, minus the rollback-on-error machinery
// this demo's single-threaded, no-network save doesn't need.
function installReferencePackage(key){
 const pkg=REFERENCE_PACKAGES[key]; if(!pkg)return;
 const collision=packageCollision(pkg);
 if(collision)return alert(`Cannot install ${pkg.name} — "${collision}" is already used by another object in this workspace.`);
 // Solution Management's Components tab mirror: every id created below is
 // recorded here, the same (type, id) shape as desktop's package_artifacts
 // - the "what did this install actually create" ledger neither this array
 // nor its desktop counterpart had before that feature existed.
 const artifacts=[];
 pkg.objects.forEach(o=>{const id=uid();data.customObjects.push({id,key:o.key,label:o.label,labelPlural:o.labelPlural,icon:o.icon,prefix:o.prefix,digits:o.digits,active:true});data[o.key]=[];artifacts.push({type:'customObject',id})});
 pkg.fields.forEach(([entity,key,label,type,options])=>{const f=stampCreate(freshFieldDef(entity,key,label,type,options));data.customFields.push(f);artifacts.push({type:'customField',id:f.id})});
 const relIds={};
 pkg.relationships.forEach(r=>{
  const relKey=`${r.source}_${r.target}`;
  const rel=stampCreate({id:uid(),key:relKey,sourceEntity:r.source,targetEntity:r.target,relType:r.relType,forwardLabel:r.forwardLabel,reverseLabel:r.reverseLabel,deleteBehavior:'restrict',showRelatedList:true,required:false,active:true,protected:false});
  data.relationshipDefinitions.push(rel);
  relIds[relKey]=rel.id;
  artifacts.push({type:'relationship',id:rel.id});
 });
 // Per-app scoped automation: a package's rules/workflows are tagged to
 // the app this install creates below, not left workspace-wide - the
 // natural default for "installed as its own focused app" content,
 // mirroring the desktop edition's identical choice in
 // industry_package_service. Created before the rules/workflows so its id
 // is already known.
 const app=freshApp(pkg.name,pkg.appIcon);
 app.description=pkg.description;
 app.objectKeys=packageObjectKeys(pkg);
 app.isPublished=true;
 pkg.rules.forEach(r=>{const rule=stampCreate({id:uid(),active:true,entity:r.entity,matchType:r.matchType,conditions:r.conditions,actions:r.actions,appId:app.id});data.fieldRules.push(rule);artifacts.push({type:'businessRule',id:rule.id})});
 pkg.workflows.forEach(w=>{const wf=stampCreate({id:uid(),active:true,conditionsMerged:true,entity:w.entity,matchType:w.matchType,conditions:w.conditions,actions:w.actions,notify:w.notify,appId:app.id});data.workflowRules.push(wf);artifacts.push({type:'workflow',id:wf.id})});
 data.apps.push(app);
 const installedId=uid();
 data.installedApps.push({id:installedId,key,packageId:pkg.packageId,name:pkg.name,industry:pkg.industry,version:pkg.version,status:'active',appId:app.id,objectKeys:app.objectKeys,artifacts,installedAt:new Date().toISOString()});
 data.appPackages=(data.appPackages||[]).filter(p=>p.key!==key);
 // Component-tagging (Phase 3): every artifact above was already tagged
 // 'local' the instant its own create call ran (see tagLocalComponent,
 // wired into the same 5 creation sites the hand-built admin screens
 // use) - this overwrites that tag with the resolved installing
 // publisher, exactly mirroring desktop's run_install retag step.
 const publisherKey=publisherKeyFromPackageId(pkg.packageId)||'lanesra';
 artifacts.forEach(a=>retagComponent(a.type,a.id,publisherKey,installedId));
 syncCustomObjectRegistry();
 renderSidebarNav();
 save();toast(`${pkg.name} installed`);
}
// Deactivate hides the package's objects from nav/creation and unpublishes
// its App, without deleting any data - the same "safe to undo" semantics
// Custom Objects' own deactivate already has. Reactivate reverses both.
function setInstalledAppActive(installedId,activate){
 const installed=(data.installedApps||[]).find(a=>a.id===installedId); if(!installed)return;
 installed.status=activate?'active':'inactive';
 (installed.objectKeys||[]).forEach(k=>{const o=customObjectByKey(k); if(o)o.active=activate});
 const app=(data.apps||[]).find(a=>a.id===installed.appId);
 if(app)app.isPublished=activate;
 if(!activate&&activeAppId===installed.appId)activeAppId=null;
 syncCustomObjectRegistry();
 renderSidebarNav();
 save();toast(activate?'App reactivated':'App deactivated');
}
// ---- App Catalog "Details" preview (before Install) ------------------------
// Mirrors desktop's PackageDetailsPanel: a plain-language read of what a
// reference package builds, how it connects to existing data, and its
// automation - built entirely off a REFERENCE_PACKAGES entry, before
// anything is installed. Deliberately its own field-label resolver rather
// than fieldLabelFor/conditionFieldsFor, which read data.customFields -
// already-installed fields this package hasn't created yet.
function isBuiltinPackageEntity(entityKey){return APP_OBJECT_TYPES.includes(entityKey)}
function packageObjectLabel(pkg,entityKey){
 if(isBuiltinPackageEntity(entityKey))return APP_OBJECT_TYPE_LABELS[entityKey]||entityKey;
 const o=pkg.objects.find(o=>o.key===entityKey);
 return o?o.labelPlural:entityKey;
}
function packageFieldLabel(pkg,entityKey,fieldKey){
 if(!fieldKey)return '';
 const f=pkg.fields.find(f=>f[0]===entityKey&&f[1]===fieldKey);
 return f?f[2]:fieldKey;
}
function packageDescribeCondition(pkg,entityKey,c){
 return `${packageFieldLabel(pkg,entityKey,c.fieldKey)} ${OPERATOR_LABELS[c.operator]||'is'}${operatorNeedsValue(c.operator)?' '+(c.compareField?packageFieldLabel(pkg,entityKey,c.compareField):badgeMaybe(c.value)):''}`;
}
function packageDescribeConditions(pkg,entityKey,conditions,matchType){
 if(!conditions||!conditions.length)return 'always';
 const units=groupConditionUnits(conditions);
 const parts=units.map(u=>u.kind==='single'?packageDescribeCondition(pkg,entityKey,conditions[u.index]):`(${u.indices.map(i=>packageDescribeCondition(pkg,entityKey,conditions[i])).join(' OR ')})`);
 return parts.join(matchType==='any'?' OR ':' AND ');
}
function packageDescribeRuleAction(pkg,entityKey,a){
 const target=a.targetField?packageFieldLabel(pkg,entityKey,a.targetField):'';
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
function packageDescribeWorkflowAction(pkg,entityKey,a){
 if(a.type==='create_record')return `create a new ${a.recordTargetEntity?packageObjectLabel(pkg,a.recordTargetEntity):'record'}`;
 if(a.type==='update_related_record')return `set ${packageFieldLabel(pkg,a.relTargetEntity,a.relTargetField)} = "${a.relValue||''}" on the related ${packageObjectLabel(pkg,a.relTargetEntity)}`;
 if(a.type==='update_field'||a.type==='set_default_field'||a.type==='clear_field'){
  if(!a.updateFieldKey)return 'set a field on this record';
  if(a.type==='clear_field')return `clear ${packageFieldLabel(pkg,entityKey,a.updateFieldKey)}`;
  const prefix=a.type==='set_default_field'?'default ':'set ';
  const suffix=a.type==='set_default_field'?' (only if currently empty)':'';
  return a.updateCopyFrom?`${prefix}${packageFieldLabel(pkg,entityKey,a.updateFieldKey)} = value copied from ${packageFieldLabel(pkg,entityKey,a.updateCopyFrom)}${suffix}`:`${prefix}${packageFieldLabel(pkg,entityKey,a.updateFieldKey)} = "${a.updateValue||''}"${suffix}`;
 }
 return `create task "${a.taskTitle||''}" (${a.daysOffset?`due ${a.daysOffset} day(s) later`:'due same day'})`;
}
// Builds the whole expanded-row preview for one package - see this
// section's own doc comment above for what it deliberately reuses vs.
// resolves fresh from the manifest.
function packageDetailsHtml(pkg){
 const builtinRelationships=pkg.relationships.filter(r=>isBuiltinPackageEntity(r.source)||isBuiltinPackageEntity(r.target));
 const internalRelationships=pkg.relationships.filter(r=>!isBuiltinPackageEntity(r.source)&&!isBuiltinPackageEntity(r.target));
 const fieldsByEntity=k=>pkg.fields.filter(f=>f[0]===k);
 const objectsHtml=pkg.objects.map(o=>{
  const n=fieldsByEntity(o.key).length;
  return `<li>${o.icon} <strong>${o.labelPlural}</strong> (${n} custom field${n===1?'':'s'}, IDs like ${o.prefix}-${'0'.repeat(o.digits)})</li>`;
 }).join('');
 const internalRelHtml=internalRelationships.length?`<ul class="muted" style="margin:8px 0 0;padding-left:18px;font-size:12px;display:grid;gap:2px">${internalRelationships.map(r=>`<li>${packageObjectLabel(pkg,r.source)} → ${packageObjectLabel(pkg,r.target)} (${r.forwardLabel})</li>`).join('')}</ul>`:'';
 const builtinRelHtml=builtinRelationships.length?`<section><h4 style="margin-bottom:6px">How it connects to your existing data</h4><ul style="margin:0;padding-left:18px;font-size:13px;display:grid;gap:4px">${builtinRelationships.map(r=>{
  const sourceIsBuiltin=isBuiltinPackageEntity(r.source);
  const newObj=sourceIsBuiltin?r.target:r.source;
  const existing=sourceIsBuiltin?r.source:r.target;
  return `<li>${packageObjectLabel(pkg,newObj)} link${sourceIsBuiltin?'s from':'s to'} your existing <strong>${APP_OBJECT_TYPE_LABELS[existing]||existing}</strong> (${r.forwardLabel})</li>`;
 }).join('')}</ul></section>`:'';
 const rulesHtml=pkg.rules.map(r=>`<li><strong>${packageObjectLabel(pkg,r.entity)}:</strong> when ${packageDescribeConditions(pkg,r.entity,r.conditions,r.matchType)}, ${r.actions.map(a=>packageDescribeRuleAction(pkg,r.entity,a)).join('; ')}.</li>`).join('');
 const workflowsHtml=pkg.workflows.map(w=>`<li>When ${packageObjectLabel(pkg,w.entity)}'s conditions are met (${packageDescribeConditions(pkg,w.entity,w.conditions,w.matchType)}), ${w.actions.map(a=>packageDescribeWorkflowAction(pkg,w.entity,a)).join(' and ')}${w.notify?' and admins are notified':''}.</li>`).join('');
 const automationHtml=(pkg.rules.length||pkg.workflows.length)?`<section><h4 style="margin-bottom:6px">Automation, in plain language</h4><ul style="margin:0;padding-left:18px;font-size:13px;display:grid;gap:6px">${rulesHtml}${workflowsHtml}</ul></section>`:'';
 const linkedBuiltins=[...new Set(builtinRelationships.flatMap(r=>[r.source,r.target]).filter(isBuiltinPackageEntity))].map(k=>APP_OBJECT_TYPE_LABELS[k]||k);
 const guidance=[];
 if(linkedBuiltins.length)guidance.push(`This package links its new objects into your existing ${linkedBuiltins.join(', ')} - install it once you have real records there, not before, so the first ${packageObjectLabel(pkg,pkg.objects[0]?.key||'')} you create has something real to connect to.`);
 guidance.push('Object names, ID prefixes and picklist options match this package\'s own defaults - rename fields, adjust numbering, or add/remove select options from Custom Objects after installing to match how your team actually talks about the work.');
 guidance.push('It installs as its own App with its own rules and workflows (see Admin → Apps) - nobody\'s permissions change automatically, review who should see it after installing.');
 guidance.push('Every rule and workflow here only reads and writes fields on the record that actually changed - a process that needs to check other linked records first (an overlap check, a running total) still needs a person to enforce it for now.');
 return `<div style="padding:8px 0;display:grid;gap:16px">
 <p class="muted" style="font-size:13px;margin-top:0">${pkg.description}</p>
 <section><h4 style="margin-bottom:6px">What this builds</h4><ul style="margin:0;padding-left:18px;font-size:13px;display:grid;gap:4px">${objectsHtml}</ul>${internalRelHtml}</section>
 ${builtinRelHtml}
 ${automationHtml}
 <section><h4 style="margin-bottom:6px">Fitting this to your organization</h4><ul class="muted" style="margin:0;padding-left:18px;font-size:13px;display:grid;gap:4px">${guidance.map(g=>`<li>${g}</li>`).join('')}</ul></section>
 </div>`;
}
let previewPackageKey=null;
function packagesTab(body){
 const catalogKeys=Object.keys(REFERENCE_PACKAGES).filter(k=>!(data.appPackages||[]).some(p=>p.key===k)&&!(data.installedApps||[]).some(a=>a.key===k));
 const imported=data.appPackages||[];
 const installed=data.installedApps||[];
 body.innerHTML=`
 <div class="panel"><h3 style="margin-top:0">Available starter packages</h3>
 <p class="muted">Bundled reference packages - each creates a small, working set of Custom Objects, fields, relationships, business rules and workflow automation for a specific industry, plus an App grouping them together. Import one to review what it contains, then install it.</p>
 ${catalogKeys.length?`<div style="display:flex;gap:12px;flex-wrap:wrap">${catalogKeys.map(k=>{const p=REFERENCE_PACKAGES[k];return `<div class="panel" style="flex:1;min-width:240px;background:var(--surface-alt,#f7f8fc)"><div style="font-weight:700">${p.appIcon} ${p.name}</div><p class="muted" style="font-size:12px">${p.description}</p><p class="muted" style="font-size:12px">${p.objects.length} objects · ${p.fields.length} fields · ${p.relationships.length} relationships · ${p.rules.length} rules · ${p.workflows.length} workflows</p><button class="btn btn-secondary" data-import="${k}">Import</button></div>`}).join('')}</div>`:'<div class="empty">Every starter package has been imported or installed.</div>'}
 </div>
 <div class="panel"><h3 style="margin-top:0">Imported packages</h3>
 <p class="muted" style="font-size:13px">Review a package's "Details" before installing - what it builds, how it connects to your existing Companies/Contacts, and its automation in plain language.</p>
 ${imported.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Package</th><th>Industry</th><th>Version</th><th>Imported</th><th></th></tr></thead><tbody>${imported.map(p=>{
   const pkg=REFERENCE_PACKAGES[p.key];
   const open=previewPackageKey===p.key;
   return `<tr><td>${pkg?.appIcon||''} ${p.name}</td><td>${p.industry}</td><td>${p.version}</td><td>${new Date(p.importedAt).toLocaleString()}</td><td><div class="actions"><button class="icon-btn" data-preview="${p.key}">${open?'Hide details':'Details'}</button><button class="btn btn-primary" data-install="${p.key}">Install</button><button class="icon-btn" data-discard="${p.key}">Discard</button></div></td></tr>${open&&pkg?`<tr><td colspan="5" style="background:var(--surface-alt,#f7f8fc)">${packageDetailsHtml(pkg)}<button class="btn btn-primary" data-install="${p.key}" style="margin-top:4px">Install this package</button></td></tr>`:''}`;
  }).join('')}</tbody></table></div>`:'<div class="empty">Nothing imported yet.</div>'}
 </div>
 <div class="panel"><h3 style="margin-top:0">Installed apps</h3>
 ${installed.length?`<div class="table-wrap"><table class="table"><thead><tr><th>App</th><th>Industry</th><th>Version</th><th>Creates</th><th>Status</th><th></th></tr></thead><tbody>${installed.map(a=>`<tr><td>${REFERENCE_PACKAGES[a.key]?.appIcon||''} ${a.name}</td><td>${a.industry}</td><td>${a.version}</td><td>${(a.objectKeys||[]).length} object${(a.objectKeys||[]).length===1?'':'s'}</td><td>${badgeMaybe(a.status==='active'?'Active':'Inactive')}</td><td>${a.status==='active'?`<button class="btn btn-secondary" data-deactivate="${a.id}">Deactivate</button>`:`<button class="btn btn-secondary" data-reactivate="${a.id}">Reactivate</button>`}</td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">Nothing installed yet.</div>'}
 </div>`;
 body.querySelectorAll('[data-preview]').forEach(b=>b.onclick=()=>{previewPackageKey=previewPackageKey===b.dataset.preview?null:b.dataset.preview;renderAdminTab()});
 body.querySelectorAll('[data-import]').forEach(b=>b.onclick=()=>{importPackage(b.dataset.import);renderAdminTab()});
 body.querySelectorAll('[data-discard]').forEach(b=>b.onclick=()=>{data.appPackages=(data.appPackages||[]).filter(p=>p.key!==b.dataset.discard);save();toast('Removed from catalog');renderAdminTab()});
 body.querySelectorAll('[data-install]').forEach(b=>b.onclick=()=>{installReferencePackage(b.dataset.install);renderAdminTab()});
 body.querySelectorAll('[data-deactivate]').forEach(b=>b.onclick=()=>{setInstalledAppActive(b.dataset.deactivate,false);renderAdminTab()});
 body.querySelectorAll('[data-reactivate]').forEach(b=>b.onclick=()=>{setInstalledAppActive(b.dataset.reactivate,true);renderAdminTab()});
}
// ---- Solution Management (Admin IA design spec) ----------------------------
// Mirrors the desktop edition's SolutionManagementAdmin.tsx: a read-only
// landing over data App Catalog already collects (installedApps/
// appPackages/artifacts), plus the Publishers registry above. "What's
// installed, what did it create, and who published it." Deliberately not
// built here either, matching desktop: a real Managed/Unmanaged
// distinction, component-tagging for hand-built objects, and any
// write/deploy action beyond registering a publisher - install/deactivate
// stay on Admin → App Catalog, the screen that already owns them.
const ARTIFACT_TYPE_LABELS={customObject:'custom object',customField:'custom field',relationship:'relationship',businessRule:'business rule',workflow:'workflow',screenLayout:'screen layout',customReport:'custom report'};
function artifactTypeLabel(type){return ARTIFACT_TYPE_LABELS[type]||type}
let solutionsSubTab='packages';
function solutionsTab(body){
 const subTabs=[['packages','Solution Packages'],['solutions','Solutions'],['components','Components'],['dependencies','Dependencies'],['publishers','Publishers']];
 body.innerHTML=`<div class="panel">
 <h3 style="margin-top:0">Solution Management</h3>
 <p class="muted" style="font-size:13px">Every industry app installed in this workspace, what it created, and who published it - plus everything you've built by hand, grouped as your Local Workspace. Install, deactivate or reactivate an app from <b>Admin → App Catalog</b>; this is where you see the result and export your own customizations. Building something to ship to another workspace on purpose? Pick exactly what goes in it under <b>Solutions</b>.</p>
 <div class="tabs">${subTabs.map(t=>`<button class="tab ${solutionsSubTab===t[0]?'active':''}" data-solutions-tab="${t[0]}">${t[1]}</button>`).join('')}</div>
 <div id="solutionsBody"></div>
 </div>`;
 body.querySelectorAll('[data-solutions-tab]').forEach(b=>b.onclick=()=>{solutionsSubTab=b.dataset.solutionsTab;renderSolutionsSubTab()});
 renderSolutionsSubTab();
}
function renderSolutionsSubTab(){
 document.querySelectorAll('[data-solutions-tab]').forEach(b=>b.classList.toggle('active',b.dataset.solutionsTab===solutionsSubTab));
 const body=$('#solutionsBody');
 ({packages:solutionPackagesSubTab,solutions:solutionsListSubTab,components:solutionComponentsSubTab,dependencies:solutionDependenciesSubTab,publishers:solutionPublishersSubTab}[solutionsSubTab])(body);
}
function solutionPackagesSubTab(body){
 const installed=data.installedApps||[];
 const components=listSolutionComponents();
 const localSummary=localWorkspaceSummary();
 const localRow=localSummary.componentCount?`<tr><td>🧩 Local Workspace</td><td>local</td><td><span class="badge">Unmanaged</span></td><td>—</td><td>—</td><td>${localSummary.componentCount}</td><td>—</td><td><button class="btn btn-secondary" id="exportLocalWorkspace" style="font-size:12px;padding:4px 8px">Export</button></td></tr>`:'';
 body.innerHTML=`<div style="margin-top:16px">${installed.length||localSummary.componentCount?`<div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Publisher</th><th>Type</th><th>Version</th><th>Status</th><th>Components</th><th>Dependencies</th><th></th></tr></thead><tbody>${installed.map(a=>{const publisher=publisherForPackageId(a.packageId);const componentCount=components.filter(c=>c.installedAppId===a.id).length;return `<tr><td>${REFERENCE_PACKAGES[a.key]?.appIcon||''} ${a.name}</td><td>${publisher?publisher.name:'—'}</td><td><span class="badge">Managed</span></td><td>${a.version}</td><td>${badgeMaybe(a.status==='active'?'Active':'Inactive')}</td><td>${componentCount}</td><td>0</td><td></td></tr>`}).join('')}${localRow}</tbody></table></div>`:'<div class="empty">Nothing installed yet. Install a reference package from <b>Admin → App Catalog</b> to see it here.</div>'}
 ${installed.length?'<p class="muted" style="font-size:12px;margin-top:10px">Releases (multiple imported versions of one package) and update-with-diff don\'t apply here - the demo installs from a fixed, single-version reference-package catalog, unlike the desktop edition which can import any hand-authored manifest version. See the desktop edition for that flow.</p>':''}
 </div>`;
 const exportBtn=$('#exportLocalWorkspace');
 if(exportBtn)exportBtn.onclick=()=>{downloadJson('local-workspace-export.lanesra.json',exportLocalWorkspace());toast('Local Workspace exported')};
}
// Named, scoped Solutions - see createSolution's own comment for the full
// design. This sub-tab lists every Solution with its curated component
// count; solutionModal handles create/rename; solutionDetailModal is
// where components actually get added/removed and where Export lives.
function solutionsListSubTab(body){
 const solutions=data.solutions||[];
 body.innerHTML=`<div style="margin-top:16px">
 <div class="panel-head" style="align-items:flex-start;gap:16px">
  <p class="muted" style="font-size:13px;margin:0;flex:1;min-width:0">A named, versioned subset of components you deliberately pick - not everything Local Workspace owns. Build one, add exactly the objects/fields/rules/workflows/screens it needs, then Export and import the file into another workspace's <b>Admin → App Catalog</b> to promote it there.</p>
  <button class="btn btn-primary" id="newSolution" style="flex-shrink:0">+ New solution</button>
 </div>
 ${solutions.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Version</th><th>Publisher</th><th>Components</th><th></th></tr></thead><tbody>${solutions.map(s=>{const publisher=data.publishers.find(p=>p.key===s.publisherKey);const count=s.memberIds.length;return `<tr><td>${s.name}</td><td>${s.version}</td><td>${publisher?publisher.name:'—'}</td><td>${count}</td><td><div class="actions"><button class="btn btn-secondary" data-open-solution="${s.id}" style="font-size:12px;padding:4px 8px">Open</button><button class="btn btn-secondary" data-export-solution="${s.id}" style="font-size:12px;padding:4px 8px" ${count?'':'disabled title="Add at least one component first"'}>Export</button><button class="btn btn-danger" data-del-solution="${s.id}" style="font-size:12px;padding:4px 8px">Delete</button></div></td></tr>`}).join('')}</tbody></table></div>`:'<div class="empty">No solutions yet. Create one to start picking exactly what ships to another workspace.</div>'}
 </div>`;
 $('#newSolution').onclick=()=>solutionModal();
 body.querySelectorAll('[data-open-solution]').forEach(b=>b.onclick=()=>solutionDetailModal(b.dataset.openSolution));
 body.querySelectorAll('[data-export-solution]').forEach(b=>b.onclick=()=>{
  const solution=data.solutions.find(s=>s.id===b.dataset.exportSolution); if(!solution)return;
  downloadJson(solutionFilename(solution.name),exportSolution(solution.id));
  toast(`${solution.name} exported`);
 });
 body.querySelectorAll('[data-del-solution]').forEach(b=>b.onclick=()=>{
  const solution=data.solutions.find(s=>s.id===b.dataset.delSolution); if(!solution)return;
  if(!confirm(`Delete solution "${solution.name}"? Its curated components themselves are not touched.`))return;
  deleteSolution(solution.id);toast('Solution deleted');renderSolutionsSubTab();
 });
}
function solutionModal(){
 const body=`<form id="solutionForm" class="form-grid">
 <div class="field"><label>Name</label><input name="name" placeholder="Field Ops Extensions" required></div>
 <div class="field full"><label>Description (optional)</label><input name="description"></div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Create solution</button></div>
 </form>`;
 modal('New solution',body);
 $('[data-close]').onclick=closeModal;
 $('#solutionForm').onsubmit=e=>{
  e.preventDefault();
  const fd=Object.fromEntries(new FormData(e.target).entries());
  const {error,solution}=createSolution(fd);
  if(error)return alert(error);
  closeModal();toast('Solution created');renderSolutionsSubTab();
  solutionDetailModal(solution.id);
 };
}
// The curation surface for one Solution: rename/re-describe/bump its
// version, see what's in it, remove a member, and add any other workspace
// component (hand-built or package-installed) that isn't in it yet.
function solutionDetailModal(solutionId){
 const solution=data.solutions.find(s=>s.id===solutionId); if(!solution)return;
 const members=solutionMembersResolved(solution);
 const memberKeys=new Set(members.map(c=>`${c.type}:${c.metadataId}`));
 const candidates=listSolutionComponents().filter(c=>!memberKeys.has(`${c.type}:${c.metadataId}`));
 const memberRow=c=>`<tr><td>${artifactTypeLabel(c.type)}</td><td>${c.publisherName}${c.isLocal?' <span class="badge">Local</span>':''}</td><td>${c.installedAppName||'Hand-built'}</td><td><button class="btn btn-secondary" data-remove-member="${c.type}|${c.metadataId}" style="font-size:12px;padding:2px 8px">Remove</button></td></tr>`;
 const candidateRow=c=>`<tr data-needle="${(artifactTypeLabel(c.type)+' '+c.publisherName+' '+(c.installedAppName||'')).toLowerCase()}"><td>${artifactTypeLabel(c.type)}</td><td>${c.publisherName}${c.isLocal?' <span class="badge">Local</span>':''}</td><td>${c.installedAppName||'Hand-built'}</td><td><button class="btn btn-primary" data-add-member="${c.type}|${c.metadataId}" style="font-size:12px;padding:2px 8px">+ Add</button></td></tr>`;
 const body=`
 <div style="display:flex;gap:8px;align-items:flex-end;flex-wrap:wrap;margin-bottom:16px">
  <div class="field" style="margin:0"><label style="font-size:12px">Name</label><input id="solutionEditName" value="${solution.name}" style="width:220px"></div>
  <div class="field" style="margin:0"><label style="font-size:12px">Version</label><input id="solutionEditVersion" value="${solution.version}" style="width:120px"></div>
  <button class="btn btn-secondary" id="saveSolutionMeta" style="font-size:12px;padding:6px 10px">Save</button>
 </div>
 <b style="font-size:13px">Curated components (${members.length})</b>
 ${members.length?`<div class="table-wrap"><table class="table" style="margin-top:8px;margin-bottom:0"><thead><tr><th>Type</th><th>Publisher</th><th>Source</th><th></th></tr></thead><tbody>${members.map(memberRow).join('')}</tbody></table></div>`:'<div class="empty">Nothing curated yet - add components below.</div>'}
 <div style="margin-top:16px"><b style="font-size:13px">Add a component</b>
 ${candidates.length?`<input id="solutionAddFilter" placeholder="Filter by type, publisher or app..." style="margin-top:6px;margin-bottom:8px;width:100%;max-width:320px">
 <div class="table-wrap" style="max-height:240px;overflow:auto"><table class="table" style="margin-bottom:0"><thead><tr><th>Type</th><th>Publisher</th><th>Source</th><th></th></tr></thead><tbody id="solutionAddRows">${candidates.map(candidateRow).join('')}</tbody></table></div>`:'<p class="empty" style="margin-top:6px">Every component in this workspace is already in this solution.</p>'}
 </div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Close</button></div>`;
 modal(`Solution: ${solution.name}`,body);
 $('[data-close]').onclick=closeModal;
 $('#saveSolutionMeta').onclick=()=>{
  const {error}=updateSolution(solution.id,{name:$('#solutionEditName').value,description:solution.description,version:$('#solutionEditVersion').value});
  if(error)return alert(error);
  toast('Solution saved');closeModal();renderSolutionsSubTab();solutionDetailModal(solution.id);
 };
 document.querySelectorAll('[data-remove-member]').forEach(b=>b.onclick=()=>{
  const [type,metadataId]=b.dataset.removeMember.split('|');
  removeSolutionMember(solution.id,type,metadataId);
  closeModal();renderSolutionsSubTab();solutionDetailModal(solution.id);
 });
 document.querySelectorAll('[data-add-member]').forEach(b=>b.onclick=()=>{
  const [type,metadataId]=b.dataset.addMember.split('|');
  addSolutionMember(solution.id,type,metadataId);
  closeModal();renderSolutionsSubTab();solutionDetailModal(solution.id);
 });
 const filterInput=$('#solutionAddFilter');
 if(filterInput)filterInput.oninput=()=>{
  const needle=filterInput.value.trim().toLowerCase();
  document.querySelectorAll('#solutionAddRows tr').forEach(tr=>{tr.hidden=needle&&!tr.dataset.needle.includes(needle)});
 };
}
function solutionComponentsSubTab(body){
 const all=listSolutionComponents();
 const byType={};
 all.forEach(a=>{byType[a.type]=(byType[a.type]||0)+1});
 body.innerHTML=`<div style="margin-top:16px">
 <p class="muted" style="font-size:13px">Every custom object, field, relationship, business rule, workflow, screen layout and report in this workspace - hand-built or installed by an app - and who owns it.</p>
 ${all.length?`<div style="display:flex;gap:12px;flex-wrap:wrap;margin-bottom:12px">${Object.entries(byType).map(([type,count])=>`<span class="badge">${count} ${artifactTypeLabel(type)}${count===1?'':'s'}</span>`).join('')}</div>
 <input id="solutionsComponentFilter" placeholder="Filter by publisher, app, type or id..." style="margin-bottom:8px;width:100%;max-width:320px">
 <div class="table-wrap"><table class="table"><thead><tr><th>Type</th><th>Publisher</th><th>Source</th></tr></thead><tbody id="solutionsComponentRows">${all.map(a=>`<tr data-needle="${(a.publisherName+' '+(a.installedAppName||'')+' '+artifactTypeLabel(a.type)+' '+a.metadataId).toLowerCase()}"><td>${artifactTypeLabel(a.type)}</td><td>${a.publisherName}${a.isLocal?' <span class="badge">Local</span>':''}</td><td>${a.installedAppName||'Hand-built'}</td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">Nothing here yet.</div>'}
 </div>`;
 const filterInput=$('#solutionsComponentFilter');
 if(filterInput)filterInput.oninput=()=>{
  const needle=filterInput.value.trim().toLowerCase();
  document.querySelectorAll('#solutionsComponentRows tr').forEach(tr=>{tr.hidden=needle&&!tr.dataset.needle.includes(needle)});
 };
}
function solutionDependenciesSubTab(body){
 // None of the bundled reference packages declare a dependency (same as
 // their desktop equivalents in reference_packages.rs - every manifest's
 // "dependencies" array is empty), so this is an honest empty state for
 // any workspace that's only ever installed starter packages.
 body.innerHTML=`<div style="margin-top:16px">
 <p class="muted" style="font-size:13px">Every dependency declared by a package imported into this workspace, and whether it's currently satisfied.</p>
 <div class="empty">No imported package declares a dependency.</div>
 </div>`;
}
function solutionPublishersSubTab(body){
 const publishers=data.publishers||[];
 body.innerHTML=`<div style="margin-top:16px">
 <div class="panel-head" style="align-items:flex-start;gap:16px"><p class="muted" style="font-size:13px;margin:0;flex:1;min-width:0">Who a package's namespace belongs to. Every package_id is expected to be "&lt;publisher-key&gt;.&lt;name&gt;" - importing a package under an unregistered key is rejected until its publisher is registered here.</p><button class="btn btn-primary" id="addPublisher" style="flex-shrink:0">+ Register publisher</button></div>
 <div class="table-wrap"><table class="table"><thead><tr><th>Key</th><th>Name</th><th>Description</th><th>Packages</th><th></th></tr></thead><tbody>${publishers.map(p=>{const packageCount=[...(data.appPackages||[]),...(data.installedApps||[])].filter(pk=>publisherKeyFromPackageId(pk.packageId)===p.key).length;return `<tr><td><code>${p.key}</code></td><td>${p.name}</td><td class="muted">${p.description||'—'}</td><td>${packageCount}</td><td>${p.isOfficial?'<span class="badge">Official</span>':''}${p.isLocal?'<span class="badge">Local</span>':''}</td></tr>`}).join('')}</tbody></table></div>
 </div>`;
 $('#addPublisher').onclick=()=>publisherModal();
}
function publisherModal(){
 const body=`<form id="publisherForm" class="form-grid">
 <div class="field"><label>Key</label><input name="key" placeholder="acme" required></div>
 <div class="field"><label>Name</label><input name="name" placeholder="Acme Corp" required></div>
 <div class="field full"><label>Description (optional)</label><input name="description"></div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Register publisher</button></div>
 </form>`;
 modal('Register publisher',body);
 $('[data-close]').onclick=closeModal;
 $('#publisherForm').onsubmit=e=>{
  e.preventDefault();
  const fd=Object.fromEntries(new FormData(e.target).entries());
  const {error}=createPublisher(fd);
  if(error)return alert(error);
  closeModal();toast('Publisher registered');renderAdminTab();
 };
}
// ---- Integration Hub (UI-only simulation) ---------------------------------
// Relabeled to match the real Integration Hub the desktop app now ships
// (Connections, Connection References, Connectors, API Access, Webhooks &
// Events, Data Exchange, External Objects, Integration Jobs, Logs &
// Monitoring, Settings - see desktop's IntegrationHubAdmin.tsx) - three
// representative areas (Connections, API Access, Integration Jobs), not
// the full ten-screen set. The static demo has no server, so nothing here
// makes a real network call, encrypts a secret, signs a webhook, or runs
// on a real schedule - every "run"/"test" is a local simulation against
// this browser's own demo data, saved to localStorage like everything
// else, matching the honesty level the rest of this demo already holds to.
let integrationsSubTab='jobs';
const JOB_TYPES=['export','import','sync'];
const JOB_TYPE_LABELS={export:'Export data out',import:'Import data in',sync:'Two-way sync'};
const SCHEDULE_OPTIONS=['manual','hourly','daily','weekly'];
const SCHEDULE_LABELS={manual:'Manual only',hourly:'Every hour',daily:'Once a day',weekly:'Once a week'};
const FORMAT_OPTIONS=['csv','json'];
const EXTERNAL_AUTH_TYPES=['none','apiKey','bearer'];
const EXTERNAL_AUTH_LABELS={none:'None',apiKey:'API key',bearer:'Bearer token'};
const WEBHOOK_EVENT_TYPES=['record.created','record.updated','record.archived','workflow.completed'];
function integrationsTab(body){
 const subTabs=[['overview','Overview'],['jobs','Integration Jobs'],['endpoints','API Access'],['external','Connections'],['webhooks','Webhooks & Events']];
 body.innerHTML=`<div class="panel">
 <h3 style="margin-top:0">Integration Hub</h3>
 <p class="muted" style="font-size:13px">Schedule recurring data jobs, issue API access for other systems to call in, configure outbound connections this workspace would use, and subscribe other systems to what changes here. This is a UI-only simulation: everything is saved to this browser and "runs"/"test calls" produce a realistic result against your demo data, but no real network request, encryption, signature or scheduled job ever actually fires — there's no server behind the online demo to run one. The desktop app's Admin → Integration Hub is the real thing: encrypted Connections, OpenAPI Connectors, hashed API clients, HMAC-signed Webhooks, a generic CSV wizard, External Objects and a real background scheduler.</p>
 <div class="tabs">${subTabs.map(t=>`<button class="tab ${integrationsSubTab===t[0]?'active':''}" data-integrations-tab="${t[0]}">${t[1]}</button>`).join('')}</div>
 <div id="integrationsBody"></div>
 </div>`;
 document.querySelectorAll('[data-integrations-tab]').forEach(b=>b.onclick=()=>{integrationsSubTab=b.dataset.integrationsTab;renderIntegrationsSubTab()});
 renderIntegrationsSubTab();
}
function renderIntegrationsSubTab(){
 document.querySelectorAll('[data-integrations-tab]').forEach(b=>b.classList.toggle('active',b.dataset.integrationsTab===integrationsSubTab));
 const body=$('#integrationsBody');
 ({overview:integrationOverviewSubTab,jobs:jobsSubTab,endpoints:endpointsSubTab,external:externalSubTab,webhooks:webhooksSubTab}[integrationsSubTab])(body);
}
// Real counts over this browser's own demo data - not fabricated numbers -
// the same "compute from what's actually here" convention the Overview
// KPIs use on desktop (integration_log_service::overview).
function integrationOverviewSubTab(body){
 const jobs=data.integrationJobs||[], endpoints=data.apiEndpoints||[], connections=data.externalConnections||[], hooks=data.webhooks||[];
 const activeJobs=jobs.filter(j=>j.active).length;
 const activeEndpoints=endpoints.filter(e=>e.active).length;
 const activeConnections=connections.filter(c=>c.active).length;
 const activeHooks=hooks.filter(w=>w.active).length;
 const totalRuns=jobs.reduce((n,j)=>n+(j.runs||[]).length,0);
 const totalDeliveries=hooks.reduce((n,w)=>n+(w.deliveries||[]).length,0);
 const totalCalls=endpoints.length?connections.reduce((n,c)=>n+(c.calls||[]).length,0):0;
 const kpi=(label,value)=>`<div class="kpi"><div class="kpi-value">${value}</div><div class="kpi-label">${label}</div></div>`;
 body.innerHTML=`<div style="margin-top:16px">
 <div class="kpi-grid" style="margin-bottom:20px">${[
   kpi('Integration Jobs',`${activeJobs} / ${jobs.length} active`),
   kpi('API endpoints',`${activeEndpoints} / ${endpoints.length} active`),
   kpi('Connections',`${activeConnections} / ${connections.length} active`),
   kpi('Webhook subscriptions',`${activeHooks} / ${hooks.length} active`),
   kpi('Simulated job runs',totalRuns),
   kpi('Simulated webhook deliveries',totalDeliveries),
 ].join('')}</div>
 <p class="muted" style="font-size:13px">These counts come from what's actually configured in this browser's demo data, the same "compute it, don't fake it" rule every other number in this demo follows - they just don't reflect a real API call, encrypted secret or network delivery, because this static demo has no server behind it to make one. Get started: <button class="link-btn" data-integrations-tab="jobs">create a job</button>, <button class="link-btn" data-integrations-tab="endpoints">issue API access</button>, <button class="link-btn" data-integrations-tab="external">add a connection</button>, or <button class="link-btn" data-integrations-tab="webhooks">subscribe a webhook</button>.</p>
 </div>`;
 body.querySelectorAll('[data-integrations-tab]').forEach(b=>b.onclick=()=>{integrationsSubTab=b.dataset.integrationsTab;renderIntegrationsSubTab()});
}
function jobsSubTab(body){
 const list=data.integrationJobs||[];
 body.innerHTML=`<div class="panel-head" style="margin-top:16px"><h3 style="margin:0;font-size:16px">Integration Jobs</h3><button class="btn btn-primary" id="addJob">+ New job</button></div>${list.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Type</th><th>Entity</th><th>Schedule</th><th>Format</th><th>Status</th><th>Last run</th><th>Actions</th></tr></thead><tbody>${list.map(j=>`<tr><td>${j.name}</td><td>${JOB_TYPE_LABELS[j.type]}</td><td>${entityLabel(j.entityKey)}</td><td>${SCHEDULE_LABELS[j.schedule]}</td><td>${j.format.toUpperCase()}</td><td>${badgeMaybe(j.active?'Active':'Inactive')}</td><td>${j.lastRun?new Date(j.lastRun).toLocaleString():'Never run'}</td><td><div class="actions"><button class="icon-btn" data-run-job="${j.id}">Run now</button><button class="icon-btn" data-history-job="${j.id}">History</button><button class="icon-btn" data-edit-job="${j.id}">Edit</button><button class="icon-btn" data-del-job="${j.id}">Delete</button></div></td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No scheduled jobs yet.</div>'}`;
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
 body.innerHTML=`<div class="panel-head" style="margin-top:16px"><h3 style="margin:0;font-size:16px">API Access</h3><button class="btn btn-primary" id="addEndpoint">+ New endpoint</button></div><p class="muted" style="font-size:13px">Expose a read/write endpoint backed by a built-in or custom object. Test call simulates the request/response locally against your demo data — nothing actually leaves your browser. On desktop, API Access issues a real <code>{client_id}.{secret}</code> bearer key (shown once, hashed at rest) against a real generic REST API.</p>${list.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Method</th><th>Path</th><th>Entity</th><th>Auth</th><th>Status</th><th>Actions</th></tr></thead><tbody>${list.map(e=>`<tr><td>${e.name}</td><td><code>${e.method}</code></td><td><code>${e.path}</code></td><td>${entityLabel(e.entityKey)}</td><td>${e.authType==='apiKey'?'API key':'None'}</td><td>${badgeMaybe(e.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-test-endpoint="${e.id}">Test call</button><button class="icon-btn" data-edit-endpoint="${e.id}">Edit</button><button class="icon-btn" data-del-endpoint="${e.id}">Delete</button></div></td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No API endpoints yet.</div>'}`;
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
 body.innerHTML=`<div class="panel-head" style="margin-top:16px"><h3 style="margin:0;font-size:16px">Connections</h3><button class="btn btn-primary" id="addConnection">+ New connection</button></div><p class="muted" style="font-size:13px">Configure an external API this workspace would call. Test request simulates a response shape locally — the online demo can't make outbound network calls, so nothing is actually sent. On desktop, a Connection's secret is encrypted at rest (AES-256-GCM) and a real Test Connection call is made.</p>${list.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Method</th><th>Base URL</th><th>Auth</th><th>Status</th><th>Actions</th></tr></thead><tbody>${list.map(c=>`<tr><td>${c.name}</td><td><code>${c.method}</code></td><td style="max-width:260px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap"><code>${c.baseUrl}</code></td><td>${EXTERNAL_AUTH_LABELS[c.authType]}</td><td>${badgeMaybe(c.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-test-conn="${c.id}">Test request</button><button class="icon-btn" data-history-conn="${c.id}">History</button><button class="icon-btn" data-edit-conn="${c.id}">Edit</button><button class="icon-btn" data-del-conn="${c.id}">Delete</button></div></td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No external connections yet.</div>'}`;
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
// Webhooks & Events - the outbound half of Integration Hub's simulation,
// alongside the inbound API Access tab. Real desktop webhooks are
// HMAC-SHA256 signed and retried with exponential backoff (webhook_service);
// here, "Send test event" simulates one delivery against whichever record
// this browser's demo data actually has, same honesty rule as every other
// Integration Hub sub-tab - no real signature, no real HTTP request.
function webhooksSubTab(body){
 const list=data.webhooks||[];
 body.innerHTML=`<div class="panel-head" style="margin-top:16px"><h3 style="margin:0;font-size:16px">Webhooks & Events</h3><button class="btn btn-primary" id="addWebhook">+ New webhook</button></div><p class="muted" style="font-size:13px">Subscribe an external system to what changes in this workspace. Send test event simulates one signed delivery locally against your demo data — nothing actually leaves your browser. On desktop, every delivery is HMAC-SHA256 signed and retried with exponential backoff on failure, with a real delivery history per webhook.</p>${list.length?`<div class="table-wrap"><table class="table"><thead><tr><th>Name</th><th>Target URL</th><th>Events</th><th>Status</th><th>Actions</th></tr></thead><tbody>${list.map(w=>`<tr><td>${w.name}</td><td style="max-width:220px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap"><code>${w.targetUrl}</code></td><td>${w.eventTypes.map(e=>`<span class="badge" style="margin-right:4px">${e}</span>`).join('')}</td><td>${badgeMaybe(w.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-test-hook="${w.id}">Send test event</button><button class="icon-btn" data-history-hook="${w.id}">Deliveries</button><button class="icon-btn" data-edit-hook="${w.id}">Edit</button><button class="icon-btn" data-del-hook="${w.id}">Delete</button></div></td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No webhooks yet.</div>'}`;
 $('#addWebhook').onclick=()=>webhookModal();
 body.querySelectorAll('[data-test-hook]').forEach(b=>b.onclick=()=>sendTestWebhookEvent(b.dataset.testHook));
 body.querySelectorAll('[data-history-hook]').forEach(b=>b.onclick=()=>webhookHistoryModal(b.dataset.historyHook));
 body.querySelectorAll('[data-edit-hook]').forEach(b=>b.onclick=()=>webhookModal(list.find(w=>w.id===b.dataset.editHook)));
 body.querySelectorAll('[data-del-hook]').forEach(b=>b.onclick=()=>{if(!confirm('Delete this webhook? Its delivery history goes with it.'))return;data.webhooks=data.webhooks.filter(w=>w.id!==b.dataset.delHook);save();webhooksSubTab(body)});
}
function webhookModal(hook){
 const isEdit=!!hook;
 const selected=hook?.eventTypes||['record.created'];
 const body=`<form id="hookForm" class="form-grid">
 <div class="field full"><label>Webhook name</label><input name="name" value="${hook?.name||''}" required></div>
 <div class="field full"><label>Target URL</label><input name="targetUrl" type="url" value="${hook?.targetUrl||''}" placeholder="https://example.com/webhooks/lanesra" required></div>
 <div class="field full"><label>Events</label>${WEBHOOK_EVENT_TYPES.map(e=>`<label class="checkbox-row" style="padding:0"><input type="checkbox" name="eventTypes" value="${e}" ${selected.includes(e)?'checked':''}> ${e}</label>`).join('')}</div>
 ${isEdit?`<div class="field"><label>Status</label><select name="active"><option value="true" ${hook.active?'selected':''}>Active</option><option value="false" ${!hook.active?'selected':''}>Inactive</option></select></div>`:''}
 <div class="modal-actions">${isEdit?'<button type="button" class="btn btn-secondary" data-delete-hook>Delete</button>':''}<button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">${isEdit?'Save webhook':'Create webhook'}</button></div>
 </form>`;
 modal(isEdit?`Edit webhook: ${hook.name}`:'New webhook',body);
 $('[data-close]').onclick=closeModal;
 $('#hookForm').onsubmit=e=>{
  e.preventDefault();
  const fd=new FormData(e.target);
  const eventTypes=fd.getAll('eventTypes');
  if(eventTypes.length===0)return alert('Pick at least one event type.');
  const targetUrl=fd.get('targetUrl');
  if(isEdit){Object.assign(hook,{name:fd.get('name'),targetUrl,eventTypes,active:fd.get('active')==='true'})}
  else{data.webhooks.push({id:uid(),name:fd.get('name'),targetUrl,eventTypes,active:true,deliveries:[]})}
  save();closeModal();toast(isEdit?'Webhook saved':'Webhook created');renderAdminTab();
 };
 if(isEdit){$('[data-delete-hook]').onclick=()=>{if(!confirm('Delete this webhook? Its delivery history goes with it.'))return;data.webhooks=data.webhooks.filter(w=>w.id!==hook.id);save();closeModal();toast('Webhook deleted');renderAdminTab()}}
}
// Simulated delivery: picks a real record of a type this event touches (or
// falls back to a generic payload if none exist yet) so the JSON preview
// looks like a genuine payload rather than a placeholder - HMAC signing and
// the actual POST are exactly what's not simulated, since there is nothing
// real to sign or send from a static site.
function sendTestWebhookEvent(id){
 const hook=(data.webhooks||[]).find(w=>w.id===id); if(!hook)return;
 const eventType=hook.eventTypes[Math.floor(Math.random()*hook.eventTypes.length)];
 const candidateKeys=allEntityTypeKeys().filter(k=>(data[k]||[]).length>0);
 const entityKey=candidateKeys[Math.floor(Math.random()*candidateKeys.length)]||'companies';
 const record=(data[entityKey]||[])[0];
 const occurredAt=new Date().toISOString();
 const payload={event_id:'evt_'+uid(),event_type:eventType,object_key:entityKey,record_id:record?.id||'sim_'+uid(),occurred_at:occurredAt,payload:record?{id:record.id,name:record.name||record.subject||record.title||undefined}:{note:'no records yet'}};
 const delivery={id:uid(),deliveredAt:occurredAt,eventType,httpStatus:200,payload};
 hook.deliveries.unshift(delivery); if(hook.deliveries.length>10)hook.deliveries.length=10;
 save();
 const body=`<p class="muted" style="font-size:13px">Simulated locally — on desktop this payload is signed with <code>X-Lanesra-Signature: sha256=...</code> (HMAC-SHA256 over this exact body using the webhook's own secret) and POSTed to the target URL for real, with retry on failure. Nothing was actually sent here.</p>
 <div><strong>Payload</strong> <span class="badge badge-success">200 OK (simulated)</span><pre style="background:#0f172a;color:#e2e8f0;border-radius:10px;padding:12px;overflow:auto;font-size:12px;max-height:280px">${JSON.stringify(payload,null,2)}</pre></div>
 <div class="modal-actions"><button class="btn btn-secondary" type="button" data-close>Close</button></div>`;
 modal(`Test event: ${hook.name}`,body);
 $('[data-close]').onclick=closeModal;
}
function webhookHistoryModal(id){
 const hook=(data.webhooks||[]).find(w=>w.id===id); if(!hook)return;
 const body=`${hook.deliveries.length?`<div class="table-wrap"><table class="table"><thead><tr><th>When</th><th>Event</th><th>Status</th></tr></thead><tbody>${hook.deliveries.map(d=>`<tr><td>${new Date(d.deliveredAt).toLocaleString()}</td><td>${d.eventType}</td><td><span class="badge">${d.httpStatus} OK</span></td></tr>`).join('')}</tbody></table></div>`:'<div class="empty">No deliveries yet — click Send test event to simulate one.</div>'}<div class="modal-actions"><button class="btn btn-secondary" type="button" data-close>Close</button></div>`;
 modal(`Deliveries: ${hook.name}`,body);
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
// Admin UX polish (spec §10): a dependency warning before deactivating a
// custom field an active business rule or workflow rule still reads
// (condition fieldKey/compareField) or writes (action targetField) - the
// concrete case where deactivating something silently breaks another
// admin-configured thing, since a rule/workflow referencing a deactivated
// field just stops finding it and never fires that clause again.
function fieldDependents(entityKey,fieldKey){
 const usesField=conditions=>(conditions||[]).some(c=>c.fieldKey===fieldKey||c.compareField===fieldKey);
 const rules=(data.fieldRules||[]).filter(r=>r.entity===entityKey&&r.active&&(usesField(r.conditions)||(r.actions||[]).some(a=>a.targetField===fieldKey)))
  .map(r=>`Business rule — IF ${describeConditions(entityKey,r.conditions,r.matchType||'all')} THEN ${(r.actions||[]).map(a=>describeRuleAction(entityKey,a)).join('; ')}`);
 const workflows=(data.workflowRules||[]).filter(r=>r.entity===entityKey&&r.active&&(usesField(r.conditions)||(r.actions||[]).some(a=>a.targetField===fieldKey)))
  .map(r=>`Workflow rule — WHEN ${describeConditions(entityKey,r.conditions,r.matchType||'all')} THEN ${(r.actions||[]).map(a=>describeWorkflowAction(a,entityKey)).join('; ')}`);
 return [...rules,...workflows];
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
 ${isEdit?`<div class="field full">${auditByline(f)}</div>`:''}
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
  if(isEdit&&field.active&&fd.active==='false'){
   const dependents=fieldDependents(field.entity,field.key);
   if(dependents.length&&!confirm(`${dependents.length} active ${dependents.length===1?'rule references':'rules reference'} this field:\n\n${dependents.map(d=>'• '+d).join('\n')}\n\nDeactivating it will stop those matching on this field. Deactivate anyway?`))return;
  }
  const shared={label:fd.label,type:fd.type,options:fd.type==='select'?fd.options:'',active:fd.active==='true',defaultValue:fd.defaultValue||'',placeholder:fd.placeholder||'',helpText:fd.helpText||'',unique:fd.unique==='true',
   required:fd.required==='true',
   maxLength:fd.type==='text'&&fd.maxLength?Number(fd.maxLength):null,
   pattern:fd.type==='text'?(fd.pattern||''):'',
   minValue:fd.type==='number'&&fd.minValue!==''?Number(fd.minValue):'',
   maxValue:fd.type==='number'&&fd.maxValue!==''?Number(fd.maxValue):'',
   searchable:fd.searchable==='true',filterable:fd.filterable==='true',reportable:fd.reportable==='true',
   hiddenByDefault:fd.hiddenByDefault==='true',
  };
  if(isEdit){Object.assign(field,shared);stampUpdate(field)}
  else{const newField=stampCreate({id:uid(),entity:cfEntity,key:slugify(fd.label),...shared});data.customFields.push(newField);tagLocalComponent('customField',newField.id)}
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
// Admin UX polish (spec §10): a bounded version history on business rules
// and workflow rules - pushRuleHistory snapshots a rule's current state
// (everything except its own history array, to avoid nesting) right before
// an edit overwrites it, capped at RULE_HISTORY_LIMIT so this can't grow
// unbounded in localStorage. ruleHistoryModal shows those snapshots newest
// first with a Restore action; restoring itself snapshots the
// about-to-be-replaced state first, so a restore is never a dead end.
const RULE_HISTORY_LIMIT=10;
function pushRuleHistory(existing){
 if(!existing)return;
 const snapshot=JSON.parse(JSON.stringify(existing)); delete snapshot.history;
 existing.history=[{...snapshot,savedAt:new Date().toISOString()},...(existing.history||[])].slice(0,RULE_HISTORY_LIMIT);
}
function ruleHistoryModal(collectionKey,ruleId,entityKey,describeAction){
 const list=collectionKey==='fieldRules'?data.fieldRules:data.workflowRules;
 const rule=list.find(r=>r.id===ruleId); if(!rule)return;
 const history=rule.history||[];
 const bodyHtml=history.length
  ?`<div class="history-list">${history.map((h,i)=>`<div class="history-item"><div class="history-meta"><strong>${new Date(h.savedAt).toLocaleString()}</strong> ${badgeMaybe(h.active?'Active':'Inactive')}</div><div class="muted" style="font-size:13px;margin:6px 0">${describeConditions(entityKey,h.conditions,h.matchType||'all')}${(h.actions||[]).length?' → '+h.actions.map(a=>describeAction(entityKey,a)).join('; '):''}</div><button class="btn btn-secondary" data-restore-history="${i}">Restore this version</button></div>`).join('')}</div>`
  :'<div class="empty">No saved versions yet — history starts recording from the next edit.</div>';
 modal('Version history',bodyHtml);
 document.querySelectorAll('[data-restore-history]').forEach(b=>b.onclick=()=>{
  const snap=history[Number(b.dataset.restoreHistory)]; if(!snap)return;
  pushRuleHistory(rule);
  Object.assign(rule,{conditions:snap.conditions,matchType:snap.matchType,actions:snap.actions,active:snap.active,notify:snap.notify,conditionsMerged:snap.conditionsMerged});
  save(); closeModal(); toast('Version restored'); renderAdminTab();
 });
}
function rulesTab(body){
 if(ruleBuilderMode){renderRuleBuilder(body);return}
 const keys=[...Object.keys(numberRules),...activeCustomObjectKeys()];
 const actionFields=actionableFieldsFor(ruleEntity);
 const list=(data.fieldRules||[]).filter(r=>r.entity===ruleEntity&&matchesAppFilter(r.appId,ruleAppFilter));
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Business rules</h3><p class="muted">Build an IF (AND/OR conditions, with one level of OR-groups) / THEN (any number of actions) rule against any built-in or custom field.</p>
 ${entityPills(keys,ruleEntity)}
 ${appFilterPills(ruleAppFilter)}
 ${actionFields.length?`<div class="table-wrap"><table class="table"><thead><tr><th>If</th><th>Then</th><th>App</th><th>Active</th><th>Actions</th></tr></thead><tbody>${list.map(r=>`<tr><td>${describeConditions(r.entity,r.conditions,r.matchType||'all')}</td><td>${(r.actions||[]).map(a=>describeRuleAction(r.entity,a)).join('; ')}</td><td>${appNameFor(r.appId)||'<span class="muted">Workspace-wide</span>'}</td><td>${badgeMaybe(r.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-edit-rule="${r.id}">Edit</button><button class="icon-btn" data-dup-rule="${r.id}" title="Duplicate">Duplicate</button><button class="icon-btn" data-history-rule="${r.id}" title="Version history">History</button><button class="icon-btn" data-del-rule="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No business rules on '+entityLabel(ruleEntity)+' match this filter</div>'}</div><button class="btn btn-secondary" id="addRule" style="margin-top:14px">+ New rule</button>`:`<div class="empty">${entityLabel(ruleEntity)} has no field a rule can act on yet.</div>`}
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{ruleEntity=b.dataset.entity;renderAdminTab()});
 wireAppFilterPills(body,()=>ruleAppFilter,v=>ruleAppFilter=v,()=>renderAdminTab());
 $('#addRule')?.addEventListener('click',()=>{ruleBuilderMode='create';renderAdminTab()});
 body.querySelectorAll('[data-edit-rule]').forEach(b=>b.onclick=()=>{ruleBuilderMode=b.dataset.editRule;renderAdminTab()});
 body.querySelectorAll('[data-dup-rule]').forEach(b=>b.onclick=()=>{
  const src=data.fieldRules.find(r=>r.id===b.dataset.dupRule); if(!src)return;
  const copy=JSON.parse(JSON.stringify(src)); copy.id=uid(); copy.active=false; delete copy.history; stampCreate(copy);
  data.fieldRules.push(copy); save(); toast('Rule duplicated as inactive — review and activate when ready');
  ruleBuilderMode=copy.id; renderAdminTab();
 });
 body.querySelectorAll('[data-history-rule]').forEach(b=>b.onclick=()=>ruleHistoryModal('fieldRules',b.dataset.historyRule,ruleEntity,describeRuleAction));
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
 const initialAppId=isEdit?(existing.appId||null):defaultAppIdFor(ruleAppFilter);
 const initialConditions=existing?.conditions?.length?existing.conditions:[{fieldKey:transitionFieldFor(entityKey),operator:'equals',value:'',compareField:null,groupId:null}];
 const initialActions=existing?.actions?.length?existing.actions:[{type:'require',targetField:actionFields[0]?.[0]||'',value:'',message:''}];
 body.innerHTML=`<div class="builder-header">
  <div>
   <div class="builder-breadcrumb">Business Rules / ${isEdit?'Edit rule':'New rule'}</div>
   <div class="builder-title-row"><h2>${isEdit?'Edit business rule':'New business rule'}</h2>${isEdit?`<span class="badge" style="${existing.active?'background:#dcfce7;color:#166534':''}">${existing.active?'Active':'Inactive'}</span>`:''}</div>
   <p class="builder-subtitle">Applies to ${entityLabel(entityKey)}.</p>
   ${appSelectHtml('ruleBuilderApp',initialAppId)}
   ${isEdit?auditByline(existing):''}
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
  const appId=$('#ruleBuilderApp')?.value||null;
  const payload={entity:entityKey,matchType,conditions,actions,appId};
  if(isEdit){pushRuleHistory(existing);Object.assign(existing,payload);stampUpdate(existing)}else{const newRule=stampCreate({id:uid(),active:true,...payload});data.fieldRules.push(newRule);tagLocalComponent('businessRule',newRule.id)}
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
// Every other entity type reachable from `entityKey` through an active,
// admin-defined Custom Relationship (data.relationshipDefinitions) - in
// either direction, since update_related_record doesn't care which side
// is "source". This is the Custom Relationships equivalent of RELATIONS'
// hardcoded built-in foreign-key graph below - added for the Industry
// Data Model reference packages, whose objects are entirely custom and so
// have no entry in RELATIONS at all.
function customRelationTargetsFor(entityKey){
 return (data.relationshipDefinitions||[]).filter(d=>d.active&&(d.sourceEntity===entityKey||d.targetEntity===entityKey))
  .map(d=>d.sourceEntity===entityKey?d.targetEntity:d.sourceEntity);
}
function relTargetsFor(entityKey){return [...new Set([...(RELATIONS[entityKey]||[]).map(x=>x.target),...customRelationTargetsFor(entityKey)])]}
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
  if(rel){
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
   if(!linked.length||!a.relTargetField)return null;
   linked.forEach(x=>{x[a.relTargetField]=a.relValue});
   return `set ${fieldLabelFor(targetEntity,a.relTargetField)} = "${a.relValue}" on ${linked.length} related ${entityLabel(targetEntity).toLowerCase()}`;
  }
  // Not in the static built-in graph - fall back to an active admin-defined
  // Custom Relationship instead (see customRelationTargetsFor above),
  // resolved through data.relationshipInstances exactly the way
  // relatedRecordsFor() reads a record's related list, from whichever side
  // `key` sits on. This is what makes update_related_record usable for the
  // Industry Data Model reference packages, whose objects are entirely
  // custom and never appear in RELATIONS.
  const def=(data.relationshipDefinitions||[]).find(d=>d.active&&((d.sourceEntity===key&&d.targetEntity===a.relTargetEntity)||(d.targetEntity===key&&d.sourceEntity===a.relTargetEntity)));
  if(!def||!a.relTargetField)return null;
  const linked=def.sourceEntity===key
   ?(data.relationshipInstances||[]).filter(i=>i.definitionId===def.id&&i.sourceId===record.id).map(i=>byId(i.targetEntity,i.targetId)).filter(Boolean)
   :(data.relationshipInstances||[]).filter(i=>i.definitionId===def.id&&i.targetId===record.id).map(i=>byId(i.sourceEntity,i.sourceId)).filter(Boolean);
  if(!linked.length)return null;
  linked.forEach(x=>{x[a.relTargetField]=a.relValue});
  return `set ${fieldLabelFor(a.relTargetEntity,a.relTargetField)} = "${a.relValue}" on ${linked.length} related ${entityLabel(a.relTargetEntity).toLowerCase()}`;
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
 const list=(data.workflowRules||[]).filter(r=>r.entity===wfEntity&&matchesAppFilter(r.appId,wfAppFilter));
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Workflow automation</h3><p class="muted">Trigger any number of actions - create a task, create a new record, update a related record, or update/default/clear a field on this record - when a saved record's changed fields match a set of AND/OR conditions (with one level of OR-groups).</p>
 ${entityPills(keys,wfEntity)}
 ${appFilterPills(wfAppFilter)}
 <div class="table-wrap"><table class="table"><thead><tr><th>When</th><th>Then</th><th>App</th><th>Notifies admins</th><th>Active</th><th>Actions</th></tr></thead><tbody>${list.map(r=>`<tr><td>${describeConditions(r.entity,r.conditions,r.matchType||'all')}</td><td>${(r.actions||[]).map(a=>describeWorkflowAction(a,r.entity)).join('; ')}</td><td>${appNameFor(r.appId)||'<span class="muted">Workspace-wide</span>'}</td><td>${r.notify?'Yes':'No'}</td><td>${badgeMaybe(r.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-edit-wf="${r.id}">Edit</button><button class="icon-btn" data-dup-wf="${r.id}" title="Duplicate">Duplicate</button><button class="icon-btn" data-history-wf="${r.id}" title="Version history">History</button><button class="icon-btn" data-del-wf="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No workflow rules on '+entityLabel(wfEntity)+' match this filter</div>'}</div>
 <button class="btn btn-secondary" id="addWf" style="margin-top:14px">+ New workflow rule</button>
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{wfEntity=b.dataset.entity;renderAdminTab()});
 wireAppFilterPills(body,()=>wfAppFilter,v=>wfAppFilter=v,()=>renderAdminTab());
 $('#addWf').onclick=()=>{wfBuilderMode='create';renderAdminTab()};
 body.querySelectorAll('[data-edit-wf]').forEach(b=>b.onclick=()=>{wfBuilderMode=b.dataset.editWf;renderAdminTab()});
 body.querySelectorAll('[data-dup-wf]').forEach(b=>b.onclick=()=>{
  const src=data.workflowRules.find(r=>r.id===b.dataset.dupWf); if(!src)return;
  const copy=JSON.parse(JSON.stringify(src)); copy.id=uid(); copy.active=false; delete copy.history; stampCreate(copy);
  data.workflowRules.push(copy); save(); toast('Workflow rule duplicated as inactive — review and activate when ready');
  wfBuilderMode=copy.id; renderAdminTab();
 });
 body.querySelectorAll('[data-history-wf]').forEach(b=>b.onclick=()=>ruleHistoryModal('workflowRules',b.dataset.historyWf,wfEntity,(entityKey,a)=>describeWorkflowAction(a,entityKey)));
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
 const initialAppId=isEdit?(existing.appId||null):defaultAppIdFor(wfAppFilter);
 const initialConditions=existing?.conditions?.length?existing.conditions:[{fieldKey:transitionFieldFor(entityKey),operator:'equals',value:'',compareField:null,groupId:null}];
 const initialActions=existing?.actions?.length?existing.actions:[emptyWorkflowAction('create_task',entityKey,recordTargets,relTargets)];
 body.innerHTML=`<div class="builder-header">
  <div>
   <div class="builder-breadcrumb">Workflow Automation / ${isEdit?'Edit workflow':'New workflow'}</div>
   <div class="builder-title-row"><h2>${isEdit?'Edit workflow rule':'New workflow rule'}</h2>${isEdit?`<span class="badge" style="${existing.active?'background:#dcfce7;color:#166534':''}">${existing.active?'Active':'Inactive'}</span>`:''}</div>
   <p class="builder-subtitle">Applies to ${entityLabel(entityKey)}.</p>
   ${appSelectHtml('wfBuilderApp',initialAppId)}
   ${isEdit?auditByline(existing):''}
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
  const appId=$('#wfBuilderApp')?.value||null;
  const payload={entity:entityKey,notify:fd.notify==='true',conditions,matchType,actions,conditionsMerged:true,appId};
  if(isEdit){pushRuleHistory(existing);Object.assign(existing,payload);stampUpdate(existing)}else{const newWf=stampCreate({id:uid(),active:true,...payload});data.workflowRules.push(newWf);tagLocalComponent('workflow',newWf.id)}
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
 body.querySelectorAll('[data-toggle-tr]').forEach(b=>b.onclick=()=>{const r=data.statusTransitionRules.find(x=>x.id===b.dataset.toggleTr);r.active=!r.active;stampUpdate(r);save();toast(r.active?'Rule activated':'Rule deactivated');renderAdminTab()});
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
  data.statusTransitionRules.push(stampCreate({id:uid(),entity:entityKey,active:true,from:fd.from||'',to:fd.to}));
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
 ${o?`<div class="field full">${auditByline(o)}</div>`:''}
 <div class="field full"><label>Prefix (include any punctuation, e.g. "ACC-" or "ACC-ab")</label><input name="prefix" value="${o?o.prefix:base.prefix+(base.year?'-'+year()+'-':'-')}" required></div>
 <div class="field"><label>Digits</label><input name="width" type="number" min="1" max="10" value="${o?o.width||base.width:base.width}"></div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Save format</button></div>
 </form>`;
 modal(`Numbering format — ${entityLabel(key)}`,body);
 $('[data-close]').onclick=closeModal;
 $('#numForm').onsubmit=e=>{e.preventDefault();const fd=Object.fromEntries(new FormData(e.target).entries());if(!fd.prefix.trim())return alert('Enter a prefix.');
  // Upsert semantics (mirrors numbering_override_repo::upsert): on an
  // existing override, created_by is preserved and only updated_by moves;
  // a brand-new override gets both set to the same actor.
  const existing=data.numberingOverrides[key];
  const rec={prefix:fd.prefix.trim(),width:Math.min(10,Math.max(1,Number(fd.width||base.width)))};
  if(existing){Object.assign(existing,rec);stampUpdate(existing)}else{data.numberingOverrides[key]=stampCreate(rec)}
  save();closeModal();toast('Numbering format updated');renderView()};
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

function publicNav(){return `<nav class="landing-nav"><div class="container nav-inner"><a class="brand" href="/"><span class="brand-mark">L</span>Lanesra OS</a><div class="nav-links"><a href="/platform">Platform</a><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a></div><div class="nav-actions"><a class="btn btn-primary mobile-try" href="/demo">Try Online →</a><button class="menu-toggle" aria-label="Open navigation" aria-expanded="false">☰</button></div></div><div class="mobile-drawer" hidden><a href="/platform">Platform</a><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a><hr><a href="/roadmap">Roadmap & Backlog</a><a href="/releases">Releases</a><a href="https://vikramgrover.com">Built by Vikram Grover</a></div></nav>`}
function publicFooter(){return `<footer class="footer"><div class="container footer-grid"><div><a class="brand footer-brand" href="/"><span class="brand-mark">L</span>Lanesra OS</a><span class="muted">The open-source platform for building your own business app - a complete CRM out of the box.</span></div><div><strong>Product</strong><a href="/platform">Platform</a><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a></div><div><strong>Development</strong><a href="/roadmap">Roadmap & Backlog</a><a href="/releases">Releases</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a></div><div><strong>Creator</strong><a href="https://vikramgrover.com">VikramGrover.com</a></div></div><div class="container footer-bottom"><span>© 2026 Lanesra OS</span><span>Created by Vikram Grover</span></div></footer>`}
function roadmapPage(){
 document.title='Roadmap & Backlog — Lanesra OS';
 setPageMeta('Everything shipped in Lanesra OS - CRM, no-code platform, App Catalog, Solution Management, Integration Hub - what\'s being built next, and what\'s still proposed. Compiled directly from the working codebase, not a wishlist.');
 const shippedGroups=[
  ['▣','Core product, data safety & deployment',[['Core CRM & sales lifecycle','Companies, Contacts, Products, Opportunities, Quotes, Orders, Invoices, Contracts, Tasks — full CRUD, the flexible Company → Opportunity → Quote → Order → Invoice path plus direct-entry shortcuts, gap-free document numbering, integer-cent money math, duplicate-name/email warnings, and dashboard KPIs.'],['Team Workspace (multi-user over LAN)','An axum HTTP server sharing the same business logic as the desktop app, cookie sessions, Docker packaging — a small team runs one server, everyone else uses a browser tab.'],['Data safety & account','Whole-workspace backup/restore as a .lanesra file, self-service password change, admin-managed users with a last-Administrator lockout guard.'],['Document output','PDF-quality print preview for quotes/orders/invoices via the browser\'s native print dialog; CSV export on every list screen and CSV import for Companies and Contacts, both routed through the same create commands the manual forms use.'],['Self-hosted internet deployment','The Docker-packaged Team Workspace server (axum) can now be exposed on the open internet on an organization\'s own domain/infrastructure, still self-hosted and never a Lanesra-run SaaS. LANESRA_TRUST_PROXY_HTTPS marks the session cookie Secure and sends Strict-Transport-Security once a reverse proxy is actually terminating TLS in front of it — off by default so the original LAN-only, plain-HTTP behavior is unchanged unless explicitly opted into. LANESRA_ALLOWED_ORIGINS adds a credentialed CORS layer for the rare case the frontend is served from a different origin than the API — also off (same-origin only) by default. Every response now also carries always-on security headers (nosniff, deny framing, a trimmed Referer). The README documents minimal Caddy and nginx reverse-proxy recipes to pair with it.'],['Record detail pages, ID hyperlinks & new fields — Products, Quotes, Orders, Invoices, Contracts, Tasks','The click-an-ID-to-open-a-record-you-can-view/edit/see-related-records-from pattern Companies and Contacts already had is now every object\'s behavior, on both the online demo and the desktop app. Every list\'s ID column is a hyperlink; each of the 6 newly-covered entities gets a detail page with an Overview panel, line items + totals for the three document types, and related-record links (a Product shows every Quote/Order/Invoice referencing it; documents show downstream documents and Tasks). Also added a batch of relevant out-of-the-box fields across 9 entities - Company phone/email/website/annual revenue/employee count/preferred contact method, Contact mobile/department/LinkedIn, and equivalents for Quotes/Orders/Invoices/Contracts/Products/Tasks.']]],
  ['⚙','No-code extensibility: fields, objects, rules & workflow',[['Admin panel & configurability','Branding & print customization; reports beyond the dashboard plus a simple custom report builder; custom fields, conditional business rules and workflow automation, generalized from Companies/Contacts to every major object; admin-configurable numbering per object; a dashboard KPI picker.'],['Custom Objects — extensibility platform, Phase A','An Administrator defines a whole new business object at runtime with its own icon and ID format, no code change — and it works through the exact same custom fields, business rules and report builder every built-in entity uses.'],['Custom Relationships — Phase B','Admins define relationships between any two record types (built-in or custom) — one-to-one / many-to-one / many-to-many cardinality, a restrict-or-archive delete policy, and a related-records list on record detail pages.'],['Richer Business Rules engine — Phase C, extended','Multi-condition AND/OR matching with one level of nested OR-groups, 10 comparison operators, and 12 action types (require, show, hide, lock, make editable, set default, set/clear value, restrict choices, block save, show error, show warning), plus rule priority, optional effective-date windows, and a "hide by default" flag on custom fields.'],['Richer Workflow Automation engine — Phase D, extended','7 trigger types (created/updated, status/field changed, date reached, due/overdue, scheduled), optional extra AND/OR conditions with OR-groups, and 8 action types (create task, update/default/clear a field, assign owner, create related record, add notification, create reminder), plus an in-app notification center.'],['Field validation, task reminders, session lock — Phase E','Custom field validation (min/max, max length, regex) at both definition and save time; Windows task reminder toasts through the standard Web Notification API; a 15-minute session inactivity auto-lock.'],['Condition engine v2','Four more comparison operators — starts with, ends with, is one of, is not one of — plus field-to-field comparison, so a condition can match against another field\'s live value instead of only a fixed one. Shared by business rules and workflow triggers, on desktop and in the online demo.'],['Status Transition Editor','Restrict which status/stage changes are allowed on any object, with a wildcard "from any status" starting point and a per-rule active toggle. No active rules leaves the field fully unrestricted; resaving the same status is never blocked.'],['Workflow action & test-mode expansion','Workflow actions reaching beyond the triggering record: create a new record (optionally linked) or update a field on already-linked records. A Test rule / Test workflow dry-run mode shows what active rules and workflows would do against hypothetical values, without touching real data.'],['Custom field extensibility','Four more settings on any custom field: a default value applied when a save leaves it empty, a "require a unique value" check (rejected at definition time for yes/no fields), placeholder text, and help text shown under the field on the record form.'],['Business Rules & Workflow Automation redesign','Both builders rebuilt as a numbered Condition/Effect (or Trigger/Action) layout with a live rule-summary panel; Workflow Automation gained a connected visual canvas (Trigger → Conditions → Actions → End) with zoom. Test and Activate/Deactivate moved into the builder header, alongside full editing, not just create.']]],
  ['⊞','The platform trio: Screen/App Builder, Dashboards & App Builder',[['Screen/App Builder — full, 4 phases','What was the single largest item on this roadmap, now the biggest thing shipped on it: a real drag-and-drop layout builder for any object\'s create/edit form, on both the desktop app and the online demo. Phase 1 — named layouts, made of tabs of field sections, assigned to roles with a required Default fallback, and Draft → Publish (desktop had no layout designer at all before this; the demo\'s previous "Screen layouts" was field-order-only). Phase 2 — 1-3 column sections with a per-field full-width span. Phase 3 — placing a custom relationship\'s related-records list on a specific tab, with anything a tab doesn\'t claim still showing in an always-visible spot rather than disappearing. Phase 4 — the same published layout also drives the record\'s read-only detail/Overview view, not just the edit form, so a custom field an admin added is now visible there too. This is exactly the mechanism behind <a href="/platform">building your own app</a> on top of a custom object, not just reordering a form.'],['Dashboard customization — full, 3 phases + online demo','Admin → Dashboards lets an Administrator build multiple named dashboard layouts — each an ordered list of widgets — and assign them by role, with a required Default fallback, reusing the exact same draft/publish/role-resolution model Screen/App Builder shipped. Phase 1 — KPI tiles. Phase 2 — chart widgets, reusing the existing Custom Reports engine. Phase 3 — record-list widgets, a short list of an object\'s most recent (or, for Tasks and Invoices, soonest-due) records that jump straight to that record on click. No dashboard published yet falls back to exactly the fixed KPI picker that existed before this feature, unchanged. Shipped on both the desktop app and the online demo.'],['App Builder — Phase 1: publish a named app from your custom objects','Admin → Apps (desktop and online demo) groups a set of already-existing objects (built-in or custom), their screens and a dashboard into one named, publishable application — Property Management, Recruitment, Asset Tracking, whatever your organization actually runs — with its own icon, a sidebar App Switcher that filters navigation down to that app\'s objects, and an app-scoped dashboard, the same way Salesforce\'s AppDefinition scopes a Lightning app. Access is a genuinely new permission model — a grant to a role or to one specific person, at Viewer or Editor — not the existing role-checkbox pattern; on the desktop app, Administrators always see every published app and everyone else needs an explicit grant, while the browser demo (no signed-in user) simply shows every published app to everyone.'],['App Builder — Phase 2: server-side access enforcement','The Viewer/Editor level Phase 1 resolves is now a real security boundary, not just a UI hint. Once an object type is placed in at least one published app, every create, update, archive and status-lifecycle command on the desktop edition — issuing/voiding an invoice, recording a payment, converting a quote to an order or an order to an invoice, setting a quote/order status — requires at least Editor access to some app containing it, checked server-side on every command, not only by the button that\'s visible. Administrators always bypass; a grant to a specific person beats a grant to their role; the strongest grant across every matching app wins. Objects never placed in an app are completely unaffected — no existing workspace changes behavior by not adopting App Builder. The desktop UI also reflects this before a click reaches the server: a Viewer sees every "New", "Edit", "Issue", "Void", "Record payment" and status/conversion button on a scoped object disabled with an explanatory tooltip.']]],
  ['◧','Online demo parity',[['Online demo: full interactive parity','The browser demo at /demo mirrors everything above as real interactive features, not just changelog copy — its own Status Transitions tab, expanded workflow actions, Test rule/Test workflow panels, the redesigned rule-builder layout with a visual canvas, custom field extensibility, and Customer 360/Contact 360 detail pages.'],['Online demo: workflow-action & custom-field parity','Workflow "Create a new record" now offers all 9 built-in entities, not just 3, and "Update a related record" walks the relationship graph in both directions so every trigger entity gets its actual related-record options. Custom fields in the demo gained the same Required/Max length/Pattern/Min/Max/Searchable/Filterable/Reportable settings the desktop edition already had.'],['Online demo: Custom Objects','An Administrator can define a whole new business object at runtime from Admin → Custom Objects - its own icon, sidebar entry and ID format, no code change - and it works through the demo\'s existing Custom Fields, Business Rules, Status Transitions and Workflow Automation tabs exactly like a built-in entity. Delete is blocked while records exist; deactivate is always safe and reversible.'],['Online demo: Custom Relationships','An Administrator can connect any two object types - built-in or custom - from Admin → Relationships, with a cardinality (many-to-one/one-to-one/many-to-many), forward/reverse labels, and a delete behavior (Restrict or Archive). Every record\'s edit form gets a "Related records" panel showing every link from either direction, with inline Link/Unlink.'],['Online demo: Reports','A new Reports section in the browser demo with the desktop edition\'s full fixed report gallery (Revenue by month, Win rate by owner, Lost reasons, AR aging, Sales by owner) plus a Custom Reports builder that can group any built-in or custom object by its status/stage or a reportable custom field, count or sum, with CSV export on every report - closing the online demo\'s last desktop-parity gap.'],['Online demo: Screen layouts (no-code UI designer)','A new capability that doesn\'t exist on desktop either: from Admin → Screen layouts, an admin drag-orders any built-in or custom object\'s create/edit fields into named sections. Editing only ever touches a draft - the live form keeps its default order until Publish, and Preview shows the draft rendered before that. A scoped, demo-first version of the "No-code Screen/UI Designer" item still proposed below for the full desktop admin extensibility spec (which also covers detail-page and tab/column layouts).'],['Online demo: Integrations (UI-only simulation)','A new Admin → Integrations section, also new to the product rather than a desktop port: scheduled data Export/Import/Sync jobs against any object with a Run now simulation and per-job history, defined-and-exposed API endpoints with a Test call that returns real demo data as JSON, and configured external API connections with a Test request and call history. Everything is a local simulation against this browser\'s data - the static demo has no server, so it\'s built and labeled that way rather than faking a real backend.'],['Online demo: workflow self-updates + custom-object workflow fix','Workflow automation gained the desktop edition\'s update_field action - set another field on the same record a workflow just triggered on (e.g. Company status becomes Customer, so Industry gets set to Active), with the new value either fixed or copied live from another field. Also fixed a bug where workflow rules on admin-defined Custom Objects were creatable but silently never fired.'],['Online demo: mobile layout fix + stale favicon fix','The new record detail pages above overflowed horizontally on a phone - a nested overflow:auto table wrapper without min-width:0 on its containing grid, the same class of bug across Order/Invoice/Quote/Product/Contract detail; fixed to match Company/Contact 360\'s existing correct mobile behavior. Also fixed the browser tab favicon, which still drew a "B" glyph left over from the product\'s original BusinessOS name.']]],
  ['⬢','Industry Data Model & App Catalog — 10 installable business apps',[['Industry Data Model: package manifest, install pipeline & App Catalog','A versioned metadata package format — objects, fields, relationships, business rules, workflows, screens, reports and a dashboard, with optional sample data — installed into an existing workspace from Admin → App Catalog. Install is not a separate product or a parallel data model: a package reuses the workspace\'s existing Company/Contact/Task core rather than creating duplicates, so Sales and an installed industry app coexist against the same customer master. Every install runs through pre-install validation (naming/numbering collisions, missing dependencies), an automatic safety backup, and a transactional install with rollback on failure; deactivating a package removes it from navigation without touching the business records it created.'],['Ten industry reference packages, desktop edition','Field Service, Property Management, Construction & Contractors, Professional Services, Dental/Clinic Practice Administration, Recruitment & Staffing, Real Estate Brokerage, Legal Practice, Nonprofit & Association Management, and Auto Repair & Service Garage — each a complete, install-ready object/field/relationship/business-rule/workflow set for that industry, reviewed and shipped one at a time with its own Rust core tests.'],['Per-app scoped automation','Business rules, workflow automation and dashboards created by an installed app are now tagged with the App that owns them, both admin builders gained an App filter, so an installed package\'s automation stays visibly contained instead of mixing into one flat unscoped list as more apps get installed.'],['App Catalog admin category, and App Builder confirms object changes','Admin → Apps split into two categories - App Builder (build your own) and App Catalog (install a reference package) - instead of one crowded tab. Separately, toggling an object on or off inside App Builder now shows a confirmation toast ("Added Contact to Field Service") instead of saving silently with no visible feedback, the actual defect behind a reported "can\'t add an existing entity to an installed app" bug (the underlying save always worked; there was just no confirmation, made worse by the mobile sidebar\'s icon-only collapsed state hiding the new nav entry).'],['Online demo: full 10-package App Catalog parity','The browser demo\'s Admin → App Catalog mirrors all ten reference packages, not just the original two (Field Service, Property Management) it shipped with - install creates the same custom objects/fields/relationships/rules/workflows client-side and tags them to the installing app, within the demo engine\'s existing structural limits (rules and workflows only fire on a watched-field edit, not record creation; a workflow\'s "create a new record" action only targets the 9 built-in types, not a custom object).'],['Solution Management: Admin IA reshuffle, Solution visibility & Publisher registry','A new Admin category surfaces the Industry Data Model\'s existing package/component/dependency data under the Solution Packages design spec\'s framing - Solution Packages, Components ("what have I customized beyond what I installed" now has a real screen) and Dependencies, plus a genuine <b>Publisher registry</b> with reserved keys, key validation and enforced namespace checks on package import (an unregistered publisher key blocks the import, naming exactly what needs registering first). The two auto-seeded publishers - <code>lanesra</code> (official, owns every bundled reference package) and <code>local</code> (the implicit home for hand-built work) - keep every existing install working unchanged. Shipped on the desktop edition first (Rust core migration 0029, wiring the previously-dead <code>app_dependencies</code> table into actual use) and mirrored in the online demo, including the artifact-tracking the demo never had before this.'],['Solution Management: component-tagging, Local Workspace, export & update-with-diff','Every custom object/field/relationship/rule/workflow now has a real owner - component-tagging (migration 0030) wires into every creation path on both platforms, so the Components tab finally shows hand-built work alongside what an install created, not just the latter. The Managed/Unmanaged distinction is real: a synthetic <b>Local Workspace</b> row in Solution Packages aggregates everything the <code>local</code> publisher owns, with a genuine <b>Export</b> action producing a re-importable manifest (round-trip verified into a fresh workspace on desktop). <b>Releases</b> shows every imported version of a package - no new table, since each version was already an immutable snapshot. <b>Update-with-diff</b> finally replaces "reinstalling an installed package is rejected outright": a real Added/Modified/Removed preview for objects and fields, applied transactionally with a pre-update safety backup, never destroying anything a newer manifest drops. Update-with-diff and Releases are desktop-only - the online demo\'s fixed, single-version reference-package catalog has no multiple versions to diff against or show a history for, called out explicitly rather than faked.'],['Solution Management: named, scoped Solutions','The Dynamics-365-style "build a solution in test, export it, import it in prod" workflow. A new <b>Solutions</b> tab lets an admin name a deliberate <em>subset</em> of components - not everything Local Workspace owns - curate exactly what belongs in it, and Export that subset alone into the same manifest shape the rest of Solution Management already uses. "Environment" needed no new modeling: a Lanesra OS workspace already is a standalone environment, the same way a D365 org is, so promoting a solution to "prod" is two workspaces exchanging the exported file through the unmodified existing Import → Validate → Install pipeline. A solution\'s package_id stays stable across repeated exports, so bumping its version and exporting again becomes a second, listable Release - upgradeable via the same update-with-diff pair any other package uses. Shipped on the desktop edition (Rust core migration 0031, <code>solution_service</code>) and mirrored in the online demo.']]],
  ['⬡','Integration Hub — connect Lanesra to the outside world',[["Connections, Connectors, API Access, Webhooks","A real, encrypted-at-rest Connection layer (generic REST, SFTP, Postgres, OData, SMTP, OAuth2), OpenAPI-imported Connectors that become reusable Workflow Automation actions, inbound API clients (hashed <code>{client_id}.{secret}</code> keys, scoped, revocable) against a generic <code>/api/v1/objects/...</code> REST API, and outbound Webhooks with HMAC-SHA256 signing, retry/backoff and delivery history - each backed by real tests (a real local TCP listener stands in for the external system, not a mock)."],["Data Exchange, External Objects, Integration Jobs","The existing Companies/Contacts-only CSV import is now a generalized, object-agnostic wizard (any built-in or Custom Object, column mapping with transforms, duplicate/upsert handling, dry-run preview) plus CSV export and reusable Mappings - all routed through the same generic record-write path the REST API uses, so permissions and business rules apply identically. External Objects surface read-only, live records from a Connection; Integration Jobs add a recurring pull-sync with a checkpoint cursor, run by a real background scheduler on the Team Workspace server (desktop keeps a manual Run Now, the same client-poll-only pattern Workflow Automation's own scheduling already has)."],["Logs & Monitoring, Settings, event streaming","A unified execution log across API calls, webhook deliveries and import/export runs, a real KPI-driven Overview, workspace-level rate-limit/retention Settings, and a Server-Sent-Events endpoint streaming executions live. Best-effort, explicitly-labeled coverage beyond the core v1 scope: SFTP and PostgreSQL connectors (proven against a real in-process SFTP server and a real local Postgres), OAuth2 client-credentials and authorization-code token exchange, a minimal SMTP action, and an OData v4 query/response layer. mTLS is the one deliberate gap, left for later as the spec itself marks it."]]],
  ['⌕','Admin UX, search & list views',[['Customer 360 / Contact 360','A dedicated detail page for every company and contact — full field overview plus every linked record (contacts, opportunities, quotes, orders, invoices, contracts, tasks) one click away, replacing edit-modal-only access.'],['Admin landing page redesign & Admin UX polish','Admin no longer opens straight into a flat tab row — a categorized landing page (Workspace, Access, Customization, Automation, Integrations) routes straight into each builder on click, with a breadcrumb (Dashboard → Admin → tool) back to the landing page; the sidebar Admin icon always resets to it. Business Rules and Workflow Automation both gained Duplicate (clones a rule/workflow as an inactive draft, opened for review), a bounded version history (last 10 saves with a one-click Restore per version — restoring itself snapshots the state it replaces, so it\'s never a dead end), and a dependency warning before deactivating a custom field an active rule or workflow still reads or writes, listing exactly which ones. Shipped on both the online demo and the desktop app (Rust core + React). The last scoped item in the Admin Automation & Customization addendum.'],['Desktop: Global search & list-view filtering','A topbar search box (Companies, Contacts, Opportunities, Products, Quotes, Orders, Invoices, Contracts, Tasks, active custom objects, plus any custom field flagged Searchable) resolves matches to a real display name via the entity-registry dispatcher and jumps straight to the record, reusing the exact same one-shot openId navigation every ID hyperlink already uses. Every list screen also gained filter controls for whichever custom fields an admin flagged Filterable — a select/boolean field filters by exact match, text by case-insensitive contains — client-side against one bulk values fetch per screen. Gives the is_searchable/is_filterable capability flags their first real use since Phase E introduced them. Desktop only; the online demo\'s own simple ⌘K search stays unaffected, as planned.'],['Saved Views & Bulk Actions','Any list screen that already had per-field filtering — Companies, Contacts, Tasks, Opportunities, and every Custom Object\'s records — can now save its current filter/sort/group-by combination as a named view, Private or Shared, with an admin-settable object default; a saved view also feeds a dashboard record-list widget directly, narrowing it to that view\'s filters instead of showing every record. Bulk operations pair with the multi-select a saved view narrows to: update a status/stage (validated through the same Status Transition rules a single edit already goes through), reassign owner, add/remove tags, and archive — each offered only where the entity actually carries that field (Contacts have no owner field, for instance, so bulk reassign isn\'t offered there). Shipped on the desktop edition (Rust core: migration 0034, <code>saved_view_service</code>, <code>bulk_action_service</code>) and mirrored in the online demo — the same view/sort/group-by and bulk update-status/reassign-owner/export/delete operations against this demo\'s simpler status-and-owner field set, without a Private/Shared distinction or a per-custom-field filter, since the demo has neither a signed-in-user concept nor desktop\'s per-field list filtering to begin with.']]],
 ];
 const planned=[
  ['App Catalog: a package detail screen before install','Today "Install" is the only way to see what a reference package actually does. Next to Install, add a Details view an admin opens first: what the package builds (its objects, at a glance), how it connects to what\'s already in the workspace (which existing entities it reuses vs. what\'s new), the automation it ships with (rules/workflows in plain language, not JSON), and concrete guidance on which existing or custom objects to add to make it fit a real organization. Content structure decided up front so it reads clearly rather than as a wall of manifest fields; UX/UI treatment matters as much as the content.',null,'M'],
  ['Admin panel: UX/UI consistency pass','A full diagnostic across the whole Admin section, both platforms, for spacing/alignment/visual inconsistencies — flagged in particular: the mobile sidebar is icon-only with no label, so a newly-added nav icon (e.g. after installing an app) is easy to miss entirely. The Solution Packages spec\'s own Mobile UX Requirements call this out directly: icons alone aren\'t sufficient for discoverability, and the collapsed rail needs accessible labels/tooltips or a labeled collapsible state.',null,'M'],
 ];
 const proposed=[
  ['The 5-layer extension model & destructive uninstall','The remaining scope from the Solution Packages design spec, now that component-tagging, the Local Workspace/Unmanaged grouping, export and update-with-diff have all shipped (see Shipped, below): a formal 5-layer extensibility model (Core → Managed → Managed Extension → Unmanaged Extension → Local Override) so a managed component can be deliberately, safely extended without editing it destructively - today any admin can already add a field to a package-installed object with no restriction, which covers the common case but has no concept of "this field belongs to a different layer than the object it\'s on." Also still open: destructive uninstall with a dependency/data-disposition review (only non-destructive deactivate/reactivate exist), and <code>is_locally_customized</code> drift detection (package_artifacts has had the column since the beginning; nothing sets it yet - update-with-diff\'s new Modified classification is the natural signal to wire it from).','Needs its own design pass on what a "layer" concretely changes about how a component behaves or renders, beyond the ownership tagging that already exists - the risk is building a taxonomy with no visible difference from what component-tagging (just shipped) already answers.','App Catalog','M'],
  ['Approval Engine','Optional approval routing for a record — a high-value order, a large quote discount, a contract — with Pending/Approved/Rejected/Cancelled states, approver comments, and the result visible on the record, in audit history, and to workflows. Explicitly optional so it never complicates the simple SMB flows that don\'t need it.','Needs a decision on where approval criteria live (a standalone rule type vs. reusing the Business Rules condition engine) and how deep multi-step/multi-approver routing should go in v1 vs. later.',null,'M'],
  ['Full drag-and-drop report builder','The shipped report builder covers pick-an-object → group-by-field (including custom fields) → count or sum. A richer builder — multiple group-bys, filters, joins across objects, a visual canvas — was scoped down to that simpler version by explicit choice.','Worth revisiting once real usage shows the count/sum + single group-by shape is genuinely too narrow.',null,'M–L'],
  ['Optional Google/Microsoft sign-in','Let a user log in with their Google or Microsoft identity alongside the existing local username/password, as an optional per-workspace toggle — local accounts remain the required baseline so offline use never depends on it.','Needs a decision first: since every workspace is self-hosted (not a shared Lanesra SaaS), each organization would have to register its own OAuth client — is that acceptable setup friction, or does this need a generic OIDC option instead of naming specific providers? Also unclear whether it applies to the Team Workspace server only, or the Tauri desktop app too (an OAuth redirect flow is awkward inside a native webview).',null,'M'],
  ['Code-signed Windows installer','The published installer is unsigned, so Windows SmartScreen flags it as an unknown publisher.','Mostly not a coding task: buy a certificate, add a signtool step to the release workflow. The real cost is procurement — identity verification lead time, a recurring fee — an ops/budget decision, not an engineering one.',null,'S (code) / ops-heavy'],
 ];
 const futureIdeas=['Projects and milestones','Inventory and suppliers','Recurring invoices','Customer portal','Plugin architecture'];
 const shippedTotal=shippedGroups.reduce((n,g)=>n+g[2].length,0);
 const latestCatIndex=shippedGroups.length-1;
 const latestCat=shippedGroups[latestCatIndex];
 // Short chip/quick-nav label for a category: the part before an em dash
 // or colon, so "The platform trio: Screen/App Builder, ..." reads as
 // "The platform trio" - derived from the title itself rather than a
 // second parallel array, so it can never drift out of sync with it.
 const shortCatLabel=g=>g[1].split(/[—:]/)[0].trim();

 // SEO/LLM structured data: a CollectionPage wrapping two ItemLists - the
 // actionable upcoming work (planned + proposed, real descriptions) and
 // the shipped categories (title, item count, one representative blurb) -
 // not every individual shipped line, which would bloat this block
 // without telling a crawler or an AI answer engine anything more useful.
 const plainText=s=>String(s).replace(/<[^>]+>/g,'');
 const truncate=(s,n)=>{const t=plainText(s);return t.length>n?t.slice(0,n).replace(/\s+\S*$/,'')+'…':t};
 let pos=0;
 const upcomingItems=[
  ...planned.map(p=>({'@type':'ListItem',position:++pos,name:p[0],description:`${truncate(p[1],220)} (status: planned, size: ${p[3]})`})),
  ...proposed.map(p=>({'@type':'ListItem',position:++pos,name:p[0],description:`${truncate(p[1],220)} (status: proposed, size: ${p[4]})`})),
 ];
 const shippedListItems=shippedGroups.map((g,i)=>({'@type':'ListItem',position:i+1,name:plainText(g[1]),description:`${g[2].length} shipped item${g[2].length===1?'':'s'}, including ${truncate(g[2][0][0],80)}`}));
 setPageJsonLd('roadmap-jsonld',{
  '@context':'https://schema.org',
  '@type':'CollectionPage',
  '@id':'https://lanesraos.com/roadmap#page',
  url:'https://lanesraos.com/roadmap',
  name:'Lanesra OS Roadmap & Backlog',
  description:'Everything shipped, being built next, and still proposed for Lanesra OS - CRM, no-code platform, App Catalog, Solution Management, Integration Hub - compiled from the working codebase.',
  isPartOf:{'@id':'https://lanesraos.com/#website'},
  about:{'@id':'https://lanesraos.com/#software'},
  mainEntity:[
   {'@type':'ItemList',name:'Building next & proposed',itemListElement:upcomingItems},
   {'@type':'ItemList',name:'Shipped categories',itemListElement:shippedListItems},
  ],
 });

 $('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Built from the working codebase</div><h1>Roadmap & backlog.</h1><p>What's being built next, what's still waiting on a decision, and everything already shipped — compiled from the actual code (core/server/src-tauri/frontend) plus the online demo, not a wishlist. Every "Shipped" entry below is running code with tests.</p><div class="status-row"><span class="status-chip">Early Access v0.36.0</span><span class="muted">Last updated August 2026</span></div>
 <a class="rm-spotlight" href="#cat-${latestCatIndex}"><span class="rm-spotlight-eyebrow"><span class="rm-spotlight-dot"></span>Just shipped</span><span class="rm-spotlight-title"><span class="rm-cat-icon">${latestCat[0]}</span>${latestCat[1]}</span><span class="rm-spotlight-arrow">See what's in it →</span></a>
 <div class="backlog-callout" id="desktop"><h3>Release status</h3><p><b>desktop-v0.12.0 is the latest tagged release</b> (installers attached, Early Access/prerelease as intended), cut with the Integration Hub work below (v0.35.0) — Saved Views & Bulk Actions (v0.36.0) has since merged to <code>main</code> but hasn't been cut into a new installer tag yet. Everything below is merged to <code>main</code> regardless of installer status. Full desktop feature list → <a href="/download">/download</a>, full release history → <a href="/releases">/releases</a>.</p><p class="muted">Repo hygiene: a real MIT <code>LICENSE</code>, <code>CONTRIBUTING.md</code>, <code>CODE_OF_CONDUCT.md</code>, <code>SECURITY.md</code>, issue/PR templates, and a root README written for someone landing on the repo, not a deploy runbook.</p></div>
 </div></section>
 <nav class="rm-subnav" aria-label="Roadmap sections"><div class="container narrow rm-subnav-inner"><a href="#building-next"><span class="rm-subnav-n">${planned.length}</span>Now building</a><a href="#deciding"><span class="rm-subnav-n">${proposed.length}</span>Deciding</a><a href="#future-ideas"><span class="rm-subnav-n">${futureIdeas.length}</span>Ideas</a><a href="#shipped" class="done"><span class="rm-subnav-n">${shippedTotal}</span>Shipped</a></div></nav>
 <section class="section"><div class="container narrow">

 <section class="rm-section" id="building-next"><div class="rm-section-head"><div class="eyebrow">In progress · up next</div><h2>What's being built next.</h2><p>Scoped and ready to build — no open product question left to resolve first.</p></div>${planned.length>0?planned.map(p=>`<div class="rm-hero-card"><div class="rm-kicker"><span class="dot"></span>Next up</div><h3>${p[0]}</h3><p>${p[1]}</p><div class="rm-tags"><span class="backlog-tag planned-tag">planned</span><span class="backlog-tag">size: ${p[3]}</span></div></div>`).join(''):`<div class="rm-empty">Nothing scoped and ready right now — every remaining item needs a product/architecture decision first. See "Deciding" below.</div>`}</section>

 <section class="rm-section" id="deciding"><div class="rm-section-head"><div class="eyebrow">Proposed</div><h2>Big calls, waiting on a decision.</h2><p>Explicitly deferred, not forgotten — each needs a product or architecture decision before it's scoped enough to build. <b>The 5-layer extension model</b> is the one worth deciding first: Solution Management's foundation (component-tagging, Local Workspace, export, update-with-diff) has already shipped — see <a href="#shipped">Shipped</a> below — so this is the natural next layer on top of it.</p></div><div class="rm-card-grid">${proposed.map(p=>`<div class="backlog-card"><div class="backlog-card-head"><h3>${p[0]}</h3><div class="backlog-card-tags"><span class="backlog-tag proposed-tag">proposed</span>${p[3]?`<span class="backlog-tag">${p[3]}</span>`:''}<span class="backlog-tag">size: ${p[4]}</span></div></div><p class="ask">${p[1]}</p><div class="backlog-solution"><div class="sol-label">Why it's still just proposed</div><ul><li>${p[2]}</li></ul></div></div>`).join('')}</div></section>

 <section class="rm-section" id="future-ideas"><div class="rm-section-head"><div class="eyebrow">Unscoped</div><h2>Ideas without a plan yet.</h2><p>Added here once there's real signal they're needed — no shape, no decision, just names worth remembering.</p></div><div class="rm-chip-cloud">${futureIdeas.map(x=>`<span class="rm-idea-chip">${x}</span>`).join('')}</div></section>

 <section class="rm-section" id="shipped"><div class="rm-section-head"><div class="eyebrow">Shipped</div><h2>Everything already running in production.</h2><p>${shippedTotal} epics across ${shippedGroups.length} areas, all merged to <code>main</code> — every line below is running code with tests, not a changelog promise.</p></div>
 <div class="rm-shipped-toolbar"><input type="search" id="rmSearch" class="rm-search-input" placeholder="Search everything shipped… (e.g. &quot;webhook&quot;, &quot;dashboard&quot;)" aria-label="Search shipped features"><span id="rmSearchCount" class="rm-search-count" aria-live="polite"></span></div>
 <div class="rm-filter-chips">${shippedGroups.map((g,i)=>`<button type="button" class="rm-filter-chip" data-cat="${i}"><span class="rm-cat-icon">${g[0]}</span>${shortCatLabel(g)}</button>`).join('')}</div>
 ${shippedGroups.map((g,i)=>`<details class="rm-cat" id="cat-${i}"${i===latestCatIndex?' open':''}><summary><h3 class="rm-cat-label"><span class="rm-cat-icon">${g[0]}</span>${g[1]}</h3><span class="rm-cat-meta"><span class="rm-cat-count">${g[2].length}</span><span class="rm-chevron"></span></span></summary><div class="rm-cat-body"><div class="backlog-shipped-list">${g[2].map(s=>`<div class="backlog-shipped-item"><div class="mark">✓</div><div><h4 class="t">${s[0]}</h4><div class="d">${s[1]}</div></div></div>`).join('')}</div></div></details>`).join('')}
 </section>

 </div></section>
 </main>${publicFooter()}`;
 bindPublicNav();
 bindRoadmapSearch();
}
// Roadmap "Shipped" search/filter: text search matches against each
// item's own rendered title+description (read at filter time via
// textContent, not baked into a data attribute - the content is static
// copy this file owns, so escaping it into an HTML attribute would only
// add risk for no benefit), hides non-matching items, and collapses any
// category left with zero matches. Clearing the box restores the default
// state (only the newest category open). Filter chips are a quick-jump,
// not a filter - they open and scroll to one category, same as the
// hero spotlight link.
function bindRoadmapSearch(){
 const input=document.getElementById('rmSearch');
 const countEl=document.getElementById('rmSearchCount');
 const cats=[...document.querySelectorAll('#shipped details.rm-cat')];
 const defaultOpenIndex=cats.length-1;
 document.querySelectorAll('.rm-filter-chip').forEach(chip=>{
  chip.onclick=()=>{
   const cat=cats[Number(chip.dataset.cat)];
   if(!cat)return;
   cat.open=true;
   cat.scrollIntoView({behavior:'smooth',block:'start'});
  };
 });
 if(!input)return;
 input.oninput=()=>{
  const q=input.value.trim().toLowerCase();
  if(!q){
   cats.forEach((cat,i)=>{cat.style.display='';cat.open=(i===defaultOpenIndex);cat.querySelectorAll('.backlog-shipped-item').forEach(item=>item.style.display='')});
   countEl.textContent='';
   return;
  }
  let matches=0;
  cats.forEach(cat=>{
   let any=false;
   cat.querySelectorAll('.backlog-shipped-item').forEach(item=>{
    const text=(item.querySelector('.t').textContent+' '+item.querySelector('.d').textContent).toLowerCase();
    const hit=text.includes(q);
    item.style.display=hit?'':'none';
    if(hit){any=true;matches++}
   });
   cat.style.display=any?'':'none';
   cat.open=any;
  });
  countEl.textContent=matches?`${matches} match${matches===1?'':'es'}`:'No matches';
 };
}
function releasesPage(){document.title='Releases — Lanesra OS';const releases=[['v0.36.0','August 2026','Saved Views & Bulk Actions — desktop and online demo',["Any list screen that already had per-field filtering - Companies, Contacts, Tasks, Opportunities, and every Custom Object\'s records - can now save its current filter/sort/group-by combination as a named view, Private or Shared, with an admin-settable object default. Selecting a view narrows and re-sorts the list live; updating or deleting a view, and setting the workspace default, are all one click from the same bar","A saved view also feeds a dashboard record-list widget directly: the admin builder can narrow any record-list widget to one saved view, so a dashboard tile shows only the subset that view\'s filters define instead of every record of that type","Bulk operations pair with the multi-select a saved view narrows to: update a status/stage (validated through the same Status Transition rules a single edit already goes through), reassign owner, add/remove tags, and archive - each offered only where the entity actually carries that field (Contacts have no owner field on either platform, so bulk reassign isn\'t offered there)","Desktop (Rust core): migration 0034 (<code>saved_views</code>), <code>saved_view_service</code> (CRUD, one-default-per-object enforcement, owner-or-admin edit/delete) and <code>bulk_action_service</code> (per-entity allowlists mirroring what each entity\'s model actually supports), plus 11 new Rust tests, all green alongside the full existing suite","Online demo: the same view/sort/group-by and bulk update-status/reassign-owner/export/delete operations against this demo\'s simpler status-and-owner field set, wired into the shared <code>tablePage</code> renderer every list screen already uses - without a Private/Shared distinction or a per-custom-field filter, since the demo has neither a signed-in-user concept nor desktop\'s per-field list filtering to begin with"]],['v0.35.0','August 2026','Integration Hub — Connections, Connectors, API Access, Webhooks, Data Exchange, Integration Jobs, Logs & Monitoring, desktop only',['A real Integration Hub ships on the desktop app (Admin → Integration Hub): encrypted-at-rest Connections (AES-256-GCM) for generic REST, SFTP, PostgreSQL, OData and SMTP, with a genuine Test Connection call against a real endpoint - not a client-side simulation like the online demo\'s existing Integrations tab, which stays exactly that and is relabeled to match this feature\'s terminology rather than implying parity it doesn\'t have','Connectors import an OpenAPI 3.x spec and derive reusable Actions, each callable as a new Workflow Automation \'Call Connector Action\' step. API Access issues hashed, scoped <code>{client_id}.{secret}</code> bearer keys against a real generic <code>/api/v1/objects/...</code> REST API (Team Workspace server only - a pure desktop install has no listening socket for an external caller to reach). Webhooks sign every delivery with HMAC-SHA256, retry with exponential backoff, and keep a full delivery history, with a one-click Test Delivery','Data Exchange generalizes the previous Companies/Contacts-only CSV import into an object-agnostic wizard (any built-in or Custom Object, auto-map-from-header, transforms, duplicate/upsert policy, dry-run preview with per-row results) plus CSV export and reusable Mappings - all routed through the same generic record-write path the REST API itself uses, so validation, business rules and permissions apply identically, not a second parallel code path','External Objects surface read-only, live records from a Connection; Integration Jobs add a recurring pull-sync on an interval with a checkpoint cursor, run by a real background scheduler thread on the Team Workspace server (desktop has no long-running process to host one, so a manual Run Now is the only way a desktop-hosted Job ever runs today - stated directly in the Jobs tab\'s own copy). A unified execution log spans API calls, webhook deliveries and import/export runs, with a real KPI Overview and a Server-Sent-Events endpoint for live streaming','Best-effort, explicitly-labeled coverage beyond that core scope: SFTP and PostgreSQL connectors proven against a real in-process SFTP server and a real local Postgres instance (the Postgres test is <code>#[ignore]</code>d by default since CI doesn\'t run <code>cargo test</code> at all here, so contributors without Postgres installed are unaffected), OAuth2 client-credentials and authorization-code token exchange against a local mock token endpoint, a minimal SMTP send action against a raw-socket SMTP double, and an OData v4 query/response layer against a mock server. mTLS is the one deliberate, named gap - the spec itself marks it Future']],['v0.34.0','August 2026','Named, scoped Solutions — the Dynamics-365-style build-in-test / export / import-in-prod workflow, desktop and online demo',['Solution Management gains a real <b>Solutions</b> tab: a named, versioned, admin-picked <em>subset</em> of components - not everything Local Workspace owns. Create a solution, curate exactly the objects/fields/relationships/rules/workflows/screens/reports it needs from anything in the workspace (hand-built or installed), and Export it on its own into the same manifest shape <code>export_local_workspace</code> already produces','No new "environment" concept was needed: a Lanesra OS workspace - one desktop install, or one Team Workspace deployment - already is a standalone environment, the same way a D365 org is. "Build in test, export it, import it in prod" is two separate workspaces exchanging the exported file through the existing Admin → App Catalog → Import → Validate → Install pipeline, unmodified','Desktop (Rust core): new <code>solutions</code>/<code>solution_members</code> tables (migration 0031), <code>solution_service</code> for admin-gated create/rename/delete and add/remove-component, and <code>industry_package_service::export_solution</code>, which shares its manifest-building pass with <code>export_local_workspace</code> via a new <code>build_export_manifest</code> helper. A solution\'s <code>package_id</code> is fixed as <code>local.solution.&lt;id&gt;</code>, stable across repeated exports, so bumping its version and exporting again produces a second, listable Release of the same package - upgradeable in the target workspace via the existing update-with-diff pair, exactly like any other package','Online demo: a parallel <code>data.solutions</code> registry with the identical create/rename/delete/add-member/remove-member/export shape, sharing the same manifest-building helper its own <code>exportLocalWorkspace</code> was refactored to use - a Solution here is a real, complete export for backup/inspection/sharing, though (like <code>exportLocalWorkspace</code>) not re-importable within the demo itself, since the demo has no pathway to install a hand-authored manifest back in','10 new Rust core tests (create/rename/delete, membership add/remove, scoped export excluding uncurated components, round-trip export into a separate "prod" workspace, two sequential version-bumped exports listing as two Releases, admin-gating) alongside the full existing suite, all green']],['v0.33.0','August 2026','Component-tagging, Local Workspace & export — the rest of Solution Management, desktop and online demo',['Component-tagging: every custom object, field, relationship, business rule and workflow now has a real owner - the desktop edition wires this into all 10 creation call sites plus the package installer (migration 0030, <code>solution_component_service</code>), and the online demo mirrors it independently across its own 7 hand-built creation sites plus <code>installReferencePackage</code>. The Solution Management <b>Components</b> tab now shows everything in the workspace, hand-built or installed, not only what an install created','The Managed/Unmanaged distinction\'s Unmanaged half, made real without a fake package row: a synthetic <b>Local Workspace</b> entry in Solution Packages aggregates every component still owned by the <code>local</code> publisher, on both platforms','A genuine <b>Export</b> action turns everything in Local Workspace into a downloadable, re-importable manifest. On desktop this reuses the exact <code>IndustryPackageManifest</code> shape <code>import_package</code>/<code>install</code> already understand - exporting one workspace\'s hand-built customizations and importing them into another was verified end to end. The online demo\'s export produces the same manifest shape for backup/inspection/sharing, documented as not re-importable within the demo itself, since the demo has no pathway to install a hand-authored manifest the way its fixed reference-package catalog installs','<b>Releases</b> (desktop only): every imported version of a package, oldest first - no new table, since each <code>app_packages</code> row was already an immutable per-version snapshot','<b>Update-with-diff</b> (desktop only), replacing "reinstalling an already-installed package_id is rejected outright": <code>plan_package_update</code> previews Added/Modified/Removed for objects and fields (the two component types with stable keys across versions) plus added-counts for relationships/rules/workflows/screen layouts/reports; <code>apply_package_update</code> applies it transactionally with a pre-update safety backup, updating matched objects/fields in place and never deleting anything a newer manifest drops. The online demo\'s reference-package catalog has no multi-version concept to diff against, so this stays desktop-only, called out explicitly in its own Solution Packages tab rather than faked','11 new Rust core tests (component-tagging, Local Workspace summary, export round-trip into a fresh workspace, update-with-diff plan/apply) alongside the full existing suite, all green']],['v0.32.0','August 2026','Solution Management: Admin IA reshuffle, Publisher registry & namespace enforcement — desktop and online demo',['Admin no longer opens into one flat category set: the desktop edition\'s landing page is regrouped into 8 categories (Workspace, Access, Data Model, Experience, Automation, Apps, Analytics, Solution Management) and the online demo into 9 (the same plus Integrations, demo-only) - pure regrouping of existing tools plus one new area','New <b>Solution Management</b> area (both platforms) answers "what have I customized beyond what I installed": a <b>Solution Packages</b> tab reframes every installed reference package with its publisher, component and dependency counts; a <b>Components</b> tab lists every custom object/field/relationship/rule/workflow an install created, grouped and searchable by type; a <b>Dependencies</b> tab shows each package\'s declared dependencies with a satisfied/unsatisfied badge','Desktop: wired the existing but previously-dead <code>app_dependencies</code> table into <code>import_package</code>, so a manifest\'s declared dependencies are now actually recorded at import time, and added a workspace-wide artifact listing so Components isn\'t limited to one installed app at a time. Online demo: added the equivalent artifact tracking to <code>installReferencePackage</code>, which didn\'t exist in any form before this release','A real <b>Publisher registry</b>, not a placeholder: every workspace auto-seeds two publishers - <code>lanesra</code> (official, owns all ten bundled reference packages) and <code>local</code> (the implicit home for hand-built customizations) - and an admin can register more from a new Publishers tab, with the same key rules enforced independently on both platforms: lowercase, starts with a letter, 2-32 characters, digits/underscores only, <code>lanesra</code> and <code>local</code> reserved','Every package import now resolves its publisher from the text before its <code>package_id</code>\'s first dot and rejects the import if that publisher isn\'t registered, naming exactly which key needs registering - real, enforced namespace validation on both the desktop Rust core (migration 0029) and the online demo, not merely documented behavior','Managed/Unmanaged, the 5-layer extension model, update-with-diff and Releases/export remain proposed - this release is the registry and visibility layer under them, not the full spec']],['v0.31.0','August 2026','Online demo: full 10-package App Catalog parity',['The browser demo\'s Admin → App Catalog now mirrors all ten industry reference packages, not just the original two (Field Service, Property Management) it launched with in v0.28.0: Construction & Contractors, Professional Services, Dental/Clinic Practice Administration, Recruitment & Staffing, Real Estate Brokerage, Legal Practice, Nonprofit & Association Management and Auto Repair & Service Garage all install client-side into the demo\'s existing engine - custom objects, fields, relationships, business rules and workflows, tagged to the installing app the same way per-app scoped automation already tags them on desktop','Documented (not silently worked around) three structural limits of the demo\'s simpler engine hit while mirroring these: workflow rules only fire on an edit where a watched field changed, not on record creation; a workflow\'s "create a new record" action only supports the 9 built-in types, not a custom-object target; and the demo\'s condition engine has no date-to-date comparison operator, only numeric-coercing greater/less-than','<b>App Builder</b> also gained real feedback: toggling an object on or off now shows a confirmation toast ("Added Contact to Field Service") instead of saving with no visible confirmation - the actual defect behind a reported "can\'t add an existing entity to an installed app" bug (the save always worked; there was simply nothing telling the user it had)']],['v0.30.0','August 2026','Auto Repair & Service Garage: tenth and final industry reference package',['Completes the Industry Data Model\'s initial reference-package set: Vehicle, Repair Order, Repair Line, Repair Inspection, Service Recommendation and Vehicle Appointment objects, 30 fields, 10 relationships, 3 business rules (repair-completion, authorization, odometer validation) and 4 workflows (appointment check-in, repair authorized, repair completed, no-show)','Surfaced and documented two real platform limits while building it: a workflow\'s create_record action can only target Company or an active custom object - never a built-in document type like Invoice, since those need a required relational field a no-code action can\'t safely synthesize - and create_record\'s name template is static, not built from the triggering record\'s own fields. Both are now called out in the reference-package module\'s own docs rather than worked around silently','10 new Rust tests covering manifest install counts and every business rule and workflow end to end']],['v0.29.1','August 2026','Real Estate Brokerage, Legal Practice, and Nonprofit & Association Management reference packages',['Three more industry reference packages on the Industry Data Model foundation, each with its own object model, business rules and workflows: <b>Real Estate Brokerage</b> (Listings, Transactions, Showings), <b>Legal Practice</b> (Matters, Time Entries, Court Dates) and <b>Nonprofit & Association Management</b> (Members, Donations, Programs)','Brings the App Catalog to nine reference packages, one short of the full initial set']],['v0.29.0','August 2026','Per-app scoped automation',['Business rules, workflow automation and dashboards created as part of an installed app are now tagged with the App that owns them - both admin builders gained an App filter, so an installed reference package\'s automation stays visibly contained as more apps get installed instead of mixing into one flat, unscoped list','Shipped on the desktop edition (Rust core migration + models + repos + services, Tauri commands, React UI) and mirrored in the online demo']],['v0.28.2','August 2026','App Builder: object-change confirmation, and App Catalog split into its own Admin category',['Admin → Apps split into two categories - <b>App Builder</b> (assemble your own app from existing objects) and <b>App Catalog</b> (install a ready-made industry reference package) - instead of one crowded tab covering both very different workflows','Toggling an object on or off inside an App Builder app already saved and refreshed navigation correctly; it just gave no visible confirmation it had, which read as a bug when the newly-added object\'s nav icon was easy to miss on the icon-only mobile sidebar. Fixed with a toast']],['v0.28.1','August 2026','App Catalog: Construction & Contractors, Professional Services, Dental/Clinic Practice Administration, and Recruitment & Staffing reference packages',['Four more reference packages available from Admin → App Catalog, each a complete object/field/relationship/business-rule/workflow set reusing the workspace\'s existing Company/Contact/Task core rather than a parallel data model: <b>Construction & Contractors</b> (Projects, Change Orders, Site Visits), <b>Professional Services</b> (Engagements, Timesheets, Deliverables), <b>Dental/Clinic Practice Administration</b> (Patients, Appointments, Treatment Plans) and <b>Recruitment & Staffing</b> (Job Openings, Candidates, Placements)','Brings the App Catalog to six reference packages']],['v0.28.0','August 2026','Industry Data Model: package manifest, install pipeline & App Catalog, with the first two reference packages',['The largest item that was still "proposed" as of v0.27.0, now shipped: a versioned metadata package format - objects, fields, relationships, business rules, workflows, screens, reports and a dashboard - installed into an existing workspace from a new Admin → App Catalog. Install runs through pre-install validation (naming/numbering collisions, missing dependencies), an automatic safety backup, and a transactional install with rollback on failure; deactivating a package removes it from navigation without touching the business records it created','Ships with the first two reference packages: <b>Field Service</b> (Work Orders, Assets, Service Appointments) and <b>Property Management</b> (Properties, Units, Leases) - both fully reusing the workspace\'s existing Company/Contact/Task core rather than creating parallel "Field Service Customer" style duplicates','Shipped on the desktop edition first (Rust core: repo, install-orchestrator service, Tauri commands, server dispatch, React admin UI) and mirrored in the online demo\'s own App Catalog']],['v0.27.0','August 2026','App Builder: package objects, screens and a dashboard into one named app with role-based access — now enforced server-side, not just in the UI',['<b>App Builder</b> ships as a real feature, not just a name: an Administrator can group any set of objects - Custom Objects or built-ins - their published Screen/App Builder layouts, and one dashboard into a named, publishable App with its own icon, then grant it to specific roles or individual users as Viewer or Editor. Every published App also filters navigation and adds an entry to a sidebar App Switcher - switching into an app only shows the objects it claims, not the whole workspace','Access grants are now enforced everywhere a record is written, not only in the UI that shows or hides a button: once an object type is placed in at least one published App, creating, editing or archiving a record of that type requires at least Editor access to some App containing it. An Administrator always bypasses; a grant made to a specific user beats one made to their role; the strongest grant across every matching App wins. Objects never placed in an App are completely unaffected - existing workspaces that haven\'t adopted App Builder see no change in behavior','Status-lifecycle actions are covered by the same check: issuing or voiding an invoice, recording a payment, setting a quote or order\'s status, and converting a quote to an order or an order to an invoice all require write access to the entity type the action actually changes - a conversion command is gated on the document it creates, not the untouched source document it reads','The desktop app reflects this before a click ever reaches the server: a Viewer-only user sees every "New", "Edit", "Issue", "Void", "Record payment" and status/conversion button on a scoped object disabled with a tooltip explaining why, instead of submitting and hitting a permission error','Shipped on the desktop edition first (Rust core, Tauri commands, React UI) and mirrored in the online demo (Admin → Apps, the sidebar App Switcher, and an app-scoped dashboard)']],['v0.26.1','August 2026','Dashboard customization Phase 2/3: chart and record-list widgets, on desktop and in the online demo',['Extends Phase 1\'s KPI-tile-only widget catalog: a <b>chart widget</b> reuses the existing Custom Reports engine directly on a dashboard - pick a saved report (any object, grouped by its status/stage or a reportable custom field, counted or summed) and it renders live, the same report definition Admin → Reports already builds and runs elsewhere','A <b>record-list widget</b> shows a short list of an object\'s most recent records - or, for Tasks and Invoices, its soonest-due ones - each row jumping straight to that record on click, the same one-shot navigation every ID hyperlink elsewhere in the app already uses','Both widget types ship on the desktop app\'s Admin → Dashboards builder and are mirrored in the online demo\'s own dashboard builder, reusing the identical draft/publish/role-resolution model Phase 1 established - a dashboard using neither widget type behaves exactly as it did before this release']],['v0.26.0','August 2026','Screen/App Builder (full, 4 phases) and Dashboard customization Phase 1 - the two largest items on the roadmap, both shipped',['<b>Screen/App Builder</b>, the single largest item that was still "proposed" - a drag-and-drop layout builder for any object\'s create/edit form, on both the desktop app and the online demo, shipped in four phases: (1) named layouts with tabs of field sections, assigned to roles with a required Default fallback, and Draft → Publish; (2) multi-column sections (1-3 columns) with a per-field full-width span; (3) placing a custom relationship\'s related-records list on a specific tab, with anything unclaimed still showing in an always-visible spot; (4) the same published layout now also drives the record\'s read-only detail/Overview view, not just the edit form - a custom field an admin added is now visible there too, something no detail page could show before this','Desktop gained a real Screen layouts builder for the first time (it previously had none at all - the online demo\'s was the only one); the demo\'s builder gained the tabs/columns/related-lists/detail-view capabilities its Phase-1-only version didn\'t have','<b>Dashboard customization</b> Phase 1 (desktop): Admin → Dashboards lets an Administrator build multiple named dashboard layouts - each an ordered list of KPI-tile widgets - and assign them by role, with a required Default fallback, reusing the exact same draft/publish/role-resolution model as Screen/App Builder. No dashboard published yet falls back to exactly the dashboard that existed before this feature, unchanged. Chart and record-list widgets, and an online demo mirror, are next','See <a href="/platform">what you can build</a> with these primitives - they\'re the same building blocks a whole custom business app is made from, not just form customization']],['v0.25.0','August 2026','Admin landing page redesign, plus duplicate/version history/dependency warnings for Business Rules and Workflow Automation',['Admin no longer opens straight into a flat 12-tab row - clicking Admin now opens a categorized landing page (Workspace, Access, Customization, Automation, Integrations), and clicking an item inside a category opens straight into that builder with a breadcrumb (Dashboard > Admin > tool) back to the landing page. The sidebar Admin icon always resets to the landing page, the same as clicking Setup in Salesforce always reopens Setup Home','Business Rules and Workflow Automation both gained a Duplicate action - clones a rule/workflow as an inactive copy and opens it for review before it can affect anything live','Both builders also gained a bounded version history (last 10 saves): every edit snapshots the rule\'s prior state, viewable from a History button with a one-click Restore per version - restoring itself snapshots the state it\'s replacing, so a restore is never a dead end','Deactivating a custom field that an active business rule or workflow rule still reads or writes now shows a confirmation listing exactly which active rules reference it, instead of silently breaking them the next time they evaluate']],['v0.24.1','August 2026','Mobile layout fix for record detail pages, and a stale favicon fix',['The new Products/Quotes/Orders/Invoices/Contracts/Tasks record detail pages (shipped in v0.24.0) overflowed horizontally on a phone - reported directly from mobile testing against the live demo. The line-items table wasn\'t wrapped in the same `.table-wrap` (overflow:auto) pattern every other data table in the app already uses, and the surrounding grid/header layout sized to that table\'s intrinsic minimum width instead of the actual viewport - a classic CSS grid/flex "min-width:0" gap. Fixed both; Company/Contact 360 (unaffected by the bug) and every other detail page now match viewport width exactly on mobile, with no change to desktop-width layout','Fixed the browser tab favicon, which still drew a "B" glyph left over from when the product was called BusinessOS (renamed to Lanesra OS in v0.5.0) - it now draws an "L" matching the in-app sidebar mark']],['v0.24.0','August 2026','Multi-condition OR-groups and a full action palette for Business Rules & Workflow Automation, on desktop and in the online demo',['Both engines\' conditions gained one level of nested OR-grouping on top of the existing AND/OR: a rule can now express "A AND (B OR C)", not just a flat AND or a flat OR - the builders show a dashed "OR group" box with a "+ OR group" control alongside "+ Add condition"','Business rule actions expanded from require/hide/lock/set-default/set-value/block-save/show-message to the full palette: Show and Make editable (explicit counterparts to Hide/Lock, most useful together with a new "Hide by default" flag on a custom field), Clear field value, Restrict choices (narrows a select field\'s options while the rule matches), and a severity split of the old generic message into Show error/Show warning - "Effects" is renamed "Action" throughout both builders','A custom field can now be flagged "Hide by default" - left off every create/edit form unless a business rule\'s Show action currently targets it, enforced server-side on desktop (a hidden-by-default+required field can never block a save) and mirrored client-side in the demo','Workflow automation gained two new field-behavior actions - Set default value (only fills a field if currently empty) and Clear a field - and, in the online demo specifically, an optional "extra conditions" section (AND/OR, same OR-groups) evaluated once the trigger itself already fired, plus true multi-action workflows (previously one action per rule)','"Trigger approval" from the design mockup is deliberately not included in this round','Desktop: Rust core (migration 0020), 8 new tests, full workspace suite green; React admin screens for both builders rebuilt to match. Online demo: business rules and workflow automation rebuilt from single-condition/single-action to fully array-based, with an in-place upgrade of any rule saved before this release - existing rules keep evaluating exactly as before until edited through the new builder']],['v0.23.1','August 2026','Online demo parity fix: workflow self-updates and custom-object workflows',['Workflow automation gained the desktop edition\'s update_field action (the companion to update_related_record) - "when this record\'s field changes, also set another field on this same record" - e.g. "when Status becomes Customer, set Industry to Active." The new value can be a fixed value or copied live from another field on the same record','Fixed a bug where a workflow rule defined on an admin-defined Custom Object could be created in Admin → Workflow Automation but would silently never fire - execution was gated on a built-ins-only lookup table left over from before Custom Objects existed as a workflow-eligible entity; workflows on custom objects now run exactly like on any built-in entity','Both fixes verified against the exact reported scenario (Company status change auto-updating a second field on the same Company) and against a custom object end to end']],['v0.23.0','August 2026','Integrations admin section in the online demo',['A new Admin → Integrations section - another capability that doesn\'t exist on the desktop edition, built for the demo first, as a UI-only simulation: the static demo has no server, so nothing here makes a real network call or runs on a real schedule','Scheduled jobs: define a data Export/Import/Sync job against any built-in or custom object with a schedule (manual, hourly, daily, weekly) and format (CSV/JSON); Run now simulates it immediately, with a per-job history log of simulated runs and record counts','API endpoints: define and "expose" a GET or POST endpoint backed by any object, with API-key or public auth; Test call shows the exact request and a realistic JSON response built from your own current demo data, entirely local','External API connections: configure an outbound connection (base URL, method, none/API-key/bearer auth) this workspace would call; Test request logs a simulated response with a call history, clearly labeled as simulated since the demo can\'t make real outbound requests','With this, the online demo has closed every gap identified in this round of parity work: Custom Objects, Custom Relationships, Reports, a no-code Screen layouts designer, and now Integrations - all four built for the demo, two catching up to desktop and two (layouts, integrations) new to the product entirely']],['v0.22.0','August 2026','No-code Screen layouts in the online demo',['A new Admin → Screen layouts tab lets an admin drag-order any built-in or custom object\'s create/edit fields into named sections - a capability that doesn\'t exist on the desktop edition either, built for the demo first','A layout has a draft and a published copy: dragging fields between/within sections, renaming a section, or adding one only ever edits the draft - the live create/edit form keeps using the plain default field order until Publish copies the draft over, and Unpublish clears it straight back to that default','A Preview button renders the draft exactly as the live form would show it, without saving anything or touching the live workspace','A published layout never drops a field it doesn\'t recognize - any field missing from the layout (a new custom field added after publishing, or a stale key) is automatically appended to a trailing "Other fields" section so nothing can go missing from a form because of a layout edit','With this, the online demo\'s only remaining gap against the desktop edition\'s admin extensibility spec is the Integrations admin section - up next, also a demo-first UI-only build']],['v0.21.0','August 2026','Reports in the online demo',['The online demo has a new top-level Reports section with the same fixed report gallery the desktop edition ships: Revenue by month, Win rate by owner, Lost reasons, AR aging and Sales by owner, each with a date-range (or "as of") filter and a bar-chart table matching desktop\'s layout','Added a Custom Reports builder to the demo, mirroring desktop\'s admin report builder: pick any object - built-in or a Custom Object - group by its status/stage or an active, reportable custom field, and count records or sum a numeric custom field; only fields an admin flagged Reportable are offered, same as desktop','Added CSV export to every report in the demo (a new capability for the demo generally, self-contained via a Blob download - no server involved)','Two disclosed substitutions where this demo\'s simpler data model doesn\'t match desktop\'s: Revenue by month/Sales by owner group by each invoice\'s due date since the demo has no separate issue date, and AR aging uses each invoice\'s full total as its balance since the demo doesn\'t track partial payments - both called out in the report\'s own subtitle rather than silently faked','Added a Lost reason field to Opportunities so the Lost Reasons report has something real to report on','With this, the online demo\'s only remaining gaps against the desktop edition are two capabilities that don\'t exist on desktop either: a no-code UI layout designer and a UI-only Integrations admin section - both underway next']],['v0.20.0','August 2026','Custom Relationships in the online demo',['An Administrator can now connect any two object types - built-in or custom - from Admin → Relationships: a cardinality (many-to-one, one-to-one or many-to-many), a forward/reverse label pair, and a choice of what happens to a link when a linked record is deleted (Restrict blocks the delete, Archive drops the link and keeps both records)','Every record\'s edit form now shows a "Related records" panel listing every linked record across every applicable relationship, from either direction, with inline Link/Unlink - the same place desktop puts its related-records card, since most objects in this demo have an edit form but no separate detail page','Cardinality is enforced on link (a many-to-one or one-to-one side can\'t be linked twice), and a relationship can\'t connect an object type to itself - both match the desktop edition\'s validation exactly','Reports beyond the dashboard remain the one desktop-only capability left in the online demo\'s parity work']],['v0.19.0','August 2026','Custom Objects in the online demo',['The online demo caught up with one of the desktop edition\'s biggest capabilities: an Administrator can now define a whole new business object at runtime - Vendors, Assets, Projects - from Admin → Custom Objects, with its own icon, sidebar entry and record-number prefix/digit width, no code change','A custom object is a full citizen of the demo\'s admin subsystems exactly like a built-in entity: it gets its own tab in Custom Fields, Business Rules, Status Transitions and Workflow Automation, and its records go through the same create/edit/list screens, auto-numbering and delete-dependency checks as Companies or Contacts','A custom object can\'t be named the same as a built-in entity (rejected at creation, matching desktop); deleting its definition is blocked while any record still exists, while deactivating is always safe and reversible since it only hides the object from navigation and new-record creation','Reports beyond the dashboard and Custom Relationships between record types remain desktop-only for now - next up in the online demo\'s parity work']],['v0.18.1','August 2026','Online demo parity fixes',['Workflow "Create a new record" now offers all 9 built-in record types in the online demo (companies, contacts, opportunities, products, quotes, orders, invoices, contracts, tasks), not just 3 - matching the desktop edition\'s full creatable set, with company-dependent types offered only when the trigger record actually carries a company','Workflow "Update a related record" now walks the demo\'s foreign-key graph in both directions, not just downward: a Contact can update its own parent Company the same way a Company can update its Contacts, and every entity gets its linked Tasks (and vice versa) through the existing relatedType/relatedId link','Custom fields in the online demo gained the validation and capability settings the desktop edition already had: Required, Max length and Pattern/regex (text fields), Min/Max value (number fields), and Searchable/Filterable/Reportable flags - enforced with native HTML5 form validation and shown in the custom fields list']],['v0.18.0','August 2026','Status transitions, richer workflow actions, test mode, a rule-builder redesign, and Customer 360',['Added a Status Transition Editor: restrict which status/stage changes are allowed on any object with a fixed-schema field (companies, contacts, opportunities, products, quotes, orders, invoices, contracts, tasks) - each rule is one from → to move, with a wildcard "any status" starting point and its own active toggle; with no active rules a field stays fully unrestricted, and resaving the same status is never blocked','Workflow automation actions expanded beyond "create a task": a workflow can now create a new record (a company, opportunity or task) or update a field on a record related to the trigger through the demo\'s existing company/contact/opportunity/quote/order relationships','Added a Test rule / Test workflow dry-run mode to both Business Rules and Workflow Automation: fill in hypothetical values for an object and see exactly which active rules or workflows would match and what they would do, without creating, changing or sending anything','Redesigned the Business Rules and Workflow Automation builders to match the desktop edition\'s rule-builder layout: numbered Condition/Effect (or Trigger/Action) sections, a live-updating rule summary panel, and - for workflows - a visual Trigger → Action → End canvas that mirrors the form as you edit it; both builders gained full editing (not just create) and header-level Test/Activate-Deactivate/Save controls','Custom fields gained four more settings: an optional default value applied whenever a save leaves the field empty, a "require a unique value" check (rejected at definition time for yes/no fields, since they only have two possible values), placeholder text, and help text shown under the field on every record form','Added Customer 360 and Contact 360: clicking a company or contact name anywhere in the app now opens a dedicated detail page with its full field overview and every linked record - contacts, opportunities, quotes, orders, invoices, contracts and tasks - each one click away, replacing edit-modal-only access','Fixed a pre-existing bug surfaced while building the above: the admin panel\'s tab row no longer freezes on whichever tab was open first - switching tabs now correctly highlights the one you\'re on']],['v0.17.0','August 2026','More operators & field-to-field comparison',['Business rules and workflows gained four more comparison operators - starts with, ends with, is one of, is not one of - on top of is/is not/contains/is empty/is not empty/greater than/less than','A condition can now compare a field against another field\'s live value instead of only a fixed value - e.g. "require Flag when Notes equals Expected Notes" - with the same live-updating preview on the record form that a fixed-value condition already had','Windows desktop edition: the shared condition engine gained the same operators and field-to-field comparison, for both business rule conditions and workflow triggers']],['v0.16.0','August 2026','Business rules & workflows now work on any field, not just status',['Business rules can now condition on any built-in field - name, industry, value, close date, whatever the object has - not only the status/stage field, with a real comparison operator (is/is not, contains, is empty, greater than, less than) chosen per field, and their require/hide action can now target a built-in field too, not just a custom one','Workflow automation\'s field-changed trigger can now watch any built-in field the same way, so "when Industry changes to X" or "when Due date is set" can create a task and notify admins, not only "when status/stage reaches a value"','Windows desktop edition: the underlying business rules and workflow engines gained the same any-built-in-field support for both conditions/triggers and actions (require, hide, lock, set default, force value, and the workflow update-field action), writing through each entity\'s own validation so nothing bypasses existing rules']],['v0.15.0','August 2026','Custom relationships, richer business rules & workflow automation',['Added admin-defined custom relationships between any two record types (companies, contacts, custom objects, and more), with one-to-one/many-to-one/many-to-many cardinality and a choice of what happens to linked records on delete','Added a related-records view on record detail pages showing every linked record through those relationships','Replaced the business rules engine: rules can now combine multiple conditions with AND/OR, use 10 comparison operators (not just equals), and lock a field, set a default or exact value, block saving entirely, or show a message — not just require or hide','Replaced the workflow automation engine: triggers now include field changes and dates reached/overdue in addition to status changes, and actions include assigning the record\'s owner, creating a related record, and posting an in-app notification, on top of creating a task','Added an in-app notification center (bell icon with unread count) for workflow-triggered notifications','Added optional validation for custom fields — a min/max range for number fields, a max length and regex pattern for text fields — plus searchable/filterable/reportable capability flags','Added Windows task reminder notifications (native toast notifications via the desktop app\'s webview)','Added a session inactivity auto-lock (15 minutes idle) requiring the current user\'s password to resume','Updated the online demo: business rules now support an "is / is not" operator, and workflow rules can optionally post an admin notification, shown in a new notification bell']],['v0.14.0','August 2026','Admin-defined Custom Objects',['Added Custom Objects: an Administrator can define an entirely new record type (its own label, fields and ID/numbering format) without any code changes','Custom Objects automatically get their own navigation section, and are full citizens of the existing custom fields, business rules and custom report builder — no per-object code was needed for any of the three','A custom object can\'t be named the same as a built-in entity, and deleting its definition is blocked while records exist (deactivating it is always safe and non-destructive)']],['v0.13.0','August 2026','Admin panel: users, roles & flexible configuration everywhere',['Added an Admin panel with user & role management, moved out of the main navigation into one dedicated section','Added an editable business profile (name, phone, address, city, logo) shown across the workspace','Generalized custom fields from Companies/Contacts to every major object: Opportunities, Quotes, Orders, Invoices, Contracts, Products and Tasks','Generalized conditional business rules and workflow automation the same way, so any object with custom fields can use them','Added admin-configurable numbering: choose the prefix and digit width used for each object\'s auto-generated ID (e.g. "ACC-000001" or "ACC-ab0001")','Added a simple custom report builder: pick an object, group by any field including custom fields, and count or sum','Added a dashboard KPI picker so admins choose which tiles show, in what selection, for the whole workspace','Updated the online demo with a full working Admin panel — mirrors every feature above in the browser']],['v0.12.0','August 2026','Branding, reports, custom fields, business rules & workflow automation',['Added business branding (logo, editable business profile) shown on the print letterhead for quotes, orders and invoices','Added reports beyond the dashboard: revenue by month, win rate by owner, lost reasons, AR aging and sales by owner','Added admin-defined custom fields on Companies and Contacts (text, number, date, yes/no, select), enforced both client- and server-side','Added conditional business rules that require or hide a custom field based on a record\'s status','Added Phase 1 workflow automation: auto-create a follow-up task when an Opportunity\'s stage or an Invoice\'s status changes']],['v0.11.0','August 2026','PDF printing & CSV import/export',['Added a browser-native "Print / Save as PDF" preview for quotes, orders and invoices, with business letterhead, line items and totals','Added CSV export on every list screen','Added CSV import for Companies and Contacts, validated row by row through the same rules as the manual forms']],['v0.10.0','August 2026','Team Workspace, backup & restore',['Added Team Workspace mode — a small team shares one server over the local network from browser tabs, with per-user sessions','Added whole-workspace backup and restore as a single file, safe to run against a live database','Added self-service password change from a "My account" screen']],['v0.9.0','August 2026','Desktop edition foundation published',['Published the Windows desktop edition source: Tauri v2 + Rust + SQLite','Implemented the full sales lifecycle on desktop — Companies, Contacts, Products, Opportunities, Quotes, Orders and Invoices','Added quote-to-order and order-to-invoice conversion, atomic document numbering and local user authentication','No packaged installer yet — desktop is available to build and run from source']],['v0.8.0','August 2026','Interactive navigation & public pages',['Made dashboard KPIs clickable with filtered drill-downs','Added a global Quick Create menu','Added mobile navigation while keeping Try Online prominent','Replaced Journey with Principles and added Compare and Download pages','Marked desktop downloads as Coming Soon','Fixed desktop sidebar navigation']],['v0.7.0','August 2026','Trust & product transparency',['Added Roadmap, Changelog and creator attribution','Added Person JSON-LD and updated discovery files']],['v0.6.0','August 2026','Record numbering & search',['Added automatically generated identifiers','Rebuilt global search as one stable result panel','Added keyboard shortcuts and wider search coverage']],['v0.5.0','July 2026','Lanesra OS rebrand',['Renamed BusinessOS to Lanesra OS','Updated product branding, metadata and documentation']],['v0.4.0','July 2026','Relationship integrity',['Added opportunity-to-contact relationship','Removed opportunity-to-contract relationship','Added company-filtered relationship dropdowns']],['v0.3.0','July 2026','Flexible sales flow',['Made opportunities optional for quotes','Made quotes optional for orders','Added products, services and line-item quantities']],['v0.2.0','June 2026','Connected sales MVP',['Added quotes, orders, invoices, contracts and dashboards','Connected core entities using clean relationships']],['v0.1.0','May 2026','First working prototype',['Launched the first browser-based MVP with sample data']]];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Release history</div><h1>Releases</h1><p>Every meaningful improvement to Lanesra OS, with detailed per-version release notes — documented publicly.</p><div class="status-row"><span class="status-chip">Latest: v0.36.0</span><span class="muted">Early Access</span></div><div class="backlog-callout" id="desktop-installer"><h3>Desktop installer</h3><p><b>desktop-v0.12.0 is the latest tagged Windows installer</b> on <a href="https://github.com/vikram2409-eng/Lanesra-OS/releases" target="_blank" rel="noopener">GitHub Releases</a> — cut alongside v0.35.0 below (Integration Hub); v0.36.0's Saved Views & Bulk Actions has since merged to <code>main</code> but hasn't been cut into a new installer tag yet. The two numbering schemes are deliberately not 1:1: version numbers on this page track every meaningful improvement across the desktop app, the online demo and this website; <code>desktop-v*</code> tags on GitHub are cut only when there's a new installer build worth shipping, so they move less often. Full desktop feature list → <a href="/download">/download</a>, full desktop release-status detail → <a href="/roadmap#desktop">/roadmap#desktop</a>.</p></div></div></section><section class="section"><div class="container changelog-list">${releases.map(r=>`<article class="release" id="${r[0].replaceAll('.','-')}"><div class="release-meta"><span class="status-chip">${r[0]}</span><span>${r[1]}</span></div><div><h2>${r[2]}</h2><ul>${r[3].map(x=>`<li>${x}</li>`).join('')}</ul></div></article>`).join('')}</div></section></main>${publicFooter()}`;bindPublicNav()}
function platformPage(){
 document.title='Platform — Lanesra OS';
 const primitives=['Custom Objects','Relationships','Screen/App Builder','Business Rules','Workflow Automation','Dashboards','Reports'];
 const apps=[
  {
   eyebrow:'Real estate & property management',
   name:'Property Management',
   tagline:'Properties, units, leases and tenants, connected the same way Companies and Quotes already are.',
   details:[
    'Custom Objects: Properties, Units, Leases, Tenants',
    'Relationships: Property → Units (one-to-many), Unit → Lease (one-to-one), Lease → Tenant (many-to-one)',
    'Screen/App Builder: a Property layout with Units and Leases as related-list tabs, so opening a property shows every unit\'s occupancy at a glance',
    'Workflow Automation: a reminder task 60 days before a lease\'s end date',
    'Dashboard: an occupancy-rate tile and a leases-expiring-this-quarter list',
   ],
   builtFrom:['Custom Objects','Relationships','Screen/App Builder','Workflow Automation','Dashboards'],
   inCatalog:true,
  },
  {
   eyebrow:'Talent & hiring',
   name:'Recruitment / ATS',
   tagline:'A candidate pipeline with the same stage-based rigor Opportunities already give the sales team.',
   details:[
    'Custom Objects: Candidates, Job Openings, Applications, Interviews',
    'Relationships: Candidate ↔ Job Opening through an Application join object (many-to-many), Application → Interviews (one-to-many)',
    'Business Rule: require a rejection reason before an Application\'s stage can move to Rejected',
    'Workflow Automation: notify the hiring manager the moment an Application\'s stage changes',
    'Dashboard: applications by stage, same shape as the built-in Sales Pipeline view',
   ],
   builtFrom:['Custom Objects','Relationships','Business Rules','Workflow Automation','Dashboards'],
   inCatalog:true,
  },
  {
   eyebrow:'Services & delivery',
   name:'Professional Services',
   tagline:'Projects and billable time, linked straight back to the Company that\'s paying for them.',
   details:[
    'Custom Objects: Projects, Milestones, Time Entries',
    'Relationships: Project → Company (many-to-one, reusing the built-in Companies object directly), Project → Milestones and → Time Entries (one-to-many)',
    'Screen/App Builder: a Project layout with a Time Entries related list right on the record, no separate timesheet screen to jump to',
    'Reports: hours logged by project and by consultant, using the same group-and-sum report builder Sales Pipeline reports already use',
    'Workflow Automation: flag a Milestone as at-risk when its due date passes with open Time Entries still against it',
   ],
   builtFrom:['Custom Objects','Relationships','Screen/App Builder','Workflow Automation','Reports'],
   inCatalog:true,
  },
  {
   eyebrow:'Operations & equipment',
   name:'Asset Tracking',
   tagline:'Every piece of equipment, where it lives, and when it\'s due for service.',
   details:[
    'Custom Objects: Assets, Locations, Maintenance Records',
    'Relationships: Asset → Location (many-to-one), Asset → Maintenance Records (one-to-many)',
    'Workflow Automation: on a schedule, create a maintenance task for any asset that\'s gone N days since its last service record',
    'Business Rule: require a Location before an Asset can be marked Active',
    'Dashboard: assets due for service this month, and a by-location breakdown',
   ],
   builtFrom:['Custom Objects','Relationships','Business Rules','Workflow Automation','Dashboards'],
  },
 ];
 $('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">One platform, any business</div><h1>Lanesra OS isn't only a CRM. It's what your CRM is built on.</h1><p>Companies, Contacts, Opportunities, Quotes, Orders, Invoices, Contracts and Tasks — the sales system you get out of the box — aren't hardcoded into Lanesra. They're one app built from the same platform every workspace has underneath it. An Administrator gets the identical building blocks, from a settings screen, to model whatever your organization actually runs on.</p><div class="hero-actions"><a class="btn btn-primary" href="/demo">Try the live demo →</a><a class="btn btn-secondary" href="#examples">See what you could build ↓</a></div></div></section>
 <section class="section"><div class="container narrow"><div class="section-head" style="text-align:center;margin:0 auto 8px"><div class="eyebrow">The building blocks</div><h2>Seven primitives. One workspace.</h2><p class="muted">Every example below is assembled from these — nothing industry-specific is baked into the product itself.</p></div><div class="platform-primitives">${primitives.map((p,i)=>`<span class="platform-primitive"><span class="n">${i+1}</span>${p}</span>${i<primitives.length-1?'<span class="platform-arrow">→</span>':''}`).join('')}</div></div></section>
 <section id="examples" class="section" style="background:var(--surface-alt,#f7f8fc)"><div class="container"><div class="section-head"><div class="eyebrow">Make it real</div><h2>What this actually becomes.</h2><p class="muted">Four businesses that look nothing like a CRM, each modeled entirely from Admin → Custom Objects, Relationships, Screen layouts, Business rules, Workflow automation and Dashboards. Three of the four are no longer just examples — install them in a few clicks from Admin → App Catalog instead of building them by hand.</p></div><div class="app-examples">${apps.map(a=>`<article class="app-example-card"><div class="eyebrow">${a.eyebrow}${a.inCatalog?' · <span class="badge">In App Catalog</span>':''}</div><h3>${a.name}</h3><p class="tagline">${a.tagline}</p><ul class="example-detail">${a.details.map(d=>`<li>${d}</li>`).join('')}</ul><div class="built-from">${a.builtFrom.map(b=>`<span>${b}</span>`).join('')}</div>${a.inCatalog?'<p class="muted" style="margin-top:10px">Install it as-is from Admin → App Catalog, or use it as a starting point and customize freely.</p>':''}</article>`).join('')}</div></div></section>
 <section class="section"><div class="container narrow"><div class="honesty-note"><h3>What's real today, and what's next</h3><p>Every building block used above — Custom Objects, Relationships, Screen/App Builder, Business Rules, Workflow Automation, Dashboards, Reports — is shipped and working right now, in both the desktop app and the <a href="/demo">online demo</a>. Open Admin → Custom Objects and you can start building any of these examples yourself, this minute.</p><p>You don't have to build them by hand anymore, though: the <b>App Catalog</b> (Admin → Apps → App Catalog) ships 10 complete, install-ready industry apps — Field Service, Property Management, Construction & Contractors, Professional Services, Practice Administration, Recruitment & Staffing, Real Estate Brokerage, Legal Practice, Nonprofit & Association Management and Auto Repair & Service Garage. Installing one runs a validated, backed-up, transactional install straight into your workspace, reusing your existing Companies and Contacts rather than creating a parallel database, and a plain <b>App Builder</b> (Admin → Apps) still groups any set of objects into your own named, publishable application with its own icon, a sidebar App Switcher and access grants.</p><p>Two more layers sit on top, both shipped on the desktop edition: <b>Solution Management</b> (Admin → Solution Management) lets you curate a named, versioned Solution from anything you've built, export it, and import it into another workspace - a Publisher registry, component-tagging and update-with-diff included - so your customizations move between test and production the way a real software vendor would ship an update. The <b>Integration Hub</b> (Admin → Integration Hub) connects that same workspace to everything else your business runs: encrypted Connections, OpenAPI-imported Connectors, a generic REST API secured by scoped keys, HMAC-signed webhooks, a CSV data-exchange wizard, and scheduled Integration Jobs. Neither requires a separate integration platform or ISV tooling subscription — <a href="/roadmap">see what's next on the roadmap</a>.</p></div></div></section>
 <section class="section"><div class="container cta"><h2>Go build something.</h2><p style="color:#cbd5e1;max-width:640px;margin:0 auto 24px">The live demo already has Admin → Custom Objects, Relationships, Screen layouts, Business rules, Workflow automation, Dashboards and Apps turned on. No account required.</p><div class="hero-actions" style="justify-content:center"><a class="btn btn-primary" href="/demo">Try the live demo →</a><a class="btn btn-secondary" href="/roadmap">View the roadmap</a></div></div></section>
 </main>${publicFooter()}`;
 bindPublicNav();
}
function principlesPage(){document.title='Principles — Lanesra OS';const principles=[['Own your data','Your customer and sales information should remain under your control—not trapped behind a subscription or vendor lock-in.'],['Offline first','Core work should continue even when the internet does not. The Windows desktop edition runs entirely on local SQLite storage, with no server or account required.'],['Relationships over spreadsheets','Customers, contacts, opportunities, quotes, orders and invoices stay linked so data remains clean and useful — and that same connected model extends to any custom record type you define.'],['Simple before powerful','Every feature must reduce effort. Complexity is added only when it clearly improves the work.'],['Configurable, not hardcoded','A business shouldn\'t need a developer to add a field, a record type, a rule or an automation. Admins reshape Lanesra from a settings screen — the software adapts to the business, not the other way around.'],['Open by default','The product roadmap, backlog, release notes and source code are all public so users can inspect how Lanesra evolves.'],['Business software deserves good design','Small businesses should not have to accept dated interfaces or confusing navigation to access serious capabilities.']];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">How Lanesra is designed</div><h1>Principles before features.</h1><p>The decisions behind Lanesra OS are guided by a small set of practical beliefs about ownership, simplicity and product quality.</p></div></section><section class="section"><div class="container principles-page-grid">${principles.map((p,i)=>`<article class="principle-card"><span>0${i+1}</span><h2>${p[0]}</h2><p>${p[1]}</p></article>`).join('')}</div></section><section class="section maintenance"><div class="container narrow"><div class="eyebrow">The business flow</div><h2>Connected by design.</h2><div class="flow-map"><strong>Customer</strong><span>→</span><div>Contacts<br>Opportunities <em>optional</em><br>Quotes <em>optional</em><br>Orders<br>Invoices<br>Contracts<br>Tasks</div></div><p class="muted" style="margin-top:18px">That same connected model isn't fixed to these nine record types — admins can add their own (Vendors, Assets, Projects…) and link them into this graph with custom relationships, so the "no dangling free text" principle holds for whatever your business actually looks like.</p></div></section></main>${publicFooter()}`;bindPublicNav()}
function comparePage(){document.title='Compare — Lanesra OS';const rows=[['Runs without internet','Partial','No','No','Yes'],['Open source','No','No','No','Yes'],['Local database','No','No','No','Yes (desktop)'],['Mandatory subscription','No','Yes','Yes','No'],['Connected sales workflow','Manual','Limited','Advanced','Yes'],['Custom record types, no code','No','Limited, paid tiers','Yes, complex/paid','Yes, built in'],['Custom screens & layouts by role','No','Limited','Yes, complex/paid','Yes, built in'],['Custom business rules & workflow automation','No','Paid tiers','Yes, needs admin training','Yes, built in'],['Build a whole custom app on the platform','No','No','Yes, paid tiers','Yes, built in'],['Ready-made industry apps to install, not just build','No','No','AppExchange, paid','Yes, 10 built in'],['Package & promote customizations, test → production','No','No','Yes, paid tiers (Change Sets/DevOps Center)','Yes, built in'],['Native REST API, webhooks & scheduled sync','No','Paid tiers','Yes, paid tiers/limits','Yes, built in'],['Designed for small business','General','Yes','Enterprise','Yes'],['Self-owned business data','File-based','Cloud-hosted','Cloud-hosted','Yes']];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Choose with context</div><h1>Where Lanesra fits.</h1><p>A factual comparison for small businesses deciding between spreadsheets, cloud CRMs and a local-first open-source system.</p></div></section><section class="section"><div class="container compare-wrap"><table class="compare-table"><thead><tr><th>Capability</th><th>Excel</th><th>HubSpot</th><th>Salesforce</th><th class="lanesra-col">Lanesra OS</th></tr></thead><tbody>${rows.map(r=>`<tr>${r.map((x,i)=>`<td class="${i===4?'lanesra-col':''}">${x}</td>`).join('')}</tr>`).join('')}</tbody></table><p class="compare-note">Comparisons are intentionally high-level. Product capabilities and commercial terms can change; review each vendor's current documentation before making a purchase decision.</p></div></section></main>${publicFooter()}`;bindPublicNav()}
function downloadPage(){document.title='Download — Lanesra OS';$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Local-first desktop edition</div><h1>Download Lanesra OS.</h1><p>The independent desktop edition runs locally with no cloud account or mandatory internet connection. It is in active early development, with an Early Access Windows installer now available.</p></div></section><section class="section"><div class="container download-grid"><article class="download-card featured"><span class="status-chip">Early access — installer available</span><h2>Windows</h2><p>Tauri + Rust + SQLite desktop app with the full sales lifecycle, Contracts, Tasks and user management working: Companies, Contacts, Products, Opportunities, Quotes, Orders, Invoices, Contracts and Tasks — plus Team Workspace mode for small teams, backup and restore, PDF printing, CSV import/export, admin-defined Custom Objects and relationships between any record types, and an Admin panel covering branding, user roles, custom fields, richer conditional business rules, richer workflow automation with in-app notifications, configurable ID formats, a Screen/App Builder for every object's create/edit and detail layouts, named dashboard layouts assigned by role with chart and record-list widgets, App Builder — grouping objects, screens and a dashboard into a named, publishable app with server-enforced Viewer/Editor access — and an App Catalog of 10 install-ready industry apps (Field Service, Property Management, Construction & Contractors, Professional Services, Practice Administration, Recruitment & Staffing, Real Estate Brokerage, Legal Practice, Nonprofit & Association Management, Auto Repair & Service Garage), each installed with pre-install validation, an automatic backup and a transactional, rollback-safe install. On top of that, Solution Management (Publishers, versioned Solutions, component-tagging, export/import and update-with-diff) lets you package and promote your own customizations between workspaces, and an Integration Hub connects the workspace to everything else you run - encrypted Connections, OpenAPI Connectors, a generic REST API, HMAC-signed webhooks, CSV data exchange and scheduled Integration Jobs. Unsigned .exe and .msi installers are on GitHub Releases (Windows will warn on first run since they aren't code-signed yet).</p><a class="btn btn-primary" href="https://github.com/vikram2409-eng/Lanesra-OS/releases" target="_blank" rel="noopener">Download for Windows</a><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS/tree/main/desktop" target="_blank" rel="noopener">View desktop source on GitHub</a></article><article class="download-card"><span class="status-chip">Planned</span><h2>macOS</h2><p>Apple silicon and Intel packaging will follow the Windows early-access release.</p><button class="btn btn-secondary" disabled>Planned</button></article><article class="download-card"><span class="status-chip">Planned</span><h2>Linux</h2><p>AppImage or Debian packaging is planned after the initial desktop release stabilizes.</p><button class="btn btn-secondary" disabled>Planned</button></article></div></section><section class="section maintenance"><div class="container narrow"><h2>What the desktop edition includes today</h2><div class="download-checks"><span>✓ No licence key</span><span>✓ No cloud account</span><span>✓ Standard SQLite database</span><span>✓ Offline from first launch</span><span>✓ Full sales lifecycle (quotes → orders → invoices)</span><span>✓ Contracts and tasks</span><span>✓ User management</span><span>✓ Team Workspace mode for small teams (Docker)</span><span>✓ Windows installer (unsigned, Early Access)</span><span>✓ Backup and restore</span><span>✓ Self-service password change</span><span>✓ PDF generation and printing</span><span>✓ CSV import and export</span><span>✓ Branding and print customization</span><span>✓ Reports, plus a custom report builder</span><span>✓ Custom fields & business rules on every object</span><span>✓ Workflow automation with in-app notifications</span><span>✓ Admin-defined Custom Objects</span><span>✓ Custom relationships between record types</span><span>✓ Screen/App Builder — layouts by role, Draft → Publish</span><span>✓ Dashboard customization — KPI, chart & record-list widgets by role</span><span>✓ App Builder — publish named apps with server-enforced access</span><span>✓ App Catalog — 10 install-ready industry apps</span><span>✓ Per-app scoped business rules, workflows & dashboards</span><span>✓ Solution Management — Publishers, versioned Solutions, export/import</span><span>✓ Integration Hub — Connections, Connectors, REST API, webhooks, scheduled sync</span><span>✓ Windows task reminder notifications</span><span>✓ Session inactivity auto-lock</span><span>✓ Admin panel: user roles & configurable numbering</span><span>✓ Open-source code</span><span>○ Code-signed installer — planned</span></div><div class="hero-actions"><a class="btn btn-secondary" href="/roadmap#desktop">View desktop roadmap</a><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS/tree/main/desktop" target="_blank" rel="noopener">View source on GitHub</a></div></div></section></main>${publicFooter()}`;bindPublicNav()}
// Every public page sets document.title itself, but only index.html ships
// a static <meta name="description"> - a crawler that fetches /roadmap
// directly (or any route beyond the homepage) would otherwise report the
// homepage's description regardless of which page it actually rendered.
// Call this once per page render with page-specific copy.
function setPageMeta(description){
 let tag=document.querySelector('meta[name="description"]');
 if(!tag){tag=document.createElement('meta');tag.setAttribute('name','description');document.head.appendChild(tag)}
 tag.setAttribute('content',description);
}
// Injects (or replaces) a page-specific JSON-LD block by id, alongside
// index.html's own untouched script tag - multiple structured-data blocks
// per page are valid and let a page-specific schema (here, the roadmap's
// CollectionPage/ItemList) coexist with the site-wide SoftwareApplication
// entity without editing the static HTML file per route.
function setPageJsonLd(id,data){
 let tag=document.getElementById(id);
 if(!tag){tag=document.createElement('script');tag.type='application/ld+json';tag.id=id;document.head.appendChild(tag)}
 tag.textContent=JSON.stringify(data);
}
function bindPublicNav(){document.querySelectorAll('.menu-toggle').forEach(btn=>{btn.onclick=()=>{const nav=btn.closest('.landing-nav');const drawer=nav.querySelector('.mobile-drawer');const open=drawer.hasAttribute('hidden');if(open)drawer.removeAttribute('hidden');else drawer.setAttribute('hidden','');btn.setAttribute('aria-expanded',String(open));btn.textContent=open?'×':'☰'}});document.querySelectorAll('.mobile-drawer a').forEach(a=>a.addEventListener('click',()=>{const drawer=a.closest('.mobile-drawer');drawer.setAttribute('hidden','');const btn=drawer.closest('.landing-nav').querySelector('.menu-toggle');btn.textContent='☰';btn.setAttribute('aria-expanded','false')}))}
const path=location.pathname.replace(/\/$/,'')||'/';
// /backlog and /changelog are retired URLs - Roadmap absorbed the backlog
// content and Changelog was renamed Releases. Netlify 301s these at the
// edge (see _redirects/netlify.toml); this is a client-side fallback for
// anyone who lands on the SPA directly (e.g. a stale bookmark hitting a
// preview deploy without the redirect rules), so old links still work.
if(path==='/backlog'){history.replaceState(null,'','/roadmap');roadmapPage()}
else if(path==='/changelog'){history.replaceState(null,'','/releases');releasesPage()}
else if(path==='/demo')appShell();else if(path==='/roadmap')roadmapPage();else if(path==='/releases')releasesPage();else if(path==='/platform'||path==='/build')platformPage();else if(path==='/principles'||path==='/journey'||path==='/our-story'||path==='/about')principlesPage();else if(path==='/compare')comparePage();else if(path==='/download')downloadPage();else landing();
