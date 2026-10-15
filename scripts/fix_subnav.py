#!/usr/bin/env python3
"""Convert subnav buttons to proper anchor links."""
import os, re
REPO = "."

# Map data-page values to URLs
PAGE_URLS = {
    'product-apps': '/platform',
    'product-agents': '/agent-foundry',
    'product-orchestration': '/product/orchestration',
    'dev-architecture': '/developers',
    'dev-apis': '/developers/apis',
    'dev-mcp': '/developers/mcp',
    'dev-solutions': '/developers/solutions',
    'dev-integrations': '/integrations',
}

def main():
    p = os.path.join(REPO, "site-modern-pages.js")
    src = open(p, encoding="utf-8").read()
    
    # Pattern: <button class="route[ on]" data-page="XXX">Label</button>
    # Replace with: <a class="route[ on]" href="/url" data-page="XXX">Label</a>
    
    def replace_button(m):
        full_class = m.group(1)  # "route" or "route on"
        page = m.group(2)        # data-page value
        label = m.group(3)       # button text
        url = PAGE_URLS.get(page, '#')
        return f'<a class="{full_class}" href="{url}" data-page="{page}">{label}</a>'
    
    # Match buttons in subnav
    pattern = r'<button class="(route(?: on)?)" data-page="([^"]+)">([^<]+)</button>'
    new_src, count = re.subn(pattern, replace_button, src)
    
    print(f"Converted {count} subnav buttons to links")
    open(p, "w", encoding="utf-8").write(new_src)
    print("Done")

if __name__ == "__main__":
    main()
