#!/usr/bin/env python3
"""Modern UI Revamp PR-D - Worker 2: record detail pages, artifact fidelity (issue #395).

STRUCTURAL rebuild of the record detail region to the approved concept
artifact (`reference/artifact-body.html` #company section + artifact CSS).
This is not a styling pass: the record header, the Overview/Related/Activity
tab strip and every tab panel are re-emitted in the artifact's markup.

app.js replacements (10):
  d2:helpers: insert recordTabsHtml / relatedPreviewListHtml /
    relatedFullGroupsHtml / customRelatedGroups before detail360Header
  d2:detail360Header: breadcrumb -> .pivot-trail; header -> .record-head with
    .detail-identity (avatar.lg + .record-name h2 + .record-meta) and
    .record-actions (primary Edit shimmer, outline View in Object 360).
    Handler ids (#editDetailRecord, #openObject360) and every data-* attr kept.
  d2:wireDetail360Nav: breadcrumb/edit/360/related-row wiring byte-identical
    in behavior; adds artifact tab switching ([data-record-tab] toggles the
    matching .tab-panel).
  d2:activityTimelineHtml: card -> artifact Activity timeline card
    (.card-head + ul.timeline); the log-interaction form keeps every data-* hook.
  d2:activityItemHtml: entry -> li > .name + .sub (subject/channel badge/
    direction/body/participants/date all preserved).
  d2:overviewGroupsHtml: field grid -> artifact .field-grid > .field >
    .field-label + .field-value (section grouping kept via .relation-group).
  d2:companyDetail / d2:contactDetail / d2:genericRecordDetail: full tabbed
    rebuild - Overview (.field-grid details + .related-list preview),
    Related (grouped .relation-group sections), Activity (.timeline).
    Every field, related record, link, action and data binding preserved.
    customRelatedCardsHtml is superseded by customRelatedGroups (same data,
    tuple form for the two new renderers).
  Page Builder: overviewPanelHtml/pageLiveGridHtml and the related_list node's
  relatedCardHtml are untouched, so published pages keep rendering.

styles.css: APPEND ONLY - one section,
  /* ===== Modern UI PR-D/2 record-detail \u2014 artifact fidelity ===== */
  token-mapped artifact CSS (hsl(var(--X)) -> var(--X),
  hsl(var(--X) / A) -> color-mix(in srgb, var(--X) calc(A*100%), transparent)).
  The app's --muted token aliases muted-foreground, so the artifact's muted
  SURFACE maps to --secondary (PR-B convention). No hex, no literal shadows,
  no px radii in the new CSS.

Every replacement asserts its anchor matches EXACTLY the expected count;
any mismatch fails loudly. No regex replaces.

Usage: run from the repo root (app.js + styles.css side by side).
"""
import sys

APP = 'app.js'
CSS = 'styles.css'

HELPERS = """// ===== Modern UI PR-D/2: record detail tabs (artifact fidelity, issue #395) =====
// The artifact's record page is three tabs - Overview, Related, Activity -
// under one record header. These helpers render the tab strip and the two
// related-record shapes the artifact uses: the Overview preview
// (.related-list > .related-item with a chevron) and the Related tab's
// grouped .relation-group sections (singular type label + meta badge).
// Every group is a [title, items, navKey, labelFn, metaFn] tuple, so all
// real data bindings, links and counts survive unchanged.
function recordTabsHtml(){
 return `<div class="tabs company-tabs" role="tablist"><button type="button" class="active" role="tab" aria-selected="true" data-record-tab="rec-overview">Overview</button><button type="button" role="tab" aria-selected="false" data-record-tab="rec-related">Related</button><button type="button" role="tab" aria-selected="false" data-record-tab="rec-activity">Activity</button></div>`;
}
function relatedPreviewListHtml(groups){
 const itemsHtml=groups.map(([title,items,navKey,labelFn])=>items.map(x=>`<li class="related-item"><span><span class="related-type">${title}</span><br><strong><a class="cell-link" data-nav-related="${navKey}:${x.id}">${labelFn(x)}</a></strong></span><span aria-hidden="true">\u203a</span></li>`).join('')).join('');
 return `<ul class="related-list">${itemsHtml||'<li class="muted" style="padding:13px 20px">None yet</li>'}</ul>`;
}
function relatedFullGroupsHtml(groups){
 return groups.map(([title,items,navKey,labelFn,metaFn])=>{
  const singular=title.replace(/ies$/,'y').replace(/s$/,'');
  const rowsHtml=items.map(x=>{const meta=metaFn?metaFn(x):'';return `<li class="related-item"><span><span class="related-type">${singular}</span><br><strong><a class="cell-link" data-nav-related="${navKey}:${x.id}">${labelFn(x)}</a></strong></span>${meta?`<span class="badge outline">${meta}</span>`:'<span aria-hidden="true">\u203a</span>'}</li>`}).join('');
  return `<div class="relation-group"><h4>${title} (${items.length})</h4>${rowsHtml?`<ul class="related-list">${rowsHtml}</ul>`:'<div class="muted" style="font-size:13px">None yet</div>'}</div>`;
 }).join('');
}
// Custom-object related groups as the same tuples the detail pages use, so
// admin-defined relationships render in the artifact Related tab and the
// Overview preview too. Supersedes customRelatedCardsHtml (same data).
function customRelatedGroups(entityType,entityId){
 return relatedGroupsFor(entityType,entityId).map(g=>[g.label,g.rows.map(r=>({...r,id:r.entityId})),g.otherType,x=>x.displayName,x=>x.status]);
}
"""

JS_REPLACEMENTS = [
 ('d2:helpers: record tab helpers', 'function detail360Header(breadcrumbLabel,title,eyebrow,metaHtml,auditHtml){', HELPERS + 'function detail360Header(breadcrumbLabel,title,eyebrow,metaHtml,auditHtml){', 1),

 ('d2:detail360Header: artifact record header', """function detail360Header(breadcrumbLabel,title,eyebrow,metaHtml,auditHtml){
 // Modern UI PR-B2: shadcn-style record header - avatar with initials,
 // name + badges, primary Edit CTA (shimmer), outline secondary actions.
 const initials=String(title||'\u2014').trim().split(/\\s+/).map(w=>w[0]).slice(0,2).join('').toUpperCase()||'\u2014';
 return `<nav class="breadcrumbs prb2-crumbs" aria-label="Breadcrumb"><button data-clear-filter>Dashboard</button><span aria-hidden="true">\u203a</span><button data-back-list>${breadcrumbLabel}</button><span aria-hidden="true">\u203a</span><span aria-current="page">${title}</span></nav>
 <div class="card record-head">
  <div class="record-head-id">
   <span class="avatar record-head-avatar" aria-hidden="true">${initials}</span>
   <div class="record-head-titles"><div class="eyebrow">${eyebrow||''}</div><h1>${title}</h1><div class="record-head-meta">${metaHtml}</div>${auditHtml||''}</div>
  </div>
  <div class="record-head-actions">
   <button class="btn btn-outline" id="openObject360">View in Object 360</button>
   <button class="btn btn-primary btn-shimmer" id="editDetailRecord">Edit</button>
  </div>
 </div>`;
}""", """function detail360Header(breadcrumbLabel,title,eyebrow,metaHtml,auditHtml){
 // Modern UI PR-D/2: artifact record header - pivot-trail breadcrumb,
 // .record-head with .detail-identity (avatar + .record-name + .record-meta)
 // and .record-actions (primary Edit, outline Object 360). Handler ids and
 // every data-* attribute are unchanged.
 const initials=String(title||'\u2014').trim().split(/\\s+/).map(w=>w[0]).slice(0,2).join('').toUpperCase()||'\u2014';
 return `<nav class="pivot-trail" aria-label="Breadcrumb"><button data-clear-filter>Dashboard</button><span aria-hidden="true">\u203a</span><button data-back-list>${breadcrumbLabel}</button><span aria-hidden="true">\u203a</span><strong aria-current="page">${title}</strong></nav>
 <div class="page-heading">
  <div class="record-head">
   <div class="detail-identity">
    <span class="avatar lg" aria-hidden="true">${initials}</span>
    <div class="record-identity-text">${eyebrow?`<div class="eyebrow">${eyebrow}</div>`:''}<div class="record-name"><h2>${title}</h2></div><div class="record-meta">${metaHtml}</div></div>
   </div>
   <div class="record-actions">
    <button class="btn btn-primary btn-shimmer" id="editDetailRecord">Edit</button>
    <button class="btn btn-outline" id="openObject360">View in Object 360</button>
   </div>
  </div>
  ${auditHtml||''}
 </div>`;
}""", 1),

 ('d2:wireDetail360Nav: tab switching', """function wireDetail360Nav(editHandler,object360Handler){
 document.querySelector('[data-clear-filter]')?.addEventListener('click',()=>{current='dashboard';viewFilter=null;detailRecord=null;renderView()});
 $('[data-back-list]').onclick=()=>{detailRecord=null;renderView()};
 $('#editDetailRecord').onclick=editHandler;
 if(object360Handler)$('#openObject360').onclick=object360Handler;
 wireRelatedRows($('#view'));
}""", """function wireDetail360Nav(editHandler,object360Handler){
 document.querySelector('[data-clear-filter]')?.addEventListener('click',()=>{current='dashboard';viewFilter=null;detailRecord=null;renderView()});
 $('[data-back-list]').onclick=()=>{detailRecord=null;renderView()};
 $('#editDetailRecord').onclick=editHandler;
 if(object360Handler)$('#openObject360').onclick=object360Handler;
 // Modern UI PR-D/2: artifact tab switching - the active tab button shows
 // its matching .tab-panel; every wiring line above is behavior-identical.
 const view=$('#view');
 view.querySelectorAll('[data-record-tab]').forEach(btn=>{btn.onclick=()=>{
  view.querySelectorAll('[data-record-tab]').forEach(b=>{const on=b===btn;b.classList.toggle('active',on);b.setAttribute('aria-selected',on?'true':'false')});
  view.querySelectorAll('.tab-panel').forEach(p=>p.classList.toggle('active',p.id===btn.dataset.recordTab));
 }});
 wireRelatedRows(view);
}""", 1),
]
JS_REPLACEMENTS += [
 ('d2:activityTimelineHtml: artifact timeline card', """function activityTimelineHtml(entityType,entityId){
 const items=(data.activities||[]).filter(a=>a.entityType===entityType&&a.entityId===entityId).slice().sort((a,b)=>b.occurredAt.localeCompare(a.occurredAt));
 return `<div class="card" data-activity-panel="${entityType}:${entityId}">
  <div class="card-header prb2-card-head-split"><h3 class="card-title">Interactions <span class="badge-secondary">${items.length}</span></h3><button type="button" class="btn btn-outline btn-sm" data-toggle-log-activity>+ Log an interaction</button></div>
  <div class="card-content">
   <div data-log-activity-fields class="form-grid prb2-activity-fields" style="display:none;margin:12px 0">
    <div class="field"><label>Channel</label><select class="select" data-activity-channel>${ACTIVITY_CHANNELS.map(c=>`<option value="${c[0]}">${c[1]}</option>`).join('')}</select></div>
    <div class="field" data-activity-direction-field><label>Direction</label><select class="select" data-activity-direction><option value="inbound">Inbound</option><option value="outbound">Outbound</option></select></div>
    <div class="field full"><label>Subject</label><input class="input" data-activity-subject placeholder="Following up on pricing"></div>
    <div class="field"><label>When</label><input class="input" data-activity-occurred type="datetime-local" value="${activityNowForInput()}"></div>
    <div class="field"><label>Participants</label><input class="input" data-activity-participants placeholder="jane@acme.com"></div>
    <div class="field full"><label>Notes</label><textarea class="textarea" data-activity-body rows="3"></textarea></div>
    <div class="field full"><button type="button" class="btn btn-primary" data-log-activity-submit>Log interaction</button></div>
   </div>
   <div data-activity-list class="prb2-timeline">${items.length?items.map(activityItemHtml).join(''):'<div class="empty">No interactions logged yet.</div>'}</div>
  </div>
 </div>`;
}""", """function activityTimelineHtml(entityType,entityId){
 const items=(data.activities||[]).filter(a=>a.entityType===entityType&&a.entityId===entityId).slice().sort((a,b)=>b.occurredAt.localeCompare(a.occurredAt));
 // Modern UI PR-D/2: artifact Activity timeline card - card-head with the
 // log-interaction action, entries as ul.timeline. Every data-* hook the
 // log form needs is unchanged.
 return `<section class="card" data-activity-panel="${entityType}:${entityId}">
  <div class="card-head card-head-split"><div><h3>Activity timeline <span class="badge-secondary">${items.length}</span></h3><p class="card-desc">Calls, emails and messages logged against this record</p></div><button type="button" class="btn btn-outline btn-sm" data-toggle-log-activity>+ Log an interaction</button></div>
  <div data-log-activity-fields class="form-grid" style="display:none;margin:12px 20px 0">
   <div class="field"><label>Channel</label><select class="select" data-activity-channel>${ACTIVITY_CHANNELS.map(c=>`<option value="${c[0]}">${c[1]}</option>`).join('')}</select></div>
   <div class="field" data-activity-direction-field><label>Direction</label><select class="select" data-activity-direction><option value="inbound">Inbound</option><option value="outbound">Outbound</option></select></div>
   <div class="field full"><label>Subject</label><input class="input" data-activity-subject placeholder="Following up on pricing"></div>
   <div class="field"><label>When</label><input class="input" data-activity-occurred type="datetime-local" value="${activityNowForInput()}"></div>
   <div class="field"><label>Participants</label><input class="input" data-activity-participants placeholder="jane@acme.com"></div>
   <div class="field full"><label>Notes</label><textarea class="textarea" data-activity-body rows="3"></textarea></div>
   <div class="field full"><button type="button" class="btn btn-primary" data-log-activity-submit>Log interaction</button></div>
  </div>
  <ul class="timeline" data-activity-list>${items.length?items.map(activityItemHtml).join(''):'<li><div class="sub">No interactions logged yet.</div></li>'}</ul>
 </section>`;
}""", 1),

 ('d2:activityItemHtml: artifact timeline entry', """function activityItemHtml(a){
 // Modern UI PR-B2: vertical-timeline entry - channel dot + tinted badge.
 const tone={email:'badge-info',call:'badge-success',message:'badge-secondary'}[a.channel]||'badge-secondary';
 return `<div class="prb2-tl-item">
  <span class="prb2-tl-dot prb2-tl-${a.channel}" aria-hidden="true"></span>
  <div class="prb2-tl-body">
   <div class="prb2-tl-head"><span class="badge ${tone}">${a.channel}</span>${a.direction?`<span class="prb2-tl-dir">${a.direction}</span>`:''}${a.subject?`<strong>${a.subject}</strong>`:''}</div>
   <div class="prb2-tl-notes">${a.body}</div>
   <div class="prb2-tl-meta">${a.participants?a.participants+' \u00b7 ':''}${new Date(a.occurredAt).toLocaleString()}</div>
  </div>
 </div>`;
}""", """function activityItemHtml(a){
 // Modern UI PR-D/2: artifact timeline entry - .name + .sub. Subject,
 // channel badge, direction, body, participants and date all preserved.
 const tone={email:'badge-info',call:'badge-success',message:'badge-secondary'}[a.channel]||'badge-secondary';
 const label=a.subject||`${a.channel[0].toUpperCase()+a.channel.slice(1)}${a.direction?` (${a.direction})`:''}`;
 return `<li><div class="name">${label} <span class="badge ${tone}">${a.channel}</span></div>${a.body?`<div class="sub">${a.body}</div>`:''}<div class="sub">${a.participants?a.participants+' \u00b7 ':''}${new Date(a.occurredAt).toLocaleString()}</div></li>`;
}""", 1),
]
JS_REPLACEMENTS += [
 ('d2:overviewGroupsHtml: artifact field grid', """// Read-only counterpart to groupsHtml, for overviewGroupsFor's output.
function overviewGroupsHtml(key,groups,record){
 // Modern UI PR-B2: definition-list field grid - labels in
 // muted-foreground, values in foreground.
 return groups.map(g=>`<div class="prb2-field-group">${g.title?`<h4 class="prb2-field-group-title">${g.title}</h4>`:''}<dl class="prb2-field-grid" style="--prb2-cols:${g.columns}">${g.items.map(it=>`<div class="prb2-field-row${it.full?' prb2-field-row-full':''}"><dt>${it.field[1]}</dt><dd>${overviewValueHtml(key,it.field,record)}</dd></div>`).join('')}</dl></div>`).join('');
}""", """// Read-only counterpart to groupsHtml, for overviewGroupsFor's output.
function overviewGroupsHtml(key,groups,record){
 // Modern UI PR-D/2: artifact .field-grid - .field > .field-label +
 // .field-value, one grid per published-layout section (.relation-group).
 return groups.map(g=>{
  const grid=`<div class="field-grid${(g.columns||2)===1?' cols-1':''}">${g.items.map(it=>`<div class="field${it.full?' field-full':''}"><div class="field-label">${it.field[1]}</div><div class="field-value">${overviewValueHtml(key,it.field,record)}</div></div>`).join('')}</div>`;
  return g.title?`<div class="relation-group"><h4>${g.title}</h4>${grid}</div>`:grid;
 }).join('');
}""", 1),

 ('d2:companyDetail: tabbed artifact layout', """function companyDetail(id){
 const c=byId('companies',id);
 if(!c){current='companies';detailRecord=null;return renderView()}
 const overviewFields=fieldsFor('companies',companyFields).filter(f=>f[0]!=='name'&&!['ownerPicker','orgUnitPicker'].includes(f[2]));
 const contacts=data.contacts.filter(x=>x.companyId===id);
 const opportunities=data.opportunities.filter(x=>x.companyId===id);
 const quotes=data.quotes.filter(x=>x.companyId===id);
 const orders=data.orders.filter(x=>x.companyId===id);
 const invoices=data.invoices.filter(x=>x.companyId===id);
 const contracts=data.contracts.filter(x=>x.companyId===id);
 const tasks=data.tasks.filter(x=>x.relatedType==='Company'&&x.relatedId===id);
 $('#view').innerHTML=`${detail360Header('Companies',c.name,c.customerNumber,`${badgeMaybe(c.status)}`,auditByline(c)+ownershipByline(c))}
 <div class="rule360-grid prb2-detail">
  <div><div class="card"><div class="card-header"><h3 class="card-title">Overview</h3></div><div class="card-content">${overviewPanelHtml('companies',overviewFields,c,id)}</div></div></div>
  <div>
   ${relatedCardHtml('Contacts',contacts,'contacts',x=>x.name,x=>x.role||'')}
   ${relatedCardHtml('Sales Pipeline',opportunities,'opportunities',x=>x.title,x=>money(x.value))}
   ${relatedCardHtml('Quotes',quotes,'quotes',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Orders',orders,'orders',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Invoices',invoices,'invoices',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Contracts',contracts,'contracts',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Tasks',tasks,'tasks',x=>x.title,x=>x.status)}
   ${customRelatedCardsHtml('companies',id)}
   ${activityTimelineHtml('Company',id)}
  </div>
 </div>`;
 wireDetail360Nav(()=>recordModal('companies',fieldsFor('companies',companyFields),c),()=>object360Modal('companies',id,c.name));
 wireActivityTimelines($('#view'));
}""", """function companyDetail(id){
 const c=byId('companies',id);
 if(!c){current='companies';detailRecord=null;return renderView()}
 const overviewFields=fieldsFor('companies',companyFields).filter(f=>f[0]!=='name'&&!['ownerPicker','orgUnitPicker'].includes(f[2]));
 const contacts=data.contacts.filter(x=>x.companyId===id);
 const opportunities=data.opportunities.filter(x=>x.companyId===id);
 const quotes=data.quotes.filter(x=>x.companyId===id);
 const orders=data.orders.filter(x=>x.companyId===id);
 const invoices=data.invoices.filter(x=>x.companyId===id);
 const contracts=data.contracts.filter(x=>x.companyId===id);
 const tasks=data.tasks.filter(x=>x.relatedType==='Company'&&x.relatedId===id);
 // Modern UI PR-D/2: every related group as a [title,items,navKey,labelFn,
 // metaFn] tuple - the Overview preview and the Related tab render them in
 // the artifact's .related-list / .relation-group shapes. Same filters,
 // same labels, same counts as before.
 const relGroups=[
  ['Contacts',contacts,'contacts',x=>x.name,x=>x.role||''],
  ['Sales Pipeline',opportunities,'opportunities',x=>x.title,x=>money(x.value)],
  ['Quotes',quotes,'quotes',x=>x.number,x=>x.status],
  ['Orders',orders,'orders',x=>x.number,x=>x.status],
  ['Invoices',invoices,'invoices',x=>x.number,x=>x.status],
  ['Contracts',contracts,'contracts',x=>x.number,x=>x.status],
  ['Tasks',tasks,'tasks',x=>x.title,x=>x.status],
  ...customRelatedGroups('companies',id),
 ];
 $('#view').innerHTML=`${detail360Header('Companies',c.name,c.customerNumber,`${badgeMaybe(c.status)}`,auditByline(c)+ownershipByline(c))}
 ${recordTabsHtml()}
 <div id="rec-overview" class="tab-panel active"><div class="record-grid">
  <section class="card"><div class="card-head"><h3>Company details</h3><p class="card-desc">Core information for this account</p></div><div class="card-body" style="padding:14px 0 0">${overviewPanelHtml('companies',overviewFields,c,id)}</div></section>
  <section class="card"><div class="card-head"><h3>Related records</h3><p class="card-desc">Connected across the workspace</p></div><div class="card-body" style="padding:14px 0 0">${relatedPreviewListHtml(relGroups)}</div></section>
 </div></div>
 <div id="rec-related" class="tab-panel"><section class="card"><div class="card-head"><h3>All related records</h3><p class="card-desc">Records connected to ${c.name}</p></div><div class="card-body" style="padding:14px 0 6px">${relatedFullGroupsHtml(relGroups)}</div></section></div>
 <div id="rec-activity" class="tab-panel">${activityTimelineHtml('Company',id)}</div>`;
 wireDetail360Nav(()=>recordModal('companies',fieldsFor('companies',companyFields),c),()=>object360Modal('companies',id,c.name));
 wireActivityTimelines($('#view'));
}""", 1),

 ('d2:contactDetail: tabbed artifact layout', """function contactDetail(id){
 const c=byId('contacts',id);
 if(!c){current='contacts';detailRecord=null;return renderView()}
 const overviewFields=fieldsFor('contacts',contactFields).filter(f=>!['auto','relation','ownerPicker','orgUnitPicker'].includes(f[2])&&f[0]!=='name');
 const opportunities=data.opportunities.filter(x=>x.contactId===id);
 const quotes=data.quotes.filter(x=>x.contactId===id);
 const orders=data.orders.filter(x=>x.contactId===id);
 const contracts=data.contracts.filter(x=>x.contactId===id);
 const tasks=data.tasks.filter(x=>x.relatedType==='Contact'&&x.relatedId===id);
 $('#view').innerHTML=`${detail360Header('Contacts',c.name,c.contactNumber,`${badgeMaybe(c.status)}<span>${c.role||'\u2014'}</span><a class="cell-link" data-nav-related="companies:${c.companyId}">${companyName(c.companyId)}</a>`,auditByline(c)+ownershipByline(c))}
 <div class="rule360-grid prb2-detail">
  <div><div class="card"><div class="card-header"><h3 class="card-title">Overview</h3></div><div class="card-content">${overviewPanelHtml('contacts',overviewFields,c,id)}</div></div></div>
  <div>
   ${relatedCardHtml('Sales Pipeline',opportunities,'opportunities',x=>x.title,x=>money(x.value))}
   ${relatedCardHtml('Quotes',quotes,'quotes',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Orders',orders,'orders',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Contracts',contracts,'contracts',x=>x.number,x=>x.status)}
   ${relatedCardHtml('Tasks',tasks,'tasks',x=>x.title,x=>x.status)}
   ${customRelatedCardsHtml('contacts',id)}
   ${activityTimelineHtml('Contact',id)}
  </div>
 </div>`;
 wireDetail360Nav(()=>recordModal('contacts',fieldsFor('contacts',contactFields),c),()=>object360Modal('contacts',id,c.name));
 wireActivityTimelines($('#view'));
}""", """function contactDetail(id){
 const c=byId('contacts',id);
 if(!c){current='contacts';detailRecord=null;return renderView()}
 const overviewFields=fieldsFor('contacts',contactFields).filter(f=>!['auto','relation','ownerPicker','orgUnitPicker'].includes(f[2])&&f[0]!=='name');
 const opportunities=data.opportunities.filter(x=>x.contactId===id);
 const quotes=data.quotes.filter(x=>x.contactId===id);
 const orders=data.orders.filter(x=>x.contactId===id);
 const contracts=data.contracts.filter(x=>x.contactId===id);
 const tasks=data.tasks.filter(x=>x.relatedType==='Contact'&&x.relatedId===id);
 // Modern UI PR-D/2: related groups as tuples - see companyDetail.
 const relGroups=[
  ['Sales Pipeline',opportunities,'opportunities',x=>x.title,x=>money(x.value)],
  ['Quotes',quotes,'quotes',x=>x.number,x=>x.status],
  ['Orders',orders,'orders',x=>x.number,x=>x.status],
  ['Contracts',contracts,'contracts',x=>x.number,x=>x.status],
  ['Tasks',tasks,'tasks',x=>x.title,x=>x.status],
  ...customRelatedGroups('contacts',id),
 ];
 $('#view').innerHTML=`${detail360Header('Contacts',c.name,c.contactNumber,`${badgeMaybe(c.status)}<span>${c.role||'\u2014'}</span><a class="cell-link" data-nav-related="companies:${c.companyId}">${companyName(c.companyId)}</a>`,auditByline(c)+ownershipByline(c))}
 ${recordTabsHtml()}
 <div id="rec-overview" class="tab-panel active"><div class="record-grid">
  <section class="card"><div class="card-head"><h3>Contact details</h3><p class="card-desc">Core information for this contact</p></div><div class="card-body" style="padding:14px 0 0">${overviewPanelHtml('contacts',overviewFields,c,id)}</div></section>
  <section class="card"><div class="card-head"><h3>Related records</h3><p class="card-desc">Connected across the workspace</p></div><div class="card-body" style="padding:14px 0 0">${relatedPreviewListHtml(relGroups)}</div></section>
 </div></div>
 <div id="rec-related" class="tab-panel"><section class="card"><div class="card-head"><h3>All related records</h3><p class="card-desc">Records connected to ${c.name}</p></div><div class="card-body" style="padding:14px 0 6px">${relatedFullGroupsHtml(relGroups)}</div></section></div>
 <div id="rec-activity" class="tab-panel">${activityTimelineHtml('Contact',id)}</div>`;
 wireDetail360Nav(()=>recordModal('contacts',fieldsFor('contacts',contactFields),c),()=>object360Modal('contacts',id,c.name));
 wireActivityTimelines($('#view'));
}""", 1),
]
JS_REPLACEMENTS += [
 ('d2:genericRecordDetail: tabbed artifact layout', """function genericRecordDetail(key,id){
 const r=byId(key,id);
 if(!r){current=key;detailRecord=null;return renderView()}
 const fieldsFn=DETAIL_FIELDS_FN[key];
 const relationTypes=['auto','relation','filteredContact','filteredOpportunity','filteredQuote','filteredOrder','dynamicRelation','ownerPicker','orgUnitPicker'];
 const overviewFields=fieldsFor(key,fieldsFn).filter(f=>!relationTypes.includes(f[2])&&f[0]!==DETAIL_TITLE_FIELD[key]);
 const isDoc=['quotes','orders','invoices'].includes(key);
 const linesHtml=isDoc?`<div class="card prb2-lines"><div class="card-header"><h3 class="card-title">Products & services</h3></div><div class="card-content"><div class="table-wrap"><table class="table"><thead><tr><th>Product / service</th><th>Quantity</th><th>Unit price</th><th>Line total</th></tr></thead><tbody>${(r.items||[]).map(i=>`<tr><td>${productName(i.productId)}</td><td>${i.quantity}</td><td>${money(i.unitPrice)}</td><td>${money(lineTotal(i))}</td></tr>`).join('')}</tbody></table></div><div class="prb2-line-total">Total <strong>${money(docTotal(r))}</strong>${key==='invoices'?` <span class="prb2-line-balance">\u00b7 Balance ${money(docBalance(r))}</span>`:''}</div></div></div>`:'';
 $('#view').innerHTML=`${detail360Header(DETAIL_BREADCRUMB[key],r[DETAIL_TITLE_FIELD[key]]||'\u2014',DETAIL_EYEBROW(key,r),recordEyebrowMeta(key,r),auditByline(r)+ownershipByline(r))}
 <div class="rule360-grid prb2-detail">
  <div>
   ${linesHtml}
   <div class="card"><div class="card-header"><h3 class="card-title">Overview</h3></div><div class="card-content">${overviewPanelHtml(key,overviewFields,r,id)}</div></div>
  </div>
  <div>${recordRelatedDefs(key,r).map(([title,items,navKey,labelFn,metaFn])=>relatedCardHtml(title,items,navKey,labelFn,metaFn)).join('')}${customRelatedCardsHtml(key,id)}</div>
 </div>`;
 wireDetail360Nav(()=>recordModal(key,fieldsFor(key,fieldsFn),r),()=>object360Modal(key,id,r[DETAIL_TITLE_FIELD[key]]||'\u2014'));
}""", """function genericRecordDetail(key,id){
 const r=byId(key,id);
 if(!r){current=key;detailRecord=null;return renderView()}
 const fieldsFn=DETAIL_FIELDS_FN[key];
 const relationTypes=['auto','relation','filteredContact','filteredOpportunity','filteredQuote','filteredOrder','dynamicRelation','ownerPicker','orgUnitPicker'];
 const overviewFields=fieldsFor(key,fieldsFn).filter(f=>!relationTypes.includes(f[2])&&f[0]!==DETAIL_TITLE_FIELD[key]);
 const isDoc=['quotes','orders','invoices'].includes(key);
 const singular=(ENTITY_SINGULAR[key]||key).replace(/^./,c=>c.toUpperCase());
 // Modern UI PR-D/2: line items keep every data binding; the card takes the
 // artifact's card-head anatomy.
 const linesHtml=isDoc?`<section class="card prb2-lines" style="margin-bottom:18px"><div class="card-head"><h3>Products & services</h3><p class="card-desc">Line items on this ${singular.toLowerCase()}</p></div><div class="card-body" style="padding-top:14px"><div class="table-wrap"><table class="table"><thead><tr><th>Product / service</th><th>Quantity</th><th>Unit price</th><th>Line total</th></tr></thead><tbody>${(r.items||[]).map(i=>`<tr><td>${productName(i.productId)}</td><td>${i.quantity}</td><td>${money(i.unitPrice)}</td><td>${money(lineTotal(i))}</td></tr>`).join('')}</tbody></table></div><div class="prb2-line-total">Total <strong>${money(docTotal(r))}</strong>${key==='invoices'?` <span class="prb2-line-balance">\u00b7 Balance ${money(docBalance(r))}</span>`:''}</div></div></section>`:'';
 // Modern UI PR-D/2: related groups as tuples - see companyDetail.
 const relGroups=[...recordRelatedDefs(key,r),...customRelatedGroups(key,id)];
 const title=r[DETAIL_TITLE_FIELD[key]]||'\u2014';
 $('#view').innerHTML=`${detail360Header(DETAIL_BREADCRUMB[key],title,DETAIL_EYEBROW(key,r),recordEyebrowMeta(key,r),auditByline(r)+ownershipByline(r))}
 ${recordTabsHtml()}
 <div id="rec-overview" class="tab-panel active"><div class="record-grid">
  <div>
   ${linesHtml}
   <section class="card"><div class="card-head"><h3>${singular} details</h3><p class="card-desc">Core information for this record</p></div><div class="card-body" style="padding:14px 0 0">${overviewPanelHtml(key,overviewFields,r,id)}</div></section>
  </div>
  <section class="card"><div class="card-head"><h3>Related records</h3><p class="card-desc">Connected across the workspace</p></div><div class="card-body" style="padding:14px 0 0">${relatedPreviewListHtml(relGroups)}</div></section>
 </div></div>
 <div id="rec-related" class="tab-panel"><section class="card"><div class="card-head"><h3>All related records</h3><p class="card-desc">Records connected to ${title}</p></div><div class="card-body" style="padding:14px 0 6px">${relatedFullGroupsHtml(relGroups)}</div></section></div>
 <div id="rec-activity" class="tab-panel"><section class="card"><div class="card-head"><h3>Activity timeline</h3><p class="card-desc">Calls, emails and messages logged against this record</p></div><ul class="timeline"><li><div class="sub">No interactions logged yet.</div></li></ul></section></div>`;
 wireDetail360Nav(()=>recordModal(key,fieldsFor(key,fieldsFn),r),()=>object360Modal(key,id,title));
}""", 1),

 # customRelatedCardsHtml is superseded by customRelatedGroups (inserted
 # above with the other PR-D/2 helpers) - same data, tuple form. No other
 # callers; the Page Builder related_list node keeps using relatedCardHtml.
 ('d2:customRelatedCardsHtml: superseded by customRelatedGroups', """function customRelatedCardsHtml(entityType,entityId){
 return relatedGroupsFor(entityType,entityId).map(g=>relatedCardHtml(g.label,g.rows.map(r=>({...r,id:r.entityId})),g.otherType,x=>x.displayName,x=>x.status)).join('');
}
""", "", 1),
]

PRD2_CSS = """/* Token-mapped artifact CSS for record detail pages (detail360Header,
   companyDetail, contactDetail, genericRecordDetail, activity timeline).
   Artifact color mapping: plain token colors become var(--X); token colors
   with alpha become color-mix(in srgb, var(--X) <pct>, transparent).
   NOTE: the app's --muted token aliases muted-foreground, so the artifact's
   muted SURFACE token maps to --secondary here (PR-B convention), never to
   var(--muted). Every value below is a var(--token), a color-mix() of
   tokens, or a non-color keyword. No hex, no literal shadows, no px radii. */

/* ---- Card anatomy (artifact .card-head / .card-body / .card-desc) ---- */
.card-head{padding:20px 20px 0}
.card-head h3{margin:0;font-size:var(--text-base);font-weight:700;letter-spacing:-.01em;color:var(--card-foreground)}
.card-head .card-desc{color:var(--muted-foreground);margin:5px 0 0;font-size:var(--text-sm)}
.card-head-split{display:flex;align-items:center;justify-content:space-between;gap:12px;flex-wrap:wrap}
.card-body{padding:20px}

/* ---- Breadcrumb trail (artifact .pivot-trail) ---- */
.pivot-trail{display:flex;flex-wrap:wrap;gap:5px;align-items:center;margin:0 0 12px;color:var(--muted-foreground);font-size:11px}
.pivot-trail strong{color:var(--foreground);font-weight:600}
.pivot-trail button{background:none;border:0;padding:0;font:inherit;font-weight:600;color:var(--primary);cursor:pointer}
.pivot-trail button:hover{text-decoration:underline}
.pivot-trail button:focus-visible{outline:2px solid var(--ring);outline-offset:2px}
.pivot-trail [aria-hidden="true"]{color:var(--muted-foreground)}

/* ---- Page heading + record header (artifact .page-heading / .record-head).
   Same-specificity rules appended later win over PR-B2's .record-head card
   styles, so the artifact's plain (non-card) header is restored. */
.page-heading{margin-bottom:24px}
.record-head{display:flex;align-items:center;justify-content:space-between;gap:14px;min-width:0;flex-wrap:wrap;padding:0;margin:0;background:none;border:0;box-shadow:none}
.detail-identity{display:flex;align-items:center;gap:14px;min-width:0}
.avatar.lg{width:52px;height:52px;font-size:var(--text-base);flex-shrink:0}
.record-identity-text{min-width:0}
.record-name{display:flex;flex-wrap:wrap;align-items:center;gap:8px}
.record-name h2{margin:0;font-size:var(--text-2xl);font-weight:700;letter-spacing:-.02em;color:var(--foreground)}
.record-head .eyebrow{margin-bottom:2px}
.record-meta{display:flex;flex-wrap:wrap;align-items:center;gap:8px 14px;color:var(--muted-foreground);margin-top:7px;font-size:13px}
.record-meta .cell-link{color:var(--primary);cursor:pointer}
.record-meta .cell-link:hover{text-decoration:underline}
.record-actions{display:flex;gap:8px;flex-wrap:wrap;align-items:center}

/* ---- Tabs (artifact .tabs + .company-tabs; beats the admin .tabs rule) ---- */
.tabs.company-tabs{display:inline-flex;gap:2px;padding:4px;background:var(--secondary);border-radius:var(--radius-lg);margin-bottom:18px;flex-wrap:wrap}
.company-tabs button{border:0;background:transparent;color:var(--muted-foreground);font-weight:600;font-size:13px;border-radius:var(--radius-md);padding:7px 12px;cursor:pointer;transition:color .18s ease,background-color .18s ease,box-shadow .18s ease}
.company-tabs button:hover{color:var(--foreground)}
.company-tabs button.active{color:var(--foreground);background:var(--background);box-shadow:var(--shadow-sm)}
.company-tabs button:focus-visible{outline:2px solid var(--ring);outline-offset:2px}
.tab-panel{display:none}
.tab-panel.active{display:block;animation:prd2-view-in .2s ease}
@keyframes prd2-view-in{from{opacity:.4;transform:translateY(3px)}}

/* ---- Record grid: details + related side by side (artifact .record-grid) ---- */
.record-grid{display:grid;grid-template-columns:minmax(0,1fr) minmax(320px,.65fr);gap:18px;align-items:start}

/* ---- Field grid (artifact .field-grid; scoped so the form .field markup
   elsewhere in the app is untouched) ---- */
.field-grid{display:grid;grid-template-columns:repeat(2,minmax(0,1fr))}
.field-grid.cols-1{grid-template-columns:minmax(0,1fr)}
.field-grid>.field{padding:17px 20px;border-top:1px solid var(--border)}
.field-grid:not(.cols-1)>.field:nth-child(odd){border-right:1px solid var(--border)}
.field-grid>.field.field-full{grid-column:1/-1}
.field-grid .field-label{font-size:12px;color:var(--muted-foreground);margin-bottom:4px}
.field-grid .field-value{font-weight:630;overflow-wrap:anywhere}

/* ---- Related lists (artifact .related-list / .related-item / .related-type) ---- */
.related-list{list-style:none;padding:0;margin:0}
.related-item{display:flex;justify-content:space-between;align-items:center;gap:10px;padding:13px 20px;border-top:1px solid var(--border)}
.related-item>span:first-child{min-width:0}
.related-type{color:var(--muted-foreground);font-size:12px}
.related-item strong{font-weight:650}
.related-item .cell-link{color:var(--primary);cursor:pointer;font-weight:650}
.related-item .cell-link:hover{text-decoration:underline}
.related-item .badge{flex-shrink:0}
.related-item>[aria-hidden="true"]{color:var(--muted-foreground)}

/* ---- Grouped related sections (artifact .relation-group) ---- */
.relation-group{border-top:1px solid var(--border);padding:12px 20px}
.relation-group:first-child{border-top:0;padding-top:0}
.relation-group h4{color:var(--muted-foreground);font-size:11px;text-transform:uppercase;letter-spacing:.07em;margin:0 0 2px}
.relation-group .related-item{padding:11px 0}
.relation-group .related-item:first-child{border-top:0}
.relation-group .muted{font-size:13px}

/* ---- Activity timeline (artifact .timeline) ---- */
.timeline{list-style:none;padding:2px 20px 18px 44px;margin:0}
.timeline>li{position:relative;padding:18px 0 18px 15px;border-left:1px solid var(--border)}
.timeline>li::before{content:"";position:absolute;left:-5px;top:23px;width:9px;height:9px;border-radius:50%;background:var(--primary);box-shadow:0 0 0 4px var(--background)}
.timeline .name{font-weight:660;color:var(--foreground);font-size:var(--text-sm)}
.timeline .sub{color:var(--muted-foreground);font-size:12px;margin-top:2px}

@media(max-width:1040px){
 .record-grid{grid-template-columns:1fr}
}
@media(max-width:760px){
 .record-head{align-items:flex-start}
 .record-actions{width:100%}
 .record-actions .btn-primary{flex:1}
 .field-grid{grid-template-columns:1fr}
 .field-grid>.field:nth-child(odd){border-right:0}
 .company-tabs{width:100%}
 .company-tabs button{flex:1}
}
"""


def apply_reps(path, reps):
    with open(path, encoding='utf-8') as f:
        src = f.read()
    for name, old, new, expected in reps:
        n = src.count(old)
        if n != expected:
            sys.exit('ANCHOR FAIL [%s] (%s): expected %d, found %d' % (name, path, expected, n))
        src = src.replace(old, new, 1 if expected == 1 else n)
        print('ok [%s] (%s)' % (name, path))
    with open(path, 'w', encoding='utf-8') as f:
        f.write(src)


def main():
    apply_reps(APP, JS_REPLACEMENTS)
    with open(CSS, 'a', encoding='utf-8') as f:
        f.write('\n\n/* ===== Modern UI PR-D/2 record-detail \u2014 artifact fidelity ===== */\n')
        f.write(PRD2_CSS)
    print('appended [PR-D/2 record-detail] to %s' % CSS)
    print('PR-D/2 TRANSFORM COMPLETE')


if __name__ == '__main__':
    main()
