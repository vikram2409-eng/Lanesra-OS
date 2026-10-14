#!/usr/bin/env python3
"""
Update website header logo to use the new LanesraOS logo image.
Handles the current main branch pattern.
"""

def patch_js(path):
    src = open(path, encoding='utf-8').read()
    if "__headerLogoV2" in src:
        return []
    
    import re
    
    # Pattern 1: <span class="brand-mark">L</span><span class="product-name">
    # Pattern 2: <span class="brand-mark">L</span>Lanesra OS
    
    changes = 0
    
    # Replace the "L" text inside brand-mark with an img tag
    # Match: <span class="brand-mark">L</span>
    pattern = r'<span class="brand-mark">L</span>'
    replacement = '<img class="brand-mark-img" src="/media/lanesraos-logo-icon.png" alt="LanesraOS" style="width:32px;height:32px;border-radius:8px;display:inline-block;vertical-align:middle;"/><!--__headerLogoV2-->'
    
    new_src, n = re.subn(pattern, replacement, src)
    changes += n
    
    if changes > 0:
        open(path, 'w', encoding='utf-8').write(new_src)
        return [f"header logo updated ({changes} occurrences)"]
    return ["brand mark pattern not found"]

if __name__ == '__main__':
    import sys
    print(patch_js(sys.argv[1] if len(sys.argv) > 1 else 'app.js'))
