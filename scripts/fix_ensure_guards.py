#!/usr/bin/env python3
"""Hotfix: ensureAdminData pushed to demo-data arrays that don't exist for
fresh visitors (empty localStorage), crashing module evaluation -> blank site.
Adds the same if(!data.X)data.X=[] guards the rest of the function uses."""
PATH = 'app.js'
src = open(PATH).read()
anchor = " if(!data.roles)data.roles=defaultRoles();\n"
assert src.count(anchor) == 1, "anchor count=%d" % src.count(anchor)
guards = "".join(
    " if(!data.%s)data.%s=[];\n" % (t, t)
    for t in ['businessGlossaryTerms', 'metricDefinitions', 'testCaseDefinitions',
              'actionTypeDefinitions', 'aiExecutionGraphs', 'tasks']
)
src = src.replace(anchor, anchor + guards)
open(PATH, 'w').write(src)
print("guards added")
