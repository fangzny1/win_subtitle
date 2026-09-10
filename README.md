# Tauri + Vue 3

This template should help get you started developing with Tauri + Vue 3 in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

# how to download model
via [sherpa-onnx-sense-voice](https://huggingface.co/csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/tree/main) to download `model.int8.onnx` `tokens.txt` and via [silero_vad](https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/silero_vad.onnx) to download `silero_vad.onnx` and make sure these file put in `/src-tauri/model/`

# Settings / 设置说明

Click the ⚙ button to open the settings panel. Settings are stored in `src-tauri/config.json` (auto-created with defaults on first run, so it is safe to delete).

点 ⚙ 齿轮按钮打开设置面板，配置保存在 `src-tauri/config.json`（首次运行自动生成默认文件，可随时删除重置）。

| Field 字段 | What it does 说明 |
|---|---|
| `ai_api_base` | Translation API base URL, must be OpenAI-compatible (`/v1/chat/completions`). Bring your own endpoint. 翻译接口地址，需兼容 OpenAI 格式，请填自己的服务地址 |
| `ai_model` | Model name sent to the API. 模型名 |
| `ai_api_key` | Bearer token (send `sk-123` etc. as-is if your server needs one). 密钥 |
| `target_lang` | Target language, e.g. `zh`. 目标语言 |
| `vad_slience_ms` | Silence length that ends a sentence (ms). 静音多长切一句 |
| `vad_max_speech_ms` | Max length of one sentence (ms); longer speech is force-cut. 单句最长多长，超长强制切断 |
| Title bar button 标题栏按钮 | Toggles the native window title bar at runtime (off = frameless overlay). 运行时开关原生标题栏 |

Notes 注意：
- AI settings (`ai_*`, `target_lang`) apply live to the next sentence, no restart needed. AI 相关修改即时生效，无需重启。
- VAD settings (`vad_*`) take effect on next Start (Stop → Start). VAD 参数需停止后重新开始才生效。
- The default `ai_api_base` is the author's campus LAN address and won't work for you — replace it with your own service. 默认地址是作者校园网内网地址，你用不了，请换成自己的翻译服务。

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
