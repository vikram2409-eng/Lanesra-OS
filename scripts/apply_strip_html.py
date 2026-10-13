#!/usr/bin/env python3
"""Compact Admin status strip HTML replacement."""
OLD = """ <div class="admin-top-grid">${setupDone?'':`<section class="card prc-admin-panel"><div class="prc-panel-head"><h3>Setup progress</h3><span class="badge badge-secondary">${setup.filter(s=>s.done).length} of ${setup.length} done</span></div><div class="admin-cat-items">${setup.map(s=>`<div class="admin-cat-item" style="cursor:default"><span class="prc-row-label"><span class="prc-status${s.done?' prc-status-done':''}">${s.done?'x':'o'}</span>${s.label}</span>${s.done?'<span class="badge badge-success">Done</span>':'<span class="badge badge-outline">To do</span>'}</div>`).join('')}</div></section>`}
 ${recent.length?`<div class="admin-cat-card"><div class="admin-cat-head"><span class="admin-cat-icon">x</span><div><h3>Recent changes</h3><p class="muted">Across custom objects, fields, rules, workflows, agents and integrations</p></div></div><div class="admin-cat-items">${recent.map(r=>`<div class="admin-cat-item" style="cursor:default;font-weight:400"><span>${r.text}</span><span class="muted" style="font-size:11px">${new Date(r.at).toLocaleDateString()}</span></div>`).join('')}</div></div>`:''}
 ${attention.length?`<section class="card prc-admin-panel"><div class="prc-panel-head"><h3>Needs attention</h3><span class="badge badge-warning">${attention.length}</span></div><div class="admin-cat-items">${attention.map(a=>`<button type="button" class="admin-cat-item" data-admin-open="${a.tab}"><span>${a.label}</span><span class="prc-row-meta"><span class="badge badge-warning">${a.count}</span><span class="admin-cat-arrow">x</span></span></button>`).join('')}</div></section>`:''}
 </div>"""
NEW = """ <section class="card admin-status-strip" aria-label="Workspace status">
  ${setupDone?'':`<div class="status-seg"><span class="status-seg-label">Setup</span><div class="status-progress" role="progressbar"><div class="status-progress-fill" style="width:${Math.round(setup.filter(s=>s.done).length/setup.length*100)}%"></div></div><span class="status-seg-value">${setup.filter(s=>s.done).length}/${setup.length}</span><button type="button" class="btn btn-ghost btn-sm" data-status-expand="setup">Details</button></div><div class="status-seg-detail" data-status-detail="setup" hidden><div class="admin-cat-items">${setup.map(s=>`<div class="admin-cat-item" style="cursor:default"><span>${s.label}</span>${s.done?'<span class="badge badge-success">Done</span>':'<span class="badge badge-outline">To do</span>'}</div>`).join('')}</div></div>`}
  ${recent.length?`<div class="status-seg"><span class="status-seg-label">Recent</span><span class="status-seg-value">${recent.length} changes</span><button type="button" class="btn btn-ghost btn-sm" data-status-expand="recent">View all</button></div><div class="status-seg-detail" data-status-detail="recent" hidden><div class="admin-cat-items">${recent.map(r=>`<div class="admin-cat-item" style="cursor:default;font-weight:400"><span>${r.text}</span><span class="muted" style="font-size:11px">${new Date(r.at).toLocaleDateString()}</span></div>`).join('')}</div></div>`:''}
  ${attention.length?`<div class="status-seg status-seg-warn"><span class="status-seg-label">Attention</span>${attention.map(a=>`<button type="button" class="badge badge-warning" data-admin-open="${a.tab}" title="${a.label}">${a.count}</button>`).join('')}</div>`:''}
 </section>"""
def patch(path):
    src = open(path, encoding='utf-8').read()
    # Match loosely: find admin-top-grid section
    import re
    m = re.search(r' <div class="admin-top-grid">.*?\n </div>', src, re.S)
    assert m, "admin-top-grid not found"
    src = src[:m.start()] + NEW + src[m.end():]
    old_wire = "document.querySelectorAll('[data-admin-open]').forEach"
    assert old_wire in src, "wiring anchor not found"
    # Add expand wiring after the admin-open wiring line
    src = src.replace(old_wire, "document.querySelectorAll('[data-status-expand]').forEach(b=>b.onclick=()=>{var d=document.querySelector('[data-status-detail=\"'+b.dataset.statusExpand+'\"]');if(d){d.hidden=!d.hidden;b.textContent=d.hidden?'View':'Hide';}});\n document.querySelectorAll('[data-admin-open]').forEach", 1)
    open(path, 'w', encoding='utf-8').write(src)
    print("strip applied")
if __name__ == '__main__':
    patch('app.js')
    print("done")
