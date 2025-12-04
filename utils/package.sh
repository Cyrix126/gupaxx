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
[[ -f linux/gupax ]]; check "linux/gupax"
[[ -f linux_b/gupax ]]; check "linux_b/gupax"
[[ -f linux_b/p2pool/p2pool ]]; check "linux_b/p2pool/p2pool"
[[ -f linux_b/xmrig/xmrig ]]; check "linux_b/xmrig/xmrig"
[[ -f linux_b/xmrig-proxy/xmrig-proxy ]]; check "linux_b/xmrig-proxy/xmrig-proxy"
[[ -f linux_b/node/monerod ]]; check "linux_b/node/monerod"
title "macOS-x64 folder check"
[[ -d macos-x64/Gupax.app ]]; check "macos-x64/Gupax.app"
[[ -d macos-x64_b/Gupax.app ]]; check "macos-x64_b/Gupax.app"
[[ -f macos-x64_b/Gupax.app/Contents/MacOS/p2pool/p2pool ]]; check "macos-x64_b/p2pool/p2pool"
[[ -f macos-x64_b/Gupax.app/Contents/MacOS/xmrig/xmrig ]]; check "macos-x64_b/xmrig/xmrig"
[[ -f macos-x64_b/Gupax.app/Contents/MacOS/xmrig-proxy/xmrig-proxy ]]; check "macos-x64_b/xmrig-proxy/xmrig-proxy"
[[ -f macos-x64_b/Gupax.app/Contents/MacOS/node/monerod ]]; check "macos-x64_b/node/monerod"
title "macOS-arm64 folder check"
[[ -d macos-arm64/Gupax.app ]]; check "macos-arm64/Gupax.app"
[[ -d macos-arm64_b/Gupax.app ]]; check "macos-arm64_b/Gupax.app"
[[ -f macos-arm64_b/Gupax.app/Contents/MacOS/p2pool/p2pool ]]; check "macos-arm64_b/p2pool/p2pool"
[[ -f macos-arm64_b/Gupax.app/Contents/MacOS/xmrig/xmrig ]]; check "macos-arm64_b/xmrig/xmrig"
## no macos-arm64 xmrig-proxy released todo
# [[ -f macos-arm64_b/Gupax.app/Contents/MacOS/xmrig-proxy/xmrig-proxy ]]; check "macos-arm64_b/xmrig-proxy/xmrig-proxy"
[[ -f macos-arm64_b/Gupax.app/Contents/MacOS/node/monerod ]]; check "macos-arm64_b/node/monerod"
title "Windows folder check"
[[ -f windows/Gupax.exe ]]; check "windows/Gupax.exe"
[[ -f windows_b/Gupax.exe ]]; check "windows_b/Gupax.exe"
[[ -f windows_b/P2Pool/p2pool.exe ]]; check "windows_b/P2Pool/p2pool.exe"
[[ -f windows_b/XMRig/xmrig.exe ]]; check "windows_b/XMRig/xmrig.exe"
[[ -f windows_b/XMRig/WinRing0x64.sys ]]; check "windows_b/XMRig/WinRing0x64.sys"
[[ -f windows_b/XMRig-Proxy/xmrig-proxy.exe ]]; check "windows_b/XMRig-Proxy/xmrig-proxy.exe"
[[ -f windows_b/Node/monerod.exe ]]; check "windows_b/Node/monerod.exe"

# Get random date for tar/zip
title "RNG Date"
RNG=$((EPOCHSECONDS-RANDOM*4)); check "RNG ... $RNG"
DATE=$(date -d @${RNG}); check "DATE ... $DATE"

# Tar Linux Bundle
title "Tar Linux"
# give execution permission
chmod +x linux/gupax
chmod +x linux_b/gupax
chmod +x linux_b/p2pool/p2pool
chmod +x linux_b/xmrig/xmrig
chmod +x linux_b/xmrig-proxy/xmrig-proxy
chmod +x linux_b/node/monerod
mv linux_b "gupax-$NEW_VER-linux-x64-bundle"; check "linux -> gupax-$NEW_VER-linux-x64-bundle"
tar -czpf "gupax-${NEW_VER}-linux-x64-bundle.tar.gz" "gupax-$NEW_VER-linux-x64-bundle" --owner=lm --group=lm ; check "tar linux-bundle"
# Tar Linux Standalone
mv linux "gupax-$NEW_VER-linux-x64-standalone" 
tar -czpf "gupax-${NEW_VER}-linux-x64-standalone.tar.gz" "gupax-$NEW_VER-linux-x64-standalone" --owner=lm --group=lm ; check "tar linux-standalone"
# Remove dir
rm -r "gupax-$NEW_VER-linux-x64-standalone"; check "rm linux dir"
rm -r "gupax-$NEW_VER-linux-x64-bundle"; check "rm linux_b dir"

# x64
# Tar macOS Bundle
title "Tar macOS-x64"
mv macos-x64_b "gupax-$NEW_VER-macos-x64-bundle"; check "macos-x64_b -> gupax-$NEW_VER-macos-x64-bundle"
tar -czpf "gupax-${NEW_VER}-macos-x64-bundle.tar.gz" "gupax-$NEW_VER-macos-x64-bundle" --owner=lm --group=lm ; check "tar macos-bundle"
# Tar macOS Standalone
mv macos-x64 "gupax-$NEW_VER-macos-x64-standalone"; check "macos-x64 -> gupax-$NEW_VER-macos-x64-standalone"
tar -czpf "gupax-${NEW_VER}-macos-x64-standalone.tar.gz" "gupax-$NEW_VER-macos-x64-standalone" --owner=lm --group=lm ; check "tar macos-x64-standalone"
# Remove dir
rm -r "gupax-$NEW_VER-macos-x64-standalone"; check "rm macos-x64 dir"
rm -r "gupax-$NEW_VER-macos-x64-bundle"; check "rm macos-x64_b dir"

# ARM
# Tar macOS Bundle
title "Tar macOS-arm64"
mv macos-arm64_b "gupax-$NEW_VER-macos-arm64-bundle"; check "macos-arm64_b -> gupax-$NEW_VER-macos-arm64-bundle"
tar -czpf "gupax-${NEW_VER}-macos-arm64-bundle.tar.gz" "gupax-$NEW_VER-macos-arm64-bundle" --owner=lm --group=lm ; check "tar macos-bundle"
# Tar macOS Standalone
mv macos-arm64 "gupax-$NEW_VER-macos-arm64-standalone"; check "macos-arm64 -> gupax-$NEW_VER-macos-arm64-standalone"
tar -czpf "gupax-${NEW_VER}-macos-arm64-standalone.tar.gz" "gupax-$NEW_VER-macos-arm64-standalone" --owner=lm --group=lm ; check "tar macos-arm64-standalone"
# Remove dir
rm -r "gupax-$NEW_VER-macos-arm64-standalone"; check "rm macos-arm64 dir"
rm -r "gupax-$NEW_VER-macos-arm64-bundle"; check "rm macos-arm64_b dir"

# Zip Windows Bundle
title "Zip Windows"
mv windows_b "gupax-$NEW_VER-windows-x64-bundle"; check "windows_b -> gupax-$NEW_VER-windows-x64-bundle"
zip -qr "gupax-${NEW_VER}-windows-x64-bundle.zip" "gupax-$NEW_VER-windows-x64-bundle"; check "zip windows-bundle"
# Zip Windows Standalone
mv windows "gupax-$NEW_VER-windows-x64-standalone"; check "windows -> gupax-$NEW_VER-windows-x64-standalone"
zip -qr "gupax-${NEW_VER}-windows-x64-standalone.zip" "gupax-$NEW_VER-windows-x64-standalone"; check "zip windows-standalone"
# Remove dir
rm -r "gupax-$NEW_VER-windows-x64-standalone"; check "rm windows dir"
rm -r "gupax-$NEW_VER-windows-x64-bundle"; check "rm windows_b dir"

# SHA256SUMS + Sign
title "Hash + Sign"
SHA256SUMS=$(sha256sum gupax* | gpg --clearsign --local-user 8EFFE4A8C0FD4B6D21C3AAB2EC6E5BB401C6362D); check "Hash + Sign"
echo "${SHA256SUMS}" > SHA256SUMS; check "Create SHA256SUMS file"
sha256sum -c SHA256SUMS; check "Verify SHA"
gpg --verify SHA256SUMS; check "Verify GPG"

# Get changelog + SHA256SUMS into clipboard
title "Clipboard"
clipboard() {
	grep -B999 -m1 "^$" CHANGELOG.md
	echo "## SHA256SUM & [PGP Signature](https://github.com/cyrix126/gupax/blob/main/pgp/cyrix126.asc)"
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
