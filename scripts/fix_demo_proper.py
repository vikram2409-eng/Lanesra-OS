#!/usr/bin/env python3
"""Properly fix demo banner: revert sticky, add links to sidebar."""
import os
REPO = "."

def main():
    # 1. Revert the banner CSS to non-sticky (remove my broken fix)
    p2 = os.path.join(REPO, "styles.css")
    with open(p2, 'rb') as f:
        css = f.read()
    
    # Remove the sticky/z-index I added
    old_banner = b'.demo-banner{background:var(--accent);color:var(--accent-foreground);padding:9px 18px;font-size:13px;display:flex;justify-content:center;gap:15px;align-items:center;position:sticky;top:0;z-index:70}'
    new_banner = b'.demo-banner{background:var(--accent);color:var(--accent-foreground);padding:9px 18px;font-size:13px;display:flex;justify-content:center;gap:15px;align-items:center}'
    if old_banner in css:
        css = css.replace(old_banner, new_banner, 1)
        print("1. Reverted banner to non-sticky")
    else:
        print("1. Banner already non-sticky (or not found)")
    
    # Add sidebar footer style for demo links
    # Add after the .demo-pill style
    old_pill = b'.demo-pill{font-size:11px;background:color-mix(in srgb,var(--primary) 35%,transparent);color:var(--sidebar-foreground);padding:4px 7px;border-radius:999px}'
    new_pill = old_pill + b'.side-demo-links{margin-top:16px;padding-top:16px;border-top:1px solid color-mix(in srgb,var(--sidebar-foreground) 15%,transparent);display:flex;flex-direction:column;gap:4px}.side-demo-links a,.side-demo-links button{display:flex;align-items:center;gap:11px;width:100%;padding:11px 12px;border:0;border-radius:10px;background:transparent;color:var(--sidebar-foreground);text-align:left;cursor:pointer;font-size:14px;text-decoration:none}.side-demo-links a:hover,.side-demo-links button:hover{background:color-mix(in srgb,var(--sidebar-foreground) 10%,transparent)}'
    if old_pill in css and b'.side-demo-links' not in css:
        css = css.replace(old_pill, new_pill, 1)
        print("2. Added sidebar demo links styles")
    
    with open(p2, 'wb') as f:
        f.write(css)
    
    # 2. Add the links to the sidebar in app.js
    p = os.path.join(REPO, "app.js")
    with open(p, 'rb') as f:
        data = f.read()
    
    # Find the sidebar structure and add demo links before the collapse button
    # The sidebar has: <div id="sideNav"></div><button type="button" class="btn btn-ghost collapse-side" ...
    # We'll add the demo links div after sideNav
    old_sidenav = b'<div id="sideNav"></div><button type="button" class="btn btn-ghost collapse-side"'
    new_sidenav = b'<div id="sideNav"></div><div class="side-demo-links"><button type="button" id="sideResetDemo">\xe2\x86\xbb Reset demo</button><a href="/">\xf0\x9f\x8c\x90 Product website</a></div><button type="button" class="btn btn-ghost collapse-side"'
    
    if old_sidenav in data:
        data = data.replace(old_sidenav, new_sidenav, 1)
        print("3. Added demo links to sidebar")
    else:
        print("3. WARNING: sideNav pattern not found")
    
    # 3b. Wire up the sidebar reset button (same handler as the banner button)
    # Find: $('#resetDemo').onclick=()=>{...
    # Add after it: wire up sideResetDemo too
    old_handler = b"$('#resetDemo').onclick=()=>{data=structuredClone(seed);"
    if old_handler in data:
        # Find the full onclick line and add the sidebar button handler after it
        # The line ends with renderView()};
        idx = data.find(old_handler)
        # Find the end of this statement (the };)
        end_idx = data.find(b'renderView()};', idx)
        if end_idx > 0:
            end_pos = end_idx + len(b'renderView()};')
            add_handler = b"const srd=document.getElementById('sideResetDemo');if(srd)srd.onclick=document.getElementById('resetDemo').onclick;"
            data = data[:end_pos] + add_handler + data[end_pos:]
            print("4. Wired up sidebar reset button")
    
    with open(p, 'wb') as f:
        f.write(data)
    
    print("Done")

if __name__ == "__main__":
    main()
