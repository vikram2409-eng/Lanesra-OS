#!/usr/bin/env python3
"""PR3-C: coordinator fixes for Ontology Graph PR3 (epic #359).

1. Correct rule pass/fail semantics in the Object 360 rules section:
   validation rules (block_save/show_error) fire when the record violates
   them, so the record passes when their conditions do NOT match.
2. One-time demo-data enrichment so company records show live
   Rules/Workflows sections and the depth-2 record graph for the flagship
   demo company (BrightPath Logistics) reaches the OB-1 bar of 20 nodes.

Applies AFTER apply_ontology_pr3a.py and apply_ontology_pr3b.py.
Each replacement is anchored on exact text and asserts exactly 1 match.
Run from the repo root:  python3 scripts/apply_ontology_pr3c.py
"""
import sys

PATH = 'app.js'


def rep(content, old, new, name):
    n = content.count(old)
    if n != 1:
        print('FAIL [%s]: expected exactly 1 match, found %d' % (name, n), file=sys.stderr)
        sys.exit(1)
    return content.replace(old, new, 1)


src = open(PATH, encoding='utf-8').read()

# --- 1. Pass/fail semantics ---------------------------------------------
old_verdict = """   const ruleRows=rules.map(r=>{
    const verdict=rootRecord?(ruleConditionsMatch(r,rootRecord)?'passing':'failing'):null;
    return `<button type="button" class="ontology-inspector-bullet" data-o360-admin="rules" title="Open in Admin → Business Rules"><span style="flex:1;text-align:left">${r.name||'Untitled rule'}</span>${verdict?`<span class="o360-chip ${verdict==='passing'?'o360-chip-pass':'o360-chip-fail'}">${verdict}</span>`:''}</button>`;
   }).join('');"""
new_verdict = """   const ruleRows=rules.map(r=>{
    // Pass/fail: validation rules (block_save/show_error) fire when the
    // record violates them, so the record passes when their conditions do
    // NOT match. Effect rules have no pass/fail - show whether they apply.
    const isValidation=(r.actions||[]).some(a=>a.type==='block_save'||a.type==='show_error');
    const matched=rootRecord?ruleConditionsMatch(r,rootRecord):null;
    const verdict=matched===null?null:(isValidation?(matched?'failing':'passing'):(matched?'applying':'not applying'));
    const chipCls=(verdict==='passing'||verdict==='applying')?'o360-chip-pass':(verdict==='failing'?'o360-chip-fail':'o360-chip-neutral');
    return `<button type="button" class="ontology-inspector-bullet" data-o360-admin="rules" title="Open in Admin → Business Rules"><span style="flex:1;text-align:left">${r.name||'Untitled rule'}</span>${verdict?`<span class="o360-chip ${chipCls}">${verdict}</span>`:''}</button>`;
   }).join('');"""
src = rep(src, old_verdict, new_verdict, 'rule-verdict-semantics')

# --- 2. Demo-data enrichment ---------------------------------------------
old_anchor = """  ogScope('Draft Writer',[]);
 }"""
new_anchor = """  ogScope('Draft Writer',[]);
 }
 // Object 360 (epic #359, PR 3): one-time demo data so company records show
 // live Rules/Workflows sections and the depth-2 record graph for the
 // flagship demo company (BrightPath Logistics) reaches the OB-1 bar of
 // 20 nodes. Same id-guarded one-time convention as the ontology block.
 if(!data.workspace.object360Seeded){
  data.workspace.object360Seeded=true;
  const o360HasId=(arr,id)=>(arr||[]).some(x=>x.id===id);
  if(!o360HasId(data.fieldRules,'fr_co_industry'))data.fieldRules.push(stampCreate({id:'fr_co_industry',name:'Require Industry',entity:'companies',matchType:'all',active:true,conditions:[{fieldKey:'industry',operator:'is_empty',value:'',compareField:null,groupId:null}],actions:[{type:'block_save',message:'Industry is required before saving a company.'}]}));
  if(!o360HasId(data.workflowRules,'wf_co_welcome'))data.workflowRules.push(stampCreate({id:'wf_co_welcome',name:'Company Welcome Sequence',entity:'companies',matchType:'all',active:true,conditions:[{fieldKey:'status',operator:'equals',value:'Customer',compareField:null,groupId:null}],actions:[{type:'create_task',taskTitle:'Send welcome package',daysOffset:1}],notify:false}));
  if(!o360HasId(data.workflowRules,'wf_co_health'))data.workflowRules.push(stampCreate({id:'wf_co_health',name:'Company Health Check',entity:'companies',matchType:'all',active:true,conditions:[{fieldKey:'industry',operator:'equals',value:'Logistics',compareField:null,groupId:null}],actions:[{type:'create_task',taskTitle:'Quarterly health check',daysOffset:7}],notify:false}));
  if(!o360HasId(data.tasks,'t_360_o1'))data.tasks.push({id:'t_360_o1',title:'Prepare Platform Modernization demo',relatedType:'Opportunity',relatedId:'o1',owner:'Maya Chen',due:'2026-10-20',priority:'High',status:'Open'});
  if(!o360HasId(data.tasks,'t_360_p1'))data.tasks.push({id:'t_360_p1',title:'Intro call with Ava Martin',relatedType:'Contact',relatedId:'p1',owner:'Maya Chen',due:'2026-10-18',priority:'Medium',status:'Open'});
 }"""
src = rep(src, old_anchor, new_anchor, 'demo-enrichment')

open(PATH, 'w', encoding='utf-8').write(src)
print('PR3-C applied: 2 replacements')
