#!/usr/bin/env python3
"""Fix demo banner overlay and add logo link."""
import os
REPO = "."

def main():
    # 1. Fix app.js: Add logo link and sidebar footer links
    p = os.path.join(REPO, "app.js")
    with open(p, 'rb') as f:
        data = f.read()
    
    # Make the LanesraOS logo/text link to lanesraos.com
    # Old: <div class="product-mark"><img class="brand-mark-img" ...
    # New: <a href="https://lanesraos.com" class="product-mark" style="text-decoration:none;color:inherit"><img ...
    old_logo = b'<div class="product-mark"><img class="brand-mark-img"'
    new_logo = b'<a href="https://lanesraos.com" class="product-mark" style="text-decoration:none;color:inherit"><img class="brand-mark-img"'
    if old_logo in data:
        data = data.replace(old_logo, new_logo, 1)
        # Close the </div> with </a> - find the product-mark closing
        # The structure is: <div class="product-mark">...<span class="demo-pill">DEMO</span></div>
        old_close = b'<span class="demo-pill">DEMO</span></div></div>'
        new_close = b'<span class="demo-pill">DEMO</span></a></div>'
        if old_close in data:
            data = data.replace(old_close, new_close, 1)
            print("1. Added lanesraos.com link to logo")
    
    with open(p, 'wb') as f:
        f.write(data)
    
    # 2. Fix styles.css: Make demo-banner sticky and above sidebar
    p2 = os.path.join(REPO, "styles.css")
    with open(p2, 'rb') as f:
        css = f.read()
    
    # Make demo-banner sticky with high z-index
    old_banner = b'.demo-banner{background:var(--accent);color:var(--accent-foreground);padding:9px 18px;font-size:13px;display:flex;justify-content:center;gap:15px;align-items:center}'
    new_banner = b'.demo-banner{background:var(--accent);color:var(--accent-foreground);padding:9px 18px;font-size:13px;display:flex;justify-content:center;gap:15px;align-items:center;position:sticky;top:0;z-index:70}'
    if old_banner in css:
        css = css.replace(old_banner, new_banner, 1)
        print("2. Made demo-banner sticky with z-index 70")
    
    with open(p2, 'wb') as f:
        f.write(css)
    
    print("Done")

if __name__ == "__main__":
    main()
