cd ../tod-bin/
git pull
cd ../tod/
makepkg --printsrcinfo > ../tod-bin/.SRCINFO
mv PKGBUILD ../tod-bin/
rm *.tar.gz
cd ../tod-bin/
git add .
git commit -m v0.3.7
git push aur
cd ../tod