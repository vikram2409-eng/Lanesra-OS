#!/usr/bin/env python3
"""Simulate enhancements: add-field, add-object, add-relationship, add-workflow
scenarios in the Ontology Graph Simulate tool. Safe to run once per branch.
"""
import re

def patch(path):
    src = open(path, encoding='utf-8').read()
    changes = []
    old_opts = '<select id="depSimChange" class="select"><option value="rename">rename</option><option value="delete">delete</option><option value="revoke access">revoke access</option></select>'
    new_opts = '<select id="depSimChange" class="select"><option value="rename">rename</option><option value="delete">delete</option><option value="revoke access">revoke access</option><option value="add-field">add field</option><option value="add-object">add object</option><option value="add-relationship">add relationship</option><option value="add-workflow">add workflow</option></select> <span id="depSimInputs"></span>'
    assert src.count(old_opts) == 1, "sim select not found"
    src = src.replace(old_opts, new_opts)
    changes.append("4 new options")
    marker = "function depSimBannerHtml(){"
    assert marker in src
    sim_fns = """
function depSimAddField(type,id,fieldName){
 const seen=new Set(),out=[];
 const stack=[{type,id}];
 while(stack.length){
  const n=stack.pop();
  const key=depNodeKey(n.type,n.id);
  if(seen.has(key))continue;
  seen.add(key);out.push(key);
  ['dependencies','dependents'].forEach(dir=>{
   systemGraphDirect(n.type,n.id,dir).forEach(h=>stack.push({type:h.type,id:h.id}));
  });
 }
 return {keys:out,label:'add field \\u2018'+fieldName+'\\u2019 to '+systemGraphNodeLabel(type,id),detail:out.length+' connected components would need review'};
}
function depSimAddObject(name,relType,relId){
 const key=depNodeKey(relType,relId);
 const seen=new Set([key]),out=[key];
 ['dependencies','dependents'].forEach(dir=>{
  systemGraphDirect(relType,relId,dir).forEach(h=>{
   const k=depNodeKey(h.type,h.id);
   if(!seen.has(k)){seen.add(k);out.push(k)}
  });
 });
 return {keys:out,label:'add object \\u2018'+name+'\\u2019 related to '+systemGraphNodeLabel(relType,relId),detail:'would connect through '+out.length+' neighboring components'};
}
function depSimAddRelationship(fType,fId,tType,tId){
 const fromKey=depNodeKey(fType,fId),toKey=depNodeKey(tType,tId);
 const visited=new Set(),stack=[{type:tType,id:tId}];
 let cycle=false;
 while(stack.length&&!cycle){
  const n=stack.pop();
  const k=depNodeKey(n.type,n.id);
  if(k===fromKey){cycle=true;break}
  if(visited.has(k))continue;
  visited.add(k);
  systemGraphDirect(n.type,n.id,'dependencies').forEach(h=>stack.push({type:h.type,id:h.id}));
 }
 return {keys:[fromKey,toKey],label:'link '+systemGraphNodeLabel(fType,fId)+' \\u2192 '+systemGraphNodeLabel(tType,tId),detail:cycle?'WARNING: circular dependency would be created':'new edge; no cycles detected'};
}
function depSimAddWorkflow(triggerType,triggerId,wfName){
 const keys=depSimulateImpact(triggerType,triggerId);
 return {keys:keys,label:'new workflow \\u2018'+wfName+'\\u2019 on '+systemGraphNodeLabel(triggerType,triggerId),detail:keys.length+' downstream components could be affected'};
}
function depSimRenderInputs(){
 const sel=document.getElementById('depSimChange');
 const host=document.getElementById('depSimInputs');
 if(!sel||!host)return;
 const v=sel.value;
 let html='';
 if(v==='add-field'){
  html='<input type="text" id="depSimFieldName" class="input" placeholder="Field name…" style="width:140px" aria-label="Field name">';
 }else if(v==='add-object'){
  html='<input type="text" id="depSimObjName" class="input" placeholder="Object name…" style="width:140px" aria-label="Object name">';
 }else if(v==='add-relationship'){
  html='<select id="depSimRelTarget" class="select" aria-label="Relate to"><option value="">Relate to…</option></select>';
 }else if(v==='add-workflow'){
  html='<input type="text" id="depSimWfName" class="input" placeholder="Workflow name…" style="width:140px" aria-label="Workflow name">';
 }
 host.innerHTML=html;
 const tgt=document.getElementById('depSimRelTarget');
 if(tgt){
  const nodes=systemGraphNodes();
  tgt.innerHTML='<option value="">Relate to…</option>'+nodes.slice(0,80).map(n=>'<option value="'+n.type+':'+n.id+'">'+systemGraphNodeLabel(n.type,n.id)+'</option>').join('');
 }
}
"""
    src = src.replace(marker, sim_fns + marker)
    changes.append("sim functions")
    old_run = """ if(simRun)simRun.onclick=()=>{
  const s=dependencyExplorerState;
  if(!s.inspectType||!s.inspectId){toast('Select a node first');return}
  const chg=document.getElementById('depSimChange').value;
  dependencyExplorerState={...s,simKeys:depSimulateImpact(s.inspectType,s.inspectId),simChange:chg,simRoot:{type:s.inspectType,id:s.inspectId}};
  renderAdminTab();
 };"""
    assert src.count(old_run) == 1, "simRun not found"
    new_run = """ if(simRun)simRun.onclick=()=>{
  const s=dependencyExplorerState;
  if(!s.inspectType||!s.inspectId){toast('Select a node first');return}
  const chg=document.getElementById('depSimChange').value;
  let simResult=null;
  if(chg==='add-field'){
   const fn=(document.getElementById('depSimFieldName')||{}).value||'new_field';
   simResult=depSimAddField(s.inspectType,s.inspectId,fn);
  }else if(chg==='add-object'){
   const nm=(document.getElementById('depSimObjName')||{}).value||'New Object';
   simResult=depSimAddObject(nm,s.inspectType,s.inspectId);
  }else if(chg==='add-relationship'){
   const tv=(document.getElementById('depSimRelTarget')||{}).value||'';
   if(!tv){toast('Pick a target object');return}
   const ci=tv.indexOf(':');
   simResult=depSimAddRelationship(s.inspectType,s.inspectId,tv.slice(0,ci),tv.slice(ci+1));
  }else if(chg==='add-workflow'){
   const wn=(document.getElementById('depSimWfName')||{}).value||'New Workflow';
   simResult=depSimAddWorkflow(s.inspectType,s.inspectId,wn);
  }
  if(simResult){
   dependencyExplorerState={...s,simKeys:simResult.keys,simChange:chg,simRoot:{type:s.inspectType,id:s.inspectId},simLabel:simResult.label,simDetail:simResult.detail};
  }else{
   dependencyExplorerState={...s,simKeys:depSimulateImpact(s.inspectType,s.inspectId),simChange:chg,simRoot:{type:s.inspectType,id:s.inspectId}};
  }
  renderAdminTab();
 };
 const simSel=document.getElementById('depSimChange');
 if(simSel){simSel.onchange=depSimRenderInputs;depSimRenderInputs()}"""
    src = src.replace(old_run, new_run)
    changes.append("simRun dispatch")
    old_banner = """ return '<div class="ontology-sim-banner"><span><strong>Preview only — no changes applied.</strong> '+n+' component'+(n===1?'':'s')+' would be affected by '+s.simChange+' of '+label+'.</span>"""
    new_banner = """ const simTitle=s.simLabel||(n+' component'+(n===1?'':'s')+' would be affected by '+s.simChange+' of '+label);
 const simSub=s.simDetail?'<br><span class="muted">'+s.simDetail+'</span>':'';
 return '<div class="ontology-sim-banner"><span><strong>Preview only — no changes applied.</strong> '+simTitle+'.'+simSub+'</span>"""
    assert src.count(old_banner) == 1, "banner not found"
    src = src.replace(old_banner, new_banner)
    changes.append("banner")
    open(path, 'w', encoding='utf-8').write(src)
    print("done:", changes)

if __name__ == '__main__':
    patch('app.js')
    print("done")
