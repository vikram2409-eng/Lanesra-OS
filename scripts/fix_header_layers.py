#!/usr/bin/env python3
"""Fix header layers: banner fixed at top, topbar sticky below it, content clears both."""
import os
REPO = "."

def main():
    p = os.path.join(REPO, "styles.css")
    with open(p, 'rb') as f:
        css = f.read()
    
    # 1. Make demo-banner fixed at top
    old_banner = b'.demo-banner{background:var(--accent);color:var(--accent-foreground);padding:9px 18px;font-size:13px;display:flex;justify-content:center;gap:15px;align-items:center}'
    new_banner = b'.demo-banner{background:var(--accent);color:var(--accent-foreground);padding:8px 18px;font-size:13px;display:flex;justify-content:center;gap:15px;align-items:center;position:fixed;top:0;left:0;right:0;z-index:100;height:36px;white-space:nowrap;overflow:hidden}'
    if old_banner in css:
        css = css.replace(old_banner, new_banner, 1)
        print("1. Banner: fixed at top, z-index 100")
    else:
        print("1. WARNING: banner pattern not found")
    
    # 2. Make topbar sticky below banner (top:36px instead of top:0)
    # There are multiple .topbar definitions; update the main ones
    # Main desktop: .topbar{height:68px;...position:sticky;top:0;z-index:10}
    old_tb1 = b'.topbar{height:68px;background:var(--card);border-bottom:1px solid var(--line);display:flex;align-items:center;justify-content:space-between;padding:0 26px;position:sticky;top:0;z-index:10}'
    new_tb1 = b'.topbar{height:68px;background:var(--card);border-bottom:1px solid var(--line);display:flex;align-items:center;justify-content:space-between;padding:0 26px;position:sticky;top:36px;z-index:90}'
    if old_tb1 in css:
        css = css.replace(old_tb1, new_tb1, 1)
        print("2. Topbar (desktop): sticky at top:36px")
    
    # Second definition: .topbar{height:64px;...position:sticky;top:0;z-index:10}
    old_tb2 = b'.topbar{height:64px;border-bottom:1px solid var(--border);display:flex;align-items:center;justify-content:space-between;gap:20px;padding:0 26px;background:var(--background);position:sticky;top:0;z-index:10}'
    new_tb2 = b'.topbar{height:64px;border-bottom:1px solid var(--border);display:flex;align-items:center;justify-content:space-between;gap:20px;padding:0 26px;background:var(--background);position:sticky;top:36px;z-index:90}'
    if old_tb2 in css:
        css = css.replace(old_tb2, new_tb2, 1)
        print("3. Topbar (alt): sticky at top:36px")
    
    # 3. Adjust sidebar to account for banner
    old_sb = b'.sidebar{background:var(--sidebar);color:var(--sidebar-foreground);padding:18px 14px;position:sticky;top:0;height:100vh}'
    new_sb = b'.sidebar{background:var(--sidebar);color:var(--sidebar-foreground);padding:18px 14px;position:sticky;top:36px;height:calc(100vh - 36px)}'
    if old_sb in css:
        css = css.replace(old_sb, new_sb, 1)
        print("4. Sidebar: sticky at top:36px, height adjusted")
    
    # 4. Add body padding-top to prevent content jump (banner is fixed, doesn't take space)
    # Add to the end
    fix = b'\n/* Fixed header: banner(36px) + topbar offset */\nbody.has-demo-banner{padding-top:36px}\n@media(max-width:900px){.sidebar.open{top:0;height:100vh}}\n'
    if b'has-demo-banner' not in css:
        css = css.rstrip() + fix
        print("5. Added body padding for fixed banner")
    
    with open(p, 'wb') as f:
        f.write(css)
    
    # 5. Add the body class in app.js when demo banner is shown
    p2 = os.path.join(REPO, "app.js")
    with open(p2, 'rb') as f:
        data = f.read()
    
    # Find where appShell sets innerHTML and add body class
    # After: $('#app').innerHTML=`<div class="demo-banner">...
    # Add: document.body.classList.add('has-demo-banner');
    old_shell = b"$('#app').innerHTML=`<div class=\"demo-banner\">"
    if old_shell in data:
        # Find the end of the innerHTML assignment (the `;)
        idx = data.find(old_shell)
        # Find the closing `; after the template literal
        # The template ends with `</div>`; - let's find it
        end_marker = b'</div>`;'
        end_idx = data.find(end_marker, idx)
        if end_idx > 0:
            insert_pos = end_idx + len(end_marker)
            add_class = b"document.body.classList.add('has-demo-banner');"
            data = data[:insert_pos] + add_class + data[insert_pos:]
            print("6. Added body class for demo banner")
    
    with open(p2, 'wb') as f:
        f.write(data)
    
    print("Done")

if __name__ == "__main__":
    main()
