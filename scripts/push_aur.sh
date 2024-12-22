#!/bin/bash

echo "=== CHECKING THAT tod-bin FOLDER EXITS ===" &&
cd ../tod-bin/ || exit &&
echo "=== PULLING LATEST AUR ===" &&
git pull &&
echo "=== CHECKING THAT tod FOLDER EXITS ===" &&
cd ../tod/ || exit &&
echo "=== MOVING PKGBUILD ===" &&
mv target/cargo-aur/PKGBUILD ./PKGBUILD
echo "=== RUNNING MAKEPKG ===" &&
makepkg --printsrcinfo > ../tod-bin/.SRCINFO &&
mv PKGBUILD ../tod-bin/ &&
echo "=== DELETING TAR.GZ ===" &&
rm target/cargo-aur/*.tar.gz &&
cd ../tod-bin/ || exit &&
echo "=== PUSHING TO AUR ===" &&
git add . &&
git commit -m "v$VERSION" &&
git push aur &&
cd ../tod || exit &&
echo "=== SUCCESS ===" 
