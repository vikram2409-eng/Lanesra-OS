#!/usr/bin/env python3
"""Graph revamp: mockup-matched CSS + toolbar structure fix.
Part 1: Adds GRAPH-REVAMP-2026 CSS to styles.css
Part 2: Fixes toolbar extra </div> bug in app.js
Safe to run once per branch (idempotent CSS check).
"""
import re

def patch_css():
    path = 'styles.css'
    src = open(path, encoding='utf-8').read()

    if '/* GRAPH-REVAMP-2026 */' in src:
        print("CSS revamp already applied, skipping")
        return

    revamp_css = """
/* GRAPH-REVAMP-2026: Mockup-matched design for Ontology Graph + Object 360 */
.ontology-graph-canvas{
  background-color:var(--canvas,#fbfcfe);
  background-image:radial-gradient(rgba(100,116,139,.18) 1px,transparent 1px);
  background-size:20px 20px;border-radius:12px;overflow:hidden;
}
.graph-toolbar-wrap{margin-bottom:12px}
.graph-toolbar{
  display:flex;align-items:center;gap:8px;flex-wrap:wrap;
  padding:10px 12px;background:#fff;border:1px solid var(--line,#e2e8f0);
  border-radius:12px;box-shadow:0 1px 2px rgba(15,23,42,.04);
}
.graph-toolbar .tool-group{display:flex;align-items:center;gap:6px}
.graph-toolbar .tool-group+.tool-group{border-left:1px solid var(--line,#e2e8f0);padding-left:10px}
.graph-toolbar .select,.graph-toolbar .input,.graph-toolbar .btn{min-height:34px;border-radius:8px;font-size:12px;font-weight:600}
.graph-toolbar .btn-sm{min-height:34px;padding:0 10px}
.toggle-chip{
  display:inline-flex;align-items:center;gap:6px;cursor:pointer;
  padding:0 12px;height:34px;border:1px solid var(--line,#e2e8f0);
  border-radius:999px;font-weight:600;font-size:12px;background:#fff;color:var(--ink);
  transition:all .18s ease;
}
.toggle-chip::before{content:"";width:14px;height:14px;border-radius:50%;background:var(--line-strong,#cbd5e1);transition:background .18s ease}
.toggle-chip.active{color:#3730a3;background:#eef2ff;border-color:#a5b4fc}
.toggle-chip.active::before{background:#4f46e5}
.toggle-chip:hover{border-color:var(--line-strong)}
.ontology-fields-toggle{display:flex;align-items:center;gap:6px;font-weight:600;font-size:12px;white-space:nowrap;cursor:pointer}
.ontology-fields-toggle input{accent-color:#4f46e5;width:16px;height:16px}
.ontology-node-dot{
  border:3px solid #fff;
  box-shadow:0 2px 8px rgba(15,23,42,.12),0 1px 3px rgba(15,23,42,.08);
  transition:transform .2s ease,box-shadow .2s ease,opacity .2s ease;
}
.ontology-node-dot:hover{transform:scale(1.08);box-shadow:0 8px 20px rgba(15,23,42,.18);z-index:5}
.ontology-node-dot.graph-hover-on{
  transform:scale(1.08);
  box-shadow:0 0 0 4px rgba(79,70,229,.15),0 8px 20px rgba(15,23,42,.18);z-index:5;
}
.ontology-node-caption{
  background:rgba(255,255,255,.92);border:1px solid var(--line,#e2e8f0);
  border-radius:6px;padding:2px 7px;font-size:10px;font-weight:600;
  box-shadow:0 1px 3px rgba(15,23,42,.06);backdrop-filter:blur(4px);transition:opacity .2s ease;
}
.ontology-graph-canvas svg line{stroke:#cbd5e1;stroke-width:1.5;transition:opacity .2s ease,stroke .2s ease}
.ontology-edge-label{font-size:10px;font-weight:600;fill:#64748b;paint-order:stroke;stroke:#fff;stroke-width:3px}
.graph-legend{display:flex;flex-wrap:wrap;align-items:center;gap:6px;padding:10px 12px;border-top:1px solid var(--line,#e2e8f0);background:#fff;border-radius:0 0 12px 12px}
.legend-chip{display:inline-flex;align-items:center;gap:6px;border:1px solid var(--line,#e2e8f0);background:#fff;color:#64748b;border-radius:999px;padding:6px 10px;font-size:10px;font-weight:600;cursor:pointer;transition:all .18s ease}
.legend-chip::before{content:"";width:8px;height:8px;border-radius:50%;background:var(--chip-color,#64748b)}
.legend-chip:hover{border-color:#a5b4fc;color:#3730a3}
.legend-chip.off{opacity:.4;text-decoration:line-through}
.ontology-legend{display:flex;flex-wrap:wrap;align-items:center;gap:6px;padding:10px 12px;border-top:1px solid var(--line,#e2e8f0);background:#fff}
.ontology-legend-item{display:inline-flex;align-items:center;gap:6px;border:1px solid var(--line,#e2e8f0);background:#fff;color:#64748b;border-radius:999px;padding:6px 10px;font-size:11px;font-weight:600;cursor:pointer;transition:all .18s ease}
.ontology-legend-item:hover{border-color:#a5b4fc;color:#3730a3}
.ontology-legend-item-off{opacity:.4;text-decoration:line-through}
.ontology-legend-count{background:#f1f5f9;border-radius:4px;padding:1px 6px;font-size:10px;font-weight:700;color:#475569}
.ontology-inspector-card{border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(15,23,42,.06)}
.ontology-inspector-head{padding:18px;border-bottom:1px solid var(--line,#e2e8f0)}
.ontology-type-pill{display:inline-flex;border-radius:999px;background:#eef2ff;color:#3730a3;padding:5px 10px;font-size:10px;font-weight:800;letter-spacing:.08em}
.ontology-inspector-head h3{font-size:18px;margin:10px 0 6px;letter-spacing:-.01em}
.ontology-inspector-head .desc{font-size:12px;line-height:1.55;color:#64748b;margin:0}
.ontology-inspector-actions{display:grid;gap:7px;padding:14px 18px;border-bottom:1px solid var(--line,#e2e8f0)}
.ontology-inspector-actions .btn{width:100%;height:38px;border-radius:8px;font-size:12px;font-weight:700;justify-content:center}
.ontology-inspector-actions .btn-primary{background:#4f46e5;border-color:#4f46e5;color:#fff}
.ontology-inspector-actions .btn-primary:hover{background:#4338ca}
.ontology-inspector-body{padding:8px 18px 18px;overflow:auto}
.ontology-inspector-group{margin-top:14px}
.ontology-inspector-group-label{display:flex;justify-content:space-between;color:#64748b;font-size:10px;letter-spacing:.08em;font-weight:800;margin-bottom:4px;text-transform:uppercase}
.ontology-inspector-bullet{display:flex;justify-content:space-between;align-items:center;width:100%;text-align:left;padding:8px 0;border:0;border-bottom:1px solid var(--line,#e2e8f0);background:transparent;font-size:12px;font-weight:600;cursor:pointer;color:var(--ink)}
.ontology-inspector-bullet:hover{color:#4f46e5}
.graph-path-banner{position:absolute;z-index:20;left:50%;top:14px;transform:translateX(-50%);display:flex;align-items:center;gap:12px;padding:9px 14px;background:#111827;color:#fff;border-radius:8px;box-shadow:0 8px 24px rgba(15,23,42,.2);font-size:12px;font-weight:600}
.graph-path-banner button{border:0;background:rgba(255,255,255,.15);color:#fff;border-radius:5px;padding:4px 10px;cursor:pointer;font-weight:600}
.graph-path-banner button:hover{background:rgba(255,255,255,.25)}
.o360-dialog{border-radius:14px;overflow:hidden;box-shadow:0 24px 64px rgba(15,23,42,.2)}
.o360-dialog-head{padding:16px 20px;border-bottom:1px solid var(--line,#e2e8f0);background:#fff}
.o360-record-overline{font-size:10px;letter-spacing:.1em;font-weight:800;color:#4f46e5;margin-bottom:4px;text-transform:uppercase}
.o360-dialog-title{font-size:22px;font-weight:700;letter-spacing:-.01em;margin:0}
.o360-breadcrumb{font-size:11px;color:#64748b;margin-top:6px}
.o360-breadcrumb button{border:0;background:none;padding:0;color:#4f46e5;font-weight:700;font-size:11px;cursor:pointer}
.o360-breadcrumb button:hover{text-decoration:underline}
.graph-toast{position:absolute;z-index:30;left:50%;top:16px;transform:translateX(-50%);display:flex;align-items:center;gap:8px;padding:10px 16px;background:#111827;color:#fff;border-radius:8px;font-size:12px;font-weight:600;box-shadow:0 8px 24px rgba(15,23,42,.2);animation:graph-toast-in .25s ease both}
@keyframes graph-toast-in{from{opacity:0;transform:translateX(-50%) translateY(-8px)}to{opacity:1;transform:translateX(-50%) translateY(0)}}
.ontology-node-dot.node-expand-in{animation:node-expand-in .3s cubic-bezier(.34,1.4,.64,1) both}
@keyframes node-expand-in{from{opacity:0;transform:scale(.5)}to{opacity:1;transform:scale(1)}}
.zoom-readout,[data-zoom-readout]{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:11px;font-weight:700;color:#64748b;min-width:44px;text-align:center}
.ontology-search-results{position:absolute;top:100%;left:0;right:0;z-index:50;background:#fff;border:1px solid var(--line,#e2e8f0);border-radius:8px;margin-top:4px;max-height:240px;overflow:auto;box-shadow:0 8px 24px rgba(15,23,42,.12)}
.ontology-search-results button{display:block;width:100%;text-align:left;padding:8px 12px;border:0;background:none;font-size:12px;cursor:pointer}
.ontology-search-results button:hover{background:#f1f5f9}
@media(max-width:768px){
  .graph-toolbar .tool-group{width:100%}
  .graph-toolbar .tool-group+.tool-group{border-left:0;border-top:1px solid var(--line,#e2e8f0);padding-left:0;padding-top:8px}
  .ontology-node-caption{font-size:9px;padding:1px 5px}
}
"""

    src = src.rstrip() + "\n" + revamp_css + "\n"
    open(path, 'w', encoding='utf-8').write(src)
    print("CSS revamp applied")

    # Verify brace balance
    css = open(path, encoding='utf-8').read()
    assert css.count('{') == css.count('}'), f"Unbalanced braces: {css.count('{')} vs {css.count('}')}"
    print(f"Braces balanced: {css.count('{')}/{css.count('}')}")

def patch_toolbar():
    path = 'app.js'
    with open(path, 'rb') as f:
        data = f.read()

    # Fix extra </div> in toolbar: </label></div></div>\n -> </label></div>\n
    old = b'</label></div></div>\n  <div class="tool-group"><div class="search-wrap"'
    new = b'</label></div>\n  <div class="tool-group"><div class="search-wrap"'

    count = data.count(old)
    if count == 1:
        data = data.replace(old, new)
        with open(path, 'wb') as f:
            f.write(data)
        print("Toolbar div fix applied")
    elif count == 0:
        print("Toolbar already fixed or pattern changed, skipping")
    else:
        print(f"WARNING: unexpected pattern count {count}")

if __name__ == '__main__':
    patch_css()
    patch_toolbar()
    print("done")
