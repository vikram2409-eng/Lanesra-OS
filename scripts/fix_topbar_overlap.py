#!/usr/bin/env python3
"""Fix mobile topbar overlapping content."""
import os
REPO = "."

def main():
    p = os.path.join(REPO, "styles.css")
    with open(p, 'rb') as f:
        css = f.read()
    
    # Add mobile-specific fix for topbar/content overlap
    # The issue: on iOS Safari, position:sticky inside grid doesn't work reliably,
    # causing page headings to slide under the topbar
    
    fix_css = b'''
/* Mobile topbar overlap fix: ensure content clears the sticky topbar */
@media(max-width:900px){
  .main-area{display:flex;flex-direction:column;min-height:100vh}
  .topbar{position:sticky;top:0;z-index:20;flex-shrink:0;background:var(--card)}
  .content,.page{flex:1;min-height:0}
  .page-head,.page-heading{scroll-margin-top:70px}
  .page-head h1,.page-heading h1{scroll-margin-top:70px}
}
'''
    
    # Add at the end of the file
    if b'Mobile topbar overlap fix' not in css:
        css = css.rstrip() + b'\n' + fix_css
        print("Added mobile topbar overlap fix")
    else:
        print("Fix already present")
    
    with open(p, 'wb') as f:
        f.write(css)
    
    print("Done")

if __name__ == "__main__":
    main()
