# Changelog

## [0.2.0](https://github.com/rchardx/aurotype/compare/v0.1.1...v0.2.0) (2026-03-04)


### Features

* **engine:** detect and discard silent recordings ([971381d](https://github.com/rchardx/aurotype/commit/971381d8bc157fcf904c490d413814b19d7a83c1))
* **frontend:** auto-dismiss copy button after 2 seconds ([b97858f](https://github.com/rchardx/aurotype/commit/b97858f67b5a0ab8f2f7d91e891403371985eb78))
* **tauri:** add structured logging system with file and stderr output ([b52ab5b](https://github.com/rchardx/aurotype/commit/b52ab5bddc310d4751ec18c1f1659523033e13d9))
* **tauri:** detect text input focus on Windows via UI Automation ([a0229ac](https://github.com/rchardx/aurotype/commit/a0229acea5e5c8739cee9bfa7b770527570e023c))


### Bug Fixes

* **engine:** disable parent PID monitoring on Windows ([25a51c9](https://github.com/rchardx/aurotype/commit/25a51c9c2e68f4aed5c12a7d5e759547b1ea4369))
* **tauri:** add log, simplelog, and windows crate dependencies to Cargo.toml ([8c3b1f6](https://github.com/rchardx/aurotype/commit/8c3b1f60e4fad0044e03842ef4db541494047962))

## [0.1.1](https://github.com/rchardx/aurotype/compare/v0.1.0...v0.1.1) (2026-03-02)


### Features

* add error handling, timeouts, cancel, and crash recovery ([952fc8e](https://github.com/rchardx/aurotype/commit/952fc8e09f896f9f8aa2d5b6c2c2250834169e15))
* add PyInstaller build pipeline for Windows release ([713a606](https://github.com/rchardx/aurotype/commit/713a606065a859bdb4e488838bddd1772ec2fde0))
* configurable LLM system prompt ([a0fdad3](https://github.com/rchardx/aurotype/commit/a0fdad307994541d69ea6e437749baa801004ee6))
* end-to-end voice input pipeline integration ([c9e4e57](https://github.com/rchardx/aurotype/commit/c9e4e57c404e53dbfc6bdb3a92f32ac0c6d3fc12))
* **engine:** add audio capture module with sounddevice ([7fc30a5](https://github.com/rchardx/aurotype/commit/7fc30a5aac7d302e079d21c90b22e6ec86ac5208))
* **engine:** add DashScope Paraformer STT provider ([3a22f02](https://github.com/rchardx/aurotype/commit/3a22f026b5a83e9cc0ddf6579c03df5cd567d14e))
* **engine:** add FastAPI sidecar skeleton with health endpoint and port handshake ([09b948f](https://github.com/rchardx/aurotype/commit/09b948fa2eacbba852bb1037829e5ca90877c7ab))
* **engine:** add LLM provider abstraction with OpenAI, SiliconFlow providers ([f9a5649](https://github.com/rchardx/aurotype/commit/f9a564988edf7801061fe12691a0804010ec6c82))
* **engine:** add STT provider abstraction with Groq and SiliconFlow providers ([7689573](https://github.com/rchardx/aurotype/commit/7689573e9de0fa142bcdf1a95c22576d40607d36))
* **engine:** integrate audio → STT → LLM pipeline ([28aa39c](https://github.com/rchardx/aurotype/commit/28aa39c4d23f75b2b0f7554074f871e8fb6f1a18))
* **engine:** replace Groq STT with Deepgram, remove all Groq support ([ccb21dd](https://github.com/rchardx/aurotype/commit/ccb21ddd0c94a1ce1b0cb01c0fb271413089ddcf))
* **engine:** replace SiliconFlow/Deepgram with DeepSeek, add configurable STT model ([04bbe96](https://github.com/rchardx/aurotype/commit/04bbe96d486088af1ea94294a3f3ab7823d5eb6a))
* **frontend:** persist default system prompt in settings.json ([b9c6738](https://github.com/rchardx/aurotype/commit/b9c6738958ec046b4f137cc0e66a92ef9ddb096b))
* persistent history with audio recording, playback, and STT retry ([3089e8c](https://github.com/rchardx/aurotype/commit/3089e8c40306c6231f943cf2994e03863fbc8530))
* replace health check polling with Tauri event-based status ([2aac8ca](https://github.com/rchardx/aurotype/commit/2aac8cab57a56577ab617a97d5acfef2296b1d7a))
* **scaffold:** initialize monorepo with Tauri, React frontend, and Python engine ([57b9f29](https://github.com/rchardx/aurotype/commit/57b9f2967a5c93e0d48b241225c723652fb6db3f))
* system prompt editor with save/reset buttons ([4168258](https://github.com/rchardx/aurotype/commit/416825897b8b867133d96edd5b0aa8abc9128c92))
* **tauri:** add clipboard text injection with focus tracking ([ef4308d](https://github.com/rchardx/aurotype/commit/ef4308d6825378876b96c2b190cb8ad768aa3779))
* **tauri:** add global hotkey and system tray with state management ([e10b2cc](https://github.com/rchardx/aurotype/commit/e10b2cca79143696910b214618c96258e48d467f))
* **tauri:** add recording history, copy fallback, and close-to-tray ([7ca031c](https://github.com/rchardx/aurotype/commit/7ca031c510af871fafeac5165d4b1460e79d37a6))
* **tauri:** add sidecar spawn, port handshake, and health check ([b3507ba](https://github.com/rchardx/aurotype/commit/b3507ba6045b11bded5cf653321d91711d9af583))
* **tauri:** configure externalBin sidecar and rebrand to Aurotype ([d593a1c](https://github.com/rchardx/aurotype/commit/d593a1c144997f97f0c1409b5af0d14fb52da49d))
* transparent float window with Rust-driven positioning ([01477c3](https://github.com/rchardx/aurotype/commit/01477c3d2691b7c754b4c9853f3a6b29fa8a61f5))
* **ui:** add recording history panel and copy fallback to Settings/FloatWindow ([2498e00](https://github.com/rchardx/aurotype/commit/2498e005d0508c563c648e37c6f48f379b4b8a3d))
* **ui:** add recording/processing float window ([970aa1f](https://github.com/rchardx/aurotype/commit/970aa1f588059d17e579e4b33db40c5f29d13ef0))
* **ui:** add settings page with provider and hotkey config ([632f23e](https://github.com/rchardx/aurotype/commit/632f23ec30d349c26a7b44e0c2e41f8bae948396))
* wire DashScope provider into config, server, and UI ([edfb98c](https://github.com/rchardx/aurotype/commit/edfb98c2814b36b0dc0c51b863de5787052bfe93))


### Bug Fixes

* center float window content and simplify glass-morphism ([93bf14e](https://github.com/rchardx/aurotype/commit/93bf14e2bd32127ad3e84587b6707e519f11c2a4))
* **ci:** add cargo test and enable Actions PR creation for release-please ([5e1bcd6](https://github.com/rchardx/aurotype/commit/5e1bcd60b31dd4fc40f17e59655292d546924fa6))
* **ci:** create sidecar stub so cargo check passes on Linux ([4866081](https://github.com/rchardx/aurotype/commit/48660812af62288657854f987ba12ebf7646681e))
* **ci:** enable LFS checkout for rust-check to resolve invalid PNG icons ([b7514fb](https://github.com/rchardx/aurotype/commit/b7514fb3999c2dd2c39fbdc418233e8302dea2a7))
* **ci:** resolve clippy collapsible_if and single_match warnings ([46ef96e](https://github.com/rchardx/aurotype/commit/46ef96e83a1fc0e2b022557d4af69922c4858728))
* correct macOS build config for PyInstaller onefile output and Tauri infoPlist ([b85fb5c](https://github.com/rchardx/aurotype/commit/b85fb5cc79cb48beef7fda12b33662505236d95f))
* **engine:** align audio RMS calculation with numpy formula ([c49f862](https://github.com/rchardx/aurotype/commit/c49f86263550f6ac79958241e63a52e4decb54c3))
* **engine:** enable Deepgram auto language detection for non-specified language ([02e2208](https://github.com/rchardx/aurotype/commit/02e220882ec7ce7179f814f4dadcfcd5e3273a7f))
* **engine:** improve provider defaults, config filtering, and short recording handling ([3bfdafa](https://github.com/rchardx/aurotype/commit/3bfdafa0df2d71df775f2742d965dd4d59eb5945))
* **engine:** update SiliconFlow STT model to FunAudioLLM/SenseVoiceSmall ([5a8e6f1](https://github.com/rchardx/aurotype/commit/5a8e6f182bf12053b51203c9ff62e53ff0c53b95))
* only set default system prompt when no saved settings exist ([fdad6ed](https://github.com/rchardx/aurotype/commit/fdad6edc37b99f077623826006436da5dcf6acf7))
* regenerate app icons with tighter crop and multi-size ico ([99e74ee](https://github.com/rchardx/aurotype/commit/99e74ee5684237b36f2f0102047ebae07879bee4))
* square float window, boost waveform visibility, remove LLM None option ([271b3e5](https://github.com/rchardx/aurotype/commit/271b3e5d39f228679788c4d07717eccd9b1e6726))
* **tauri:** implement macOS window refocus and accessibility permission ([6191102](https://github.com/rchardx/aurotype/commit/61911028bacdddd98faef930df721b449d6151b1))
* **tauri:** resolve macOS recording race, crash on inject, sidecar cleanup, and mic permission ([4bf196e](https://github.com/rchardx/aurotype/commit/4bf196ed5e326d234e3b5f164ce0ab6d4aca787a))
* **tauri:** rewrite hotkey handler for Windows compatibility ([cfdc1f1](https://github.com/rchardx/aurotype/commit/cfdc1f1bce1b536ed3186cfb0ebdbaff5b69942a))
* **ui:** remove debug console.log and implement error auto-dismiss in float window ([300e713](https://github.com/rchardx/aurotype/commit/300e71314afc3b6a53ef015253c34be2a1841959))
