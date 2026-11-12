// Zero-dependency static server for the repo-root website (WP-0702).
// Serves the website + online demo the way Netlify does: files from the
// repo root, SPA fallback to index.html for anything else (so
// /demo/warehouse loads the demo shell), so Playwright can drive the
// warehouse demo at http://localhost:<port>/demo/warehouse.
// The Team Workspace server (lanesra-server, the Playwright config's own
// webServer) does NOT serve this site — the warehouse demo is a static
// Netlify site — so the lane spec starts this server itself in beforeAll.
// Usage: node static-site-server.mjs [port]  (default 4173)
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const port = Number(process.argv[2] || process.env.STATIC_SITE_PORT || 4173);
const types = {
  '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript',
  '.css': 'text/css', '.json': 'application/json', '.png': 'image/png',
  '.jpg': 'image/jpeg', '.svg': 'image/svg+xml', '.ico': 'image/x-icon',
  '.txt': 'text/plain', '.xml': 'application/xml',
};
const server = http.createServer((req, res) => {
  const url = decodeURIComponent((req.url || '/').split('?')[0]);
  let file = path.join(root, url);
  // Never serve outside the repo root; anything else is the SPA fallback.
  if (!file.startsWith(root) || !fs.existsSync(file) || fs.statSync(file).isDirectory()) {
    file = path.join(root, 'index.html');
  }
  const ext = path.extname(file);
  fs.readFile(file, (err, body) => {
    if (err) { res.writeHead(404); res.end('not found'); return; }
    res.writeHead(200, { 'content-type': types[ext] || 'application/octet-stream' });
    res.end(body);
  });
});
server.listen(port, () => console.log(`static site on http://localhost:${port} (root ${root})`));
