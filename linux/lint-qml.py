#!/usr/bin/env python3
"""Check plugin syntax against the installed Omarchy shell, without starting another shell."""
from pathlib import Path
import os
import subprocess
import tempfile
root=Path(__file__).resolve().parent
shell=Path(os.environ.get("OMARCHY_PATH", "/usr/share/omarchy"))/"shell"
with tempfile.TemporaryDirectory(prefix="omarchy-qml-lint-") as directory:
    imports=Path(directory)
    (imports/"qs").symlink_to(shell,target_is_directory=True)
    subprocess.run(["/usr/lib/qt6/bin/qmllint","--syntax","error","--syntax.duplicate-ids","error","--max-warnings","999999","-I",str(imports),*(str(path) for path in sorted((root/"omarchy").rglob("*.qml")))],check=True)
