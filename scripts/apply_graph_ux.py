#!/usr/bin/env python3
"""Graph UX overhaul + Admin reorg: dropdown hover fix, depth default 1,
Palantir-style graph interactions, Admin landing reorganization.
Applies to app.js and styles.css. Safe to run once per branch.
"""
import re, sys

def patch_admin_reorg(src, changes):
    m = re.search(r'const ADMIN_CATEGORIES=\[(.*?)\n\];', src, re.S)
    assert m, "ADMIN_CATEGORIES not found"
    block = m.group(1)
    cats = re.findall(r"\{[^{}]*?key:'([^']+)'[^{}]*?\}", block)
    assert len(cats) == 15, f"expected 15 categories, got {len(cats)}"
    cat_text = {}
    for key in cats:
        cm = re.search(r"\{key:'" + re.escape(key) + r"',.*?\}", block)
        assert cm, f"category {key} not found"
        cat_text[key] = cm.group(0)
    new_order = ['workspace','access','data-model','ontology','experience','automation',
                 'apps','ai-agent-foundry','ai','voice-first','assistant',
                 'analytics','integrations','solutions','testing']
    assert set(new_order) == set(cats), "category key mismatch"
    new_block = ",\n ".join(cat_text[k] for k in new_order)
    src = src.replace(m.group(0), "const ADMIN_CATEGORIES=[\n " + new_block + "\n];")
    changes.append("reordered ADMIN_CATEGORIES (15)")
    old_health = """ <section class="card prc-admin-panel"><div class="prc-panel-head"><h3>Platform health</h3></div><div class="kpi-grid">${adminPlatformHealthHtml()}</div></section>"""
    assert src.count(old_health) == 1, "platform health section not found"
    src = src.replace(old_health, "")
    changes.append("removed Platform health from bottom position")
    old_heading_end = """<button class="btn btn-ghost btn-sm" data-clear-filter type="button">Back to dashboard</button></div></div>"""
    assert src.count(old_heading_end) == 1, "page heading end not found"
    health_strip = old_heading_end + """
 <section class="card admin-health-strip"><div class="admin-health-items">${adminPlatformHealthHtml()}</div></section>"""
    src = src.replace(old_heading_end, health_strip)
    changes.append("added compact Platform health strip at top")
    old_seq = """${setupDone?'':`<section class="card prc-admin-panel"><div class="prc-panel-head"><h3>Setup progress</h3>"""
    new_seq = """<div class="admin-top-grid">${setupDone?'':`<section class="card prc-admin-panel"><div class="prc-panel-head"><h3>Setup progress</h3>"""
    src = src.replace(old_seq, new_seq)
    old_recent_wrap = """ <div class="admin-landing-grid prc-grid-stack">
 ${adminRecentNavHtml()}
 ${recent.length?`<div class="admin-cat-card"><div class="admin-cat-head"><span class="admin-cat-icon">📝</span><div><h3>Recent changes</h3><p class="muted">Across custom objects, fields, rules, workflows, agents and integrations</p></div></div><div class="admin-cat-items">${recent.map(r=>`<div class="admin-cat-item" style="cursor:default;font-weight:400"><span>${r.text}</span><span class="muted" style="font-size:11px">${new Date(r.at).toLocaleDateString()}</span></div>`).join('')}</div></div>`:''}
 </div>"""
    assert src.count(old_recent_wrap) == 1, "admin landing grid not found"
    src = src.replace(old_recent_wrap, "")
    changes.append("removed old admin-landing-grid")
    recent_card = """${recent.length?`<div class="admin-cat-card"><div class="admin-cat-head"><span class="admin-cat-icon">📝</span><div><h3>Recent changes</h3><p class="muted">Across custom objects, fields, rules, workflows, agents and integrations</p></div></div><div class="admin-cat-items">${recent.map(r=>`<div class="admin-cat-item" style="cursor:default;font-weight:400"><span>${r.text}</span><span class="muted" style="font-size:11px">${new Date(r.at).toLocaleDateString()}</span></div>`).join('')}</div></div>`:''}"""
    old_between = """</div></section>`}
 ${attention.length?`"""
    assert src.count(old_between) == 1, "setup/attention boundary not found"
    new_between = """</div></section>`}
 """ + recent_card + """
 ${attention.length?`"""
    src = src.replace(old_between, new_between)
    changes.append("inserted Recent changes into top grid")
    m_att = re.search(r"</div></section>`:\'\'\}\s*<div class=\"settings-grid\">", src)
    assert m_att, "attention/settings boundary not found"
    src = src[:m_att.start()] + "</div></section>`:\'\'}\n </div>\n <div class=\"settings-grid\">" + src[m_att.end():]
    changes.append("closed admin-top-grid (3 columns)")
    return src

def patch_app_js_full(path):
    src = open(path, encoding='utf-8').read()
    changes = []
    src = patch_admin_reorg(src, changes)
    old_dm = "(parseInt(d,10)||2)"
    assert src.count(old_dm) >= 1, "depMaxDepth fallback not found"
    src = src.replace(old_dm, "(parseInt(d,10)||1)")
    changes.append("depMaxDepth fallback 2->1")
    open(path, 'w', encoding='utf-8').write(src)
    print(f"admin reorg: {len(changes)} change groups")
    for c in changes: print("  -", c)

def patch_app_js(path):
    src = open(path, encoding='utf-8').read()
    changes = []
    depth_patches = [
        ("let dependencyExplorerState={type:'custom_object',id:null,depth:'2',",
         "let dependencyExplorerState={type:'custom_object',id:null,depth:'1',"),
        ("dependencyExplorerState={type:'custom_object',id:entityTypeKey,depth:'2',",
         "dependencyExplorerState={type:'custom_object',id:entityTypeKey,depth:'1',"),
        ("String(s.depth||'2')===d?' selected':''",
         "String(s.depth||'1')===d?' selected':''"),
        ("['depth',s.depth||'2'],", "['depth',s.depth||'1'],"),
        ("dependencyExplorerState={type,id,depth:'2',",
         "dependencyExplorerState={type,id,depth:'1',"),
        ("depth:dependencyExplorerState.depth||'2',",
         "depth:dependencyExplorerState.depth||'1',"),
        ("String(dependencyExplorerState.depth||'2')",
         "String(dependencyExplorerState.depth||'1')"),
        ("maxDepth:(depth>=1&&depth<=3)?depth:2,",
         "maxDepth:(depth>=1&&depth<=3)?depth:1,"),
    ]
    for old, new in depth_patches:
        c = src.count(old)
        if c:
            src = src.replace(old, new)
            changes.append(f"depth: replaced {c}x")
    old_dot = '\'\" \'+ui.inspectAttr+\'=\"\'+n.type+\':\'+n.id+\'\" style=\"left:\''
    new_dot = '\'\" \'+ui.inspectAttr+\'=\"\'+n.type+\':\'+n.id+\'\" data-node-key=\"\'+n.key+\'\" style=\"left:\''
    assert src.count(old_dot) >= 1
    src = src.replace(old_dot, new_dot)
    changes.append("node dot data-node-key")
    target_old = '\'<div class="ontology-node-caption\'+(isDimmed?\' ontology-node-dimmed\':\'\')+\'\" style=\"left:\''
    target_new = '\'<div class="ontology-node-caption\'+(isDimmed?\' ontology-node-dimmed\':\'\')+\'\" data-node-key=\"\'+n.key+\'\" style=\"left:\''
    assert src.count(target_old) >= 1
    src = src.replace(target_old, target_new)
    changes.append("caption data-node-key")
    old_line = 'edgeSvg.push(\'<line x1="\''
    new_line = 'edgeSvg.push(\'<line data-edge-from="\'+e.fromKey+\'\" data-edge-to="\'+e.toKey+\'\" x1="\''
    assert src.count(old_line) >= 1
    src = src.replace(old_line, new_line)
    changes.append("edge line data attrs")
    old_el = '(eDimmed?\' ontology-node-dimmed\':\'\')+\'\" style=\"left:\'+((a.x+b.x)/2)'
    new_el = '(eDimmed?\' ontology-node-dimmed\':\'\')+\'\" data-edge-from=\"\'+e.fromKey+\'\" data-edge-to=\"\'+e.toKey+\'\" style=\"left:\'+((a.x+b.x)/2)'
    assert src.count(old_el) >= 1
    src = src.replace(old_el, new_el)
    changes.append("edge label data attrs")
    marker = "// Thin wrapper: the Ontology Graph's ui over the shared renderer."
    assert marker in src
    hover_fn = """
// Graph UX: hover spotlight + double-click to expand.
function wireGraphHoverFx(host, onExpand){
 const canvas=host?host.querySelector('.ontology-graph-canvas'):null;
 if(!canvas||canvas.dataset.hoverWired)return;canvas.dataset.hoverWired='1';
 const clear=()=>{canvas.classList.remove('graph-hover-dim');canvas.querySelectorAll('.graph-hover-on').forEach(el=>el.classList.remove('graph-hover-on'))};
 canvas.addEventListener('mouseover',e=>{
  const dot=e.target.closest('.ontology-node-dot');if(!dot||!dot.dataset.nodeKey)return;
  const key=dot.dataset.nodeKey;
  const connected=new Set([key]);
  canvas.querySelectorAll('line[data-edge-from]').forEach(l=>{
   if(l.dataset.edgeFrom===key)connected.add(l.dataset.edgeTo);
   else if(l.dataset.edgeTo===key)connected.add(l.dataset.edgeFrom);
  });
  canvas.classList.add('graph-hover-dim');
  canvas.querySelectorAll('.ontology-node-dot,.ontology-node-caption').forEach(el=>{
   el.classList.toggle('graph-hover-on',connected.has(el.dataset.nodeKey));
  });
  canvas.querySelectorAll('line[data-edge-from],.ontology-edge-label[data-edge-from]').forEach(el=>{
   el.classList.toggle('graph-hover-on',el.dataset.edgeFrom===key||el.dataset.edgeTo===key);
  });
 });
 canvas.addEventListener('mouseleave',clear);
 canvas.addEventListener('mouseout',e=>{if(!e.relatedTarget||!canvas.contains(e.relatedTarget))clear()});
 if(onExpand){
  canvas.addEventListener('dblclick',e=>{
   const dot=e.target.closest('.ontology-node-dot');if(!dot||!dot.dataset.nodeKey)return;
   e.preventDefault();e.stopPropagation();onExpand(dot.dataset.nodeKey);
  });
 }
}
"""
    src = src.replace(marker, hover_fn + marker)
    changes.append("wireGraphHoverFx")
    old_o360 = "object360State={...object360State,collapsed:c};\n  renderObject360Body();\n });"
    assert src.count(old_o360) >= 1
    src = src.replace(old_o360, old_o360 + "\n wireGraphHoverFx(body,(k)=>{const ex={...(object360State.expanded||{})};ex[k]=true;object360State={...object360State,expanded:ex};renderObject360Body()});", 1)
    changes.append("o360 hover/dblclick")
    old_dep = "dependencyExplorerState={...dependencyExplorerState,inspectType:type,inspectId:id};renderAdminTab()});"
    assert src.count(old_dep) >= 1
    src = src.replace(old_dep, old_dep + "\n wireGraphHoverFx(body,(k)=>{const ex={...(dependencyExplorerState.expanded||{})};ex[k]=true;dependencyExplorerState={...dependencyExplorerState,expanded:ex};renderAdminTab()});", 1)
    changes.append("dep hover/dblclick")
    open(path, 'w', encoding='utf-8').write(src)
    print(f"app.js graph: {len(changes)} groups")

def patch_styles_css(path):
    src = open(path, encoding='utf-8').read()
    additions = """
/* ===== Graph UX: dropdown hover bridge ===== */
@media(min-width:901px){.nav-dropdown::before{content:'';position:absolute;top:-8px;left:0;right:0;height:8px}}
/* ===== Graph UX: hover spotlight ===== */
.ontology-node-dot{transition:transform .18s ease,box-shadow .18s ease,opacity .2s ease;cursor:pointer}
.ontology-node-dot:hover{transform:scale(1.32);z-index:6}
.ontology-node-caption,.ontology-edge-label{transition:opacity .2s ease}
.ontology-graph-canvas.graph-hover-dim .ontology-node-dot:not(.graph-hover-on){opacity:.14}
.ontology-graph-canvas.graph-hover-dim .ontology-node-caption:not(.graph-hover-on){opacity:.1}
.ontology-graph-canvas.graph-hover-dim .ontology-edge-label:not(.graph-hover-on){opacity:.05}
.ontology-graph-canvas.graph-hover-dim svg line:not(.graph-hover-on){stroke-opacity:.06!important}
.ontology-node-dot.graph-hover-on{box-shadow:0 0 0 5px color-mix(in srgb,currentColor 22%,transparent),0 4px 12px rgba(0,0,0,.28)}
.ontology-node-caption.graph-hover-on{font-weight:700}
/* ===== Admin reorg: health strip + 3-col grid ===== */
.admin-health-strip{padding:12px 18px;margin-bottom:16px}
.admin-health-items{display:flex;gap:10px;flex-wrap:wrap}
.admin-health-items .kpi{flex:1 1 0;min-width:130px;padding:10px 14px;display:flex;align-items:center;gap:12px;background:var(--card);border:1px solid var(--line);border-radius:12px}
.admin-health-items .kpi-value{font-size:22px;font-weight:800;line-height:1}
.admin-health-items .kpi-label{font-size:12px;color:var(--muted);font-weight:600}
.admin-top-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px;margin-bottom:20px;align-items:start}
@media(max-width:1100px){.admin-top-grid{grid-template-columns:minmax(0,1fr)}}
.admin-top-grid .card{margin:0}
"""
    clean = re.sub(r'/\*.*?\*/', '', src, flags=re.S)
    assert clean.count('{') == clean.count('}'), "braces unbalanced"
    if '.nav-dropdown::before' not in src:
        src = src.rstrip('\n') + '\n' + additions
    open(path, 'w', encoding='utf-8').write(src)
    print("styles.css updated")

if __name__ == '__main__':
    patch_app_js('app.js')
    patch_app_js_full('app.js')
    patch_styles_css('styles.css')
    print("done")
