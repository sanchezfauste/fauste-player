#!/usr/bin/env bash
# Builds the snap in CI (the stores workflow), telling a Snap Store outage
# from a real failure: snapd, LXD and snapcraft come from the store, and so
# do the base and the build snaps, and the store is often unreachable.
#
#   scripts/package/snap-build.sh setup              snapd, LXD and snapcraft
#   scripts/package/snap-build.sh build              snapcraft pack in LXD
#   scripts/package/snap-build.sh store-error <log>  exit 0 when the end of
#                                                    the log is a store error
#
# setup retries an install while the store is unreachable
# (SNAP_STORE_ATTEMPTS times, default 6, waiting SNAP_STORE_DELAY seconds,
# default 30, doubled after each attempt; an install that takes more than
# SNAP_STORE_TIMEOUT seconds, default 600, counts as a stalled store). setup
# and build exit 75 when they fail because of the store, and with another
# non-zero status otherwise.
set -euo pipefail

OUTAGE=75
attempts="${SNAP_STORE_ATTEMPTS:-6}"
delay="${SNAP_STORE_DELAY:-30}"
snap_bin="${SNAP_BIN_DIR:-/snap/bin}"
install_timeout="${SNAP_STORE_TIMEOUT:-600}"

# The errors of snapd and snapcraft when the store or Canonical's image
# server cannot be reached.
store_error() {
  tail -n 40 "$1" | grep -qiE \
    'unable to contact snap store|api\.snapcraft\.io|Client\.Timeout exceeded|store is unavailable|cloud-images\.ubuntu\.com|images\.lxd\.canonical\.com'
}

# Runs a command with its output on stderr (the job's log) and copied into
# the file $1, and prints its exit status.
run() {
  local log="$1" status
  shift
  status="$(mktemp)"
  { s=0; "$@" 2>&1 || s=$?; echo "${s}" > "${status}"; } | tee "${log}" >&2
  cat "${status}"
  rm -f "${status}"
}

# `sudo snap install <args>`, retried while the store is unreachable.
install() {
  local log status try=1 wait="${delay}"
  log="$(mktemp)"
  while :; do
    echo "snap install $* (attempt ${try} of ${attempts})"
    status="$(run "${log}" timeout "${install_timeout}" sudo snap install "$@")"
    if [ "${status}" = 0 ]; then
      rm -f "${log}"
      return 0
    fi
    # 124: the install hung, which is how a stalled store looks.
    if [ "${status}" != 124 ] && ! store_error "${log}"; then
      rm -f "${log}"
      return 1
    fi
    if [ "${try}" -ge "${attempts}" ]; then
      rm -f "${log}"
      echo "the Snap Store is unreachable after ${attempts} attempts" >&2
      return "${OUTAGE}"
    fi
    echo "the Snap Store is unreachable; trying again in ${wait} s" >&2
    sleep "${wait}"
    try=$((try + 1))
    wait=$((wait * 2))
  done
}

setup() {
  if [ ! -x /usr/bin/snap ]; then
    sudo apt-get update -q
    sudo apt-get install -qy snapd
  fi
  # The runners' / is not owned by root, which trips up snap-confine.
  if [ "$(stat -c %u:%g /)" != 0:0 ]; then
    sudo chown root:root /
  fi
  sudo groupadd --force --system lxd
  sudo usermod --append --groups lxd "$(id -un)"
  # Installed snaps are used as they are: a refresh needs the store too.
  if [ ! -x "${snap_bin}/lxd" ]; then
    install lxd || return
  fi
  sudo lxd init --auto
  # Docker's firewall rules block the LXD bridge.
  sudo iptables -P FORWARD ACCEPT
  if [ ! -x "${snap_bin}/snapcraft" ]; then
    install snapcraft --classic || return
  fi
}

build() {
  local log status
  log="$(mktemp)"
  # A fresh login through sudo, so the new lxd group applies.
  status="$(run "${log}" sudo -u "$(id -un)" -E env SNAPCRAFT_BUILD_ENVIRONMENT=lxd \
    snapcraft pack)"
  if [ "${status}" != 0 ] && store_error "${log}"; then
    status="${OUTAGE}"
  fi
  rm -f "${log}"
  return "${status}"
}

case "${1:-}" in
  setup) setup ;;
  build) build ;;
  store-error) store_error "${2:?usage: snap-build.sh store-error <log>}" ;;
  *)
    echo "usage: snap-build.sh setup | build | store-error <log>" >&2
    exit 2
    ;;
esac
