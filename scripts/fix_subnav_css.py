#!/usr/bin/env python3
"""Fix subnav CSS to style anchor tags like buttons."""
import os
REPO = "."

def main():
    p = os.path.join(REPO, "site-modern.css")
    src = open(p, encoding="utf-8").read()
    
    # Fix 1: Main subnav button styles -> include anchors
    old1 = '.modern-site .subnav button{flex:1;border:0;border-radius:8px;background:transparent;color:var(--muted);padding:10px 12px;font-size:12px;font-weight:800;cursor:pointer}'
    new1 = '.modern-site .subnav button,.modern-site .subnav a{flex:1;border:0;border-radius:8px;background:transparent;color:var(--muted);padding:10px 12px;font-size:12px;font-weight:800;cursor:pointer;text-decoration:none;text-align:center;display:block}'
    if old1 in src:
        src = src.replace(old1, new1, 1)
        print("1. Fixed main subnav styles for anchors")
    
    # Fix 2: Active state
    old2 = '.modern-site .subnav button.on{color:var(--text);background:var(--surface);box-shadow:0 1px 3px rgba(20,22,29,.1)}'
    new2 = '.modern-site .subnav button.on,.modern-site .subnav a.on{color:var(--text);background:var(--surface);box-shadow:0 1px 3px rgba(20,22,29,.1)}'
    if old2 in src:
        src = src.replace(old2, new2, 1)
        print("2. Fixed active state for anchors")
    
    # Fix 3: Mobile styles (940px)
    old3 = '.modern-site .subnav button{white-space:normal;padding:9px 6px}'
    new3 = '.modern-site .subnav button,.modern-site .subnav a{white-space:normal;padding:9px 6px}'
    if old3 in src:
        src = src.replace(old3, new3, 1)
        print("3. Fixed mobile subnav styles")
    
    # Fix 4: Mobile grid last-child (620px)
    old4 = '.modern-site .subnav button:last-child:nth-child(odd){grid-column:1/-1}'
    new4 = '.modern-site .subnav button:last-child:nth-child(odd),.modern-site .subnav a:last-child:nth-child(odd){grid-column:1/-1}'
    if old4 in src:
        src = src.replace(old4, new4, 1)
        print("4. Fixed mobile grid for anchors")
    
    open(p, "w", encoding="utf-8").write(src)
    print("Done")

if __name__ == "__main__":
    main()
