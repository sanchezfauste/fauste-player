#!/usr/bin/env bash
# Checks that a Windows executable is a GUI program (PE Subsystem 2), so
# that it opens no console window (feedback 2 spec O36).
#
#   scripts/check-windows-gui.sh <file.exe>
#   scripts/check-windows-gui.sh --self-test
set -euo pipefail

# Prints the PE Subsystem field: 2 is GUI, 3 is console.
subsystem_of() {
  local exe="$1" pe
  pe="$(od -An -tu4 -j60 -N4 "${exe}" | tr -d ' ')"
  od -An -tu2 -j"$((pe + 92))" -N2 "${exe}" | tr -d ' '
}

# A file with a PE header whose Subsystem is $1.
fake_pe() {
  local file="$1" subsystem="$2"
  : > "${file}"
  printf 'MZ' >> "${file}"
  head -c 58 /dev/zero >> "${file}"
  printf '\x80\x00\x00\x00' >> "${file}"          # e_lfanew = 128 at offset 60
  head -c 64 /dev/zero >> "${file}"               # up to offset 128
  head -c 92 /dev/zero >> "${file}"               # PE signature + headers
  printf "\\x0$((subsystem))\\x00" >> "${file}"   # Subsystem at pe + 92
}

if [ "${1:-}" = "--self-test" ]; then
  tmp="$(mktemp)"
  trap 'rm -f "${tmp}"' EXIT
  fake_pe "${tmp}" 2
  [ "$(subsystem_of "${tmp}")" = "2" ] || { echo "self-test: GUI not recognised" >&2; exit 1; }
  fake_pe "${tmp}" 3
  [ "$(subsystem_of "${tmp}")" = "3" ] || { echo "self-test: console not recognised" >&2; exit 1; }
  echo "ok"
  exit 0
fi

exe="${1:?usage: check-windows-gui.sh <file.exe> | --self-test}"
found="$(subsystem_of "${exe}")"
if [ "${found}" != "2" ]; then
  echo "${exe} is not a GUI program (PE subsystem ${found}); it would open a console window" >&2
  exit 1
fi
