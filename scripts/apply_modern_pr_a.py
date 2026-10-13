#!/usr/bin/env python3
"""
Modern UI Revamp PR-A - app.js transform (Worker 2: JS layer).

Applies the PR-A JavaScript changes to app.js via anchored string
replacements:

  1. Shell HTML: sidebar collapse button, theme toggle button,
     data-tooltip on #globalSearch, #cmdPalette overlay markup.
  2. toggleSidebarCollapse() + restore (localStorage lanesra-sidebar-collapsed).
  3. toggleTheme()/applyTheme()/readSavedTheme() (localStorage lanesra-theme,
     default Light, applied before first render).
  4. Command palette (#cmdPalette): Ctrl/Cmd+K + #globalSearch trigger,
     fuzzy page/action index, arrow/Enter/Esc keyboard, .cmd-item render.
  5. applyWorkspaceTheme(): shadcn-style token aliases + theme.tokens.dark
     pairs applied on documentElement when dataset.theme==='dark'.
  6. Theme Studio TS-1: per-field dark-mode inputs, numeric --radius
     override, shadow scale select, density preview row, motion budget
     select (contrast gating still checks the light pairs only).
  7. Theme Studio TS-2: 'lanesra-dark' + 'minimal-light' presets.

Every replacement asserts its anchor matches EXACTLY once; any mismatch
fails loudly so a stale app.js can never be half-patched.

Usage: run from the repo root (PATH='app.js').
"""

import sys

PATH = 'app.js'

REPLACEMENTS = []


def add(name, old, new):
    REPLACEMENTS.append((name, old, new))


# ---------------------------------------------------------------------------
# 1a. Sidebar collapse button (after .side-brand)
# ---------------------------------------------------------------------------
add('shell: sidebar collapse button',
    r'''<div class="side-brand"><span class="brand-mark">L</span><span>Lanesra OS</span><span class="demo-pill">DEMO</span></div>''',
    r'''<div class="side-brand"><span class="brand-mark">L</span><span>Lanesra OS</span><span class="demo-pill">DEMO</span></div><button class="btn-icon sidebar-toggle" id="sidebarCollapse" aria-label="Collapse sidebar" data-tooltip="Collapse sidebar">«</button>''')

# ---------------------------------------------------------------------------
# 1b. data-tooltip on the topbar search (the palette trigger)
# ---------------------------------------------------------------------------
add('shell: globalSearch tooltip',
    r'''<input id="globalSearch" autocomplete="off" placeholder="Search companies, contacts, deals…  ⌘K">''',
    r'''<input id="globalSearch" autocomplete="off" placeholder="Search companies, contacts, deals…  ⌘K" data-tooltip="Search (Ctrl+K)">''')

# ---------------------------------------------------------------------------
# 1c. Theme toggle button in .top-actions, before the avatar
# ---------------------------------------------------------------------------
add('shell: theme toggle button',
    r'''<button class="icon-btn" id="helpButton" aria-label="Help">?</button><div class="avatar">MC</div>''',
    r'''<button class="icon-btn" id="helpButton" aria-label="Help">?</button><button class="icon-btn" id="themeToggle" aria-label="Toggle theme" data-tooltip="Toggle light/dark theme">🌙</button><div class="avatar">MC</div>''')

# ---------------------------------------------------------------------------
# 1d. Command palette overlay markup (end of the shell template)
# ---------------------------------------------------------------------------
add('shell: cmdPalette overlay',
    r'''<div id="voiceConversation" class="voice-convo" hidden></div>`;''',
    r'''<div id="voiceConversation" class="voice-convo" hidden></div><div id="cmdPalette" hidden><div class="cmd-palette" role="dialog" aria-modal="true" aria-label="Command palette"><input class="cmd-input" id="cmdInput" autocomplete="off" placeholder="Type a command or search…" aria-label="Command palette search"><div class="cmd-list" id="cmdList" role="listbox"></div></div></div>`;''')

# ---------------------------------------------------------------------------
# 2+3+4. Theme / sidebar / command-palette logic, before appShell()
# ---------------------------------------------------------------------------
add('js: shell logic block (theme, sidebar, palette)',
    r'''function appShell(){
 document.title='Lanesra OS Demo';''',
    r'''// ---------------------------------------------------------------------------
// Modern UI PR-A: app-shell controls - theme toggle, collapsible sidebar,
// command palette (Ctrl/Cmd+K). Shell state persists in localStorage
// (best-effort: a fresh visitor with an empty store gets the Light default
// and an expanded sidebar).
// ---------------------------------------------------------------------------
function readSavedTheme(){
 try{
  const t=localStorage.getItem('lanesra-theme');
  return t==='dark'||t==='light'?t:'light';
 }catch(e){return 'light'}
}
function applyTheme(t){
 const theme=t==='dark'?'dark':'light';
 document.documentElement.dataset.theme=theme;
 try{localStorage.setItem('lanesra-theme',theme)}catch(e){}
 const btn=document.getElementById('themeToggle');
 if(btn)btn.textContent=theme==='dark'?'☀️':'🌙';
 // Re-apply the workspace theme so theme.tokens.dark pairs take effect
 // (or are cleared) the moment the mode flips - no re-render needed.
 if(typeof applyWorkspaceTheme==='function')applyWorkspaceTheme();
}
function toggleTheme(){
 applyTheme(document.documentElement.dataset.theme==='dark'?'light':'dark');
}
function toggleSidebarCollapse(){
 const sidebar=document.getElementById('appSidebar');
 if(!sidebar)return;
 const collapsed=sidebar.classList.toggle('collapsed');
 try{localStorage.setItem('lanesra-sidebar-collapsed',collapsed?'1':'0')}catch(e){}
 const btn=document.getElementById('sidebarCollapse');
 if(btn){btn.textContent=collapsed?'»':'«';btn.setAttribute('aria-label',collapsed?'Expand sidebar':'Collapse sidebar')}
}
function restoreSidebarCollapsed(){
 let collapsed=false;
 try{collapsed=localStorage.getItem('lanesra-sidebar-collapsed')==='1'}catch(e){}
 const sidebar=document.getElementById('appSidebar');
 if(!sidebar)return;
 sidebar.classList.toggle('collapsed',collapsed);
 const btn=document.getElementById('sidebarCollapse');
 if(btn){btn.textContent=collapsed?'»':'«';btn.setAttribute('aria-label',collapsed?'Expand sidebar':'Collapse sidebar')}
}
// ---------------------------------------------------------------------------
// Command palette: Ctrl/Cmd+K (or clicking the topbar search) opens a
// keyboard-first overlay with a static index of app pages, admin tabs and
// quick actions. Fuzzy-matched; Up/Down + Enter + Esc.
// ---------------------------------------------------------------------------
const CMD_PALETTE_INDEX=[
 {group:'Pages',label:'Dashboard',hint:'Home',run:()=>cmdGo('dashboard')},
 {group:'Pages',label:'Companies',hint:'Records',run:()=>cmdGo('companies')},
 {group:'Pages',label:'Contacts',hint:'Records',run:()=>cmdGo('contacts')},
 {group:'Pages',label:'Sales Pipeline',hint:'Opportunities',run:()=>cmdGo('pipeline')},
 {group:'Pages',label:'Products',hint:'Catalog',run:()=>cmdGo('products')},
 {group:'Pages',label:'Quotes',hint:'Documents',run:()=>cmdGo('quotes')},
 {group:'Pages',label:'Orders',hint:'Documents',run:()=>cmdGo('orders')},
 {group:'Pages',label:'Invoices',hint:'Documents',run:()=>cmdGo('invoices')},
 {group:'Pages',label:'Contracts',hint:'Documents',run:()=>cmdGo('contracts')},
 {group:'Pages',label:'Tasks',hint:'Records',run:()=>cmdGo('tasks')},
 {group:'Pages',label:'Reports',hint:'Analytics',run:()=>cmdGo('reports')},
 {group:'Pages',label:'Assistant',hint:'AI chat',run:()=>cmdGo('assistant')},
 {group:'Pages',label:'Partner Tiers',hint:'Admin · Custom Objects',run:()=>cmdGoAdmin('objects')},
 {group:'Pages',label:'Admin',hint:'Control center',run:()=>cmdGoAdmin(null)},
 {group:'Admin',label:'Ontology Graph',hint:'Relationship map',run:()=>cmdGoAdmin('dependencyExplorer')},
 {group:'Admin',label:'Theme Studio',hint:'Themes & tokens',run:()=>cmdGoAdmin('themeStudio')},
 {group:'Admin',label:'Custom Objects',hint:'Data model',run:()=>cmdGoAdmin('objects')},
 {group:'Admin',label:'AI Agents',hint:'Automation',run:()=>cmdGoAdmin('aiAgents')},
 {group:'Admin',label:'Users & roles',hint:'Organization',run:()=>cmdGoAdmin('users')},
 {group:'Admin',label:'Integrations',hint:'Setup',run:()=>cmdGoAdmin('integrations')},
 {group:'Admin',label:'Workflow automation',hint:'Automation',run:()=>cmdGoAdmin('workflow')},
 {group:'Admin',label:'Business rules',hint:'Automation',run:()=>cmdGoAdmin('rules')},
 {group:'Admin',label:'Page Builder',hint:'Layouts',run:()=>cmdGoAdmin('pageBuilder')},
 {group:'Admin',label:'Dashboard KPIs',hint:'Analytics',run:()=>cmdGoAdmin('kpis')},
 {group:'Actions',label:'New company',hint:'Create record',run:()=>recordModal('companies',fieldsFor('companies',companyFields))},
 {group:'Actions',label:'New contact',hint:'Create record',run:()=>recordModal('contacts',fieldsFor('contacts',contactFields))},
 {group:'Actions',label:'Toggle theme',hint:'Light / dark',run:()=>toggleTheme()},
 {group:'Actions',label:'Go to Ontology Graph',hint:'Relationship map',run:()=>openOntologyGraphFor('companies')},
];
let cmdPaletteOpen=false,cmdPaletteItems=[],cmdPaletteSel=0;
// Navigation goes through the app's existing router (current/adminTab +
// renderView()), the same mechanism the sidebar and inline search use.
function cmdGo(section){current=section;viewFilter=null;detailRecord=null;renderView()}
function cmdGoAdmin(tab){current='admin';adminView=tab?'tool':'landing';if(tab)adminTab=tab;renderView()}
// Subsequence fuzzy match: score, or -1 when q is not a subsequence of
// text. Consecutive character matches score higher.
function cmdFuzzyScore(q,text){
 q=q.toLowerCase();text=text.toLowerCase();
 let qi=0,score=0,last=-2;
 for(let i=0;i<text.length&&qi<q.length;i++){
  if(text[i]===q[qi]){score+=last===i-1?2:1;last=i;qi++}
 }
 return qi===q.length?score:-1;
}
function cmdPaletteResults(q){
 q=q.trim().toLowerCase();
 const scored=CMD_PALETTE_INDEX.map(item=>{
  if(!q)return {item,score:0};
  const s=Math.max(cmdFuzzyScore(q,item.label),cmdFuzzyScore(q,item.group+' '+item.label));
  return {item,score:s};
 }).filter(x=>x.score>=0);
 scored.sort((a,b)=>b.score-a.score);
 return scored.map(x=>x.item);
}
function openCmdPalette(){
 const overlay=document.getElementById('cmdPalette');
 const input=document.getElementById('cmdInput');
 if(!overlay||!input||cmdPaletteOpen)return;
 cmdPaletteOpen=true;
 overlay.hidden=false;
 input.value='';
 cmdPaletteSel=0;
 renderCmdList();
 input.focus();
}
function closeCmdPalette(){
 const overlay=document.getElementById('cmdPalette');
 const input=document.getElementById('cmdInput');
 if(!overlay||!cmdPaletteOpen)return;
 cmdPaletteOpen=false;
 overlay.hidden=true;
 if(input)input.blur();
}
function renderCmdList(){
 const input=document.getElementById('cmdInput');
 const list=document.getElementById('cmdList');
 if(!list)return;
 cmdPaletteItems=cmdPaletteResults(input?input.value:'');
 if(cmdPaletteSel>=cmdPaletteItems.length)cmdPaletteSel=0;
 let html='',lastGroup='';
 cmdPaletteItems.forEach((item,i)=>{
  if(item.group!==lastGroup){html+=`<div class="cmd-group">${item.group}</div>`;lastGroup=item.group}
  const sel=i===cmdPaletteSel;
  html+=`<div class="cmd-item${sel?' selected':''}" role="option" aria-selected="${sel}" data-cmd="${i}"><span>${item.label}</span>${item.hint?`<small>${item.hint}</small>`:''}</div>`;
 });
 list.innerHTML=html||'<div class="cmd-empty">No matches</div>';
 list.querySelectorAll('[data-cmd]').forEach(el=>{
  const n=Number(el.dataset.cmd);
  el.onclick=()=>{cmdPaletteSel=n;cmdRunSelected()};
  el.onmouseenter=()=>{if(cmdPaletteSel!==n){cmdPaletteSel=n;renderCmdList()}};
 });
 const sel=list.querySelector('.cmd-item.selected');
 if(sel&&sel.scrollIntoView)sel.scrollIntoView({block:'nearest'});
}
function cmdRunSelected(){
 const item=cmdPaletteItems[cmdPaletteSel];
 if(!item)return;
 closeCmdPalette();
 try{item.run()}catch(e){console.error('[cmd-palette]',e)}
}
function bindCmdPalette(){
 const overlay=document.getElementById('cmdPalette');
 const input=document.getElementById('cmdInput');
 if(!overlay||!input)return;
 const topSearch=document.getElementById('globalSearch');
 // The topbar search is the palette trigger: focusing/clicking it opens
 // the palette instead of the legacy inline dropdown.
 if(topSearch)topSearch.addEventListener('focus',()=>{if(!cmdPaletteOpen)openCmdPalette()});
 document.addEventListener('keydown',e=>{
  if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='k'){e.preventDefault();openCmdPalette()}
  else if(e.key==='Escape'&&cmdPaletteOpen){closeCmdPalette()}
 });
 input.addEventListener('input',()=>{cmdPaletteSel=0;renderCmdList()});
 input.addEventListener('keydown',e=>{
  if(e.key==='ArrowDown'){e.preventDefault();cmdPaletteSel=Math.min(cmdPaletteSel+1,cmdPaletteItems.length-1);renderCmdList()}
  else if(e.key==='ArrowUp'){e.preventDefault();cmdPaletteSel=Math.max(cmdPaletteSel-1,0);renderCmdList()}
  else if(e.key==='Enter'){e.preventDefault();cmdRunSelected()}
 });
 overlay.addEventListener('click',e=>{if(e.target===overlay)closeCmdPalette()});
}
function appShell(){
 document.title='Lanesra OS Demo';''')

# ---------------------------------------------------------------------------
# 2+3. Wire shell controls in appShell(); restore persisted state first
# ---------------------------------------------------------------------------
add('appShell: wire shell controls',
    r''' renderSidebarNav();
 bindMobileSidebar();
 applyWorkspaceTheme();
''',
    r''' // Modern UI PR-A: restore persisted shell state (theme + sidebar) before
 // the first render, so a returning visitor never flashes the wrong theme.
 // Best-effort: a fresh visitor with an empty store gets Light + expanded.
 applyTheme(readSavedTheme());
 restoreSidebarCollapsed();
 renderSidebarNav();
 bindMobileSidebar();
 applyWorkspaceTheme();
 $('#sidebarCollapse').onclick=toggleSidebarCollapse;
 $('#themeToggle').onclick=toggleTheme;
 bindCmdPalette();
''')

# ---------------------------------------------------------------------------
# 4. Ctrl/Cmd+K opens the palette (replaces focus-inline-search behavior)
# ---------------------------------------------------------------------------
add('palette: Ctrl+K opens palette',
    r'''document.addEventListener('keydown',e=>{if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='k'){e.preventDefault();searchInput.focus();runSearch()}});''',
    r'''document.addEventListener('keydown',e=>{if((e.metaKey||e.ctrlKey)&&e.key.toLowerCase()==='k'){e.preventDefault();openCmdPalette()}});''')
# ---------------------------------------------------------------------------
# 5a. applyWorkspaceTheme(): clear dark overrides when no theme published
# ---------------------------------------------------------------------------
add('applyWorkspaceTheme: clear dark overrides without theme',
    r''' if(!theme){appEl.removeAttribute('style');if(sidebar)sidebar.style.background='';return}''',
    r''' if(!theme){appEl.removeAttribute('style');if(sidebar)sidebar.style.background='';applyWorkspaceThemeDark(null);return}''')

# ---------------------------------------------------------------------------
# 5b. applyWorkspaceTheme(): shadcn-style token aliases + dark pairs
# ---------------------------------------------------------------------------
add('applyWorkspaceTheme: token aliases + dark pairs',
    r''' if(sidebar)sidebar.style.background=c.surface_sidebar;
}''',
    r''' if(sidebar)sidebar.style.background=c.surface_sidebar;
 // Modern UI PR-A: shadcn-style token aliases for the new component
 // classes (.btn, .card, .input, ...), derived from the same published
 // theme color fields. The legacy --text/--brand/... writes above stay
 // untouched.
 appEl.style.setProperty('--foreground',c.text_primary);
 appEl.style.setProperty('--card',c.surface_card);
 appEl.style.setProperty('--primary',c.brand_primary);
 appEl.style.setProperty('--secondary',c.brand_secondary);
 appEl.style.setProperty('--border',c.border_default);
 appEl.style.setProperty('--muted-foreground',c.text_secondary);
 appEl.style.setProperty('--sidebar',c.surface_sidebar);
 // Dark-mode pairs: when the shell is in dark mode the theme's dark
 // values override the same token names on documentElement, cascading
 // over the [data-theme="dark"] block in styles.css.
 applyWorkspaceThemeDark(theme.tokens);
}
// Modern UI PR-A: applies theme.tokens.dark pairs (when present and the
// shell is in dark mode) onto documentElement with the same token names.
// Previous overrides are always cleared first so switching themes never
// leaks stale dark values.
function applyWorkspaceThemeDark(tokens){
 const root=document.documentElement;
 const names=['--foreground','--card','--primary','--secondary','--border','--muted-foreground','--sidebar'];
 names.forEach(k=>root.style.removeProperty(k));
 const dark=tokens&&tokens.dark;
 if(!dark||root.dataset.theme!=='dark')return;
 const c=(tokens&&tokens.color)||{};
 const pairs=[['--foreground',dark.text_primary||c.text_primary],['--card',dark.surface_card||c.surface_card],['--primary',dark.brand_primary||c.brand_primary],['--secondary',dark.brand_secondary||c.brand_secondary],['--border',dark.border_default||c.border_default],['--muted-foreground',dark.text_secondary||c.text_secondary],['--sidebar',dark.surface_sidebar||c.surface_sidebar]];
 pairs.forEach(([k,v])=>{if(v)root.style.setProperty(k,v)});
}''')

# ---------------------------------------------------------------------------
# 6a. TS-1: dark-mode value field for each of the 12 color fields
# ---------------------------------------------------------------------------
add('TS-1: dark color fields',
    r'''<div class="field"><label>${label}</label><div style="display:flex;gap:6px;align-items:center"><input type="color" data-theme-color="${key}" value="${d.tokens.color[key]}" style="width:36px;height:30px;padding:0"><input data-theme-color-text="${key}" value="${d.tokens.color[key]}" style="flex:1"></div></div>''',
    r'''<div class="field"><label>${label}</label><div style="display:flex;gap:6px;align-items:center"><input type="color" data-theme-color="${key}" value="${d.tokens.color[key]}" style="width:36px;height:30px;padding:0" aria-label="${label} (light)"><input data-theme-color-text="${key}" value="${d.tokens.color[key]}" style="flex:1" aria-label="${label} hex (light)"></div><div style="display:flex;gap:6px;align-items:center;margin-top:4px"><span class="muted" style="font-size:11px;width:30px">Dark</span><input type="color" data-theme-color-dark="${key}" value="${(d.tokens.dark||{})[key]||d.tokens.color[key]}" style="width:36px;height:30px;padding:0" aria-label="${label} (dark)"><input data-theme-color-dark-text="${key}" value="${(d.tokens.dark||{})[key]||''}" placeholder="Inherit light" style="flex:1" aria-label="${label} hex (dark)"></div></div>''')

# ---------------------------------------------------------------------------
# 6b. TS-1: radius override, shadow scale, density preview, motion budget
# ---------------------------------------------------------------------------
add('TS-1: radius/shadow/density/motion controls',
    ''' <h4>Shape &amp; density</h4>
 <div class="field"><label>Corner radius</label><select id="themeDraftRadius"><option value="sharp" ${d.tokens.shape.radius_scale==='sharp'?'selected':''}>Sharp</option><option value="soft" ${d.tokens.shape.radius_scale==='soft'?'selected':''}>Soft</option><option value="rounded" ${d.tokens.shape.radius_scale==='rounded'?'selected':''}>Rounded</option></select></div>
 <div class="field"><label>Density</label><select id="themeDraftDensity"><option value="comfortable" ${d.tokens.density==='comfortable'?'selected':''}>Comfortable</option><option value="compact" ${d.tokens.density==='compact'?'selected':''}>Compact</option><option value="dense" ${d.tokens.density==='dense'?'selected':''}>Dense</option></select></div>''',
    ''' <h4>Shape &amp; density</h4>
 <div class="field"><label>Corner radius</label><select id="themeDraftRadius"><option value="sharp" ${d.tokens.shape.radius_scale==='sharp'?'selected':''}>Sharp</option><option value="soft" ${d.tokens.shape.radius_scale==='soft'?'selected':''}>Soft</option><option value="rounded" ${d.tokens.shape.radius_scale==='rounded'?'selected':''}>Rounded</option></select></div>
 <div class="field"><label>Radius override <span class="muted" style="font-size:11px">(px/rem, optional)</span></label><input id="themeDraftRadiusPx" placeholder="e.g. 10px or 0.625rem" value="${d.tokens.shape.radius_px||''}"></div>
 <div class="field"><label>Shadow scale</label><select id="themeDraftShadow"><option value="subtle" ${(d.tokens.shadow||'balanced')==='subtle'?'selected':''}>Subtle</option><option value="balanced" ${(d.tokens.shadow||'balanced')==='balanced'?'selected':''}>Balanced</option><option value="dramatic" ${(d.tokens.shadow||'balanced')==='dramatic'?'selected':''}>Dramatic</option></select><div id="themeShadowPreview" style="margin-top:8px">${themeShadowPreviewHtml(d.tokens)}</div></div>
 <div class="field"><label>Density</label><select id="themeDraftDensity"><option value="comfortable" ${d.tokens.density==='comfortable'?'selected':''}>Comfortable</option><option value="compact" ${d.tokens.density==='compact'?'selected':''}>Compact</option><option value="dense" ${d.tokens.density==='dense'?'selected':''}>Dense</option></select></div>
 <div class="field"><label>Density preview</label><div id="themeDensityPreview">${themeDensityPreviewHtml(d.tokens)}</div></div>
 <h4>Motion</h4>
 <div class="field"><label>Motion budget</label><select id="themeDraftMotion"><option value="full" ${(d.tokens.motion||'full')==='full'?'selected':''}>Full — all transitions &amp; effects</option><option value="reduced" ${(d.tokens.motion||'full')==='reduced'?'selected':''}>Reduced — subtle only</option><option value="none" ${(d.tokens.motion||'full')==='none'?'selected':''}>None — no animation</option></select><p class="muted" style="font-size:12px;margin:4px 0 0">Reduced keeps motion subtle; None disables transitions and effects app-wide.</p></div>''')

# ---------------------------------------------------------------------------
# 6c. TS-1: live preview helpers for shadow + density
# ---------------------------------------------------------------------------
add('TS-1: shadow/density preview helpers',
    r'''function bindThemeDraftPanel(){''',
    r'''// Modern UI PR-A (TS-1): live previews for the shadow scale and
// density pickers in the Theme Studio token editor.
function themeShadowPreviewHtml(tokens){
 const shadow=tokens.shadow||'balanced';
 const css=shadow==='dramatic'?'0 12px 32px rgba(0,0,0,.18)':shadow==='subtle'?'0 1px 2px rgba(0,0,0,.06)':'0 4px 12px rgba(0,0,0,.10)';
 const c=tokens.color||{};
 return `<div style="background:${c.surface_card||'#fff'};border:1px solid ${c.border_default||'#e5e7eb'};border-radius:10px;padding:12px 14px;box-shadow:${css}"><strong style="color:${c.text_primary||'#111'};font-size:13px">Sample card</strong><p class="muted" style="margin:4px 0 0;font-size:12px">Shadow scale: ${shadow}</p></div>`;
}
function themeDensityPreviewHtml(tokens){
 const pad=tokens.density==='dense'?'4px 8px':tokens.density==='compact'?'8px 12px':'12px 16px';
 return `<div style="border:1px solid var(--line);border-radius:8px;overflow:hidden;font-size:13px"><div style="padding:${pad};border-bottom:1px solid var(--line);display:flex;justify-content:space-between"><span>Acme Corp</span><span class="muted">$48,000</span></div><div style="padding:${pad};display:flex;justify-content:space-between"><span>BrightPath Logistics</span><span class="muted">$112,500</span></div></div>`;
}
function bindThemeDraftPanel(){''')

# ---------------------------------------------------------------------------
# 6d. TS-1: initialize the dark bucket for pre-existing drafts
# ---------------------------------------------------------------------------
add('TS-1: dark bucket init',
    r'''function bindThemeDraftPanel(){
 $('#themePreview').innerHTML=themePreviewHtml(themeDraft.tokens);''',
    r'''function bindThemeDraftPanel(){
 $('#themePreview').innerHTML=themePreviewHtml(themeDraft.tokens);
 // Modern UI PR-A (TS-1): drafts predating dark pairs get an empty dark
 // bucket; an absent key means "inherit the light value".
 if(!themeDraft.tokens.dark)themeDraft.tokens.dark={};''')

# ---------------------------------------------------------------------------
# 6e. TS-1: dark-pair input bindings (contrast gating: light pairs only)
# ---------------------------------------------------------------------------
add('TS-1: dark input bindings',
    r''' THEME_COLOR_FIELDS.forEach(([key])=>{
  const colorInput=document.querySelector(`[data-theme-color="${key}"]`);
  const textInput=document.querySelector(`[data-theme-color-text="${key}"]`);
  const sync=val=>{themeDraft.tokens.color[key]=val;colorInput.value=val;textInput.value=val;$('#themePreview').innerHTML=themePreviewHtml(themeDraft.tokens)};
  colorInput.oninput=e=>sync(e.target.value);
  textInput.oninput=e=>sync(e.target.value);
 });''',
    r''' THEME_COLOR_FIELDS.forEach(([key])=>{
  const colorInput=document.querySelector(`[data-theme-color="${key}"]`);
  const textInput=document.querySelector(`[data-theme-color-text="${key}"]`);
  const sync=val=>{themeDraft.tokens.color[key]=val;colorInput.value=val;textInput.value=val;$('#themePreview').innerHTML=themePreviewHtml(themeDraft.tokens)};
  colorInput.oninput=e=>sync(e.target.value);
  textInput.oninput=e=>sync(e.target.value);
  // Modern UI PR-A (TS-1): dark-mode pair for the same token. Clearing
  // the dark text field deletes the override (inherits the light value).
  // Contrast gating keeps checking the light pairs only.
  const darkColorInput=document.querySelector(`[data-theme-color-dark="${key}"]`);
  const darkTextInput=document.querySelector(`[data-theme-color-dark-text="${key}"]`);
  darkColorInput.oninput=e=>{themeDraft.tokens.dark[key]=e.target.value;darkColorInput.value=e.target.value;darkTextInput.value=e.target.value};
  darkTextInput.oninput=e=>{const v=e.target.value.trim();if(!v){delete themeDraft.tokens.dark[key];darkColorInput.value=themeDraft.tokens.color[key]}else{themeDraft.tokens.dark[key]=v;darkColorInput.value=v}};
 });''')

# ---------------------------------------------------------------------------
# 6f. TS-1: bindings for radius override, shadow, density preview, motion
# ---------------------------------------------------------------------------
add('TS-1: radius/shadow/density/motion bindings',
    r''' $('#themeDraftRadius').onchange=e=>themeDraft.tokens.shape.radius_scale=e.target.value;
 $('#themeDraftDensity').onchange=e=>themeDraft.tokens.density=e.target.value;''',
    r''' $('#themeDraftRadius').onchange=e=>themeDraft.tokens.shape.radius_scale=e.target.value;
 $('#themeDraftRadiusPx').oninput=e=>{themeDraft.tokens.shape.radius_px=e.target.value.trim()||null};
 $('#themeDraftShadow').onchange=e=>{themeDraft.tokens.shadow=e.target.value;$('#themeShadowPreview').innerHTML=themeShadowPreviewHtml(themeDraft.tokens)};
 $('#themeDraftDensity').onchange=e=>{themeDraft.tokens.density=e.target.value;$('#themeDensityPreview').innerHTML=themeDensityPreviewHtml(themeDraft.tokens)};
 $('#themeDraftMotion').onchange=e=>{themeDraft.tokens.motion=e.target.value};''')

# ---------------------------------------------------------------------------
# 7a. TS-2: Lanesra Dark + Minimal Light presets
# ---------------------------------------------------------------------------
add('TS-2: new presets',
    '''];
const THEME_COLOR_FIELDS=''',
    ''' ['lanesra-dark','Lanesra Dark','Dark navy + purple brand: the concept dark mode.',{color:{brand_primary:'#635BFF',brand_secondary:'#8B5CF6',surface_app:'#FFFFFF',surface_card:'#FFFFFF',surface_sidebar:'#111827',border_default:'#E4E4E7',text_primary:'#09090B',text_secondary:'#71717A',status_success:'#16A34A',status_warning:'#D97706',status_danger:'#EF4444',status_info:'#3B82F6'},dark:{brand_primary:'#8B5CF6',brand_secondary:'#A78BFA',surface_app:'#0A0F1E',surface_card:'#111A30',surface_sidebar:'#0A0F1E',border_default:'#26314F',text_primary:'#F4F4F5',text_secondary:'#9AA4BD',status_success:'#4ADE80',status_warning:'#FBBF24',status_danger:'#F87171',status_info:'#60A5FA'},typography:{font_family:'-apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif',base_size_px:16},shape:{radius_scale:'rounded'},shadow:'balanced',density:'comfortable',motion:'reduced',chart:{palette:['#8B5CF6','#4ADE80','#FBBF24','#F87171','#60A5FA','#C4B5FD']}}],
 ['minimal-light','Minimal Light','Pure shadcn zinc: monochrome, minimal.',{color:{brand_primary:'#18181B',brand_secondary:'#71717A',surface_app:'#FFFFFF',surface_card:'#FFFFFF',surface_sidebar:'#18181B',border_default:'#E4E4E7',text_primary:'#09090B',text_secondary:'#71717A',status_success:'#16A34A',status_warning:'#D97706',status_danger:'#EF4444',status_info:'#3B82F6'},dark:{brand_primary:'#FAFAFA',brand_secondary:'#A1A1AA',surface_app:'#09090B',surface_card:'#18181B',surface_sidebar:'#09090B',border_default:'#27272A',text_primary:'#FAFAFA',text_secondary:'#A1A1AA',status_success:'#4ADE80',status_warning:'#FBBF24',status_danger:'#F87171',status_info:'#60A5FA'},typography:{font_family:'-apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif',base_size_px:16},shape:{radius_scale:'rounded'},shadow:'subtle',density:'comfortable',motion:'reduced',chart:{palette:['#18181B','#3F3F46','#52525B','#71717A','#A1A1AA','#D4D4D8']}}],
];
const THEME_COLOR_FIELDS=''')

# ---------------------------------------------------------------------------
# 7b. TS-2: presets comment (4 -> 6)
# ---------------------------------------------------------------------------
add('TS-2: presets comment',
    r'''// demo mirror - the 4 curated presets, transcribed verbatim from
// theme_service::built_in_presets() (Rust), as plain data rather than''',
    r'''// demo mirror - the curated presets (Orbit/Slate/Ember/Aurora transcribed
// verbatim from theme_service::built_in_presets() (Rust); Lanesra Dark +
// Minimal Light added by Modern UI PR-A), as plain data rather than''')


def main():
    with open(PATH, encoding='utf-8') as f:
        src = f.read()
    for name, old, new in REPLACEMENTS:
        n = src.count(old)
        if n != 1:
            sys.exit('ANCHOR FAIL [%s]: expected exactly 1 match, found %d' % (name, n))
        src = src.replace(old, new, 1)
        print('ok [%s]' % name)
    with open(PATH, 'w', encoding='utf-8') as f:
        f.write(src)
    print('APPLIED %d replacements -> %s' % (len(REPLACEMENTS), PATH))


if __name__ == '__main__':
    main()
