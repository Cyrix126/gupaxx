#!/usr/bin/env bash 
## to be executed once you get zip files containing binaries from github CI and put them in /tmp/gupax_*

[[ -d skel ]]; check "skel"
[[ -f linux.zip ]]; check "linux zip"
[[ -f windows.zip ]]; check "windows zip"
[[ -f macos.zip ]]; check "macos zip"
unzip linux.zip; unzip macos.zip; unzip windows.zip
tar -xf windows.tar
mv gupax.exe skel/windows/Gupax.exe
mv gupax_b.exe skel/windows_b/Gupax.exe
tar -xf linux.tar
mv gupax skel/linux/gupax
mv gupax_b skel/linux_b/gupax
tar -xf macos.tar
mv Gupax-macos-x64.app/Contents/Info.plist skel/macos-x64/Gupax.app/Contents/Info.plist
mv Gupax-macos-x64.app/Contents/MacOS/gupax skel/macos-x64/Gupax.app/Contents/MacOS/gupax
mv Gupax-macos-x64.app_b/Contents/Info.plist skel/macos-x64_b/Gupax.app/Contents/Info.plist
mv Gupax-macos-x64.app_b/Contents/MacOS/gupax skel/macos-x64_b/Gupax.app/Contents/MacOS/gupax
mv Gupax-macos-arm64.app/Contents/Info.plist skel/macos-arm64/Gupax.app/Contents/Info.plist
mv Gupax-macos-arm64.app/Contents/MacOS/gupax skel/macos-arm64/Gupax.app/Contents/MacOS/gupax
mv Gupax-macos-arm64.app_b/Contents/Info.plist skel/macos-arm64_b/Gupax.app/Contents/Info.plist
mv Gupax-macos-arm64.app_b/Contents/MacOS/gupax skel/macos-arm64_b/Gupax.app/Contents/MacOS/gupax
rm -r Gupax-macos-x64.app
rm -r Gupax-macos-arm64.app
rm -r Gupax-macos-x64.app_b
rm -r Gupax-macos-arm64.app_b
rm linux.zip; rm macos.zip; rm windows.zip
# windows unzip only the exe so not tar to delete.
rm linux.tar; rm macos.tar; rm windows.tar
