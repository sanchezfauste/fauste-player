#!/usr/bin/env bash
# Builds the Android app: the Rust library for arm64-v8a and x86_64 into
# mobile/android/app/src/main/jniLibs, then the APK with Gradle.
#
#   scripts/package/android.sh <debug|release>
#
# Needs the Android SDK (ANDROID_HOME, or sdk.dir in
# mobile/android/local.properties) with NDK r27 (ANDROID_NDK_HOME, or the
# newest ndk/27.* under the SDK), the Rust targets aarch64-linux-android and
# x86_64-linux-android, cargo-ndk and a JDK 17 or newer. The APK lands in
# mobile/android/app/build/outputs/apk/<profile>/. Judge audio only on
# release builds: debug builds drop out (mobile spec §11).
set -euo pipefail

profile="${1:?usage: android.sh <debug|release>}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
app="${root}/mobile/android"
version="$(sed -n 's/^version = "\(.*\)"$/\1/p' "${root}/Cargo.toml" | head -n1)"

case "${profile}" in
  debug) cargo_profile=(); gradle_task=assembleDebug ;;
  release) cargo_profile=(--release); gradle_task=assembleRelease ;;
  *) echo "android.sh: unknown profile ${profile} (debug or release)" >&2; exit 2 ;;
esac

sdk="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-${HOME}/Android/Sdk}}"
if [[ ! -f "${app}/local.properties" ]]; then
  export ANDROID_HOME="${sdk}"
fi
if [[ -z "${ANDROID_NDK_HOME:-}" ]]; then
  ndk="$(ls -d "${sdk}"/ndk/27.* 2>/dev/null | sort -V | tail -n1 || true)"
  if [[ -z "${ndk}" ]]; then
    echo "android.sh: no NDK r27 found; set ANDROID_NDK_HOME" >&2
    exit 1
  fi
  export ANDROID_NDK_HOME="${ndk}"
fi

# cargo-ndk leaves a library already in jniLibs in place (a debug build's
# would ship in a release APK): start from an empty folder.
rm -rf "${app}/app/src/main/jniLibs"

# API 29: midir's libamidi and AAudio's newer calls need it (spec §11).
(cd "${root}" && cargo ndk -P 29 -t arm64-v8a -t x86_64 \
  -o "${app}/app/src/main/jniLibs" build "${cargo_profile[@]}" -p fp-mobile)

(cd "${app}" && ./gradlew --no-daemon -q "${gradle_task}" -PversionName="${version}")
find "${app}/app/build/outputs/apk/${profile}" -name '*.apk' -print
