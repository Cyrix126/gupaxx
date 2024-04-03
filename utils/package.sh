#!/usr/bin/env bash

START_TIME=$EPOCHSECONDS

title() { printf "\n\e[1;93m%s\e[0m\n" "============================ $1 ============================"; }
check() {
	local CODE=$?
	if [[ $CODE = 0 ]]; then
		printf "${BASH_LINENO} | %s ... \e[1;92mOK\e[0m\n" "$1"
	else
		printf "${BASH_LINENO} | %s ... \e[1;91mFAIL\e[0m\n" "$1"
		exit $CODE
	fi
}
int() {
	exit 1
}

trap 'int' INT

title "Basic checks"
# Check for needed files
[[ -d skel ]]; check "skel"
[[ -f skel/CHANGELOG.md ]]; check "skel/CHANGELOG.md"
[[ $1 = v* ]]; check "\$1 ... $1"
NEW_VER="$1"
cd skel; check "CD into skel"

# Check that [skel] directory contains everything
# and that the naming schemes are correct
title "Linux folder check"
[[ -f linux/gupaxx ]]; check "linux/gupaxx"
[[ -f linux/Gupaxx.AppImage ]]; check "linux/Gupaxx.AppImage"
OUTPUT=$(cat linux/Gupaxx.AppImage)
[[ $OUTPUT = "./gupaxx" ]]; check "linux/Gupaxx.AppImage = ./gupaxx"
[[ -f linux/p2pool/p2pool ]]; check "linux/p2pool/p2pool"
[[ -f linux/xmrig/xmrig ]]; check "linux/xmrig/xmrig"
title "macOS-x64 folder check"
[[ -d macos-x64/Gupaxx.app ]]; check "macos-x64/Gupaxx.app"
[[ -f macos-x64/Gupaxx.app/Contents/MacOS/p2pool/p2pool ]]; check "macos-x64/p2pool/p2pool"
[[ -f macos-x64/Gupaxx.app/Contents/MacOS/xmrig/xmrig ]]; check "macos-x64/xmrig/xmrig"
title "macOS-arm64 folder check"
[[ -d macos-arm64/Gupaxx.app ]]; check "macos-arm64/Gupaxx.app"
[[ -f macos-arm64/Gupaxx.app/Contents/MacOS/p2pool/p2pool ]]; check "macos-arm64/p2pool/p2pool"
[[ -f macos-arm64/Gupaxx.app/Contents/MacOS/xmrig/xmrig ]]; check "macos-arm64/xmrig/xmrig"
title "Windows folder check"
[[ -f windows/Gupaxx.exe ]]; check "windows/Gupaxx.exe"
[[ -f windows/P2Pool/p2pool.exe ]]; check "windows/P2Pool/p2pool.exe"
[[ -f windows/XMRig/xmrig.exe ]]; check "windows/XMRig/xmrig.exe"


# Tar Linux Bundle
title "Tar Linux"
# give execution permission
chmod +x linux/gupaxx
chmod +x linux/p2pool/p2pool
chmod +x linux/xmrig/xmrig
mv linux "gupaxx-$NEW_VER-linux-x64-bundle"; check "linux -> gupaxx-$NEW_VER-linux-x64-bundle"
tar -czpf "gupaxx-${NEW_VER}-linux-x64-bundle.tar.gz" "gupaxx-$NEW_VER-linux-x64-bundle" --owner=lm --group=lm ; check "tar linux-bundle"
# Tar Linux Standalone
mv "gupaxx-$NEW_VER-linux-x64-bundle" "gupaxx-$NEW_VER-linux-x64-standalone"; check "gupaxx-$NEW_VER-linux-x64-bundle -> gupaxx-$NEW_VER-linux-x64-standalone"
rm -r "gupaxx-$NEW_VER-linux-x64-standalone/p2pool"; check "rm gupaxx-$NEW_VER-linux-x64-standalone/p2pool"
rm -r "gupaxx-$NEW_VER-linux-x64-standalone/xmrig"; check "rm gupaxx-$NEW_VER-linux-x64-standalone/xmrig"
tar -czpf "gupaxx-${NEW_VER}-linux-x64-standalone.tar.gz" "gupaxx-$NEW_VER-linux-x64-standalone" --owner=lm --group=lm ; check "tar linux-standalone"
# Remove dir
rm -r "gupaxx-$NEW_VER-linux-x64-standalone"; check "rm linux dir"

# x64
# Tar macOS Bundle
title "Tar macOS-x64"
mv macos-x64 "gupaxx-$NEW_VER-macos-x64-bundle"; check "macos-x64 -> gupaxx-$NEW_VER-macos-x64-bundle"
tar -czpf "gupaxx-${NEW_VER}-macos-x64-bundle.tar.gz" "gupaxx-$NEW_VER-macos-x64-bundle" --owner=lm --group=lm ; check "tar macos-bundle"
# Tar macOS Standalone
mv "gupaxx-$NEW_VER-macos-x64-bundle" "gupaxx-$NEW_VER-macos-x64-standalone"; check "gupaxx-$NEW_VER-macos-x64-bundle -> gupaxx-$NEW_VER-macos-x64-standalone"
rm -r "gupaxx-$NEW_VER-macos-x64-standalone/Gupaxx.app/Contents/MacOS/p2pool"; check "rm gupaxx-$NEW_VER-macos-x64-standalone/Gupaxx.app/Contents/MacOS/p2pool"
rm -r "gupaxx-$NEW_VER-macos-x64-standalone/Gupaxx.app/Contents/MacOS/xmrig"; check "rm gupaxx-$NEW_VER-macos-x64-standalone/Gupaxx.app/Contents/MacOS/xmrig/xmrig"
tar -czpf "gupaxx-${NEW_VER}-macos-x64-standalone.tar.gz" "gupaxx-$NEW_VER-macos-x64-standalone" --owner=lm --group=lm ; check "tar macos-x64-standalone"
# Remove dir
rm -r "gupaxx-$NEW_VER-macos-x64-standalone"; check "rm macos-x64 dir"

# ARM
# Tar macOS Bundle
title "Tar macOS-arm64"
mv macos-arm64 "gupaxx-$NEW_VER-macos-arm64-bundle"; check "macos-arm64 -> gupaxx-$NEW_VER-macos-arm64-bundle"
tar -czpf "gupaxx-${NEW_VER}-macos-arm64-bundle.tar.gz" "gupaxx-$NEW_VER-macos-arm64-bundle" --owner=lm --group=lm ; check "tar macos-arm64-bundle"
# Tar macOS Standalone
mv "gupaxx-$NEW_VER-macos-arm64-bundle" "gupaxx-$NEW_VER-macos-arm64-standalone"; check "gupaxx-$NEW_VER-macos-arm64-bundle -> gupaxx-$NEW_VER-macos-arm64-standalone"
rm -r "gupaxx-$NEW_VER-macos-arm64-standalone/Gupaxx.app/Contents/MacOS/p2pool"; check "rm gupaxx-$NEW_VER-macos-arm64-standalone/Gupaxx.app/Contents/MacOS/p2pool"
rm -r "gupaxx-$NEW_VER-macos-arm64-standalone/Gupaxx.app/Contents/MacOS/xmrig"; check "rm gupaxx-$NEW_VER-macos-arm64-standalone/Gupaxx.app/Contents/MacOS/xmrig/xmrig"
tar -czpf "gupaxx-${NEW_VER}-macos-arm64-standalone.tar.gz" "gupaxx-$NEW_VER-macos-arm64-standalone" --owner=lm --group=lm ; check "tar macos-arm64-standalone"
# Remove dir
rm -r "gupaxx-$NEW_VER-macos-arm64-standalone"; check "rm macos dir"

# Zip Windows Bundle
title "Zip Windows"
mv windows "gupaxx-$NEW_VER-windows-x64-bundle"; check "windows -> gupaxx-$NEW_VER-windows-x64-bundle"
zip -qr "gupaxx-${NEW_VER}-windows-x64-bundle.zip" "gupaxx-$NEW_VER-windows-x64-bundle"; check "zip windows-bundle"
# Zip Windows Standalone
mv "gupaxx-$NEW_VER-windows-x64-bundle" "gupaxx-$NEW_VER-windows-x64-standalone"; check "gupaxx-$NEW_VER-windows-x64-bundle -> gupaxx-$NEW_VER-windows-x64-standalone"
rm -r "gupaxx-$NEW_VER-windows-x64-standalone/P2Pool"; check "rm gupaxx-$NEW_VER-windows-x64-standalone/p2pool"
rm -r "gupaxx-$NEW_VER-windows-x64-standalone/XMRig"; check "rm gupaxx-$NEW_VER-windows-x64-standalone/xmrig"
zip -qr "gupaxx-${NEW_VER}-windows-x64-standalone.zip" "gupaxx-$NEW_VER-windows-x64-standalone"; check "zip windows-standalone"
# Remove dir
rm -r "gupaxx-$NEW_VER-windows-x64-standalone"; check "rm windows dir"

# SHA256SUMS + Sign
title "Hash + Sign"
SHA256SUMS=$(sha256sum gupaxx* | gpg --clearsign --local-user 8EFFE4A8C0FD4B6D21C3AAB2EC6E5BB401C6362D); check "Hash + Sign"
echo "${SHA256SUMS}" > SHA256SUMS; check "Create SHA256SUMS file"
sha256sum -c SHA256SUMS; check "Verify SHA"
gpg --verify SHA256SUMS; check "Verify GPG"

# Get changelog + SHA256SUMS into clipboard
title "Clipboard"
clipboard() {
	grep -B999 -m1 "^$" CHANGELOG.md
	echo "## SHA256SUM & [PGP Signature](https://github.com/cyrix126/gupaxx/blob/main/pgp/cyrix126.asc)"
	echo '```'
	cat SHA256SUMS
	echo '```'
}
CHANGELOG=$(clipboard); check "Create changelog + sign"
echo "$CHANGELOG" | wl-copy  $clipboard
check "Changelog into clipboard"

# Reset timezone
title "End"
printf "\n%s\n" "package.sh ... Took [$((EPOCHSECONDS-START_TIME))] seconds ... OK!"
