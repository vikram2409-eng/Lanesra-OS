#!/usr/bin/env python3
"""Convert roadmap/releases to modern template via targeted replacement."""
import os

REPO = "."

def main():
    p = os.path.join(REPO, "app.js")
    src = open(p, encoding="utf-8").read()
    n = 0
    
    # 1. roadmapPage wrapper
    old1 = '<div class="site-dark">${publicNav()}<main class="mkt-page">'
    new1 = '<div class="modern-site" id="modernSite">${window.MODERN_HEADER||\'\'}<main><div class="site-dark"><div class="mkt-page">'
    if old1 in src:
        src = src.replace(old1, new1, 1)
        n += 1
        print("1. roadmapPage wrapper updated")
    else:
        print("1. NOT FOUND")
    
    # 2. roadmapPage footer
    old2 = '</main>${publicFooter()}</div>`;'
    new2 = '</div></div></main>${window.MODERN_FOOTER||\'\'}</div>`;'
    if old2 in src:
        src = src.replace(old2, new2, 1)
        n += 1
        print("2. roadmapPage footer updated")
    
    # 3. roadmapPage binding
    old3 = 'bindPublicNav();'
    new3 = "bindModernChrome(document.getElementById('modernSite'));"
    if old3 in src:
        src = src.replace(old3, new3, 1)
        n += 1
        print("3. roadmapPage binding updated")
    
    # 4. Add bindModernChrome helper
    if "function bindModernChrome(root)" not in src:
        helper = """// Shared chrome bindings for .modern-site pages.
function bindModernChrome(root){
  if(!root) return;
  try{
    const saved = localStorage.getItem('modern-theme');
    if(saved){ root.setAttribute('data-theme', saved); }
    else if(window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches){ root.setAttribute('data-theme', 'dark'); }
  }catch(e){}
  const themeBtn = root.querySelector('.theme-toggle');
  if(themeBtn) themeBtn.addEventListener('click', () => {
    const cur = root.getAttribute('data-theme') || 'light';
    const next = cur === 'dark' ? 'light' : 'dark';
    root.setAttribute('data-theme', next);
    try{ localStorage.setItem('modern-theme', next); }catch(e){}
  });
  const menuToggle = root.querySelector('.menu-toggle');
  const mobilePanel = root.querySelector('.mobile-panel');
  if(menuToggle && mobilePanel) menuToggle.addEventListener('click', () => {
    const open = mobilePanel.classList.toggle('open');
    menuToggle.setAttribute('aria-expanded', open ? 'true' : 'false');
    document.body.style.overflow = open ? 'hidden' : '';
  });
  if(mobilePanel) mobilePanel.querySelectorAll('a').forEach(a => {
    a.addEventListener('click', () => {
      mobilePanel.classList.remove('open');
      document.body.style.overflow = '';
      if(menuToggle) menuToggle.setAttribute('aria-expanded', 'false');
    });
  });
  root.querySelectorAll('.copy-code').forEach(btn => {
    btn.addEventListener('click', () => {
      const target = document.getElementById(btn.dataset.copy);
      if(!target || !navigator.clipboard) return;
      navigator.clipboard.writeText(target.innerText).then(() => {
        const old = btn.textContent; btn.textContent = 'Copied';
        setTimeout(() => { btn.textContent = old; }, 1400);
      });
    });
  });
}
"""
        src = src.replace("function modernSitePage(pageId){", helper + "function modernSitePage(pageId){", 1)
        n += 1
        print("4. Added bindModernChrome")
    
    open(p, "w", encoding="utf-8").write(src)
    print(f"Done: {n} changes")

if __name__ == "__main__":
    main()
