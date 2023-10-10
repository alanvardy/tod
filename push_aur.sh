#!/bin/bash

cd ../tod-bin/ || exit
git pull
cd ../tod/ || exit
mv target/cargo-aur/PKGBUILD ./PKGBUILD
makepkg --printsrcinfo > ../tod-bin/.SRCINFO
mv PKGBUILD ../tod-bin/
rm target/cargo-aur/*.tar.gz
cd ../tod-bin/ || exit
git add .
git commit -m "new version"
git push aur
cd ../tod || exit
