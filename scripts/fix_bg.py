#!/usr/bin/env python3
"""Fix releasesPage background: remove nested publicNav, fix wrapper."""
import re, os
REPO = "."
p = os.path.join(REPO, "app.js")
src = open(p, encoding='utf-8').read()
n = 0


# 1. Fix releasesPage wrapper - remove nested publicNav
# Find releasesPage function
idx = src.find('function releasesPage()')
# Look for the nested publicNav pattern
m = re.search(r"\$\('#app'\)\.innerHTML=`<div class=\"modern-site\" id=\"modernSite\">(\$\{window\.MODERN_HEADER\|\|''\})<main><div class=\"site-dark\">(\$\{publicNav\(\)\})<main class=\"mkt-page\">", src[idx:idx+200000])
if m:
    pos = idx + m.start()
    old = m.group(0)
    new = "$('#app').innerHTML=`<div class=\"modern-site\" id=\"modernSite\">${window.MODERN_HEADER||''}<main><div class=\"site-dark\"><div class=\"mkt-page\">"
    src = src[:pos] + new + src[pos+len(old):]
    n += 1
    print("1. Fixed releasesPage nested nav")


# 2. Fix roadmapPage binding (if still old)
if 'bindPublicNav();' in src:
    # Find the one in roadmapPage (first occurrence after roadmapPage)
    idx_rm = src.find('function roadmapPage()')
    pos = src.find('bindPublicNav();', idx_rm)
    if pos > 0 and pos < idx_rm + 200000:
        src = src[:pos] + "bindModernChrome(document.getElementById('modernSite'));" + src[pos+len('bindPublicNav();'):]
        n += 1
        print("2. Fixed roadmapPage binding")


# 3. Apply docs article browser CSS patch
import subprocess
r = subprocess.run(["git", "apply", "scripts/site_css_docs.patch"], capture_output=True, text=True)
print("3. git apply site_css_docs.patch:", r.returncode)
if r.stdout:
    print(r.stdout)
if r.stderr:
    print(r.stderr)
n += 1


open(p, 'w', encoding='utf-8').write(src)
print(f"Done: {n} fixes")
