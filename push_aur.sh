#!/bin/bash

cd ../tod-bin/ || exit
git pull
cd ../tod/ || exit
makepkg --printsrcinfo > ../tod-bin/.SRCINFO
mv PKGBUILD ../tod-bin/
rm ./*.tar.gz
cd ../tod-bin/ || exit
git add .
git commit -m "new version"
git push aur
cd ../tod || exit