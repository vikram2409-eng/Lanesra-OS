#!/usr/bin/env python3
"""Bump desktop app version 0.14.0 -> 0.15.0."""

import json

def bump():
    # tauri.conf.json
    p1 = "desktop/src-tauri/tauri.conf.json"
    d1 = json.load(open(p1))
    old = d1["version"]
    d1["version"] = "0.15.0"
    json.dump(d1, open(p1, "w"), indent=2)
    print(f"tauri.conf.json: {old} -> 0.15.0")
    
    # Cargo.toml
    p2 = "desktop/src-tauri/Cargo.toml"
    src = open(p2).read()
    src = src.replace('version = "0.14.0"', 'version = "0.15.0"', 1)
    open(p2, "w").write(src)
    print("Cargo.toml updated")
    
    # package.json
    p3 = "desktop/package.json"
    d3 = json.load(open(p3))
    d3["version"] = "0.15.0"
    json.dump(d3, open(p3, "w"), indent=2)
    print("package.json updated")

if __name__ == "__main__":
    bump()
