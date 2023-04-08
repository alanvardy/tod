cd ../tod-bin/
git pull
cd ../tod/
makepkg --printsrcinfo > ../tod-bin/.SRCINFO
mv PKGBUILD ../tod-bin/
rm *.tar.gz
cd ../tod-bin/
git add .
git commit -m new version
git push aur
cd ../tod