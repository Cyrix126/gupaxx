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

set +ex

echo
ls --color=always /tmp/${FOLDER}
echo "/tmp/${FOLDER} ... OK"
