#!/usr/bin/env python3
"""Apply videos + docs navigation."""
import subprocess, os
REPO = "."
p = os.path.join(REPO, "scripts", "site_videos_docs.patch")
subprocess.run(["git", "apply", p], cwd=REPO, check=True)
print("Applied")
