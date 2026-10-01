# Changelog

## [0.4.0](https://github.com/sanchezfauste/fauste-player/compare/v0.3.1...v0.4.0) (2026-10-01)


### Features

* **analysis:** trimming never removes audio; shorter segues ([c8fa45b](https://github.com/sanchezfauste/fauste-player/commit/c8fa45b510d2b8e9f5b10f6016ef2c8339a21727))
* **app:** ask before analysing tracks of an older version ([08212f1](https://github.com/sanchezfauste/fauste-player/commit/08212f1c81cf5d8988525d68053c8f21439ccb1c))
* **app:** look for missing files again ([4de2bb4](https://github.com/sanchezfauste/fauste-player/commit/4de2bb4fb0d0963ced2c428f82625a9c460b105c))
* **app:** start the remote control server ([0c6f587](https://github.com/sanchezfauste/fauste-player/commit/0c6f5879e89aa9e5916559f5cc974655f7b7e729))
* **control:** a control-surface crate that reads MIDI messages ([c88dae5](https://github.com/sanchezfauste/fauste-player/commit/c88dae588ab19515c06feec5d7464ceba86c499d))
* **control:** a MIDI service with hot-plug, feedback and learn ([c87a68e](https://github.com/sanchezfauste/fauste-player/commit/c87a68e44f9a54ccfd662789f9ec4d55a4bcfc40))
* **control:** LED feedback for bound buttons ([76418d6](https://github.com/sanchezfauste/fauste-player/commit/76418d6ff69aaa6cbc517c5ec0b32bbb3f3df1a0))
* **control:** MIDI faders move volumes with soft takeover ([5a4fb66](https://github.com/sanchezfauste/fauste-player/commit/5a4fb66bf39b0a27ac4d7556a31e8840b686c902))
* **control:** route MIDI buttons to player commands ([0696431](https://github.com/sanchezfauste/fauste-player/commit/069643186db525eea520919f5e995f270e84915a))
* **control:** which control a message offers to MIDI learn ([bd5f32c](https://github.com/sanchezfauste/fauste-player/commit/bd5f32cd614fec5c58e814453e3537975b763257))
* MIDI control surfaces ([24bf6a0](https://github.com/sanchezfauste/fauste-player/commit/24bf6a0e3ea7e63a1fe54378bc657d7ab232d888))
* missing files explained and recovered, loaded waveforms first, stopped players show their next ([47dba88](https://github.com/sanchezfauste/fauste-player/commit/47dba88285d15438b11228f57864785ebcfe2ef5))
* **model:** insert and assign tracks already in the library ([a2e0728](https://github.com/sanchezfauste/fauste-player/commit/a2e0728b90d68f28f484bfe241cea366956aff33))
* **model:** MIDI configuration and a shared fader curve ([5eda331](https://github.com/sanchezfauste/fauste-player/commit/5eda3311a36f0d33ef122822ab89c417f5a5e273))
* **model:** OSC remote configuration and source allow-lists ([775725d](https://github.com/sanchezfauste/fauste-player/commit/775725da9258695a24c0602e6dcfa637c3422973))
* **model:** record each player's history ([53f7414](https://github.com/sanchezfauste/fauste-player/commit/53f7414ddc117499ca032dee26e19e3b760bd2a3))
* **model:** remote control configuration ([f2c2b93](https://github.com/sanchezfauste/fauste-player/commit/f2c2b93c25d35f11b84ce14529e7928f437b2dfc))
* **model:** repeat an entry until the operator moves on; stop after it ([ff56fbf](https://github.com/sanchezfauste/fauste-player/commit/ff56fbf5daf58f07efa9a7aa021f6d6ec430c2de))
* **model:** repeat and stop-after flags on playlist entries ([a0f9659](https://github.com/sanchezfauste/fauste-player/commit/a0f9659dcb1e3b1c5f8cbca3a870edb0d7c40c2e))
* **model:** restart the current entry and go back to the previous one ([04d91d8](https://github.com/sanchezfauste/fauste-player/commit/04d91d8cae6eb50b57edab9a9b3dee6c7fb60c9d))
* **model:** say which transport actions are available ([f1a35ba](https://github.com/sanchezfauste/fauste-player/commit/f1a35bad00de4118a03cb1e85c307bec7e0f7b00))
* **model:** tuning.missing_recheck_ms ([6ef70b4](https://github.com/sanchezfauste/fauste-player/commit/6ef70b47eddf22e1d04ef697d0ac1196c8422292))
* per-track repeat and stop after ([75003af](https://github.com/sanchezfauste/fauste-player/commit/75003afb1e963485649ebb4c90aea334e65ee7b9))
* **remote:** crate with the control trait and the API's JSON contract ([03b02aa](https://github.com/sanchezfauste/fauste-player/commit/03b02aa71fecff3fcd28dedbfff84b088648b481))
* **remote:** edit playlists, cart pages and markers ([d8a774d](https://github.com/sanchezfauste/fauste-player/commit/d8a774d2572609e188268627b9f1a885e8f26a13))
* **remote:** editing routes and Settings &gt; Remote ([3338789](https://github.com/sanchezfauste/fauste-player/commit/33387893f622c64acbfbac5d34cb03daf11ac734))
* **remote:** event stream over SSE ([be49a10](https://github.com/sanchezfauste/fauste-player/commit/be49a10ec34744823230283af271a2d7213117dc))
* **remote:** HTTP API to read and operate the player ([ee97a9e](https://github.com/sanchezfauste/fauste-player/commit/ee97a9eef71111d2620c80973df91f99b49ad533))
* **remote:** HTTP routes to read and operate players and carts ([0085f00](https://github.com/sanchezfauste/fauste-player/commit/0085f00837ad1034ac4881dd199534a860541b06))
* **remote:** live events over SSE and OSC control ([4c2a5ed](https://github.com/sanchezfauste/fauste-player/commit/4c2a5ed543497bc4daa7b02f5e43dd93f4a623b4))
* **remote:** OSC addresses, values and subscriptions ([fb0e6bb](https://github.com/sanchezfauste/fauste-player/commit/fb0e6bb27255f9bfc20bb0d088cd78075c357c76))
* **remote:** OSC server with subscriptions ([e8f1bae](https://github.com/sanchezfauste/fauste-player/commit/e8f1bae1d96b91ae2f27ec42e023aa6653573ea2))
* **remote:** reduce a waveform to n buckets, and send it without a JSON tree ([68dd94a](https://github.com/sanchezfauste/fauste-player/commit/68dd94a110159f26f25fe6c78b5e55b2baad72e4))
* **remote:** resource events from snapshot diffs ([ec53675](https://github.com/sanchezfauste/fauste-player/commit/ec53675c3967667fdcad9c35f85cb15af26f7b53))
* **remote:** server thread that follows the configuration ([9f1bffc](https://github.com/sanchezfauste/fauste-player/commit/9f1bffc860ec93b5fd168c4d3fc8634dd6b5507b))
* **remote:** token, origin and host checks with CORS and limits ([5ed95c2](https://github.com/sanchezfauste/fauste-player/commit/5ed95c2a27f2fa69db39b50673ef673286badc66))
* **remote:** turn operations into checked model commands ([88ae81f](https://github.com/sanchezfauste/fauste-player/commit/88ae81fcc1d4704f947176a7daf3ab11e631baa2))
* restart and previous with button availability ([8704ec2](https://github.com/sanchezfauste/fauste-player/commit/8704ec2a100ca0159f645120dc6b5cabf80c8b94))
* **store:** drop config fields this version does not use ([953dd24](https://github.com/sanchezfauste/fauste-player/commit/953dd2492d05483e5494c79064f527397576016c))
* **ui:** a stopped player shows the track Play will start ([756a3bc](https://github.com/sanchezfauste/fauste-player/commit/756a3bc0f84ba2c9472331a01b77c33b9ff1ab12))
* **ui:** a view mapping for waveform zoom ([417a2da](https://github.com/sanchezfauste/fauste-player/commit/417a2da428c6ab579f3f61ce6d2d913f8d505857))
* **ui:** drag on the waveform to seek on release; dim trimmed audio ([a98031a](https://github.com/sanchezfauste/fauste-player/commit/a98031aba6201f635e00f960e868028d65a32fe2))
* **ui:** elapsed / total under the waveform ([a543b08](https://github.com/sanchezfauste/fauste-player/commit/a543b08eb8f4828fa732abd55a630460eb1b5113))
* **ui:** give the meter muted zone colours ([e6dde85](https://github.com/sanchezfauste/fauste-player/commit/e6dde856e1d07deea53453f25754fdbd2d52a82b))
* **ui:** interface polish from operator feedback ([bc576b2](https://github.com/sanchezfauste/fauste-player/commit/bc576b2b81fe4793e1970ec6cd5382e17ad4ea43))
* **ui:** make slate the default waveform colour ([e5803f7](https://github.com/sanchezfauste/fauste-player/commit/e5803f73ae7e168d118b05826ec5037bc806ec0a))
* **ui:** meter and fader column with a dB scale ([d9d16f0](https://github.com/sanchezfauste/fauste-player/commit/d9d16f0aa969bf359472612357d760639d8c68c3))
* **ui:** move the meter and fader into a column on the right ([bcf9d3e](https://github.com/sanchezfauste/fauste-player/commit/bcf9d3eed9f011a44ddba6481a7d3ef87354adea))
* **ui:** own icons for missing files and outdated analyses ([bf9fe46](https://github.com/sanchezfauste/fauste-player/commit/bf9fe4650c20266cd3e4e2bc581b31164b5c7cb6))
* **ui:** proportional playlist columns ([81739e5](https://github.com/sanchezfauste/fauste-player/commit/81739e55d521cc43c4da93299a27fd2136fbcc5d))
* **ui:** proportional playlist columns and follow the current track ([075da68](https://github.com/sanchezfauste/fauste-player/commit/075da684dc4c25fb3ec8a1a781a1f6bae78a889d))
* **ui:** repeat and stop-after in the track table ([e726520](https://github.com/sanchezfauste/fauste-player/commit/e7265204c2d3696b03c872507fc6b2e6b62c2715))
* **ui:** restart and previous buttons, dimmed when unavailable ([9d9d855](https://github.com/sanchezfauste/fauste-player/commit/9d9d855d27771db77215bf69c88c63dfd093610e))
* **ui:** restart and previous shortcuts, ignored when unavailable ([12e9a5d](https://github.com/sanchezfauste/fauste-player/commit/12e9a5df8848ffebbd1473babcb2ca07459a041c))
* **ui:** say why a track is unavailable ([c6db4c1](https://github.com/sanchezfauste/fauste-player/commit/c6db4c12061358e45163b4d9d341c7428eb67f17))
* **ui:** Settings &gt; MIDI and MIDI control at start-up ([9acba92](https://github.com/sanchezfauste/fauste-player/commit/9acba9282ee56425e57eed2ce17092ccb092f3eb))
* **ui:** Settings &gt; Remote ([ab35d3c](https://github.com/sanchezfauste/fauste-player/commit/ab35d3c47c9564b48e35d846be49ee2127786c97))
* **ui:** show the version and an About window ([34fc916](https://github.com/sanchezfauste/fauste-player/commit/34fc9163505367ad215ec3a5c26259c472f5833a))
* **ui:** the playlist follows the current track ([8f39032](https://github.com/sanchezfauste/fauste-player/commit/8f390320293fed9980fd61ef8bbe5cd1ec1f0682))
* **ui:** waveform drag, zoom and trimmed regions ([a038b48](https://github.com/sanchezfauste/fauste-player/commit/a038b48c10e717a909f54f3c8211f1c4708c54d3))
* **ui:** zoom and pan the waveform with the wheel ([7de230a](https://github.com/sanchezfauste/fauste-player/commit/7de230a66ae9162e6e78467fd5ba68987bdaa631))


### Bug Fixes

* **analysis:** never trim audio, and start segues relative to the track ([1f570af](https://github.com/sanchezfauste/fauste-player/commit/1f570afd0dbe57934b47148d4a26aa7fd4cafc8e))
* **analysis:** tracks on a player are analysed ahead of the library ([ecddf8e](https://github.com/sanchezfauste/fauste-player/commit/ecddf8ea00cdc531fb007d64514a45e20946e009))
* **app:** analyse again tracks marked by an older analysis version ([c70d5e5](https://github.com/sanchezfauste/fauste-player/commit/c70d5e5fbdb8dd3c79a71f8ad917d9ea86141ed6))
* **app:** look for missing files on a probe thread ([68b1326](https://github.com/sanchezfauste/fauste-player/commit/68b13262f2527105cdbf5074ad76010a1ac74da9))
* **app:** review follow-ups for the file probe and promotion ([8b297ce](https://github.com/sanchezfauste/fauste-player/commit/8b297cecfb61cc6bf96e1a34fd7ba728a4aa9dfd))
* **app:** tracks with a missing file do not count as outdated ([1cc4ee8](https://github.com/sanchezfauste/fauste-player/commit/1cc4ee875e49825133bf39a971329455c57aa597))
* **backends:** label unnamed devices and fall back to the address ([023aa4c](https://github.com/sanchezfauste/fauste-player/commit/023aa4cdb9eb31f9ee868321ecd2b5db5d8db219))
* **backends:** tell same-named output devices apart ([d11f551](https://github.com/sanchezfauste/fauste-player/commit/d11f551946a73c5ddb8a89d16f650e653bbc526b))
* **control:** keep MIDI faders in control and leave other ports alone ([5d5cb4d](https://github.com/sanchezfauste/fauste-player/commit/5d5cb4dc80a8c98c3f89e8d6cb3c1fd3fe88f629))
* **decode:** drop the Opus pre-skip in Ogg and Matroska ([1cec52a](https://github.com/sanchezfauste/fauste-player/commit/1cec52a33a6992602e63f2a703aaa6d1cb09bcd3))
* **engine:** keep repeat passes and commands in step at a boundary ([05f2e3e](https://github.com/sanchezfauste/fauste-player/commit/05f2e3e7e2dea217de273f598d7606cd7944c9ed))
* **model:** a position restored at the end comes back at the cue-in ([998e66e](https://github.com/sanchezfauste/fauste-player/commit/998e66e3d9cd4660a7f711494cb46c91e9b8f99a))
* **model:** previous skips missing and unreadable files ([f817624](https://github.com/sanchezfauste/fauste-player/commit/f8176247c70ba1a886876e830879669601278ddb))
* **remote:** edit a cart page and a cart as one atomic command ([e31b07b](https://github.com/sanchezfauste/fauste-player/commit/e31b07be099171747edfabefff01eb881f21ee30))
* **remote:** idempotent on/off commands and no wildcard origin without a token ([7d10096](https://github.com/sanchezfauste/fauste-player/commit/7d100964d787151554c346e8fe197621da8f4e09))
* **remote:** JSON 405, bind retry, case-insensitive bearer, cart times like the UI ([5522c1e](https://github.com/sanchezfauste/fauste-player/commit/5522c1e5fb7ba94331615c8df5f4f591f234c300))
* **remote:** log a busy port once, not on every retry ([97e043c](https://github.com/sanchezfauste/fauste-player/commit/97e043c686059ce44f76d6de5410647b8d78f8f7))
* **remote:** one in-flight request limit for the whole server ([5006bd2](https://github.com/sanchezfauste/fauste-player/commit/5006bd28376f48dd7a73a143061eb0a3c401e3b9))
* **remote:** OSC clears the addresses that disappear ([56cd347](https://github.com/sanchezfauste/fauste-player/commit/56cd347b585a143053be42b99a94270b98ad54e4))
* **remote:** refuse deeply nested OSC packets and bound the cost of floods ([f435371](https://github.com/sanchezfauste/fauste-player/commit/f43537150651cea18498d3d21701c45c1179d704))
* **remote:** review follow-ups, long recordings, meters and screenshot ([0a8703a](https://github.com/sanchezfauste/fauste-player/commit/0a8703affe4c310e499a08104019b06a19e38366))
* **ui:** a discreet alignment mark on the meter ([4d428d9](https://github.com/sanchezfauste/fauste-player/commit/4d428d956848eca9e0791a95d58f8eaa16f473b5))
* **ui:** a stopped player's waveform shows times without seeking ([32f51fd](https://github.com/sanchezfauste/fauste-player/commit/32f51fd1d7f1cdf7bf759c612150b2ec0702ffe1))
* **ui:** count table scrolls and drags, and keep resizes out of widths ([2d26486](https://github.com/sanchezfauste/fauste-player/commit/2d26486d8db69b25d0fbc71e254f9f301da2627e))
* **ui:** draw a labelled dB scale with lines across both meter bars ([6adbd0c](https://github.com/sanchezfauste/fauste-player/commit/6adbd0c417551415b02b47d9cf60dd6d81109f83))
* **ui:** draw SINGLE and CONT as one segmented control ([4501cbe](https://github.com/sanchezfauste/fauste-player/commit/4501cbe05e9a52a2f0227d25c321f3dfff7c4cf7))
* **ui:** each meter type shows only the settings its standard defines ([de1be28](https://github.com/sanchezfauste/fauste-player/commit/de1be28a1565198eb8e92de64718cd9cf888e4ef))
* **ui:** fit the countdown between the grid and the meter ([6b6547b](https://github.com/sanchezfauste/fauste-player/commit/6b6547b6cf9f3fe6d624f13d11677d59bd1d3a6d))
* **ui:** keep countdowns and times at a fixed width ([2895ec7](https://github.com/sanchezfauste/fauste-player/commit/2895ec72d5f530b4bfc586a94ce12bae6596c887))
* **ui:** keep waveform drags and zoom from acting on the wrong track ([f61407c](https://github.com/sanchezfauste/fauste-player/commit/f61407cbb523f0fe5789aec993cf3a263412d798))
* **ui:** redraw the stop-after-current icon as play then stop ([95c146a](https://github.com/sanchezfauste/fauste-player/commit/95c146a80864bff43b4b43506a328c8b71872aef))
* **ui:** Settings &gt; Remote applies a pending edit when another section opens ([09e5ae3](https://github.com/sanchezfauste/fauste-player/commit/09e5ae343ef2d363893fceecfa68d667cc95a55a))
* **ui:** Settings &gt; Remote buttons look like the other sections' ([fa77283](https://github.com/sanchezfauste/fauste-player/commit/fa77283f13be43ad7bab7241ea27bc621406b3d6))
* **ui:** Settings &gt; Remote keeps what is in use until a value is valid ([318ee59](https://github.com/sanchezfauste/fauste-player/commit/318ee59bd17b62ea0d1975743bc1d83e033d60d9))


### Performance

* **decode:** decode Opus with a real FFT, not an O(n²) DFT ([d6f0005](https://github.com/sanchezfauste/fauste-player/commit/d6f0005360753dc6d058585bf729989bf78fe8f4))


### Documentation

* a 1920x1080 main screenshot made in Xvfb ([df150a3](https://github.com/sanchezfauste/fauste-player/commit/df150a37d70f46dbf9c76d6f4337e5238a0b68e3))
* CLAUDE.md lists the vendored Opus decoder ([4f5deb0](https://github.com/sanchezfauste/fauste-player/commit/4f5deb07973ba7931caced5e26c2cdfb835530f7))
* describe MIDI control surfaces ([66d32de](https://github.com/sanchezfauste/fauste-player/commit/66d32deb548124fb1592d1f08fc290d6de3d58df))
* describe peak trimming and relative segues ([c479c20](https://github.com/sanchezfauste/fauste-player/commit/c479c20e6aec447d84fb195e14af66ff875048f5))
* describe per-track repeat and stop after ([683ec74](https://github.com/sanchezfauste/fauste-player/commit/683ec744641860985423dea50ac60259abc6498b))
* describe proportional columns and follow current ([00c9afe](https://github.com/sanchezfauste/fauste-player/commit/00c9afeea9da6c1a03c43b5843fb9067e24e9868))
* describe remote events and OSC ([a94f3d2](https://github.com/sanchezfauste/fauste-player/commit/a94f3d21b69f62f9adf0ccecc54a509ee527e578))
* describe restart, previous and button availability ([a0df87e](https://github.com/sanchezfauste/fauste-player/commit/a0df87e50bbec128333d346f48baa6f695d2886a))
* describe the interface polish ([eeb3fec](https://github.com/sanchezfauste/fauste-player/commit/eeb3fecd61ad0755e12ad07970fdaa1a992f9c9f))
* describe the meter column ([ad9f81a](https://github.com/sanchezfauste/fauste-player/commit/ad9f81ac6d79597c61ab6cce102be41f88b17684))
* describe the remote control API ([8d2e79b](https://github.com/sanchezfauste/fauste-player/commit/8d2e79ba03ae8eae384363c3e44e5c853ca3d9c6))
* describe waveform drag, zoom and trimmed regions ([ad58e3e](https://github.com/sanchezfauste/fauste-player/commit/ad58e3e00e29a334923c5472b7efa0e745380d72))
* **docs:** design and plan the operator feedback work ([23c0ef6](https://github.com/sanchezfauste/fauste-player/commit/23c0ef669036a82e3334a779c0cbe3610ad4c6f4))
* **docs:** design the operator feedback work ([cfcd2f6](https://github.com/sanchezfauste/fauste-player/commit/cfcd2f6d4a85204d4609dfc93e3f002e65f60e87))
* **docs:** plan MIDI control ([b6c1e6a](https://github.com/sanchezfauste/fauste-player/commit/b6c1e6a32ec1790738367267e3ea04ae56667966))
* **docs:** plan per-entry repeat and stop ([6b06495](https://github.com/sanchezfauste/fauste-player/commit/6b06495263bb74d457deb4e69ad6599c7d65b7f2))
* **docs:** plan proportional columns and follow current ([a339c2f](https://github.com/sanchezfauste/fauste-player/commit/a339c2f02719ff6040fa6981ca569a1d6a66540f))
* **docs:** plan restart, previous and button availability ([b4eecd6](https://github.com/sanchezfauste/fauste-player/commit/b4eecd6afca93f36b199171b144b26eef68c229b))
* **docs:** plan the marker tuning ([02a0f7e](https://github.com/sanchezfauste/fauste-player/commit/02a0f7ee23faeff733d3b91da9a2f0b7f4be7cea))
* **docs:** plan the operator feedback work ([7e348a9](https://github.com/sanchezfauste/fauste-player/commit/7e348a94fdc8cc9cd2fc94fdaf4d940b56a0f92a))
* **docs:** plan the waveform drag and zoom ([5638059](https://github.com/sanchezfauste/fauste-player/commit/56380593677f3550955136abd9ead8f2552fe731))
* plan 4 adds long recordings and the re-analysis notice ([660d994](https://github.com/sanchezfauste/fauste-player/commit/660d99453c31d1659aa4f3264ae751fb49a91a99))
* plan 4, remote follow-ups, restore at the end, meter and screenshot ([4a29405](https://github.com/sanchezfauste/fauste-player/commit/4a294054687f7e02c6c5e49e951bdf1b7f23b1ea))
* plan the missing-file tooltip and recheck ([ed35c47](https://github.com/sanchezfauste/fauste-player/commit/ed35c47554bf98f7aa37de50ed677970d496bdf0))
* **plan:** remote control plan 1, HTTP foundation ([888495c](https://github.com/sanchezfauste/fauste-player/commit/888495c40d42f0dca0a240c1eb7ad306ab36f764))
* **plan:** remote control plan 2, events and OSC ([5de4d98](https://github.com/sanchezfauste/fauste-player/commit/5de4d989a0d6cf824e63e4bc6432903b24e64c8d))
* **plan:** remote control plan 3, editing and Settings ([5e8db60](https://github.com/sanchezfauste/fauste-player/commit/5e8db604c00b3717affb9a6128a9fb87425525b2))
* remote editing, Settings &gt; Remote, and a main screen on air ([27eb705](https://github.com/sanchezfauste/fauste-player/commit/27eb7056b068c64cc32918908048f704d4c28f94))
* **spec:** remote control over HTTP and OSC ([e3231ce](https://github.com/sanchezfauste/fauste-player/commit/e3231ce24140014b13c3cfa00e569128789e684c))

## [0.3.1](https://github.com/sanchezfauste/fauste-player/compare/v0.3.0...v0.3.1) (2026-09-30)


### Bug Fixes

* **ui:** settle the meter and waveform review minors ([821044c](https://github.com/sanchezfauste/fauste-player/commit/821044c87da0d77e235f075090403a7e99b6955d))
* **ui:** settle the meter and waveform review minors ([c926384](https://github.com/sanchezfauste/fauste-player/commit/c9263843fe45fc790d7a7fec63ac402e319873d3))


### Documentation

* record the follow-up review's minors ([104e511](https://github.com/sanchezfauste/fauste-player/commit/104e51124d21ac7ecab3a9008c5a04789c2bae6e))
* refresh the main screen screenshot ([819fce8](https://github.com/sanchezfauste/fauste-player/commit/819fce86f75896deafe770d3d24fce2a8a88e3e1))

## [0.3.0](https://github.com/sanchezfauste/fauste-player/compare/v0.2.0...v0.3.0) (2026-09-29)


### Features

* **analysis:** keep the RMS level of each waveform bucket ([a806936](https://github.com/sanchezfauste/fauste-player/commit/a8069363b990ad7bf680c4dcbd41e7468bd1366d))
* **app:** accept the new audio formats, read DSF tags, document decoding ([59715d2](https://github.com/sanchezfauste/fauste-player/commit/59715d2027dca63e56c834662df9c67d72989d88))
* **decode:** play DSD, WavPack, Monkey's Audio and Opus ([9149106](https://github.com/sanchezfauste/fauste-player/commit/91491067d7b695265182b0abac70a51bd538986e))
* **engine:** K-System meters and a maximum level per player ([bc68eea](https://github.com/sanchezfauste/fauste-player/commit/bc68eea1a7126592d18900671f3f975bf7ec373f))
* **ui:** draw the waveform continuously with its RMS body ([df8bd1c](https://github.com/sanchezfauste/fauste-player/commit/df8bd1c4ef22941cb826330f8e12fdc6f75fdf17))
* **ui:** meters on their standard's scale, with a maximum readout ([0373c72](https://github.com/sanchezfauste/fauste-player/commit/0373c72e4c4ed514ef1117557f24f2760cf33778))
* **ui:** standard meter scales, the K-System and a maximum readout ([de93d65](https://github.com/sanchezfauste/fauste-player/commit/de93d65fafd35b681060d0647b0d8b3070c08af3))
* **ui:** waveform with a peak outline and an RMS body, drawn continuously ([b345cef](https://github.com/sanchezfauste/fauste-player/commit/b345cefbfeb272ea612b1d9b1123430c1bb25da4))


### Bug Fixes

* **analysis:** read the INTRO tag from Opus, WavPack, Monkey's Audio and DSF ([3555ca0](https://github.com/sanchezfauste/fauste-player/commit/3555ca0dfcf8ac9cd070cf9ece26dff60be49965))
* **ci:** build the release packages on every platform, and run the nightly fuzz ([0a2b870](https://github.com/sanchezfauste/fauste-player/commit/0a2b870c800666d66a889889a2daa13580551cf8))
* **decode:** look past a leading ID3v2 tag, and never guess a format ([91075a5](https://github.com/sanchezfauste/fauste-player/commit/91075a592d8906779f632a00f675228194099736))
* **decode:** place surround DSD channels by their layout ([0622f2d](https://github.com/sanchezfauste/fauste-player/commit/0622f2db7ab4c9174d9f48b95aa7d73bea73d702))
* **decode:** refuse DSDIFF property chunks that overrun their parent ([493cdcb](https://github.com/sanchezfauste/fauste-player/commit/493cdcb37768d6e3415698c42801a572b0b44a28))
* **decode:** trim encoder delay once, in the decoder ([1bc8c8b](https://github.com/sanchezfauste/fauste-player/commit/1bc8c8b07111961b11d4b165cf3cb58a0b93ae24))
* **packaging:** give each installer notice its own component ([759dcd2](https://github.com/sanchezfauste/fauste-player/commit/759dcd2522ac93f419a7143a9c947b4f6bea2198))
* **ui:** mark the alignment level on every meter, and turn red where each scale does ([7e77951](https://github.com/sanchezfauste/fauste-player/commit/7e77951d7939ef6a99dc1b619950ac9a680d33f7))


### Documentation

* describe the standard meter scales and the K-System ([57c3bfe](https://github.com/sanchezfauste/fauste-player/commit/57c3bfee36b6c23334dbbf4e80f1dd0ae3d6ba08))
* record the waveform plan's ruling and deferred minors ([ab5e9b0](https://github.com/sanchezfauste/fauste-player/commit/ab5e9b06153fafdad2b08badb1468b1b259be502))

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
