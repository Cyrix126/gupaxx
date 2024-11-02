#!/usr/bin/env bash

# Sets up a packaging environment in [/tmp]

# Make sure we're in the [gupaxx/utils] directory
set -ex
[[ $PWD = */gupaxx ]]

# Make sure the folder doesn't already exist
GIT_COMMIT=$(cat .git/refs/heads/main)
FOLDER="gupaxx_${GIT_COMMIT}"
[[ ! -e /tmp/${FOLDER} ]]

mkdir /tmp/${FOLDER}
cp -r utils/* /tmp/${FOLDER}/
cp CHANGELOG.md /tmp/${FOLDER}/skel/

## Download XMRig Binaries
# download xmrig into directory linux
wget https://github.com/xmrig/xmrig/releases/download/v6.21.1/xmrig-6.21.1-linux-static-x64.tar.gz
tar -xf xmrig-6.21.1-linux-static-x64.tar.gz
mv xmrig-6.21.1/xmrig /tmp/${FOLDER}/skel/linux_b/xmrig/xmrig
rm -r xmrig-6.21.1
rm xmrig-6.21.1-linux-static-x64.tar.gz 
# download xmrig into directory macos-arm64
wget https://github.com/xmrig/xmrig/releases/download/v6.21.1/xmrig-6.21.1-macos-arm64.tar.gz
tar -xf xmrig-6.21.1-macos-arm64.tar.gz
mv xmrig-6.21.1/xmrig /tmp/${FOLDER}/skel/macos-arm64_b/Gupaxx.app/Contents/MacOS/xmrig/xmrig
rm -r xmrig-6.21.1
rm xmrig-6.21.1-macos-arm64.tar.gz
# download xmrig into directory macos-x64
wget https://github.com/xmrig/xmrig/releases/download/v6.21.1/xmrig-6.21.1-macos-x64.tar.gz
tar -xf xmrig-6.21.1-macos-x64.tar.gz
mv xmrig-6.21.1/xmrig /tmp/${FOLDER}/skel/macos-x64_b/Gupaxx.app/Contents/MacOS/xmrig/xmrig
rm -r xmrig-6.21.1
rm xmrig-6.21.1-macos-x64.tar.gz
# download xmrig into directory windows
wget https://github.com/xmrig/xmrig/releases/download/v6.21.1/xmrig-6.21.1-msvc-win64.zip
unzip xmrig-6.21.1-msvc-win64.zip
mv xmrig-6.21.1/xmrig.exe /tmp/${FOLDER}/skel/windows_b/XMRig/xmrig.exe
mv xmrig-6.21.1/WinRing0x64.sys /tmp/${FOLDER}/skel/windows_b/XMRig/WinRing0x64.sys
rm -r xmrig-6.21.1
rm xmrig-6.21.1-msvc-win64.zip

## Download XMRig-Proxy Binaries
wget https://github.com/xmrig/xmrig-proxy/releases/download/v6.21.1/xmrig-proxy-6.21.1-jammy-x64.tar.gz
tar -xf xmrig-proxy-6.21.1-jammy-x64.tar.gz
mv xmrig-proxy-6.21.1/xmrig-proxy /tmp/${FOLDER}/skel/linux_b/xmrig-proxy/xmrig-proxy
rm -r xmrig-proxy-6.21.1
rm xmrig-proxy-6.21.1-jammy-x64.tar.gz 
## no release for arm64 mac-osx xmrig-proxy, todo make CI build it.
# download xmrig into directory macos-arm64
# wget https://github.com/xmrig/xmrig-proxy/releases/download/v6.21.1/xmrig-proxy-6.21.1-macos-arm64.tar.gz
# tar -xf xmrig-proxy-6.21.1-macos-arm64.tar.gz
# mv xmrig-proxy-6.21.1/xmrig-proxy /tmp/${FOLDER}/skel/macos-arm64_b/Gupaxx.app/Contents/MacOS/xmrig-proxy/xmrig-proxy
# rm -r xmrig-proxy-6.21.1
# rm xmrig-proxy-6.21.1-macos-arm64.tar.gz
# download xmrig into directory macos-x64
wget https://github.com/xmrig/xmrig-proxy/releases/download/v6.21.1/xmrig-proxy-6.21.1-macos-x64.tar.gz
tar -xf xmrig-proxy-6.21.1-macos-x64.tar.gz
mv xmrig-proxy-6.21.1/xmrig-proxy /tmp/${FOLDER}/skel/macos-x64_b/Gupaxx.app/Contents/MacOS/xmrig-proxy/xmrig-proxy
rm -r xmrig-proxy-6.21.1
rm xmrig-proxy-6.21.1-macos-x64.tar.gz
# download xmrig into directory windows
wget https://github.com/xmrig/xmrig-proxy/releases/download/v6.21.1/xmrig-proxy-6.21.1-msvc-win64.zip
unzip xmrig-proxy-6.21.1-msvc-win64.zip
mv xmrig-proxy-6.21.1/xmrig-proxy.exe /tmp/${FOLDER}/skel/windows_b/XMRig-Proxy/xmrig-proxy.exe
rm -r xmrig-proxy-6.21.1
rm xmrig-proxy-6.21.1-msvc-win64.zip

## Download P2Pool Binaries
# download p2pool into directory linux
wget https://github.com/SChernykh/p2pool/releases/download/v4.2/p2pool-v4.2-linux-x64.tar.gz
tar -xf p2pool-v4.2-linux-x64.tar.gz
mv p2pool-v4.2-linux-x64/p2pool /tmp/${FOLDER}/skel/linux_b/p2pool/p2pool
rm -r p2pool-v4.2-linux-x64
rm p2pool-v4.2-linux-x64.tar.gz
# download p2pool into directory macos-arm64
wget https://github.com/SChernykh/p2pool/releases/download/v4.2/p2pool-v4.2-macos-aarch64.tar.gz
tar -xf p2pool-v4.2-macos-aarch64.tar.gz
mv p2pool-v4.2-macos-aarch64/p2pool /tmp/${FOLDER}/skel/macos-arm64_b/Gupaxx.app/Contents/MacOS/p2pool/p2pool
rm -r p2pool-v4.2-macos-aarch64
rm p2pool-v4.2-macos-aarch64.tar.gz
# download p2pool into directory macos-x64
wget https://github.com/SChernykh/p2pool/releases/download/v4.2/p2pool-v4.2-macos-x64.tar.gz
tar -xf p2pool-v4.2-macos-x64.tar.gz
mv p2pool-v4.2-macos-x64/p2pool /tmp/${FOLDER}/skel/macos-x64_b/Gupaxx.app/Contents/MacOS/p2pool/p2pool
rm -r p2pool-v4.2-macos-x64
rm p2pool-v4.2-macos-x64.tar.gz
# download p2pool into directory windows
wget https://github.com/SChernykh/p2pool/releases/download/v4.2/p2pool-v4.2-windows-x64.zip
unzip p2pool-v4.2-windows-x64.zip
mv p2pool-v4.2-windows-x64/p2pool.exe /tmp/${FOLDER}/skel/windows_b/P2Pool/p2pool.exe
rm -r p2pool-v4.2-windows-x64
rm p2pool-v4.2-windows-x64.zip

## Download Monero Binaries
# download monero into directory linux
wget https://downloads.getmonero.org/cli/monero-linux-x64-v0.18.3.4.tar.bz2
tar -xf monero-linux-x64-v0.18.3.4.tar.bz2
mv monero-x86_64-linux-gnu-v0.18.3.4/monerod /tmp/${FOLDER}/skel/linux_b/node/monerod
rm -r monero-x86_64-linux-gnu-v0.18.3.4
rm monero-linux-x64-v0.18.3.4.tar.bz2
# download monero into directory macos-arm64
wget https://downloads.getmonero.org/cli/monero-mac-armv8-v0.18.3.4.tar.bz2
tar -xf monero-mac-armv8-v0.18.3.4.tar.bz2
mv monero-aarch64-apple-darwin11-v0.18.3.4/monerod /tmp/${FOLDER}/skel/macos-arm64_b/Gupaxx.app/Contents/MacOS/node/monerod
rm -r monero-aarch64-apple-darwin11-v0.18.3.4
rm monero-mac-armv8-v0.18.3.4.tar.bz2
# download monero into directory macos-x64
wget https://downloads.getmonero.org/cli/monero-mac-x64-v0.18.3.4.tar.bz2
tar -xf monero-mac-x64-v0.18.3.4.tar.bz2
mv monero-x86_64-apple-darwin11-v0.18.3.4/monerod /tmp/${FOLDER}/skel/macos-x64_b/Gupaxx.app/Contents/MacOS/node/monerod
rm -r monero-x86_64-apple-darwin11-v0.18.3.4
rm monero-mac-x64-v0.18.3.4.tar.bz2
# download monero into directory windows
wget https://downloads.getmonero.org/cli/monero-win-x64-v0.18.3.4.zip
unzip monero-win-x64-v0.18.3.4.zip
mv monero-x86_64-w64-mingw32-v0.18.3.4/monerod.exe /tmp/${FOLDER}/skel/windows_b/Node/monerod.exe
rm -r monero-x86_64-w64-mingw32-v0.18.3.4
rm monero-win-x64-v0.18.3.4.zip

set +ex

echo
ls --color=always /tmp/${FOLDER}
echo "/tmp/${FOLDER} ... OK"
