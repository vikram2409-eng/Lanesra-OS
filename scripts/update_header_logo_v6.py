#!/usr/bin/env python3
"""
Update website header logo - v6. Uses regular quotes (not escaped).
Actual pattern: <span class="mark brand-mark">L</span>
"""

def patch_js(path):
    src = open(path, encoding='utf-8').read()
    if "__headerLogoV6" in src:
        return []
    
    changes = 0
    
    # Pattern 1: product header
    p1_old = '<span class="mark brand-mark">L</span>'
    p1_new = '<img class="brand-mark-img" src="/media/lanesraos-logo-icon.png" alt="LanesraOS" style="width:32px;height:32px;border-radius:8px;display:inline-block;vertical-align:middle;"/><!--__headerLogoV6-->'
    
    # Pattern 2: marketing nav
    p2_old = '<span class="mkt-brand-mark">L</span>'
    p2_new = '<img class="brand-mark-img" src="/media/lanesraos-logo-icon.png" alt="LanesraOS" style="width:32px;height:32px;border-radius:8px;display:inline-block;vertical-align:middle;"/>'
    
    n1 = src.count(p1_old)
    if n1 > 0:
        src = src.replace(p1_old, p1_new)
        changes += n1
    
    n2 = src.count(p2_old)
    if n2 > 0:
        src = src.replace(p2_old, p2_new)
        changes += n2
    
    if changes > 0:
        open(path, 'w', encoding='utf-8').write(src)
        return [f"header logo updated ({changes}: {n1} product, {n2} marketing)"]
    return ["patterns not found - v6"]

if __name__ == '__main__':
    import sys
    print(patch_js(sys.argv[1] if len(sys.argv) > 1 else 'app.js'))
