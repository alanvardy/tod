#!/usr/bin/env bash
# Purpose: This script automates the process of creating a PR to release to AUR
# Usage: NAME=tod VERSION=0.6.15 ./push_aur.sh

if [ -z "${NAME}" ]; then
  echo "Error: NAME environment variable is not set."
  echo "Usage: NAME=tod VERSION=0.6.15 ./push_aur.sh"
  exit 1
fi

echo "=== CHECKING THAT ${NAME}-bin FOLDER EXISTS ===" &&
if [ ! -d "../${NAME}-bin/" ]; then
  echo "Error: Folder '../${NAME}-bin/' does not exist."
  exit 1
fi
echo "=== CHECKING THAT ${NAME} FOLDER EXISTS ===" &&
if [ ! -d "../${NAME}/" ]; then
  echo "Error: Folder '../${NAME}/' does not exist."
  exit 1
fi
cd "../${NAME}/" || exit &&
echo "=== MOVING PKGBUILD ===" &&
if [ ! -f "target/cargo-aur/PKGBUILD" ]; then
  echo "Error: File 'target/cargo-aur/PKGBUILD' does not exist."
  exit 1
fi
mv target/cargo-aur/PKGBUILD ./PKGBUILD
echo "=== CHECKING THAT ${NAME} FOLDER EXITS ===" &&
cd "../${NAME}/" || exit &&
echo "=== MOVING PKGBUILD ===" &&
mv target/cargo-aur/PKGBUILD ./PKGBUILD
echo "=== RUNNING MAKEPKG ===" &&
makepkg --printsrcinfo > "../${NAME}-bin/.SRCINFO" &&
mv PKGBUILD "../${NAME}-bin/" &&
echo "=== DELETING TAR.GZ ===" &&
rm target/cargo-aur/*.tar.gz &&
cd "../${NAME}-bin/" || exit &&
echo "=== PUSHING TO AUR ===" &&
if [ -z "${VERSION}" ]; then
  echo "Error: VERSION environment variable is not set."
  exit 1
fi
git add . &&
git commit -m "v$VERSION" &&
git push aur &&
cd "../${NAME}" || exit &&
echo "=== Successfully pushed ${NAME} v${VERSION} to AUR ==="
