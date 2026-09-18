#!/usr/bin/env bash
set -euo pipefail

target_triple="armv7-unknown-linux-gnueabihf"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target_dir="${CARGO_TARGET_DIR:-${repo_root}/target}"

for tool in rustup cargo arm-linux-gnueabihf-gcc arm-linux-gnueabihf-ar readelf size sha256sum; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "missing required tool: ${tool}" >&2
    exit 2
  fi
done

rustup target add "${target_triple}"

export CC_armv7_unknown_linux_gnueabihf="$(command -v arm-linux-gnueabihf-gcc)"
export AR_armv7_unknown_linux_gnueabihf="$(command -v arm-linux-gnueabihf-ar)"
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="${CC_armv7_unknown_linux_gnueabihf}"
export CARGO_TARGET_DIR="${target_dir}"

cd "${repo_root}"
cargo build -p ailive-gun-spirit-host --release --locked --target "${target_triple}"

artifact="${target_dir}/${target_triple}/release/ailive-gun-spirit-host"
test -f "${artifact}"

readelf -h "${artifact}" | grep -E 'Class:|Machine:|Flags:'
readelf -d "${artifact}" | grep NEEDED || true
size "${artifact}"
sha256sum "${artifact}"

readelf -h "${artifact}" | grep -q 'Class:.*ELF32'
readelf -h "${artifact}" | grep -q 'Machine:.*ARM'
readelf -h "${artifact}" | grep -q 'hard-float ABI'
