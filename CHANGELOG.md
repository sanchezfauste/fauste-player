# Changelog

## [0.2.0](https://github.com/sanchezfauste/fauste-player/compare/v0.1.0...v0.2.0) (2026-09-27)


### Features

* **analysis:** read the INTRO tag as the intro end ([b2772ab](https://github.com/sanchezfauste/fauste-player/commit/b2772abea80da393fa900b32e85690c670283617))
* **app:** application id, icon, desktop entry and playlist arguments ([184399e](https://github.com/sanchezfauste/fauste-player/commit/184399e8b2fff1cc9950987af4cb3dedcab4b71d))
* **backends:** Core Audio hog mode for bit-perfect devices on macOS ([cb19d50](https://github.com/sanchezfauste/fauste-player/commit/cb19d503a2bd10fb41e54d5b5dbb7440e526c5c1))
* **backends:** exclusive access, real sample formats and exclusive-capable devices ([ac5036e](https://github.com/sanchezfauste/fauste-player/commit/ac5036eeeb5adb0c19275998204850722368ef89))
* **backends:** offer every audio system of the platform ([d29f5af](https://github.com/sanchezfauste/fauste-player/commit/d29f5af270fcc32e2c33ee7692bcd576fef88ba9))
* **backends:** WASAPI exclusive mode for bit-perfect devices on Windows ([cd3a7a8](https://github.com/sanchezfauste/fauste-player/commit/cd3a7a874c566fd5507de68f13f5afe0d5c58e01))
* **engine:** a VU that moves like IEC 60268-17 ([18af556](https://github.com/sanchezfauste/fauste-player/commit/18af5564e7317f03a6483fa3f8ce320dd945a53e))
* **engine:** bit-perfect buses follow the file rate while idle ([6cd5387](https://github.com/sanchezfauste/fauste-player/commit/6cd5387f2a053350d43442e25f70f082ae130a72))
* **engine:** bound worker sources and loop them gaplessly ([c41ec6a](https://github.com/sanchezfauste/fauste-player/commit/c41ec6afd48b404a0d2aec04b6d5f4e05444fb2e))
* **engine:** loudness that meets EBU Tech 3341 as a live meter ([44e2e5b](https://github.com/sanchezfauste/fauste-player/commit/44e2e5bf075d65fb314844dbcfdeaa936e52d375))
* **engine:** measure what each source puts out for the level meters ([c9df38b](https://github.com/sanchezfauste/fauste-player/commit/c9df38bcdc9548b8bcc4810cb17a3c200c98390b))
* **engine:** play the cartwall on its own routes ([cd7e825](https://github.com/sanchezfauste/fauste-player/commit/cd7e8251ed5efa11b8a14bb1387de74904b96a8b))
* **engine:** programme meters that meet EBU Tech 3205 ([31d4c14](https://github.com/sanchezfauste/fauste-player/commit/31d4c14b2d918ea0afeb4ef99c232eeb89880ee7))
* **engine:** report when a player is bit-perfect, and prove the output is bit-exact ([50299c2](https://github.com/sanchezfauste/fauste-player/commit/50299c21b75077ba8f6918cd32869958b5c25cf4))
* **engine:** standard meter ballistics and EBU R128 loudness ([c4f5099](https://github.com/sanchezfauste/fauste-player/commit/c4f5099d60e2249afe977f6d918d6a3a913c1e1a))
* **engine:** the conductor runs each player's meter and publishes its reading ([a31cb0f](https://github.com/sanchezfauste/fauste-player/commit/a31cb0f4f48b69da42fcbb6238e5bb316973eeb3))
* **engine:** true peak with the ITU-R BS.1770-5 interpolator ([4478418](https://github.com/sanchezfauste/fauste-player/commit/4478418d55577f407b513e9c2e55077b731d5537))
* **model:** add cartwall pages, carts and rules C1–C10 ([c12a941](https://github.com/sanchezfauste/fauste-player/commit/c12a94162cf83fa001015dc5fbfddc4d3df63b5b))
* **model:** add configurable keyboard shortcuts ([cb724ed](https://github.com/sanchezfauste/fauste-player/commit/cb724edcaf8ff1d568861eb72fe45298fa9d7dd1))
* **model:** add manual marker commands ([b25218f](https://github.com/sanchezfauste/fauste-player/commit/b25218f547e7c4a8c60131f77c0d961aa179975a))
* **model:** configurable level meters ([e642051](https://github.com/sanchezfauste/fauste-player/commit/e642051e7e28f00ef351c20190c5874ae56fd53f))
* **model:** record each file's rate and sample size and pass it to the engine ([577fa28](https://github.com/sanchezfauste/fauste-player/commit/577fa288b34b2cf3331896cae82d126e746161a5))
* **store:** persist cart pages and the cartwall session ([d57d115](https://github.com/sanchezfauste/fauste-player/commit/d57d115ddbc2cab4f60df3ead69ca1f006b3c561))
* **store:** read M3U/M3U8/PLS and cart page files, write M3U8 ([039b0db](https://github.com/sanchezfauste/fauste-player/commit/039b0dbdbc5c865c634ada6872ad14532f771b14))
* **ui:** add Cartwall and Keyboard shortcuts settings, and a language selector ([a174e64](https://github.com/sanchezfauste/fauste-player/commit/a174e64821d8d35d1a3488aca9efa18ec74bed53))
* **ui:** add the cartwall strip ([6a30ccd](https://github.com/sanchezfauste/fauste-player/commit/6a30ccdbe37880dd3be766131b9b26292e11d220))
* **ui:** dispatch keys from the configured shortcuts ([6ba70f7](https://github.com/sanchezfauste/fauste-player/commit/6ba70f74e9884d48c057d7836c8e6f771bf20e4a))
* **ui:** edit markers on the waveform ([039af23](https://github.com/sanchezfauste/fauste-player/commit/039af23441e6926debbe434e370e3ccb1b0e8b55))
* **ui:** import M3U/M3U8/PLS playlists and export M3U8 ([7d4cc05](https://github.com/sanchezfauste/fauste-player/commit/7d4cc05151c7c309118c896981b3999d6288a888))
* **ui:** light the BP badge and choose bit-perfect devices in Settings ([3f85b0c](https://github.com/sanchezfauste/fauste-player/commit/3f85b0ce7b964fad2e84cf2414838cc4816336ea))
* **ui:** standard level meters and a Meters section in Settings ([855d532](https://github.com/sanchezfauste/fauste-player/commit/855d532228a7c4f052d1e68419d0a1ccfd690454))


### Bug Fixes

* address review of hardening, docs and release pipeline ([e675712](https://github.com/sanchezfauste/fauste-player/commit/e675712753be323e3271920884a7c004d479181c))
* address review of the Phase 2 engine and analysis ([405103e](https://github.com/sanchezfauste/fauste-player/commit/405103eb9f6fb66ac9373a244e31edb221e311a1))
* address review of the Phase 2 model and store ([6d87c9d](https://github.com/sanchezfauste/fauste-player/commit/6d87c9ddffea5893291fd68b6d68b2f8167e98ef))
* **app:** one instance per data folder, and packaging fixes from review ([21c2a49](https://github.com/sanchezfauste/fauste-player/commit/21c2a496547f3667e04a22f63cb635fad9852554))
* **backends:** harden the exclusive modes after review ([7b716df](https://github.com/sanchezfauste/fauste-player/commit/7b716df257591ec8c3ff63d968a500b3a558c154))
* **backends:** treat systems without output devices as unavailable and recover lost hosts ([ea1fdb4](https://github.com/sanchezfauste/fauste-player/commit/ea1fdb419875420331e7a96ab634ff3d8a283d28))
* close Phase 1 review minors and avoid duplicate on-air next ([aa44115](https://github.com/sanchezfauste/fauste-player/commit/aa44115c7b4f9060510f94c6e6794c980d54fa18))
* **engine:** a transition at the next frame to play has not happened yet ([e03d5ad](https://github.com/sanchezfauste/fauste-player/commit/e03d5ad13d0e4c60bb73372240043d468d374551))
* **engine:** make bit-perfect output work on real devices ([c78b56d](https://github.com/sanchezfauste/fauste-player/commit/c78b56d3e684dfdfa3d8549f7a1a12104029c6e6))
* **engine:** meter review fixes ([c4e1ed5](https://github.com/sanchezfauste/fauste-player/commit/c4e1ed593030758bdec7c90b60132fcf2c2c77f4))
* **engine:** meter review fixes for the standards work ([4e893d2](https://github.com/sanchezfauste/fauste-player/commit/4e893d238642ac0ba2b8c35390be18392a6e72cb))
* **model:** players are independent, even on the same playlist ([da2f331](https://github.com/sanchezfauste/fauste-player/commit/da2f331efc3bd3d030cd5f4cf7cef1b2b88972d5))
* **model:** players are independent, even on the same playlist ([315d865](https://github.com/sanchezfauste/fauste-player/commit/315d865f4b8d88101dc1e57c7c1c2c33fbe6d6c1))
* **store:** bump the playlists schema for per-player played marks ([4ae97d8](https://github.com/sanchezfauste/fauste-player/commit/4ae97d8bb96b66ada5b285043f85767f8cd23103))
* **ui:** address review of the Phase 2 interface ([850d39c](https://github.com/sanchezfauste/fauste-player/commit/850d39cf1855f1ea7da7e580ea2113b8ccc5b7d7))


### Code Refactoring

* **engine:** time every source in the frames of its own bus ([9394b44](https://github.com/sanchezfauste/fauste-player/commit/9394b4497663e013093b4c9ee2b9422df3a3be9c))


### Documentation

* add Phase 1 plan 5 (hardening, documentation, release) ([41eaca7](https://github.com/sanchezfauste/fauste-player/commit/41eaca74f6cc1ceb5f703e1d6c642ac566d94da8))
* add Phase 2 plan 2 (engine and analysis) ([a12b706](https://github.com/sanchezfauste/fauste-player/commit/a12b706d52b8b5e2b8953dda07c3fa8ef955d810))
* add README, user guide, technical docs and agent guides ([128e53b](https://github.com/sanchezfauste/fauste-player/commit/128e53bc4271c5a2d0542844d8ab402ed8f1df4a))
* bit-perfect output on Windows and macOS ([3d9f55b](https://github.com/sanchezfauste/fauste-player/commit/3d9f55b58eea425aa85b9b5c7c2228362c81c7d9))
* design and plan standard level meters ([f4b09fc](https://github.com/sanchezfauste/fauste-player/commit/f4b09fc4ae77a5c00f193955361f99b819156aec))
* design Phase 4 (bit-perfect output) and plan its portable core ([5eb2736](https://github.com/sanchezfauste/fauste-player/commit/5eb2736bd79e77f5ff44b8ad24393ad20da55e55))
* document bit-perfect output for users and developers ([fe7f583](https://github.com/sanchezfauste/fauste-player/commit/fe7f5838b465c138a111403480a63096c852d9e7))
* document carts.json, shortcuts, cartwall config and playlist files ([6a9ed59](https://github.com/sanchezfauste/fauste-player/commit/6a9ed59f71fabf016c02b90adcfd29c40a7a4c32))
* document the cartwall, shortcuts, marker editing and playlist files ([9f686bd](https://github.com/sanchezfauste/fauste-player/commit/9f686bd76e78f93dad173cba267b1bd16d8d2391))
* level meters for users and developers ([5e373e9](https://github.com/sanchezfauste/fauste-player/commit/5e373e961b05efd4ecbff4ea8af7b4093662905f))
* meters faithful to ITU-R BS.1770-5, EBU Tech 3341 and Tech 3205 ([81ba542](https://github.com/sanchezfauste/fauste-player/commit/81ba542560ee84288c67cc02f7ebbec8088ca005))
* plan Phase 3 (native audio systems through cpal hosts) ([e2b7808](https://github.com/sanchezfauste/fauste-player/commit/e2b78081753704133c07fae45ef907d3a65f04bc))
* plan Phase 4 plan 2 (WASAPI exclusive and Core Audio hog mode) ([a7b5df7](https://github.com/sanchezfauste/fauste-player/commit/a7b5df70ce221a7a0869af5af5406338cb91cc3d))
* plan Phase 5 (installable packages for every platform) ([b4ae88a](https://github.com/sanchezfauste/fauste-player/commit/b4ae88a0797df20c0b0b0c14bc51c80c953717f8))
* work through pull requests, with docs updated in every change ([7e92808](https://github.com/sanchezfauste/fauste-player/commit/7e928085a7a92b96298ec005f9608c8a8a320616))
* work through pull requests, with docs updated in every change ([c9a244b](https://github.com/sanchezfauste/fauste-player/commit/c9a244be8b9f8204dd30698c2d3ffa00d47116ae))


### Build System

* Linux packages (deb, rpm, AppImage) and a Flatpak manifest ([545abda](https://github.com/sanchezfauste/fauste-player/commit/545abda4fb4bc1dfea60846d3ebfb2feffc5f2d0))
* macOS universal app in a disk image, signed and notarised when set up ([8363f28](https://github.com/sanchezfauste/fauste-player/commit/8363f28e233eec16446d1f3e4721a856c87430ea))
* Windows installer (MSI) through cargo-wix, signed when a certificate is set ([7fa9562](https://github.com/sanchezfauste/fauste-player/commit/7fa9562448a4b70d65b2660060d442fbf1e32341))

## Changelog

All notable changes to this project are documented in this file. It is
maintained by [release-please](https://github.com/googleapis/release-please)
from [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/);
versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Version 0.1.0 is the Phase 1 baseline (multi-player playout with segues, CUE
pre-listen, automatic markers, autosave and crash recovery, and an egui
interface in English and Spanish). Releases are listed below from the first
one published after it.
