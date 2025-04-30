#!/usr/bin/env bash
if [ -z "${NAME}" ]; then
  echo "Error: NAME environment variable is not set."
  echo "Usage: NAME=tod VERSION=0.6.15 ./push_aur.sh"
  exit 1
fi

echo "=== CHECKING THAT ${NAME}-bin FOLDER EXITS ===" &&
cd "../${NAME}-bin/" || exit &&
echo "=== PULLING LATEST AUR ===" &&
git pull &&
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
git add . &&
git commit -m "v$VERSION" &&
git push aur &&
cd "../${NAME}" || exit &&
echo "=== SUCCESS ===" 
