#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
command -v makepkg >/dev/null || { echo 'Execute no Arch/Omarchy com base-devel e git instalados.' >&2; exit 1; }
case "${1:-}" in ''|--install) ;; *) echo 'Uso: bash scripts/package-omarchy.sh [--install]' >&2; exit 2;; esac
if [[ -n "$(git status --porcelain --untracked-files=no)" ]]; then
  echo 'Faça commit das alterações antes de empacotar: o pacote usa o HEAD do Git.' >&2
  exit 1
fi
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
git archive --format=tar.gz --prefix=source/ HEAD -o "$work/source.tar.gz"
cp packaging/arch/PKGBUILD "$work/PKGBUILD"
(cd "$work"; makepkg --geninteg >> PKGBUILD)
args=(-s --noconfirm)
[[ "${1:-}" != --install ]] || args+=(-i)
(cd "$work"; makepkg "${args[@]}")
mkdir -p dist/linux
cp "$work"/*.pkg.tar.zst dist/linux/
(cd dist/linux; sha256sum -- *.pkg.tar.zst > SHA256SUMS)
printf 'Pacotes e hashes: %s/dist/linux\n' "$PWD"
