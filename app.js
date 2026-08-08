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
 ]
};
const storeKey='lanesra-os-demo-v6';
let data = JSON.parse(localStorage.getItem(storeKey)||'null') || structuredClone(seed);
let current='dashboard';
let viewFilter=null;
const save=()=>localStorage.setItem(storeKey,JSON.stringify(data));
const uid=()=>Math.random().toString(36).slice(2,10);
const pad=(n,w=4)=>String(n).padStart(w,'0');
const year=()=>new Date().getFullYear();
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
function nextNumber(key){
 const r=numberRules[key]; if(!r)return '';
 const base=r.year?`${r.prefix}-${year()}-`:`${r.prefix}-`;
 const nums=(data[key]||[]).map(x=>String(x[r.field]||'')).filter(v=>v.startsWith(base)).map(v=>Number(v.slice(base.length))).filter(Number.isFinite);
 return base+pad((nums.length?Math.max(...nums):0)+1,r.width);
}
function ensureNumbers(){
 Object.entries(numberRules).forEach(([key,r])=>{
  (data[key]||[]).slice().reverse().forEach(x=>{if(!x[r.field])x[r.field]=nextNumber(key)});
 });
 save();
}
const icons={dashboard:'▦',companies:'◫',contacts:'◎',pipeline:'⌁',products:'◇',quotes:'▤',orders:'▣',invoices:'$',contracts:'▧',tasks:'✓'};
const labels={dashboard:'Dashboard',companies:'Companies',contacts:'Contacts',pipeline:'Sales Pipeline',products:'Products',quotes:'Quotes',orders:'Orders',invoices:'Invoices',contracts:'Contracts',tasks:'Tasks'};

function landing(){
 document.title='Lanesra OS — Modern open-source sales management';
 $('#app').innerHTML=`
 ${publicNav()}
 <main>
 <section class="hero"><div class="container hero-grid"><div><div class="eyebrow">Open-source business software</div><h1>Run your business without complicated software.</h1><p>Lanesra OS gives small businesses one modern workspace for customers, opportunities, products, quotes, orders, invoices, contracts and daily follow-ups.</p><div class="hero-actions"><a class="btn btn-primary" href="/demo">Try the live demo →</a><a class="btn btn-secondary" href="/download">Desktop edition — Windows installer available</a></div><div class="trust-row"><span>✓ Free to use</span><span>✓ No licence key</span><span>✓ Desktop edition works offline</span><span>✓ Own your data</span></div></div><div class="mock"><div class="mock-top"><span class="dot"></span><span class="dot"></span><span class="dot"></span></div><div class="mock-body"><div class="mock-grid"><div class="mock-card"><small class="muted">Pipeline</small><br><strong>$192K</strong></div><div class="mock-card"><small class="muted">Revenue</small><br><strong>$84K</strong></div><div class="mock-card mock-chart">${[40,70,52,88,62,100].map(h=>`<div class="bar" style="height:${h}%"></div>`).join('')}</div></div></div></div></div></section>
 <section id="features" class="section"><div class="container"><div class="section-head"><div class="eyebrow">Complete sales journey</div><h2>Everything connected from first conversation to invoice.</h2><p class="muted">No maze of modules. No enterprise setup project. Just the essentials your team uses every day.</p></div><div class="feature-grid">${[
 ['◎','Companies & Contacts','Keep customer profiles, people, notes and activities together.'],['⌁','Sales Pipeline','Move opportunities visually from lead to won.'],['◇','Products & Services','Maintain reusable pricing, categories and tax settings.'],['▤','Quotes','Create professional commercial proposals and track acceptance.'],['▣','Orders','Convert approved quotes into trackable sales orders.'],['$','Invoices','Issue invoices and monitor paid, open and overdue balances.'],['▧','Contracts','Track agreement values, dates, files and renewals.'],['✓','Tasks & Activities','Manage calls, meetings, follow-ups and priorities.'],['▦','Sales Dashboard','See pipeline, revenue, customers and next actions instantly.']].map(x=>`<article class="feature-card"><div class="feature-icon">${x[0]}</div><h3>${x[1]}</h3><p class="muted">${x[2]}</p></article>`).join('')}</div></div></section>
 <section id="desktop" class="section"><div class="container split"><div class="choice-card"><div class="eyebrow">Try online</div><h2>Explore a working business</h2><p class="muted">Open the live demo with realistic sample customers, opportunities, quotes, invoices and contracts. No registration required.</p><ul><li>Sample company included</li><li>Create and edit records</li><li>Reset demo anytime</li></ul><a class="btn btn-primary" href="/demo">Open live demo</a></div><div class="choice-card dark"><div class="eyebrow" style="color:#a5b4fc">Desktop edition</div><h2>Your software. Your computer. Your data.</h2><p style="color:#cbd5e1">A private desktop edition is available now for Windows (Early Access, unsigned installer), with macOS and Linux to follow. The source is public on GitHub today.</p><ul><li>No cloud account required</li><li>Works without internet</li><li>No activation or subscription</li></ul><a class="btn btn-secondary" href="/download">Desktop status — Windows installer available</a></div></div></section>
 <section id="open-source" class="section"><div class="container cta"><div class="eyebrow" style="color:#a5b4fc">Open source by design</div><h2>Inspect it. Run it. Improve it.</h2><p style="color:#cbd5e1;max-width:700px;margin:0 auto 24px">Lanesra OS is designed to be transparent, community-driven and free from licence keys or mandatory telemetry.</p><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank" rel="noopener">View GitHub repository</a></div></section>
 </main>${publicFooter()}`;
 bindPublicNav();
}


function appShell(){
 document.title='Lanesra OS Demo';
 $('#app').innerHTML=`<div class="demo-banner">You are exploring the sample workspace. Changes stay in this browser. <button class="link-btn" id="resetDemo">Reset demo</button><a class="link-btn" href="/">Product website</a></div><div class="app-shell"><aside class="sidebar"><div class="side-brand"><span class="brand-mark">L</span><span>Lanesra OS</span><span class="demo-pill">DEMO</span></div><nav class="side-nav">${Object.keys(labels).map(k=>`<button data-nav="${k}"><b>${icons[k]}</b><span>${labels[k]}</span></button>`).join('')}</nav><div class="side-bottom"><div class="side-meta"><strong>Early Access v0.9.0</strong><div class="side-product-links"><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/roadmap">Roadmap</a><a href="/changelog">Changelog</a></div><span>Created by <a href="https://vikramgrover.com">Vikram Grover</a></span></div><button class="btn btn-secondary" style="width:100%" onclick="location.href='/'">← Website</button></div></aside><main class="app-main"><header class="topbar"><div class="search"><input id="globalSearch" autocomplete="off" placeholder="Search companies, contacts, deals…  ⌘K"><div id="searchResults" class="search-results" hidden></div></div><div class="top-actions"><button class="icon-btn" id="helpButton" aria-label="Help">?</button><div class="avatar">MC</div></div></header><div class="content" id="view"></div></main></div>`;
 document.querySelectorAll('[data-nav]').forEach(b=>b.onclick=()=>{current=b.dataset.nav;viewFilter=null;renderView()});
 $('#resetDemo').onclick=()=>{data=structuredClone(seed);save();toast('Demo data restored');renderView()};
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
 renderView();
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
 const configs={
 companies:{cols:[['customerNumber','Customer ID'],['name','Company'],['industry','Industry'],['city','City'],['owner','Owner'],['status','Status']],fields:companyFields},
 contacts:{cols:[['contactNumber','Contact ID'],['name','Contact'],['companyId','Company','company'],['role','Role'],['email','Email'],['status','Status']],fields:contactFields},
 products:{cols:[['productNumber','Product ID'],['name','Product / Service'],['type','Type'],['sku','SKU'],['price','Price','money'],['status','Status']],fields:productFields},
 quotes:{cols:[['number','Quote'],['companyId','Customer','company'],['opportunityId','Opportunity','opportunity'],['amount','Amount','docmoney'],['status','Status']],fields:quoteFields,document:true},
 orders:{cols:[['number','Order'],['companyId','Customer','company'],['quoteId','Quote','quote'],['amount','Amount','docmoney'],['status','Status']],fields:orderFields,document:true},
 invoices:{cols:[['number','Invoice'],['companyId','Customer','company'],['orderId','Order','order'],['amount','Amount','docmoney'],['status','Status']],fields:invoiceFields,document:true},
 contracts:{cols:[['number','Contract'],['companyId','Customer','company'],['title','Title'],['value','Value','money'],['status','Status'],['end','End date']],fields:contractFields},
 tasks:{cols:[['title','Task'],['relatedId','Related to','related'],['owner','Owner'],['due','Due'],['priority','Priority'],['status','Status']],fields:taskFields}
 };
 tablePage(current,configs[current]);
}
function dashboard(){
 const openPipe=data.opportunities.filter(o=>!['Won','Lost'].includes(o.stage)).reduce((s,o)=>s+Number(o.value||0),0);
 const won=data.opportunities.filter(o=>o.stage==='Won').reduce((s,o)=>s+Number(o.value||0),0);
 const outstanding=data.invoices.filter(i=>!['Paid','Cancelled'].includes(i.status)).reduce((s,i)=>s+docTotal(i),0);
 const openTasks=data.tasks.filter(t=>!['Completed','Cancelled'].includes(t.status)).length;
 $('#view').innerHTML=`<div class="page-head"><div><div class="eyebrow">Northstar Digital Solutions</div><h1>Good afternoon, Maya</h1><p class="muted">Here is what needs your attention today.</p></div><div class="quick-create"><button class="btn btn-primary" id="quickNew">+ New</button><div class="quick-menu" id="quickMenu" hidden>${[['companies','Company'],['contacts','Contact'],['opportunities','Opportunity'],['quotes','Quote'],['orders','Order'],['invoices','Invoice'],['contracts','Contract'],['tasks','Task']].map(x=>`<button data-create="${x[0]}">${x[1]}</button>`).join('')}</div></div></div><div class="kpi-grid"><button class="kpi kpi-link" data-kpi-nav="pipeline" data-kpi-filter="open"><div class="kpi-label">Open pipeline</div><div class="kpi-value">${money(openPipe)}</div><span>View open opportunities →</span></button><button class="kpi kpi-link" data-kpi-nav="pipeline" data-kpi-filter="won"><div class="kpi-label">Won revenue</div><div class="kpi-value">${money(won)}</div><span>View won opportunities →</span></button><button class="kpi kpi-link" data-kpi-nav="invoices" data-kpi-filter="outstanding"><div class="kpi-label">Outstanding invoices</div><div class="kpi-value">${money(outstanding)}</div><span>View outstanding invoices →</span></button><button class="kpi kpi-link" data-kpi-nav="tasks" data-kpi-filter="open"><div class="kpi-label">Open tasks</div><div class="kpi-value">${openTasks}</div><span>View open tasks →</span></button></div><div class="grid-2"><section class="panel"><div class="panel-head"><h3>Pipeline snapshot</h3><button class="link-btn" data-nav2="pipeline" data-filter2="open">Open pipeline</button></div>${data.opportunities.filter(o=>!['Won','Lost'].includes(o.stage)).slice(0,5).map(o=>`<div class="deal"><div style="display:flex;justify-content:space-between"><strong>${o.title}</strong><strong>${money(o.value)}</strong></div><small class="muted">${companyName(o.companyId)} · ${o.stage}</small></div>`).join('')}</section><section class="panel"><div class="panel-head"><h3>Tasks requiring attention</h3><button class="link-btn" data-nav2="tasks" data-filter2="open">View tasks</button></div>${data.tasks.filter(t=>!['Completed','Cancelled'].includes(t.status)).map(t=>`<div class="deal"><strong>${t.title}</strong><small class="muted">${relatedLabel(t)} · ${t.due}</small></div>`).join('')}</section></div>`;
 document.querySelectorAll('[data-kpi-nav]').forEach(b=>b.onclick=()=>{current=b.dataset.kpiNav;viewFilter=b.dataset.kpiFilter;renderView()});
 document.querySelectorAll('[data-nav2]').forEach(b=>b.onclick=()=>{current=b.dataset.nav2;viewFilter=b.dataset.filter2||null;renderView()});
 const quick=$('#quickNew'),menu=$('#quickMenu'); quick.onclick=e=>{e.stopPropagation();menu.hidden=!menu.hidden};
 menu.querySelectorAll('[data-create]').forEach(b=>b.onclick=()=>{const k=b.dataset.create;menu.hidden=true;if(k==='opportunities')recordModal('opportunities',opportunityFields());else{const cfg={companies:companyFields,contacts:contactFields,quotes:quoteFields,orders:orderFields,invoices:invoiceFields,contracts:contractFields,tasks:taskFields}[k];recordModal(k,cfg())}});
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
 $('#addDeal').onclick=()=>recordModal('opportunities',opportunityFields());
 document.querySelectorAll('[data-edit]').forEach(b=>b.onclick=()=>recordModal('opportunities',opportunityFields(),byId('opportunities',b.dataset.edit)));
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
 $('#recordForm').onsubmit=e=>{e.preventDefault();const obj=Object.fromEntries(new FormData(e.target).entries());
 const relationError=validateRelationships(key,obj);if(relationError)return alert(relationError);fields.filter(f=>f[2]==='number').forEach(f=>obj[f[0]]=Number(obj[f[0]]||0));if(isDoc){obj.items=[...document.querySelectorAll('.line-row')].map(r=>({productId:$('.line-product',r).value,quantity:Number($('.line-qty',r).value||1),unitPrice:Number($('.line-price',r).value||0)})).filter(i=>i.productId);if(!obj.items.length)return alert('Add at least one product or service.')}if(record.id)Object.assign(byId(key,record.id),obj);else{const rule=numberRules[key];if(rule&&!obj[rule.field])obj[rule.field]=nextNumber(key);data[key].unshift({id:uid(),...obj})}save();closeModal();toast('Record saved');renderView()};
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

function publicNav(){return `<nav class="landing-nav"><div class="container nav-inner"><a class="brand" href="/"><span class="brand-mark">L</span>Lanesra OS</a><div class="nav-links"><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a></div><div class="nav-actions"><a class="btn btn-primary mobile-try" href="/demo">Try Online →</a><button class="menu-toggle" aria-label="Open navigation" aria-expanded="false">☰</button></div></div><div class="mobile-drawer" hidden><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a><hr><a href="/roadmap">Roadmap</a><a href="/changelog">Changelog</a><a href="https://vikramgrover.com">Built by Vikram Grover</a></div></nav>`}
function publicFooter(){return `<footer class="footer"><div class="container footer-grid"><div><a class="brand footer-brand" href="/"><span class="brand-mark">L</span>Lanesra OS</a><span class="muted">Modern, open-source business software for small businesses.</span></div><div><strong>Product</strong><a href="/#features">Features</a><a href="/principles">Principles</a><a href="/compare">Compare</a><a href="/download">Download</a></div><div><strong>Development</strong><a href="/roadmap">Roadmap</a><a href="/changelog">Changelog</a><a href="https://github.com/vikram2409-eng/Lanesra-OS" target="_blank">GitHub</a></div><div><strong>Creator</strong><a href="https://vikramgrover.com">VikramGrover.com</a></div></div><div class="container footer-bottom"><span>© 2026 Lanesra OS</span><span>Created by Vikram Grover</span></div></footer>`}
function roadmapPage(){document.title='Roadmap — Lanesra OS';$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Built in public</div><h1>Product roadmap</h1><p>What is available now, what is being built next, and where Lanesra OS is heading.</p><div class="status-row"><span class="status-chip">Early Access v0.9.0</span><span class="muted">Last updated August 2026</span></div></div></section><section class="section roadmap-board"><div class="container roadmap-columns"><div><h2>Available now</h2>${['Companies and contacts','Sales pipeline','Products and services','Quotes, orders and invoices','Contracts and tasks','Interactive dashboards','Connected record relationships','Auto-generated numbering','Windows desktop installer (Early Access, unsigned)','Backup and restore','Self-service password change'].map(x=>`<p class="roadmap-row done">✓ ${x}</p>`).join('')}</div><div id="desktop"><h2>Building now</h2>${['Code-signed installer','CSV import and export','Improved document printing'].map(x=>`<p class="roadmap-row active">↻ ${x}</p>`).join('')}<p class="muted"><strong>Windows desktop edition: full sales lifecycle, Contracts, Tasks, user management and Team Workspace mode all working.</strong> An unsigned installer is available now. <a href="https://github.com/vikram2409-eng/Lanesra-OS/releases" target="_blank" rel="noopener">Download it →</a> · <a href="https://github.com/vikram2409-eng/Lanesra-OS/tree/main/desktop" target="_blank" rel="noopener">View desktop source →</a></p></div><div><h2>Planned</h2>${['Projects and milestones','Inventory and suppliers','Recurring invoices','Custom fields','Customer portal','Plugin architecture'].map(x=>`<p class="roadmap-row">○ ${x}</p>`).join('')}</div></div></section></main>${publicFooter()}`;bindPublicNav()}
function changelogPage(){document.title='Changelog — Lanesra OS';const releases=[['v0.9.0','August 2026','Desktop edition foundation published',['Published the Windows desktop edition source: Tauri v2 + Rust + SQLite','Implemented the full sales lifecycle on desktop — Companies, Contacts, Products, Opportunities, Quotes, Orders and Invoices','Added quote-to-order and order-to-invoice conversion, atomic document numbering and local user authentication','No packaged installer yet — desktop is available to build and run from source']],['v0.8.0','August 2026','Interactive navigation & public pages',['Made dashboard KPIs clickable with filtered drill-downs','Added a global Quick Create menu','Added mobile navigation while keeping Try Online prominent','Replaced Journey with Principles and added Compare and Download pages','Marked desktop downloads as Coming Soon','Fixed desktop sidebar navigation']],['v0.7.0','August 2026','Trust & product transparency',['Added Roadmap, Changelog and creator attribution','Added Person JSON-LD and updated discovery files']],['v0.6.0','August 2026','Record numbering & search',['Added automatically generated identifiers','Rebuilt global search as one stable result panel','Added keyboard shortcuts and wider search coverage']],['v0.5.0','July 2026','Lanesra OS rebrand',['Renamed BusinessOS to Lanesra OS','Updated product branding, metadata and documentation']],['v0.4.0','July 2026','Relationship integrity',['Added opportunity-to-contact relationship','Removed opportunity-to-contract relationship','Added company-filtered relationship dropdowns']],['v0.3.0','July 2026','Flexible sales flow',['Made opportunities optional for quotes','Made quotes optional for orders','Added products, services and line-item quantities']],['v0.2.0','June 2026','Connected sales MVP',['Added quotes, orders, invoices, contracts and dashboards','Connected core entities using clean relationships']],['v0.1.0','May 2026','First working prototype',['Launched the first browser-based MVP with sample data']]];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Release history</div><h1>Changelog</h1><p>Every meaningful improvement to Lanesra OS, documented publicly.</p><div class="status-row"><span class="status-chip">Latest: v0.9.0</span><span class="muted">Early Access</span></div></div></section><section class="section"><div class="container changelog-list">${releases.map(r=>`<article class="release" id="${r[0].replaceAll('.','-')}"><div class="release-meta"><span class="status-chip">${r[0]}</span><span>${r[1]}</span></div><div><h2>${r[2]}</h2><ul>${r[3].map(x=>`<li>${x}</li>`).join('')}</ul></div></article>`).join('')}</div></section></main>${publicFooter()}`;bindPublicNav()}
function principlesPage(){document.title='Principles — Lanesra OS';const principles=[['Own your data','Your customer and sales information should remain under your control—not trapped behind a subscription or vendor lock-in.'],['Offline first','Core work should continue even when the internet does not. The downloadable edition is being designed around local SQLite storage.'],['Relationships over spreadsheets','Customers, contacts, opportunities, quotes, orders and invoices stay linked so data remains clean and useful.'],['Simple before powerful','Every feature must reduce effort. Complexity is added only when it clearly improves the work.'],['Open by default','The product roadmap, changelog and source code are public so users can inspect how Lanesra evolves.'],['Business software deserves good design','Small businesses should not have to accept dated interfaces or confusing navigation to access serious capabilities.']];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">How Lanesra is designed</div><h1>Principles before features.</h1><p>The decisions behind Lanesra OS are guided by a small set of practical beliefs about ownership, simplicity and product quality.</p></div></section><section class="section"><div class="container principles-page-grid">${principles.map((p,i)=>`<article class="principle-card"><span>0${i+1}</span><h2>${p[0]}</h2><p>${p[1]}</p></article>`).join('')}</div></section><section class="section maintenance"><div class="container narrow"><div class="eyebrow">The business flow</div><h2>Connected by design.</h2><div class="flow-map"><strong>Customer</strong><span>→</span><div>Contacts<br>Opportunities <em>optional</em><br>Quotes <em>optional</em><br>Orders<br>Invoices<br>Contracts<br>Tasks</div></div></div></section></main>${publicFooter()}`;bindPublicNav()}
function comparePage(){document.title='Compare — Lanesra OS';const rows=[['Runs without internet','Partial','No','No','Yes'],['Open source','No','No','No','Yes'],['Local database','No','No','No','Planned desktop'],['Mandatory subscription','No','Yes','Yes','No'],['Connected sales workflow','Manual','Limited','Advanced','Yes'],['Designed for small business','General','Yes','Enterprise','Yes'],['Self-owned business data','File-based','Cloud-hosted','Cloud-hosted','Yes']];$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Choose with context</div><h1>Where Lanesra fits.</h1><p>A factual comparison for small businesses deciding between spreadsheets, cloud CRMs and a local-first open-source system.</p></div></section><section class="section"><div class="container compare-wrap"><table class="compare-table"><thead><tr><th>Capability</th><th>Excel</th><th>HubSpot</th><th>Salesforce</th><th class="lanesra-col">Lanesra OS</th></tr></thead><tbody>${rows.map(r=>`<tr>${r.map((x,i)=>`<td class="${i===4?'lanesra-col':''}">${x}</td>`).join('')}</tr>`).join('')}</tbody></table><p class="compare-note">Comparisons are intentionally high-level. Product capabilities and commercial terms can change; review each vendor's current documentation before making a purchase decision.</p></div></section></main>${publicFooter()}`;bindPublicNav()}
function downloadPage(){document.title='Download — Lanesra OS';$('#app').innerHTML=`${publicNav()}<main class="page-site"><section class="page-hero"><div class="container narrow"><div class="eyebrow">Local-first desktop edition</div><h1>Download Lanesra OS.</h1><p>The independent desktop edition runs locally with no cloud account or mandatory internet connection. It is in active early development, with an Early Access Windows installer now available.</p></div></section><section class="section"><div class="container download-grid"><article class="download-card featured"><span class="status-chip">Early access — installer available</span><h2>Windows</h2><p>Tauri + Rust + SQLite desktop app with the full sales lifecycle, Contracts, Tasks and user management working: Companies, Contacts, Products, Opportunities, Quotes, Orders, Invoices, Contracts and Tasks. Unsigned .exe and .msi installers are on GitHub Releases (Windows will warn on first run since they aren't code-signed yet).</p><a class="btn btn-primary" href="https://github.com/vikram2409-eng/Lanesra-OS/releases" target="_blank" rel="noopener">Download for Windows</a><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS/tree/main/desktop" target="_blank" rel="noopener">View desktop source on GitHub</a></article><article class="download-card"><span class="status-chip">Planned</span><h2>macOS</h2><p>Apple silicon and Intel packaging will follow the Windows early-access release.</p><button class="btn btn-secondary" disabled>Planned</button></article><article class="download-card"><span class="status-chip">Planned</span><h2>Linux</h2><p>AppImage or Debian packaging is planned after the initial desktop release stabilizes.</p><button class="btn btn-secondary" disabled>Planned</button></article></div></section><section class="section maintenance"><div class="container narrow"><h2>What the desktop edition includes today</h2><div class="download-checks"><span>✓ No licence key</span><span>✓ No cloud account</span><span>✓ Standard SQLite database</span><span>✓ Offline from first launch</span><span>✓ Full sales lifecycle (quotes → orders → invoices)</span><span>✓ Contracts and tasks</span><span>✓ User management</span><span>✓ Team Workspace mode for small teams (Docker)</span><span>✓ Windows installer (unsigned, Early Access)</span><span>✓ Backup and restore</span><span>✓ Self-service password change</span><span>✓ Open-source code</span><span>○ Code-signed installer — planned</span></div><div class="hero-actions"><a class="btn btn-secondary" href="/roadmap#desktop">View desktop roadmap</a><a class="btn btn-secondary" href="https://github.com/vikram2409-eng/Lanesra-OS/tree/main/desktop" target="_blank" rel="noopener">View source on GitHub</a></div></div></section></main>${publicFooter()}`;bindPublicNav()}
function bindPublicNav(){document.querySelectorAll('.menu-toggle').forEach(btn=>{btn.onclick=()=>{const nav=btn.closest('.landing-nav');const drawer=nav.querySelector('.mobile-drawer');const open=drawer.hasAttribute('hidden');if(open)drawer.removeAttribute('hidden');else drawer.setAttribute('hidden','');btn.setAttribute('aria-expanded',String(open));btn.textContent=open?'×':'☰'}});document.querySelectorAll('.mobile-drawer a').forEach(a=>a.addEventListener('click',()=>{const drawer=a.closest('.mobile-drawer');drawer.setAttribute('hidden','');const btn=drawer.closest('.landing-nav').querySelector('.menu-toggle');btn.textContent='☰';btn.setAttribute('aria-expanded','false')}))}
const path=location.pathname.replace(/\/$/,'')||'/'; if(path==='/demo')appShell();else if(path==='/roadmap')roadmapPage();else if(path==='/changelog')changelogPage();else if(path==='/principles'||path==='/journey'||path==='/our-story'||path==='/about')principlesPage();else if(path==='/compare')comparePage();else if(path==='/download')downloadPage();else landing();
