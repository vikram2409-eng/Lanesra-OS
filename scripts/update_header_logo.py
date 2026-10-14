#!/usr/bin/env python3
"""
Update website header logo to use the new LanesraOS logo image.
"""

def patch_js(path):
    src = open(path, encoding='utf-8').read()
    if "__headerLogoUpdated" in src:
        return []
    
    # Replace the text "L" brand mark with the logo image
    old = '<span class="brand-mark">L</span>Lanesra OS'
    new = '<img class="brand-mark-img" src="/media/lanesraos-logo-icon.png" alt="LanesraOS" style="width:32px;height:32px;border-radius:8px;"/>Lanesra OS<!--__headerLogoUpdated-->'
    
    if old in src:
        src = src.replace(old, new)
        open(path, 'w', encoding='utf-8').write(src)
        return ["header logo updated to use image"]
    return ["brand mark not found"]

if __name__ == '__main__':
    import sys
    print(patch_js(sys.argv[1] if len(sys.argv) > 1 else 'app.js'))
