#!/usr/bin/env python3
"""
Update website header logo - v4. Handles actual patterns:
- <span class=\"mark brand-mark\">L</span>
- <span class=\"mkt-brand-mark\">L</span>
"""

def patch_js(path):
    src = open(path, encoding='utf-8').read()
    if "__headerLogoV4" in src:
        return []
    
    import re
    changes = 0
    
    # Pattern 1: product header - <span class=\"mark brand-mark\">L</span>
    p1 = r'<span class=\\"mark brand-mark\\">L</span>'
    r1 = '<img class=\\"brand-mark-img\\" src=\\"/media/lanesraos-logo-icon.png\\" alt=\\"LanesraOS\\" style=\\"width:32px;height:32px;border-radius:8px;display:inline-block;vertical-align:middle;\\"/><!--__headerLogoV4-->'
    src, n1 = re.subn(p1, r1, src)
    changes += n1
    
    # Pattern 2: marketing nav - <span class=\"mkt-brand-mark\">L</span>
    p2 = r'<span class=\\"mkt-brand-mark\\">L</span>'
    r2 = '<img class=\\"brand-mark-img\\" src=\\"/media/lanesraos-logo-icon.png\\" alt=\\"LanesraOS\\" style=\\"width:32px;height:32px;border-radius:8px;display:inline-block;vertical-align:middle;\\"/>'
    src, n2 = re.subn(p2, r2, src)
    changes += n2
    
    if changes > 0:
        open(path, 'w', encoding='utf-8').write(src)
        return [f"header logo updated ({changes} occurrences: {n1} product, {n2} marketing)"]
    return ["patterns not found - v4"]

if __name__ == '__main__':
    import sys
    print(patch_js(sys.argv[1] if len(sys.argv) > 1 else 'app.js'))
