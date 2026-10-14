#!/usr/bin/env python3
"""
Update website header logo - v3. Handles escaped quotes in JS strings.
"""

def patch_js(path):
    src = open(path, encoding='utf-8').read()
    if "__headerLogoV3" in src:
        return []
    
    import re
    changes = 0
    
    # The HTML is inside JS strings, so quotes are escaped as \"
    # Pattern: <span class=\"brand-mark\">L</span>
    pattern = r'<span class=\\"brand-mark\\">L</span>'
    replacement = '<img class=\\"brand-mark-img\\" src=\\"/media/lanesraos-logo-icon.png\\" alt=\\"LanesraOS\\" style=\\"width:32px;height:32px;border-radius:8px;display:inline-block;vertical-align:middle;\\"/><!--__headerLogoV3-->'
    
    new_src, n = re.subn(pattern, replacement, src)
    changes += n
    
    if changes > 0:
        open(path, 'w', encoding='utf-8').write(new_src)
        return [f"header logo updated ({changes} occurrences)"]
    return ["brand mark pattern not found - v3"]

if __name__ == '__main__':
    import sys
    print(patch_js(sys.argv[1] if len(sys.argv) > 1 else 'app.js'))
