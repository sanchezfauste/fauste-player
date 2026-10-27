# Mobile Plan 6 — Android Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A release can carry a signed Android APK and AAB (mobile spec
§7), built by the release workflow once the maintainer adds a signing
key, and the app loses its last interim parts: a real `versionCode`, its
own notification icon and links that open.

**Architecture:** Gradle signs release builds with a key passed through
environment variables (`FAUSTE_ANDROID_KEYSTORE`,
`FAUSTE_ANDROID_KEYSTORE_PASSWORD`, `FAUSTE_ANDROID_KEY_ALIAS`,
`FAUSTE_ANDROID_KEY_PASSWORD`) and falls back to the debug key without
them. `scripts/package/android.sh release` derives the `versionCode` from
the workspace version and builds the APK and the AAB. `release-build.yml`
gains an `android` job: a dry run on packaging pull requests, and on a
release it uploads the APK and AAB only when the `ANDROID_KEYSTORE*`
secrets exist (a debug-signed APK is never published: Android would
refuse to update it with a later signed one). `AndroidPlatform::open_url`
calls `MainActivity.openUrl` through JNI, which starts a view intent.

**Tech Stack:** Gradle Kotlin DSL, bash, GitHub Actions, `jni` 0.21.

**Spec:** `docs/superpowers/specs/2026-10-25-mobile-design.md` (§7, §8).

## Global Constraints

- Nothing is published without the maintainer's key; no secret is
  created here.
- `unsafe` only at fp-mobile's JNI call, with a `// SAFETY:` comment.
- The desktop packages and workflows are unchanged.

## Review Focus

- A version with a pre-release suffix (`0.13.0-rc.1`) still gives a valid,
  increasing `versionCode`.
- The release job uploads nothing without the secrets, and never prints
  them; the keystore file is removed after the build.
- `open_url` from a helper thread: the activity reference may be gone
  (between activities); it degrades to an error, never a crash.

---

### Task 1: Release signing and the version code

**Files:** `mobile/android/app/build.gradle.kts`, `scripts/package/android.sh`

- [ ] **Step 1:** `android.sh`: `version_code` = major × 1 000 000 + minor
  × 1 000 + patch (pre-release suffix dropped); `--self-test` checks it for
  `0.12.0`, `1.2.3`, `0.13.0-rc.1`. Run the self-test: FAIL, then PASS.
- [ ] **Step 2:** Gradle: `versionCode` from `-PversionCode`; a `release`
  signing config from the environment when all four variables are set,
  else the debug key with a warning. `release` builds `assembleRelease`
  and `bundleRelease`.
- [ ] **Step 3:** Build `release` locally without a key (debug-signed),
  then with a throwaway keystore (`apksigner verify --print-certs` shows
  it). Commit `build(mobile): sign Android releases with a release key`.

### Task 2: The release workflow

**Files:** `.github/workflows/release-build.yml`, `scripts/package/android.sh`

- [ ] **Step 1:** An `android` job (Ubuntu, JDK 17, the Android targets,
  cargo-ndk): decodes `ANDROID_KEYSTORE` (base64) into a temporary file
  when set, builds `release`, renames to
  `fauste-player-<version>-android.apk` / `.aab` with `.sha256`, uploads
  only on a release with the key; deletes the keystore in an `always()`
  step.
- [ ] **Step 2:** `actionlint` (if installed) and the `android.sh`
  self-test. Commit `ci: build the Android release in release-build`.

### Task 3: Notification icon and links

**Files:** `mobile/android/app/src/main/res/drawable/ic_notification.xml`,
`PlaybackService.kt`, `MainActivity.kt`, `crates/fp-mobile/src/{android.rs,platform.rs,jni.rs}`

- [ ] **Step 1:** A white vector icon (the app's play mark) as the small
  icon.
- [ ] **Step 2:** `MainActivity.openUrl(String)` starts `ACTION_VIEW`
  (`FLAG_ACTIVITY_NEW_TASK`, errors logged). `android_main` stores the
  VM and activity pointers; `open_url` attaches the thread and calls it;
  without an activity it returns an error.
- [ ] **Step 3:** Android clippy, debug APK. Commit
  `feat(mobile): own notification icon and links that open`.

### Task 4: Docs

- [ ] `docs/user/android.md` (+ SUMMARY): install, permissions, the
  phone screens, the media browser, outputs, what stays desktop-only.
- [ ] README: the Android row and roadmap row 9.
- [ ] `docs/technical/release-process.md`: the Android secrets and how to
  make the key; `docs/technical/mobile.md`: drop the interim parts done.
- [ ] Commit `docs: the Android app in the guide and the release process`.
