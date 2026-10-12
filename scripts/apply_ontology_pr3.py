#!/usr/bin/env python3
"""Apply Ontology Graph PR3 (power tools + Object 360, epic #359, issues #369-#374).

Chains, in order:
  apply_ontology_pr3a.py — power tools (zoom/pan, search, find path,
      change-impact simulation, shareable URL state)
  apply_ontology_pr3b.py — Object 360 rebuild on the shared graph engine
      (record driver, rules/workflows live states, agent access chips,
      pivot title fix + breadcrumb, cross-links)
  apply_ontology_pr3c.py — coordinator fixes (rule pass/fail semantics,
      demo-data enrichment for the Object 360 flagship record)

Each sub-script anchors on exact text and asserts exact match counts,
failing loudly on any mismatch. Run from the repo root:
  python3 scripts/apply_ontology_pr3.py
"""
import runpy
import sys

for script in (
    'scripts/apply_ontology_pr3a.py',
    'scripts/apply_ontology_pr3b.py',
    'scripts/apply_ontology_pr3c.py',
):
    sys.argv = [script]  # pr3b accepts an optional path arg; default app.js
    runpy.run_path(script, run_name='__main__')
print('PR3 full transform applied (a -> b -> c)')
