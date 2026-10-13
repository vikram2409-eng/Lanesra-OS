#!/usr/bin/env python3
"""Fix Object 360: graph node clicks refresh inspector; fix button alignment."""
def patch(path):
    src = open(path, encoding='utf-8').read()
    old_attach = """ body.querySelectorAll('[data-o360g-inspect]').forEach(el=>el.onclick=()=>{
  const v=el.dataset.o360gInspect,ci=v.indexOf(':');
  const entityType=v.slice(0,ci),entityId=v.slice(ci+1);
  object360State={...object360State,inspect:{entityType,entityId,label:driver.label(entityType,entityId)}};
  renderObject360Body();
 });"""
    assert src.count(old_attach) == 1, "handler not found"
    new_attach = """ const o360InspectHandler=(el)=>{
  const v=el.dataset.o360gInspect,ci=v.indexOf(':');
  const entityType=v.slice(0,ci),entityId=v.slice(ci+1);
  object360State={...object360State,inspect:{entityType,entityId,label:driver.label(entityType,entityId)}};
  renderObject360Body();
 };
 body.querySelectorAll('[data-o360g-inspect]').forEach(el=>el.onclick=()=>o360InspectHandler(el));
 const o360GraphHost=document.querySelector('#o360MiniGraph');
 if(o360GraphHost)o360GraphHost.querySelectorAll('[data-o360g-inspect]').forEach(el=>el.onclick=(e)=>{e.stopPropagation();o360InspectHandler(el)});"""
    src = src.replace(old_attach, new_attach)
    open(path, 'w', encoding='utf-8').write(src)
    print("o360 click fix applied")
def patch_css(path):
    src = open(path, encoding='utf-8').read()
    additions = """
.ontology-inspector-actions{display:flex;flex-wrap:wrap;gap:8px;align-items:center}
.ontology-inspector-actions .btn{flex:0 1 auto;min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:100%}
.ontology-inspector-actions .btn-primary{flex:1 1 auto}
#o360OpenOntology{flex:1 1 100%;justify-content:center}
"""
    if '.ontology-inspector-actions{display:flex' not in src:
        src = src.rstrip('\n') + '\n' + additions
    open(path, 'w', encoding='utf-8').write(src)
    print("css done")
if __name__ == '__main__':
    patch('app.js')
    patch_css('styles.css')
    print("done")
