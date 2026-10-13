#!/usr/bin/env python3
"""Modernize list-page filter bar: apply design-system .select to view bar
dropdowns and tighten the view-bar layout. Safe to run once per branch.
"""
import re

def patch(path):
    src = open(path, encoding='utf-8').read()
    changes = []
    old = '<select id="viewSelect" style="min-width:160px">'
    new = '<select id="viewSelect" class="select view-bar-select" style="min-width:170px;width:auto">'
    assert src.count(old) == 1, "viewSelect not found"
    src = src.replace(old, new)
    changes.append("viewSelect")
    old = '<select id="viewStatusFilter">'
    new = '<select id="viewStatusFilter" class="select view-bar-select" style="width:auto">'
    assert src.count(old) == 1, "viewStatusFilter not found"
    src = src.replace(old, new)
    changes.append("viewStatusFilter")
    old = '<select id="viewSort">'
    new = '<select id="viewSort" class="select view-bar-select" style="width:auto">'
    assert src.count(old) == 1, "viewSort not found"
    src = src.replace(old, new)
    changes.append("viewSort")
    old = '<select id="viewGroup">'
    new = '<select id="viewGroup" class="select view-bar-select" style="width:auto">'
    assert src.count(old) == 1, "viewGroup not found"
    src = src.replace(old, new)
    changes.append("viewGroup")
    old_bar = '<div class="view-bar" style="display:flex;flex-wrap:wrap;gap:10px;align-items:center;margin:14px 0">'
    new_bar = '<div class="view-bar">'
    assert src.count(old_bar) == 1, "view-bar div not found"
    src = src.replace(old_bar, new_bar)
    changes.append("view-bar container")
    old_label = '<label style="display:flex;align-items:center;gap:6px;font-size:13px">'
    new_label = '<label class="view-bar-label">'
    c = src.count(old_label)
    assert c >= 1, "labels not found"
    src = src.replace(old_label, new_label)
    changes.append(f"labels {c}x")
    open(path, 'w', encoding='utf-8').write(src)
    print("app.js filter done:", changes)

def patch_css(path):
    src = open(path, encoding='utf-8').read()
    additions = """
/* ===== Filter bar modernization ===== */
.view-bar{display:flex;flex-wrap:wrap;gap:8px;align-items:center;margin:14px 0;padding:10px 12px;background:var(--card);border:1px solid var(--line);border-radius:var(--radius-lg)}
.view-bar-label{display:inline-flex;align-items:center;gap:8px;font-size:13px;font-weight:600;color:var(--muted)}
.view-bar-select{width:auto;min-height:36px;padding:7px 30px 7px 12px;font-size:13px;border-color:transparent;background-color:var(--secondary)}
.view-bar-select:hover{border-color:var(--line)}
.view-bar-select:focus{border-color:var(--ring)}
"""
    clean = re.sub(r'/\*.*?\*/', '', src, flags=re.S)
    assert clean.count('{') == clean.count('}'), "braces unbalanced"
    if '.view-bar{display:flex' not in src:
        src = src.rstrip('\n') + '\n' + additions
    open(path, 'w', encoding='utf-8').write(src)
    print("styles.css filter done")

if __name__ == '__main__':
    patch('app.js')
    patch_css('styles.css')
    print("done")
