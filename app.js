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
  {id:'cf1',entity:'opportunities',key:'leadSource',label:'Lead Source',type:'select',options:'Referral|Website|Event|Cold Outreach|Partner',active:true}
 ],
 fieldRules:[
  {id:'fr1',entity:'opportunities',triggerField:'stage',fieldKey:'leadSource',triggerValue:'Won',effect:'require',operator:'equals'}
 ],
 workflowRules:[
  {id:'wf1',entity:'opportunities',triggerField:'stage',toValue:'Won',taskTitle:'Kick off onboarding',daysOffset:2,notify:true,operator:'equals'},
  {id:'wf2',entity:'invoices',triggerField:'status',toValue:'Overdue',taskTitle:'Follow up on overdue invoice',daysOffset:0,notify:false,operator:'equals'}
 ],
 numberingOverrides:{},
 kpiPrefs:[],
 notifications:[]
};
const storeKey='lanesra-os-demo-v9';
let data = JSON.parse(localStorage.getItem(storeKey)||'null') || structuredClone(seed);
let current='dashboard';
let viewFilter=null;
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
function effectiveRule(key){
 const base=numberRules[key]; if(!base)return null;
 const o=(data.numberingOverrides||{})[key];
 return o?{prefix:o.prefix,width:o.width||base.width,field:base.field,custom:true}:{...base,custom:false};
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
 if(!data.numberingOverrides)data.numberingOverrides={};
 if(!data.kpiPrefs)data.kpiPrefs=[];
 if(!data.notifications)data.notifications=[];
 (data.fieldRules||[]).forEach(r=>{if(!r.operator)r.operator='equals';if(!r.triggerField)r.triggerField=transitionFieldFor(r.entity)});
 (data.workflowRules||[]).forEach(r=>{if(!r.operator)r.operator='equals';if(!r.triggerField)r.triggerField=transitionFieldFor(r.entity)});
 save();
}
const icons={dashboard:'▦',companies:'◫',contacts:'◎',pipeline:'⌁',products:'◇',quotes:'▤',orders:'▣',invoices:'$',contracts:'▧',tasks:'✓'};
const labels={dashboard:'Dashboard',companies:'Companies',contacts:'Contacts',pipeline:'Sales Pipeline',products:'Products',quotes:'Quotes',orders:'Orders',invoices:'Invoices',contracts:'Contracts',tasks:'Tasks'};
// Admin panel: entities that support custom fields & business rules are every object with numbering.
// Workflow automation is limited to the entities Tasks can relate to (see relatedTypeFor).
let adminTab='profile';
let cfEntity='companies';
let ruleEntity='companies';
let wfEntity='companies';
const relatedTypeFor={companies:'Company',contacts:'Contact',opportunities:'Opportunity',quotes:'Quote',orders:'Order',invoices:'Invoice',contracts:'Contract'};
function transitionFieldFor(key){return key==='opportunities'?'stage':'status'}
function fieldsFnFor(key){return {companies:companyFields,contacts:contactFields,opportunities:opportunityFields,products:productFields,quotes:quoteFields,orders:orderFields,invoices:invoiceFields,contracts:contractFields,tasks:taskFields}[key]}
function slugify(label){
 const parts=String(label).trim().split(/[^a-zA-Z0-9]+/).filter(Boolean);
 if(!parts.length)return 'field'+uid();
 return parts[0].toLowerCase()+parts.slice(1).map(w=>w[0].toUpperCase()+w.slice(1).toLowerCase()).join('');
}
function customFieldsFor(entityKey){
 return (data.customFields||[]).filter(f=>f.entity===entityKey&&f.active).map(f=>{
  if(f.type==='boolean')return [f.key,f.label,'select','Yes|No'];
  if(f.type==='select')return [f.key,f.label,'select',f.options||''];
  if(f.type==='number')return [f.key,f.label,'number'];
  if(f.type==='date')return [f.key,f.label,'date'];
  return [f.key,f.label,'text'];
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
const OPERATOR_LABELS={equals:'is',not_equals:'is not',contains:'contains',not_contains:'does not contain',is_empty:'is empty',is_not_empty:'is not empty',greater_than:'is greater than',less_than:'is less than'};
function operatorsForType(type){
 if(type==='select')return ['equals','not_equals'];
 if(type==='number'||type==='date')return ['equals','not_equals','greater_than','less_than','is_empty','is_not_empty'];
 return ['equals','not_equals','contains','not_contains','is_empty','is_not_empty'];
}
function operatorNeedsValue(op){return op!=='is_empty'&&op!=='is_not_empty'}
function operatorMatch(op,value,target){
 const v=value??'', t=target??'';
 switch(op){
  case 'not_equals':return String(v)!==String(t);
  case 'contains':return String(v).toLowerCase().includes(String(t).toLowerCase());
  case 'not_contains':return !String(v).toLowerCase().includes(String(t).toLowerCase());
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
 const rules=(data.fieldRules||[]).filter(r=>r.entity===entityKey);
 if(!rules.length)return;
 function apply(){
  rules.forEach(r=>{
   const triggerField=r.triggerField||transitionFieldFor(entityKey);
   const trigger=form.elements[triggerField]; const input=form.elements[r.fieldKey];
   if(!trigger||!input)return;
   const wrap=input.closest('.field'); const label=wrap?.querySelector('label');
   const match=operatorMatch(r.operator,trigger.value,r.triggerValue);
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
 const watchFields=[...new Set(rules.map(r=>r.triggerField||transitionFieldFor(entityKey)))];
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
 $('#app').innerHTML=`<div class="demo-banner">You are exploring the sample workspace. Changes stay in this browser. <button class="link-btn" id="resetDemo">Reset demo</button><a class="link-btn" href="/">Product website</a></div><div class="app-shell"><aside class="sidebar"><div class="side-brand"><span class="brand-mark">L</span><span>Lanesra OS</span><span class="demo-pill">DEMO</span></div><nav class="side-nav">${Object.keys(labels).map(k=>`<button data-nav="${k}"><b>${icons[k]}</b><span>${labels[k]}</span></button>`).join('')}<button data-nav="admin" class="admin-nav-btn"><b>⚙</b><span>Admin</span></button></nav><div class="side-bottom"><div class="side-meta"><strong>Early Access v0.16.0</strong><div class="side-product-links"><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/roadmap">Roadmap</a><a href="/changelog">Changelog</a></div><span>Created by <a href="https://vikramgrover.com">Vikram Grover</a></span></div><button class="btn btn-secondary" style="width:100%" onclick="location.href='/'">← Website</button></div></aside><main class="app-main"><header class="topbar"><div class="search"><input id="globalSearch" autocomplete="off" placeholder="Search companies, contacts, deals…  ⌘K"><div id="searchResults" class="search-results" hidden></div></div><div class="top-actions"><div class="notif-wrap"><button class="icon-btn" id="notifButton" aria-label="Notifications">🔔<span id="notifBadge" class="notif-badge" hidden></span></button><div id="notifPanel" class="notif-panel" hidden></div></div><button class="icon-btn" id="helpButton" aria-label="Help">?</button><div class="avatar">MC</div></div></header><div class="content" id="view"></div></main></div>`;
 document.querySelectorAll('[data-nav]').forEach(b=>b.onclick=()=>{current=b.dataset.nav;viewFilter=null;renderView()});
 $('#resetDemo').onclick=()=>{data=structuredClone(seed);save();toast('Demo data restored');refreshNotifBadge();renderView()};
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
  searchBox.querySelectorAll('[data-result]').forEach(b=>b.onclick=()=>{const m=shown[Number(b.dataset.result)]; current=m.key==='opportunities'?'pipeline':m.key; closeSearch(); searchInput.value=''; renderView()});
 }
 searchInput.addEventListener('input',runSearch);
 searchInput.addEventListener('keydown',e=>{if(e.key==='Escape'){closeSearch();searchInput.blur()}});
 document.addEventListener('click',e=>{if(!e.target.closest('.search'))closeSearch()});
 document.addEventListener('keydown',e=>{if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='k'){e.preventDefault();searchInput.focus();runSearch()}});
 $('#helpButton').onclick=()=>modal('Help & product links',`<div class="help-list"><a href="/principles">Product principles</a><a href="/compare">Compare Lanesra</a><a href="/roadmap">Roadmap</a><a href="/changelog">Changelog</a><a href="/">Product website</a><button class="btn btn-secondary" onclick="document.getElementById('modal').remove()">Close</button></div>`);
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
function renderView(){
 document.querySelectorAll('[data-nav]').forEach(b=>b.classList.toggle('active',b.dataset.nav===current));
 if(current==='dashboard') return dashboard();
 if(current==='pipeline') return pipeline();
 if(current==='admin') return adminPage();
 const configs={
 companies:{cols:[['customerNumber','Customer ID'],['name','Company'],['industry','Industry'],['city','City'],['owner','Owner'],['status','Status']],fields:()=>fieldsFor('companies',companyFields)},
 contacts:{cols:[['contactNumber','Contact ID'],['name','Contact'],['companyId','Company','company'],['role','Role'],['email','Email'],['status','Status']],fields:()=>fieldsFor('contacts',contactFields)},
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
 document.querySelectorAll('[data-kpi-nav]').forEach(b=>b.onclick=()=>{current=b.dataset.kpiNav;viewFilter=b.dataset.kpiFilter;renderView()});
 document.querySelectorAll('[data-nav2]').forEach(b=>b.onclick=()=>{current=b.dataset.nav2;viewFilter=b.dataset.filter2||null;renderView()});
 const quick=$('#quickNew'),menu=$('#quickMenu'); quick.onclick=e=>{e.stopPropagation();menu.hidden=!menu.hidden};
 menu.querySelectorAll('[data-create]').forEach(b=>b.onclick=()=>{const k=b.dataset.create;menu.hidden=true;if(k==='opportunities')recordModal('opportunities',fieldsFor('opportunities',opportunityFields));else{const fn={companies:companyFields,contacts:contactFields,quotes:quoteFields,orders:orderFields,invoices:invoiceFields,contracts:contractFields,tasks:taskFields}[k];recordModal(k,fieldsFor(k,fn))}});
 document.addEventListener('click',()=>{if(menu)menu.hidden=true},{once:true});
}
function companyFields(){return [['customerNumber','Customer ID','auto'],['name','Company name'],['industry','Industry'],['city','City'],['owner','Owner'],['status','Status','select','Lead|Prospect|Customer|Inactive']]}
function contactFields(){return [['contactNumber','Contact ID','auto'],['name','Full name'],['companyId','Company','relation','companies'],['role','Role'],['email','Email'],['phone','Phone'],['status','Status','select','Active|Inactive']]}
function opportunityFields(){return [['opportunityNumber','Opportunity ID','auto'],['title','Opportunity title'],['companyId','Customer','relation','companies'],['contactId','Primary contact (optional)','filteredContact'],['value','Value','number'],['stage','Stage','select','Lead|Qualified|Discovery|Proposal|Negotiation|Won|Lost'],['probability','Probability %','number'],['close','Expected close','date'],['owner','Owner'],['status','Status','select','Open|On Hold|Won|Lost']]}
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
function cellValue(r,c){const [key,,type]=c;if(type==='money')return money(r[key]);if(type==='docmoney')return money(docTotal(r));if(type==='company')return companyName(r[key]);if(type==='opportunity')return opportunityName(r[key]);if(type==='quote')return quoteName(r[key]);if(type==='order')return orderName(r[key]);if(type==='related')return relatedLabel(r);return badgeMaybe(r[key])}
function tablePage(key,cfg){
 let arr=data[key];
 if(key==='tasks'&&viewFilter==='open')arr=arr.filter(x=>!['Completed','Cancelled'].includes(x.status));
 if(key==='invoices'&&viewFilter==='outstanding')arr=arr.filter(x=>!['Paid','Cancelled'].includes(x.status));
 $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><span>${viewFilter?viewFilter.charAt(0).toUpperCase()+viewFilter.slice(1):labels[key]}</span></div><h1>${viewFilter==='open'&&key==='tasks'?'Open Tasks':viewFilter==='outstanding'?'Outstanding Invoices':labels[key]}</h1><p class="muted">${arr.length} connected records in the sample workspace</p></div><button class="btn btn-primary" id="addRecord">+ New ${labels[key].replace(/s$/,'')}</button></div><div class="table-wrap"><table class="table"><thead><tr>${cfg.cols.map(c=>`<th>${c[1]}</th>`).join('')}<th>Actions</th></tr></thead><tbody>${arr.map(r=>`<tr>${cfg.cols.map(c=>`<td>${cellValue(r,c)}</td>`).join('')}<td><div class="actions"><button class="icon-btn" data-edit="${r.id}">Edit</button><button class="icon-btn" data-del="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${arr.length?'':'<div class="empty">No records yet</div>'}</div>`;
 document.querySelector('[data-clear-filter]')?.addEventListener('click',()=>{current='dashboard';viewFilter=null;renderView()});
 $('#addRecord').onclick=()=>recordModal(key,cfg.fields());
 document.querySelectorAll('[data-edit]').forEach(b=>b.onclick=()=>recordModal(key,cfg.fields(),byId(key,b.dataset.edit)));
 document.querySelectorAll('[data-del]').forEach(b=>b.onclick=()=>remove(key,b.dataset.del));
}
function badgeMaybe(v){const vals=['Active','Inactive','Customer','Prospect','Lead','Sent','Accepted','Draft','Paid','Overdue','Open','Completed','High','Medium','Low','Urgent','Renewal Due','In Progress','Won','Lost','Confirmed','Cancelled'];return vals.includes(String(v))?`<span class="badge">${v}</span>`:(v??'—')}
function fieldHtml(f,record){const [name,label,type,opts]=f;const val=record[name]??'';if(type==='auto')return `<div class="field"><label>${label}</label><input name="${name}" value="${val}" readonly placeholder="Generated automatically"><small class="field-help">Generated when the record is saved</small></div>`;if(type==='select')return `<div class="field"><label>${label}</label><select name="${name}">${opts.split('|').map(o=>`<option value="${o}" ${val===o?'selected':''}>${o}</option>`).join('')}</select></div>`;if(type==='relation')return selectHtml(name,label,data[opts],val);if(type==='filteredContact')return `<div class="field"><label>${label}</label><select name="${name}" data-filter="contact">${optionalOptions(data.contacts.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No contact')}</select></div>`;if(type==='filteredOpportunity')return `<div class="field"><label>${label}</label><select name="${name}" data-filter="opportunity">${optionalOptions(data.opportunities.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No opportunity',x=>x.title)}</select></div>`;if(type==='filteredQuote')return `<div class="field"><label>${label}</label><select name="${name}" data-filter="quote">${optionalOptions(data.quotes.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No source quote',x=>x.number+' · '+money(docTotal(x)))}</select></div>`;if(type==='filteredOrder')return `<div class="field"><label>${label}</label><select name="${name}" data-filter="order">${optionalOptions(data.orders.filter(x=>!record.companyId||x.companyId===record.companyId),val,'No source order',x=>x.number+' · '+money(docTotal(x)))}</select></div>`;if(type==='dynamicRelation')return `<div class="field"><label>${label}</label><select name="${name}" data-dynamic-related></select></div>`;return `<div class="field ${name==='title'?'full':''}"><label>${label}</label><input name="${name}" type="${type||'text'}" value="${val}" ${['name','title','number'].includes(name)?'required':''}></div>`}
function lineItemsHtml(items=[]){const rows=(items.length?items:[{productId:'',quantity:1,unitPrice:0}]).map(lineRow).join('');return `<div class="full line-items"><div class="line-head"><h3>Products & services</h3><button type="button" class="btn btn-secondary" id="addLine">+ Add line</button></div><div id="lineRows">${rows}</div><div class="line-total">Total <strong id="docTotal">${money(items.reduce((s,i)=>s+lineTotal(i),0))}</strong></div></div>`}
function lineRow(i={productId:'',quantity:1,unitPrice:0}){return `<div class="line-row"><div class="field"><label>Product / service</label><select class="line-product">${options(data.products.filter(p=>p.status==='Active'),i.productId)}</select></div><div class="field"><label>Quantity</label><input class="line-qty" type="number" min="0.01" step="0.01" value="${i.quantity??1}"></div><div class="field"><label>Unit price</label><input class="line-price" type="number" min="0" step="0.01" value="${i.unitPrice??0}"></div><div class="line-subtotal">${money(lineTotal(i))}</div><button type="button" class="icon-btn line-remove">Remove</button></div>`}
function recordModal(key,fields,record={}){
 const isDoc=['quotes','orders','invoices'].includes(key);
 if(!record.id&&numberRules[key])record={...record,[numberRules[key].field]:nextNumber(key)};
 const form=`<form id="recordForm"><div class="form-grid">${fields.map(f=>fieldHtml(f,record)).join('')}${isDoc?lineItemsHtml(record.items||[]):''}</div><div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Save record</button></div></form>`;
 modal(record.id?'Edit record':'Create record',form); $('[data-close]').onclick=closeModal;
 wireRelations(record); if(isDoc)wireLines();
 applyFieldRules(key,$('#recordForm'));
 $('#recordForm').onsubmit=e=>{e.preventDefault();const obj=Object.fromEntries(new FormData(e.target).entries());
 const relationError=validateRelationships(key,obj);if(relationError)return alert(relationError);fields.filter(f=>f[2]==='number').forEach(f=>obj[f[0]]=Number(obj[f[0]]||0));if(isDoc){obj.items=[...document.querySelectorAll('.line-row')].map(r=>({productId:$('.line-product',r).value,quantity:Number($('.line-qty',r).value||1),unitPrice:Number($('.line-price',r).value||0)})).filter(i=>i.productId);if(!obj.items.length)return alert('Add at least one product or service.')}
 const wasEdit=!!record.id, before=wasEdit?{...record}:null;
 if(wasEdit)Object.assign(byId(key,record.id),obj);else{const rule=numberRules[key];if(rule&&!obj[rule.field])obj[rule.field]=nextNumber(key);data[key].unshift({id:uid(),...obj})}
 if(wasEdit&&relatedTypeFor[key]&&before){
  // Each workflow rule watches its own field (not just status/stage) - fire
  // only when that field actually changed and the rule's operator matches.
  (data.workflowRules||[]).filter(r=>r.entity===key).forEach(r=>{
   const wf=r.triggerField||transitionFieldFor(key);
   if(obj[wf]===undefined||obj[wf]===before[wf])return;
   if(!operatorMatch(r.operator||'equals',obj[wf],r.toValue))return;
   const due=new Date();due.setDate(due.getDate()+Number(r.daysOffset||0));
   data.tasks.unshift({id:uid(),taskNumber:nextNumber('tasks'),title:r.taskTitle,relatedType:relatedTypeFor[key],relatedId:record.id,owner:record.owner||'Unassigned',due:due.toISOString().slice(0,10),priority:'Medium',status:'Open'});
   if(r.notify){
    const label=obj.name||obj.title||obj.number||entityLabel(key);
    data.notifications.unshift({id:uid(),message:`${entityLabel(key).replace(/s$/,'')} "${label}" — ${fieldLabelFor(key,wf)} ${OPERATOR_LABELS[r.operator||'equals']}${operatorNeedsValue(r.operator||'equals')?` "${r.toValue}"`:''} — task "${r.taskTitle}" created`,createdAt:new Date().toISOString(),read:false});
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
function remove(key,id){const refs=dependencies(key,id);if(refs.length)return alert(`This record is connected to ${refs.join(', ')}. Update or delete those records first.`);if(confirm('Delete this record?')){data[key]=data[key].filter(x=>x.id!==id);save();toast('Record deleted');renderView()}}
function modal(title,body){document.body.insertAdjacentHTML('beforeend',`<div class="modal-backdrop" id="modal"><div class="modal"><div class="modal-head"><h2>${title}</h2><button class="icon-btn" onclick="document.getElementById('modal').remove()">✕</button></div>${body}</div></div>`)}
function closeModal(){document.getElementById('modal')?.remove()}
function toast(msg){document.body.insertAdjacentHTML('beforeend',`<div class="toast">${msg}</div>`);setTimeout(()=>$('.toast')?.remove(),2200)}

// ---- Admin panel ---------------------------------------------------------
function entityLabel(key){return key==='opportunities'?labels.pipeline:labels[key]}
function entityPills(keys,active){return `<div class="entity-tabs">${keys.map(k=>`<button class="pill-tab ${k===active?'active':''}" data-entity="${k}">${entityLabel(k)}</button>`).join('')}</div>`}
function adminPage(){
 document.title='Admin — Lanesra OS Demo';
 const tabs=[['profile','Business profile'],['users','Users & roles'],['fields','Custom fields'],['rules','Business rules'],['workflow','Workflow automation'],['numbering','Numbering'],['kpis','Dashboard KPIs']];
 $('#view').innerHTML=`<div class="page-head"><div><div class="breadcrumbs"><button data-clear-filter>Dashboard</button><span>›</span><span>Admin</span></div><h1>Admin panel</h1><p class="muted">Configure your workspace, users and automation. Changes save immediately in this browser.</p></div></div><div class="tabs">${tabs.map(t=>`<button class="tab ${adminTab===t[0]?'active':''}" data-admin-tab="${t[0]}">${t[1]}</button>`).join('')}</div><div id="adminBody" class="admin-body"></div>`;
 $('[data-clear-filter]').onclick=()=>{current='dashboard';viewFilter=null;renderView()};
 document.querySelectorAll('[data-admin-tab]').forEach(b=>b.onclick=()=>{adminTab=b.dataset.adminTab;renderAdminTab()});
 renderAdminTab();
}
function renderAdminTab(){
 const body=$('#adminBody');
 ({profile:profileTab,users:usersTab,fields:fieldsTab,rules:rulesTab,workflow:workflowTab,numbering:numberingTab,kpis:kpisTab}[adminTab])(body);
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
function fieldsTab(body){
 const keys=Object.keys(numberRules);
 const list=(data.customFields||[]).filter(f=>f.entity===cfEntity);
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Custom fields</h3><p class="muted">Add fields to any object. They appear automatically on that object's create/edit form.</p>
 ${entityPills(keys,cfEntity)}
 <div class="table-wrap"><table class="table"><thead><tr><th>Field</th><th>Type</th><th>Options</th><th>Active</th><th>Actions</th></tr></thead><tbody>${list.map(f=>`<tr><td>${f.label}</td><td>${f.type}</td><td>${f.type==='select'?f.options:'—'}</td><td>${badgeMaybe(f.active?'Active':'Inactive')}</td><td><div class="actions"><button class="icon-btn" data-edit-field="${f.id}">Edit</button><button class="icon-btn" data-del-field="${f.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No custom fields on '+entityLabel(cfEntity)+' yet</div>'}</div>
 <button class="btn btn-secondary" id="addField" style="margin-top:14px">+ New field</button>
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{cfEntity=b.dataset.entity;renderAdminTab()});
 $('#addField').onclick=()=>customFieldModal();
 body.querySelectorAll('[data-edit-field]').forEach(b=>b.onclick=()=>customFieldModal(data.customFields.find(f=>f.id===b.dataset.editField)));
 body.querySelectorAll('[data-del-field]').forEach(b=>b.onclick=()=>{if(confirm('Delete this custom field? Saved values remain on records but the field will no longer appear.')){data.customFields=data.customFields.filter(f=>f.id!==b.dataset.delField);save();toast('Field deleted');renderAdminTab()}});
}
function customFieldModal(field){
 const isEdit=!!field;
 const f=field||{entity:cfEntity,label:'',type:'text',options:'',active:true};
 const body=`<form id="cfForm" class="form-grid">
 <div class="field"><label>Field label</label><input name="label" value="${f.label}" required></div>
 <div class="field"><label>Type</label><select name="type">${['text','number','date','boolean','select'].map(t=>`<option value="${t}" ${f.type===t?'selected':''}>${t}</option>`).join('')}</select></div>
 <div class="field full" id="cfOptionsWrap" ${f.type==='select'?'':'style="display:none"'}><label>Options (separate with |)</label><input name="options" value="${f.options||''}" placeholder="Referral|Website|Event"></div>
 <div class="field"><label>Active</label><select name="active"><option value="true" ${f.active?'selected':''}>Active</option><option value="false" ${!f.active?'selected':''}>Inactive</option></select></div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">${isEdit?'Save field':'Add field'}</button></div>
 </form>`;
 modal(isEdit?'Edit custom field':`New custom field on ${entityLabel(cfEntity)}`,body);
 $('[data-close]').onclick=closeModal;
 const cfForm=$('#cfForm'), typeSelect=cfForm.elements.type, optWrap=$('#cfOptionsWrap');
 typeSelect.onchange=()=>{optWrap.style.display=typeSelect.value==='select'?'':'none'};
 cfForm.onsubmit=e=>{
  e.preventDefault();
  const fd=Object.fromEntries(new FormData(e.target).entries());
  if(fd.type==='select'&&!fd.options.trim())return alert('Add at least one option, separated by |.');
  if(isEdit){Object.assign(field,{label:fd.label,type:fd.type,options:fd.type==='select'?fd.options:'',active:fd.active==='true'})}
  else{data.customFields.push({id:uid(),entity:cfEntity,key:slugify(fd.label),label:fd.label,type:fd.type,options:fd.type==='select'?fd.options:'',active:fd.active==='true'})}
  save();closeModal();toast('Custom field saved');renderView();
 };
}
function rulesTab(body){
 const keys=Object.keys(numberRules);
 const actionFields=actionableFieldsFor(ruleEntity);
 const list=(data.fieldRules||[]).filter(r=>r.entity===ruleEntity);
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Business rules</h3><p class="muted">Require or hide a field - built-in or custom - based on any other field's value, with a real comparison operator, not just the status/stage field.</p>
 ${entityPills(keys,ruleEntity)}
 ${actionFields.length?`<div class="table-wrap"><table class="table"><thead><tr><th>When</th><th>Then</th><th>Field</th><th>Actions</th></tr></thead><tbody>${list.map(r=>`<tr><td>${fieldLabelFor(r.entity,r.triggerField||transitionFieldFor(r.entity))} ${OPERATOR_LABELS[r.operator]||'is'}${operatorNeedsValue(r.operator)?' '+badgeMaybe(r.triggerValue):''}</td><td>${r.effect==='require'?'Require':'Hide'}</td><td>${fieldLabelFor(r.entity,r.fieldKey)}</td><td><div class="actions"><button class="icon-btn" data-del-rule="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No business rules on '+entityLabel(ruleEntity)+' yet</div>'}</div><button class="btn btn-secondary" id="addRule" style="margin-top:14px">+ New rule</button>`:`<div class="empty">${entityLabel(ruleEntity)} has no field a rule can require/hide yet.</div>`}
 <p class="muted" style="margin-top:14px">This demo shows the core require/hide rule. The Windows desktop edition also supports multi-condition AND/OR rules and lock/set-value/block-save/show-message actions — see the <a href="/roadmap">roadmap</a>.</p>
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{ruleEntity=b.dataset.entity;renderAdminTab()});
 $('#addRule')?.addEventListener('click',()=>ruleModal(ruleEntity));
 body.querySelectorAll('[data-del-rule]').forEach(b=>b.onclick=()=>{data.fieldRules=data.fieldRules.filter(r=>r.id!==b.dataset.delRule);save();toast('Rule deleted');renderAdminTab()});
}
// Renders the operator + value pair for whichever condition field is
// currently selected - re-invoked on every "Field" change so the operator
// choices and the value widget (select/number/date/text) always match the
// chosen field's real type, the same as the desktop edition's condition row.
function conditionDynamicHtml(condFields,fieldKey,operator,value){
 const field=condFields.find(f=>f[0]===fieldKey);
 const ops=operatorsForType(field?field[2]:'text');
 const op=ops.includes(operator)?operator:ops[0];
 return `<div class="field"><label>Operator</label><select name="operator">${ops.map(o=>`<option value="${o}" ${o===op?'selected':''}>${OPERATOR_LABELS[o]}</option>`).join('')}</select></div>
 <div class="field" id="condValueWrap" style="${operatorNeedsValue(op)?'':'display:none'}"><label>Value</label>${fieldValueHtml('triggerValue',field,value)}</div>`;
}
function ruleModal(entityKey){
 const condFields=conditionFieldsFor(entityKey);
 const actionFields=actionableFieldsFor(entityKey);
 const defaultField=transitionFieldFor(entityKey);
 const body=`<form id="ruleForm" class="form-grid">
 <div class="field"><label>Field</label><select name="triggerField" id="condField">${condFields.map(f=>`<option value="${f[0]}" ${f[0]===defaultField?'selected':''}>${f[1]}</option>`).join('')}</select></div>
 <div id="condDynamic" style="display:contents">${conditionDynamicHtml(condFields,defaultField,'equals','')}</div>
 <div class="field"><label>Then</label><select name="targetField">${actionFields.map(f=>`<option value="${f[0]}">${f[1]}</option>`).join('')}</select></div>
 <div class="field"><label>Effect</label><select name="effect"><option value="require">Require the field</option><option value="hide">Hide the field</option></select></div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Add rule</button></div>
 </form>`;
 modal(`New business rule on ${entityLabel(entityKey)}`,body);
 $('[data-close]').onclick=closeModal;
 const form=$('#ruleForm');
 function wireOperatorToggle(){
  form.elements.operator.onchange=()=>{
   const wrap=$('#condValueWrap',form); if(wrap)wrap.style.display=operatorNeedsValue(form.elements.operator.value)?'':'none';
  };
 }
 form.elements.triggerField.onchange=()=>{
  $('#condDynamic',form).innerHTML=conditionDynamicHtml(condFields,form.elements.triggerField.value,'equals','');
  wireOperatorToggle();
 };
 wireOperatorToggle();
 form.onsubmit=e=>{e.preventDefault();const fd=Object.fromEntries(new FormData(form).entries());
  data.fieldRules.push({id:uid(),entity:entityKey,triggerField:fd.triggerField,operator:fd.operator,triggerValue:fd.triggerValue||'',fieldKey:fd.targetField,effect:fd.effect});
  save();closeModal();toast('Business rule added');renderView()};
}
function workflowTab(body){
 const keys=Object.keys(relatedTypeFor);
 const list=(data.workflowRules||[]).filter(r=>r.entity===wfEntity);
 body.innerHTML=`<div class="panel"><h3 style="margin-top:0">Workflow automation</h3><p class="muted">Automatically create a follow-up task, and optionally notify admins, when any built-in or custom field on a record changes and matches a comparison you choose - not just the status/stage field.</p>
 ${entityPills(keys,wfEntity)}
 <div class="table-wrap"><table class="table"><thead><tr><th>When</th><th>Creates task</th><th>Due</th><th>Notifies admins</th><th>Actions</th></tr></thead><tbody>${list.map(r=>`<tr><td>${fieldLabelFor(r.entity,r.triggerField||transitionFieldFor(r.entity))} ${OPERATOR_LABELS[r.operator||'equals']}${operatorNeedsValue(r.operator||'equals')?' '+badgeMaybe(r.toValue):''}</td><td>${r.taskTitle}</td><td>${r.daysOffset==0?'Same day':r.daysOffset+' day(s) later'}</td><td>${r.notify?'Yes':'No'}</td><td><div class="actions"><button class="icon-btn" data-del-wf="${r.id}">Delete</button></div></td></tr>`).join('')}</tbody></table>${list.length?'':'<div class="empty">No workflow rules on '+entityLabel(wfEntity)+' yet</div>'}</div>
 <button class="btn btn-secondary" id="addWf" style="margin-top:14px">+ New workflow rule</button>
 <p class="muted" style="margin-top:14px">This demo's action is task creation plus an optional admin notification. The Windows desktop edition adds triggers for due dates and schedules, plus actions like updating a field, assigning the record's owner, and creating a related record — see the <a href="/roadmap">roadmap</a>.</p>
 </div>`;
 body.querySelectorAll('[data-entity]').forEach(b=>b.onclick=()=>{wfEntity=b.dataset.entity;renderAdminTab()});
 $('#addWf').onclick=()=>workflowModal(wfEntity);
 body.querySelectorAll('[data-del-wf]').forEach(b=>b.onclick=()=>{data.workflowRules=data.workflowRules.filter(r=>r.id!==b.dataset.delWf);save();toast('Workflow rule deleted');renderAdminTab()});
}
function workflowModal(entityKey){
 const condFields=conditionFieldsFor(entityKey);
 const defaultField=transitionFieldFor(entityKey);
 const body=`<form id="wfForm" class="form-grid">
 <div class="field"><label>Watch field</label><select name="triggerField" id="wfField">${condFields.map(f=>`<option value="${f[0]}" ${f[0]===defaultField?'selected':''}>${f[1]}</option>`).join('')}</select></div>
 <div id="wfDynamic" style="display:contents">${conditionDynamicHtml(condFields,defaultField,'equals','').replace('name="triggerValue"','name="toValue"')}</div>
 <div class="field full"><label>Task title</label><input name="taskTitle" required placeholder="e.g. Kick off onboarding"></div>
 <div class="field"><label>Due (days after change)</label><input name="daysOffset" type="number" min="0" value="0"></div>
 <div class="field"><label>Also notify admins?</label><select name="notify"><option value="false">No</option><option value="true">Yes</option></select></div>
 <div class="modal-actions"><button type="button" class="btn btn-secondary" data-close>Cancel</button><button class="btn btn-primary">Add rule</button></div>
 </form>`;
 modal(`New workflow rule on ${entityLabel(entityKey)}`,body);
 $('[data-close]').onclick=closeModal;
 const form=$('#wfForm');
 function wireOperatorToggle(){
  form.elements.operator.onchange=()=>{
   const wrap=$('#condValueWrap',form); if(wrap)wrap.style.display=operatorNeedsValue(form.elements.operator.value)?'':'none';
  };
 }
 form.elements.triggerField.onchange=()=>{
  $('#wfDynamic',form).innerHTML=conditionDynamicHtml(condFields,form.elements.triggerField.value,'equals','').replace('name="triggerValue"','name="toValue"');
  wireOperatorToggle();
 };
 wireOperatorToggle();
 form.onsubmit=e=>{e.preventDefault();const fd=Object.fromEntries(new FormData(form).entries());
  data.workflowRules.push({id:uid(),entity:entityKey,triggerField:fd.triggerField,operator:fd.operator,toValue:fd.toValue||'',taskTitle:fd.taskTitle,daysOffset:Number(fd.daysOffset||0),notify:fd.notify==='true'});
  save();closeModal();toast('Workflow rule added');renderView()};
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

function publicNav(){return `<nav class="landing-nav"><div class="container nav-inner"><a class="brand" href="/"><span class="brand-mark">L</span>Lanesra OS</a><div class="nav-links"><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a></div><div class="nav-actions"><a class="btn btn-primary mobile-try" href="/demo">Try Online →</a><button class="menu-toggle" aria-label="Open navigation" aria-expanded="false">☰</button></div></div><div class="mobile-drawer" hidden><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a><hr><a href="/roadmap">Roadmap</a><a href="/changelog">Changelog</a><a href="https://vikramgrover.com">Built by Vikram Grover</a></div></nav>`}
function publicFooter(){return `<footer class="footer"><div class="container footer-grid"><div><a class="brand footer-brand" href="/"><span class="brand-mark">L</span>Lanesra OS</a><span class="muted">Modern, open-source business software for small businesses.</span></div><div><strong>Product</strong><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a></div><div><strong>Development</strong><a href="/roadmap">Roadmap</a><a href="/changelog">Changelog</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a></div><div><strong>Creator</strong><a href="https://vikramgrover.com">VikramGrover.com</a></div></div><div class="container footer-bottom"><span>© 2026 Lanesra OS</span><span>Created by Vikram Grover</span></div></footer>`}
function roadmapPage(){document.title='Roadmap — Lanesra OS';$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Built in public</div><h1>Product roadmap</h1><p>What is available now, what is being built next, and where Lanesra OS is heading.</p><div class="status-row"><span class="status-chip">Early Access v0.16.0</span><span class="muted">Last updated August 2026</span></div></div></section><section class="section roadmap-board"><div class="container roadmap-columns"><div><h2>Available now</h2>${['Companies and contacts','Sales pipeline','Products and services','Quotes, orders and invoices','Contracts and tasks','Interactive dashboards','Connected record relationships','Configurable numbering — admin-controlled prefix and digit width per object','Windows desktop installer (Early Access, unsigned)','Team Workspace mode for small teams (Docker)','Backup and restore','Self-service password change','PDF generation and printing','CSV import and export','Branding and print customization (logo, editable business profile with phone)','Reports beyond the dashboard, plus a simple custom report builder','Custom fields on every major object, not just Companies and Contacts','Conditional business rules on every object with custom fields','Workflow automation across Companies, Contacts, Opportunities, Quotes, Orders, Invoices and Contracts','Admin panel: user roles, dashboard KPI picker, and every configuration screen in one place','Admin-defined Custom Objects — build entirely new record types with their own fields, numbering and screens, no code required','Custom relationships between any two record types, with cardinality (one-to-one, many-to-one, many-to-many) and delete-behavior rules','Richer conditional business rules — multiple AND/OR conditions, 10 comparison operators, and lock/set-value/block-save/show-message actions','Richer workflow automation — triggers for field changes and due dates, actions to assign the record\'s owner or create a related record, plus an in-app notification center','Business rules and workflow triggers on any built-in field, not just status/stage — with a comparison operator chosen per field','Windows task reminder notifications','Session inactivity auto-lock'].map(x=>`<p class="roadmap-row done">✓ ${x}</p>`).join('')}</div><div id="desktop"><h2>Building now</h2>${['Code-signed installer'].map(x=>`<p class="roadmap-row active">↻ ${x}</p>`).join('')}<p class="muted"><strong>Windows desktop edition: full sales lifecycle, Contracts, Tasks, user management, Team Workspace mode, backup/restore, PDF printing, CSV import/export, admin-defined Custom Objects and relationships, and an admin-configurable layer (branding, reports, custom fields, business rules, workflow automation, notifications) all working.</strong> An unsigned installer is available now. <a href="https://github.com/vikram2409-eng/Lanesra-OS/releases" target="_blank" rel="noopener">Download it →</a> · <a href="https://github.com/vikram2409-eng/Lanesra-OS/tree/main/desktop" target="_blank" rel="noopener">View desktop source →</a></p></div><div><h2>Planned</h2>${['Global search and list-view filtering','No-code screen/layout designer','Full drag-and-drop report/dashboard builder','Projects and milestones','Inventory and suppliers','Recurring invoices','Customer portal','Plugin architecture'].map(x=>`<p class="roadmap-row">○ ${x}</p>`).join('')}</div></div></section></main>${publicFooter()}`;bindPublicNav()}
function changelogPage(){document.title='Changelog — Lanesra OS';const releases=[['v0.16.0','August 2026','Business rules & workflows now work on any field, not just status',['Business rules can now condition on any built-in field - name, industry, value, close date, whatever the object has - not only the status/stage field, with a real comparison operator (is/is not, contains, is empty, greater than, less than) chosen per field, and their require/hide action can now target a built-in field too, not just a custom one','Workflow automation\'s field-changed trigger can now watch any built-in field the same way, so "when Industry changes to X" or "when Due date is set" can create a task and notify admins, not only "when status/stage reaches a value"','Windows desktop edition: the underlying business rules and workflow engines gained the same any-built-in-field support for both conditions/triggers and actions (require, hide, lock, set default, force value, and the workflow update-field action), writing through each entity\'s own validation so nothing bypasses existing rules']],['v0.15.0','August 2026','Custom relationships, richer business rules & workflow automation',['Added admin-defined custom relationships between any two record types (companies, contacts, custom objects, and more), with one-to-one/many-to-one/many-to-many cardinality and a choice of what happens to linked records on delete','Added a related-records view on record detail pages showing every linked record through those relationships','Replaced the business rules engine: rules can now combine multiple conditions with AND/OR, use 10 comparison operators (not just equals), and lock a field, set a default or exact value, block saving entirely, or show a message — not just require or hide','Replaced the workflow automation engine: triggers now include field changes and dates reached/overdue in addition to status changes, and actions include assigning the record\'s owner, creating a related record, and posting an in-app notification, on top of creating a task','Added an in-app notification center (bell icon with unread count) for workflow-triggered notifications','Added optional validation for custom fields — a min/max range for number fields, a max length and regex pattern for text fields — plus searchable/filterable/reportable capability flags','Added Windows task reminder notifications (native toast notifications via the desktop app\'s webview)','Added a session inactivity auto-lock (15 minutes idle) requiring the current user\'s password to resume','Updated the online demo: business rules now support an "is / is not" operator, and workflow rules can optionally post an admin notification, shown in a new notification bell']],['v0.14.0','August 2026','Admin-defined Custom Objects',['Added Custom Objects: an Administrator can define an entirely new record type (its own label, fields and ID/numbering format) without any code changes','Custom Objects automatically get their own navigation section, and are full citizens of the existing custom fields, business rules and custom report builder — no per-object code was needed for any of the three','A custom object can\'t be named the same as a built-in entity, and deleting its definition is blocked while records exist (deactivating it is always safe and non-destructive)']],['v0.13.0','August 2026','Admin panel: users, roles & flexible configuration everywhere',['Added an Admin panel with user & role management, moved out of the main navigation into one dedicated section','Added an editable business profile (name, phone, address, city, logo) shown across the workspace','Generalized custom fields from Companies/Contacts to every major object: Opportunities, Quotes, Orders, Invoices, Contracts, Products and Tasks','Generalized conditional business rules and workflow automation the same way, so any object with custom fields can use them','Added admin-configurable numbering: choose the prefix and digit width used for each object\'s auto-generated ID (e.g. "ACC-000001" or "ACC-ab0001")','Added a simple custom report builder: pick an object, group by any field including custom fields, and count or sum','Added a dashboard KPI picker so admins choose which tiles show, in what selection, for the whole workspace','Updated the online demo with a full working Admin panel — mirrors every feature above in the browser']],['v0.12.0','August 2026','Branding, reports, custom fields, business rules & workflow automation',['Added business branding (logo, editable business profile) shown on the print letterhead for quotes, orders and invoices','Added reports beyond the dashboard: revenue by month, win rate by owner, lost reasons, AR aging and sales by owner','Added admin-defined custom fields on Companies and Contacts (text, number, date, yes/no, select), enforced both client- and server-side','Added conditional business rules that require or hide a custom field based on a record\'s status','Added Phase 1 workflow automation: auto-create a follow-up task when an Opportunity\'s stage or an Invoice\'s status changes']],['v0.11.0','August 2026','PDF printing & CSV import/export',['Added a browser-native "Print / Save as PDF" preview for quotes, orders and invoices, with business letterhead, line items and totals','Added CSV export on every list screen','Added CSV import for Companies and Contacts, validated row by row through the same rules as the manual forms']],['v0.10.0','August 2026','Team Workspace, backup & restore',['Added Team Workspace mode — a small team shares one server over the local network from browser tabs, with per-user sessions','Added whole-workspace backup and restore as a single file, safe to run against a live database','Added self-service password change from a "My account" screen']],['v0.9.0','August 2026','Desktop edition foundation published',['Published the Windows desktop edition source: Tauri v2 + Rust + SQLite','Implemented the full sales lifecycle on desktop — Companies, Contacts, Products, Opportunities, Quotes, Orders and Invoices','Added quote-to-order and order-to-invoice conversion, atomic document numbering and local user authentication','No packaged installer yet — desktop is available to build and run from source']],['v0.8.0','August 2026','Interactive navigation & public pages',['Made dashboard KPIs clickable with filtered drill-downs','Added a global Quick Create menu','Added mobile navigation while keeping Try Online prominent','Replaced Journey with Principles and added Compare and Download pages','Marked desktop downloads as Coming Soon','Fixed desktop sidebar navigation']],['v0.7.0','August 2026','Trust & product transparency',['Added Roadmap, Changelog and creator attribution','Added Person JSON-LD and updated discovery files']],['v0.6.0','August 2026','Record numbering & search',['Added automatically generated identifiers','Rebuilt global search as one stable result panel','Added keyboard shortcuts and wider search coverage']],['v0.5.0','July 2026','Lanesra OS rebrand',['Renamed BusinessOS to Lanesra OS','Updated product branding, metadata and documentation']],['v0.4.0','July 2026','Relationship integrity',['Added opportunity-to-contact relationship','Removed opportunity-to-contract relationship','Added company-filtered relationship dropdowns']],['v0.3.0','July 2026','Flexible sales flow',['Made opportunities optional for quotes','Made quotes optional for orders','Added products, services and line-item quantities']],['v0.2.0','June 2026','Connected sales MVP',['Added quotes, orders, invoices, contracts and dashboards','Connected core entities using clean relationships']],['v0.1.0','May 2026','First working prototype',['Launched the first browser-based MVP with sample data']]];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Release history</div><h1>Changelog</h1><p>Every meaningful improvement to Lanesra OS, documented publicly.</p><div class="status-row"><span class="status-chip">Latest: v0.16.0</span><span class="muted">Early Access</span></div></div></section><section class="section"><div class="container changelog-list">${releases.map(r=>`<article class="release" id="${r[0].replaceAll('.','-')}"><div class="release-meta"><span class="status-chip">${r[0]}</span><span>${r[1]}</span></div><div><h2>${r[2]}</h2><ul>${r[3].map(x=>`<li>${x}</li>`).join('')}</ul></div></article>`).join('')}</div></section></main>${publicFooter()}`;bindPublicNav()}
function principlesPage(){document.title='Principles — Lanesra OS';const principles=[['Own your data','Your customer and sales information should remain under your control—not trapped behind a subscription or vendor lock-in.'],['Offline first','Core work should continue even when the internet does not. The Windows desktop edition runs entirely on local SQLite storage, with no server or account required.'],['Relationships over spreadsheets','Customers, contacts, opportunities, quotes, orders and invoices stay linked so data remains clean and useful — and that same connected model extends to any custom record type you define.'],['Simple before powerful','Every feature must reduce effort. Complexity is added only when it clearly improves the work.'],['Configurable, not hardcoded','A business shouldn\'t need a developer to add a field, a record type, a rule or an automation. Admins reshape Lanesra from a settings screen — the software adapts to the business, not the other way around.'],['Open by default','The product roadmap, changelog and source code are public so users can inspect how Lanesra evolves.'],['Business software deserves good design','Small businesses should not have to accept dated interfaces or confusing navigation to access serious capabilities.']];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">How Lanesra is designed</div><h1>Principles before features.</h1><p>The decisions behind Lanesra OS are guided by a small set of practical beliefs about ownership, simplicity and product quality.</p></div></section><section class="section"><div class="container principles-page-grid">${principles.map((p,i)=>`<article class="principle-card"><span>0${i+1}</span><h2>${p[0]}</h2><p>${p[1]}</p></article>`).join('')}</div></section><section class="section maintenance"><div class="container narrow"><div class="eyebrow">The business flow</div><h2>Connected by design.</h2><div class="flow-map"><strong>Customer</strong><span>→</span><div>Contacts<br>Opportunities <em>optional</em><br>Quotes <em>optional</em><br>Orders<br>Invoices<br>Contracts<br>Tasks</div></div><p class="muted" style="margin-top:18px">That same connected model isn't fixed to these nine record types — admins can add their own (Vendors, Assets, Projects…) and link them into this graph with custom relationships, so the "no dangling free text" principle holds for whatever your business actually looks like.</p></div></section></main>${publicFooter()}`;bindPublicNav()}
function comparePage(){document.title='Compare — Lanesra OS';const rows=[['Runs without internet','Partial','No','No','Yes'],['Open source','No','No','No','Yes'],['Local database','No','No','No','Yes (desktop)'],['Mandatory subscription','No','Yes','Yes','No'],['Connected sales workflow','Manual','Limited','Advanced','Yes'],['Custom record types, no code','No','Limited, paid tiers','Yes, complex/paid','Yes, built in'],['Custom business rules & workflow automation','No','Paid tiers','Yes, needs admin training','Yes, built in'],['Designed for small business','General','Yes','Enterprise','Yes'],['Self-owned business data','File-based','Cloud-hosted','Cloud-hosted','Yes']];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Choose with context</div><h1>Where Lanesra fits.</h1><p>A factual comparison for small businesses deciding between spreadsheets, cloud CRMs and a local-first open-source system.</p></div></section><section class="section"><div class="container compare-wrap"><table class="compare-table"><thead><tr><th>Capability</th><th>Excel</th><th>HubSpot</th><th>Salesforce</th><th class="lanesra-col">Lanesra OS</th></tr></thead><tbody>${rows.map(r=>`<tr>${r.map((x,i)=>`<td class="${i===4?'lanesra-col':''}">${x}</td>`).join('')}</tr>`).join('')}</tbody></table><p class="compare-note">Comparisons are intentionally high-level. Product capabilities and commercial terms can change; review each vendor's current documentation before making a purchase decision.</p></div></section></main>${publicFooter()}`;bindPublicNav()}
function downloadPage(){document.title='Download — Lanesra OS';$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Local-first desktop edition</div><h1>Download Lanesra OS.</h1><p>The independent desktop edition runs locally with no cloud account or mandatory internet connection. It is in active early development, with an Early Access Windows installer now available.</p></div></section><section class="section"><div class="container download-grid"><article class="download-card featured"><span class="status-chip">Early access — installer available</span><h2>Windows</h2><p>Tauri + Rust + SQLite desktop app with the full sales lifecycle, Contracts, Tasks and user management working: Companies, Contacts, Products, Opportunities, Quotes, Orders, Invoices, Contracts and Tasks — plus Team Workspace mode for small teams, backup and restore, PDF printing, CSV import/export, admin-defined Custom Objects and relationships between any record types, and an Admin panel covering branding, user roles, custom fields, richer conditional business rules, richer workflow automation with in-app notifications, configurable ID formats and a dashboard KPI picker. Unsigned .exe and .msi installers are on GitHub Releases (Windows will warn on first run since they aren't code-signed yet).</p><a class="btn btn-primary" href="https://github.com/vikram2409-eng/Lanesra-OS/releases" target="_blank" rel="noopener">Download for Windows</a><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS/tree/main/desktop" target="_blank" rel="noopener">View desktop source on GitHub</a></article><article class="download-card"><span class="status-chip">Planned</span><h2>macOS</h2><p>Apple silicon and Intel packaging will follow the Windows early-access release.</p><button class="btn btn-secondary" disabled>Planned</button></article><article class="download-card"><span class="status-chip">Planned</span><h2>Linux</h2><p>AppImage or Debian packaging is planned after the initial desktop release stabilizes.</p><button class="btn btn-secondary" disabled>Planned</button></article></div></section><section class="section maintenance"><div class="container narrow"><h2>What the desktop edition includes today</h2><div class="download-checks"><span>✓ No licence key</span><span>✓ No cloud account</span><span>✓ Standard SQLite database</span><span>✓ Offline from first launch</span><span>✓ Full sales lifecycle (quotes → orders → invoices)</span><span>✓ Contracts and tasks</span><span>✓ User management</span><span>✓ Team Workspace mode for small teams (Docker)</span><span>✓ Windows installer (unsigned, Early Access)</span><span>✓ Backup and restore</span><span>✓ Self-service password change</span><span>✓ PDF generation and printing</span><span>✓ CSV import and export</span><span>✓ Branding and print customization</span><span>✓ Reports, plus a custom report builder</span><span>✓ Custom fields & business rules on every object</span><span>✓ Workflow automation with in-app notifications</span><span>✓ Admin-defined Custom Objects</span><span>✓ Custom relationships between record types</span><span>✓ Windows task reminder notifications</span><span>✓ Session inactivity auto-lock</span><span>✓ Admin panel: user roles & configurable numbering</span><span>✓ Open-source code</span><span>○ Code-signed installer — planned</span></div><div class="hero-actions"><a class="btn btn-secondary" href="/roadmap#desktop">View desktop roadmap</a><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS/tree/main/desktop" target="_blank" rel="noopener">View source on GitHub</a></div></div></section></main>${publicFooter()}`;bindPublicNav()}
function bindPublicNav(){document.querySelectorAll('.menu-toggle').forEach(btn=>{btn.onclick=()=>{const nav=btn.closest('.landing-nav');const drawer=nav.querySelector('.mobile-drawer');const open=drawer.hasAttribute('hidden');if(open)drawer.removeAttribute('hidden');else drawer.setAttribute('hidden','');btn.setAttribute('aria-expanded',String(open));btn.textContent=open?'×':'☰'}});document.querySelectorAll('.mobile-drawer a').forEach(a=>a.addEventListener('click',()=>{const drawer=a.closest('.mobile-drawer');drawer.setAttribute('hidden','');const btn=drawer.closest('.landing-nav').querySelector('.menu-toggle');btn.textContent='☰';btn.setAttribute('aria-expanded','false')}))}
const path=location.pathname.replace(/\/$/,'')||'/'; if(path==='/demo')appShell();else if(path==='/roadmap')roadmapPage();else if(path==='/changelog')changelogPage();else if(path==='/principles'||path==='/journey'||path==='/our-story'||path==='/about')principlesPage();else if(path==='/compare')comparePage();else if(path==='/download')downloadPage();else landing();
