#!/usr/bin/env python3
import re
def patch(path):
    src = open(path, encoding='utf-8').read()
    old_tab = "let integrationsSubTab='jobs';"
    assert src.count(old_tab) == 1, "integrationsSubTab not found"
    src = src.replace(old_tab, "let integrationsSubTab='overview';")
    open(path, 'w', encoding='utf-8').write(src)
    print("overview default set")
def patch_css(path):
    src = open(path, encoding='utf-8').read()
    additions = """
.admin-status-strip{display:flex;flex-wrap:wrap;gap:8px 20px;align-items:center;padding:10px 16px;margin-bottom:16px}
.status-seg{display:flex;align-items:center;gap:10px;min-width:0}
.status-seg + .status-seg{border-left:1px solid var(--line);padding-left:20px}
.status-seg-label{font-size:11px;font-weight:700;text-transform:uppercase;letter-spacing:.06em;color:var(--muted)}
.status-seg-value{font-size:13px;font-weight:600}
.status-progress{width:90px;height:6px;border-radius:99px;background:var(--secondary);overflow:hidden}
.status-progress-fill{height:100%;border-radius:99px;background:var(--primary);transition:width .3s ease}
.status-seg-warn .badge{cursor:pointer}
.status-seg-detail{flex-basis:100%;border-top:1px solid var(--line);padding-top:10px;margin-top:4px}
@media(max-width:900px){.status-seg + .status-seg{border-left:0;padding-left:0}}
"""
    clean = re.sub(r'/\*.*?\*/', '', src, flags=re.S)
    assert clean.count('{') == clean.count('}'), "braces unbalanced"
    if '.admin-status-strip{' not in src:
        src = src.rstrip('\n') + '\n' + additions
    open(path, 'w', encoding='utf-8').write(src)
    print("css done")
if __name__ == '__main__':
    patch('app.js')
    patch_css('styles.css')
    print("done")
