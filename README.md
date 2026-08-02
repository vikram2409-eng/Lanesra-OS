# Lanesra OS MVP

A modern, open-source sales and business management system for small businesses.

## Included in this MVP

- Public product landing page
- No-registration live demo at `/demo`
- Companies, contacts, pipeline, products, quotes, orders, invoices, contracts and tasks
- Dashboard and global search
- Realistic sample data
- Browser-local persistence and Reset Demo
- SEO, robots.txt, sitemap.xml, llms.txt, llms-full.txt and PWA manifest
- Netlify drag-and-drop deployment

## Deploy to Netlify

1. Zip the contents of this folder, not the parent folder.
2. Open Netlify and choose **Add new site → Deploy manually**.
3. Drop the ZIP file into Netlify.
4. Attach `lanesraos.com` under Domain Management.

No build command or environment variable is required for this web MVP.

## Local preview

Because this is a single-page app, preview it with any local static server, for example:

```bash
python3 -m http.server 8080
```

Then open `http://localhost:8080`.

## Current limitation

The downloadable Tauri/SQLite desktop application is the next milestone and is not included in this first web package.


## Public product pages

- `/principles` — The product decisions and beliefs behind Lanesra OS
- `/compare` — A factual market-positioning comparison
- `/download` — Desktop platform status and planned capabilities
- `/roadmap` — Current, building and planned capabilities
- `/changelog` — Release-by-release updates

The desktop edition is still in development and is not included in this package.
