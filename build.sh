#!/usr/bin/env bash
# Build the plugin binary, package it into a release zip, and optionally
# install it to the local OpenDeck plugins directory (--install).
set -e

# figure out where this file is located even if it is being run from another location
# or as a symlink
# shellcheck disable=SC2296
SOURCE="${BASH_SOURCE[0]:-${(%):-%x}}"
while [ -h "$SOURCE" ]; do # resolve $SOURCE until the file is no longer a symlink
  DIR="$( cd -P "$( dirname "$SOURCE" )" >/dev/null && pwd )"
  SOURCE="$(readlink "$SOURCE")"
  [[ $SOURCE != /* ]] && SOURCE="$DIR/$SOURCE" # if $SOURCE was a relative symlink, we need to resolve it relative to the path where the symlink file was located
done
THIS_DIR="$( cd -P "$( dirname "$SOURCE" )" >/dev/null && pwd )"

TARGET="x86_64-unknown-linux-gnu"
PLUGIN_DIR="${HOME}/.config/opendeck/plugins/linux-gpu-monitor.sdPlugin"

cargo build --release --target "${TARGET}"
mkdir -p "${THIS_DIR}/assets/${TARGET}/bin"
cp "${THIS_DIR}/target/${TARGET}/release/linux-gpu-monitor" "${THIS_DIR}/assets/${TARGET}/bin/"

# Create release zip
cd "${THIS_DIR}"
rm -rf linux-gpu-monitor.sdPlugin
cp -r assets linux-gpu-monitor.sdPlugin
zip -r linux-gpu-monitor.zip linux-gpu-monitor.sdPlugin
rm -rf linux-gpu-monitor.sdPlugin
echo "Created linux-gpu-monitor.zip"

# Install to local OpenDeck plugins directory
if [ "$1" = "--install" ]; then
  rm -rf "${PLUGIN_DIR}"
  cp -r "${THIS_DIR}/assets" "${PLUGIN_DIR}"
  echo "Installed to ${PLUGIN_DIR}"
  echo "Reload the plugin in OpenDeck to apply changes."
fi
