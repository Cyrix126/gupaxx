#!/bin/bash

## to be executed once you get zip files containing binairies from github CI and put them in /tmp/gupaxx_*

[[ -d skel ]]; check "skel"
[[ -f linux.zip ]]; check "linux zip"
[[ -f windows.zip ]]; check "windows zip"
[[ -f macos.zip ]]; check "macos zip"
unzip linux.zip; unzip macos.zip; unzip windows.zip
mv gupaxx.exe skel/windows/Gupaxx.exe
tar -xf linux.tar
mv gupaxx skel/linux/gupaxx
tar -xf macos.tar
mv Gupaxx-macos-x64.app/Contents/Info.plist skel/macos-x64/Gupaxx.app/Contents/Info.plist
mv Gupaxx-macos-x64.app/Contents/MacOS/gupaxx skel/macos-x64/Gupaxx.app/Contents/MacOS/gupaxx
mv Gupaxx-macos-arm64.app/Contents/MacOS/gupaxx skel/macos-arm64/Gupaxx.app/Contents/MacOS/gupaxx
mv Gupaxx-macos-arm64.app/Contents/Info.plist skel/macos-arm64/Gupaxx.app/Contents/Info.plist
rm linux.zip; rm macos.zip; rm windows.zip
# windows unzip only the exe so not tar to delete.
rm linux.tar; rm macos.tar
