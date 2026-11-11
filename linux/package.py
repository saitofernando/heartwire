#!/usr/bin/env python3
"""Build a private working-tree snapshot, package it with pacman, and install its native panel."""
from pathlib import Path
import argparse
import hashlib
import os
import shutil
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parents[1]
NAME = "heartwire"
PROFILE = os.environ.get("OMARCHY_BUILD_PROFILE", "omarchy-dev")
if PROFILE not in ("omarchy-dev", "release"):
    raise SystemExit("Invalid build profile")

def install_file(source, destination, mode=0o644):
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    destination.chmod(mode)

def payload(directory):
    if directory.resolve() == Path("/"):
        raise SystemExit("Refusing to write a payload directly to the root filesystem")
    install_file(ROOT / "linux" / NAME, directory / "usr/bin" / NAME, 0o755)
    target = Path(os.environ.get("CARGO_TARGET_DIR", str(ROOT / "target"))) / PROFILE
    install_file(target / "heartwire-backend", directory / "usr/lib/heartwire/heartwire-backend", 0o755)
    plugin = ROOT / "linux/omarchy"
    destination = directory / "usr/share" / NAME / "omarchy"
    for source in plugin.rglob("*"):
        if source.is_file() and (source.suffix in (".qml", ".js", ".json", ".svg", ".so", ".qmltypes") or source.name in ("qmldir", "LICENSE")):
            install_file(source, destination / source.relative_to(plugin))
    install_file(plugin / "assets/heartwire-symbolic.svg", directory / "usr/share/icons/hicolor/scalable/status/heartwire-symbolic.svg")
    install_file(ROOT / "design/heartwire/app.svg", directory / "usr/share/icons/hicolor/scalable/apps/heartwire.svg")
    desktop = ROOT / "linux" / (NAME + ".desktop")
    install_file(desktop, directory / "usr/share/applications" / desktop.name)
    for name in ("LICENSE", "NOTICE", "THIRD_PARTY_NOTICES.linux.md"):
        install_file(ROOT / name, directory / "usr/share/licenses" / NAME / name)
    for source in (ROOT / "licenses").rglob("*"):
        if source.is_file():
            install_file(source, directory / "usr/share/licenses" / NAME / "licenses" / source.relative_to(ROOT / "licenses"))
    for name in ("README.md", "README.en.md", "CHANGELOG.md", "CHANGELOG.en.md", "VALIDACAO.md", "VALIDACAO.en.md"):
        install_file(ROOT / name, directory / "usr/share/doc" / NAME / name)
    for source in (ROOT / "docs").rglob("*"):
        if source.is_file():
            install_file(source, directory / "usr/share/doc" / NAME / "docs" / source.relative_to(ROOT / "docs"))

def run(*args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--install", action="store_true")
    parser.add_argument("--payload", type=Path)
    args = parser.parse_args()
    if args.payload:
        payload(args.payload)
        return
    build = ROOT / ".build/omarchy-package"
    build.mkdir(parents=True, exist_ok=True)
    # Include edited/new source, excluding ignored private files, dependencies and caches.
    listed = subprocess.check_output(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=ROOT)
    paths = sorted(set(Path(os.fsdecode(value)) for value in listed.split(b"\0") if value))
    with tarfile.open(build / "source.tar.gz", "w:gz") as archive:
        for relative in paths:
            source = ROOT / relative
            if source.is_file() or source.is_symlink():
                archive.add(source, arcname=str(Path("source") / relative), recursive=False)
    shutil.copyfile(ROOT / "packaging/arch/PKGBUILD", build / "PKGBUILD")
    with (build / "PKGBUILD").open("a") as file:
        run("makepkg", "--geninteg", cwd=build, stdout=file)
    environment = dict(os.environ)
    environment["CARGO_TARGET_DIR"] = str(ROOT / "target")
    # The target cache lives outside this extracted source tree and remains reusable.
    if (build / "src").exists():
        shutil.rmtree(build / "src")
    run("makepkg", "--noconfirm", "--force", cwd=build, env=environment)
    packages = [Path(line) for line in subprocess.check_output(["makepkg", "--packagelist"], cwd=build, text=True).splitlines() if "-debug-" not in Path(line).name]
    dist = ROOT / "dist/linux"
    dist.mkdir(parents=True, exist_ok=True)
    checksums = []
    for package in packages:
        shutil.copy2(package, dist / package.name)
        checksums.append(hashlib.sha256(package.read_bytes()).hexdigest() + "  " + package.name)
    (dist / "SHA256SUMS").write_text("\n".join(checksums) + "\n")
    print("Packages:", dist)
    if not args.install:
        return
    elevation = "sudo" if sys.stdin.isatty() else "pkexec"
    run(elevation, "pacman", "-U", "--noconfirm", *(str(package) for package in packages))
    user_plugin = Path.home() / ".config/omarchy/plugins" / ("saito." + NAME)
    if user_plugin.exists():
        backup = ROOT / ".build/plugin-backups" / str(time.time_ns())
        shutil.copytree(user_plugin, backup)
        print("Previous panel:", backup)
    # Replace the directory atomically; never truncate a library mapped by the shell.
    staged = user_plugin.with_name(user_plugin.name + ".new-" + str(os.getpid()))
    previous = user_plugin.with_name(user_plugin.name + ".old-" + str(os.getpid()))
    shutil.copytree("/usr/share/" + NAME + "/omarchy", staged)
    if user_plugin.exists():
        user_plugin.rename(previous)
    staged.rename(user_plugin)
    if previous.exists():
        shutil.rmtree(previous)
    run("omarchy-shell", "shell", "rescanPlugins")
    for attempt in range(4):
        result = subprocess.run(["omarchy", "plugin", "enable", "saito." + NAME, "--section", "right", "--before", "omarchy.agents"])
        if result.returncode == 0:
            break
        time.sleep(0.5)
    else:
        raise SystemExit("Could not enable the installed panel")
    # keepLoaded services and native QML libraries need one shell restart after an update.
    run("omarchy", "restart", "shell")

if __name__ == "__main__":
    main()
