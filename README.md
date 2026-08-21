<p align="center">
  <picture>
    <source srcset="https://raw.githubusercontent.com/rpaff/LlamaLaunch/main/assets/logo-dark.png" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/rpaff/LlamaLaunch/main/assets/logo-light.png" alt="LlamaLaunch Logo" width="280" style="background:#1a1a1a; padding:20px; border-radius:12px;">
  </picture>
</p>

<p align="center">
  <strong>Fast, beautiful launcher for llama.cpp — with zero terminal hassle.</strong>
</p>

<p align="center">
<a href="https://github.com/rpaff/LlamaLaunch/releases/latest" target="_blank"><img src="https://img.shields.io/github/v/release/rpaff/LlamaLaunch?style=for-the-badge&logo=github&color=4caf50" alt="Latest Release"></a>
<a href="https://github.com/rpaff/LlamaLaunch/blob/main/LICENSE" target="_blank"><img src="https://img.shields.io/github/license/rpaff/LlamaLaunch?style=for-the-badge&logo=github&color=9e9e9e" alt="License"></a>
<a href="https://www.rust-lang.org/" target="_blank"><img src="https://img.shields.io/badge/rust-%23000?style=for-the-badge&logo=rust&logoColor=#E84E2B" alt="Rust"></a>
</p>

---

# 🚀 LlamaLaunch

**[English](#-english)** | [Русский](#-русский)

---

## 🇬🇧 English

A sleek, cross-platform graphical launcher built in Rust for **[llama.cpp](https://github.com/ggerganov/llama.cpp)**. Say goodbye to typing terminal commands — manage and launch your local LLM models from a clean, intuitive interface.

### ✨ Features

- **One-click model launching** — Start any configured model instantly. No more typing long command-line arguments.
- **Model library management** — Add, edit, and remove multiple model configurations with custom names, descriptions, and launch parameters.
- **Real-time log streaming** — Watch stdout/stderr in a live log panel with auto-scroll support. Every line arrives the moment it's written.
- **Zero-console process spawning** — Launches llama-server.exe silently (no flashing terminal window) thanks to Windows `CREATE_NO_WINDOW` flag.
- **Portable config** — All settings are stored in a `config.json` file alongside the executable. Just copy the folder and go.
- **Beautiful native UI** — Powered by [`egui`](https://github.com/emilk/egui), the UI is lightweight, fast, and feels right at home on any desktop.
- **Optimized build** — Compiled with LTO and size optimization (`opt-level = "s"`) for small binaries and fast startup.

### 📥 Get Started

The first release is already out! Grab it here:

👉 **[Download v0.1.0 Release](https://github.com/rpaff/LlamaLaunch/releases/tag/v0.1.0)**

**Quick start:**
1. Download the latest release from the link above.
2. Extract the archive to any folder.
3. Open **Llama Launch**, click **Settings**.
4. Set the path to your `llama-server.exe`.
5. Add your model configs (name, description, launch args like `--model models/mymodel.gguf --ctx-size 4096`).
6. Click **Start** on any card and enjoy!

### 🛠️ Tech Stack

| Component | Technology |
|-----------|------------|
| Language | [Rust](https://www.rust-lang.org/) (Edition 2021) |
| GUI Framework | [`eframe` / `egui`](https://github.com/emilk/egui) |
| File Dialog | [`rfd`](https://crates.io/crates/rfd) |
| Serialization | [`serde` + `serde_json`](https://crates.io/crates/serde) |

### 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.  
Made with ❤️ by [rpaff](https://github.com/rpaff).

---

## 🇷🇺 Русский

Элегантный кроссплатформенный графический лаунчер, написанный на Rust для **[llama.cpp](https://github.com/ggerganov/llama.cpp)**. Забудьте про командную строку — управляйте и запускайте ваши локальные LLM-модели из чистого и интуитивного интерфейса.

### ✨ Возможности

- **Запуск моделей в один клик** — Запускайте любую настроенную модель мгновенно, без набора длинных аргументов командной строки.
- **Управление библиотекой моделей** — Добавляйте, редактируйте и удаляйте конфигурации моделей с произвольными именами, описаниями и параметрами запуска.
- **Трансляция логов в реальном времени** — Наблюдайте за stdout/stderr в панели логов с автопрокруткой. Каждая строка появляется мгновенно.
- **Запуск без консольного окна** — llama-server.exe запускается тихим способом (без выскакивающего окна терминала) благодаря флагу `CREATE_NO_WINDOW` в Windows.
- **Портативная конфигурация** — Все настройки хранятся в файле `config.json` рядом с исполняемым файлом. Просто скопируйте папку и перенесите.
- **Красивый нативный интерфейс** — Основан на [`egui`](https://github.com/emilk/egui), легковесном и быстром GUI-фреймворке, который ощущается родным на любом десктопе.
- **Оптимизированная сборка** — Компилируется с LTO и оптимизацией по размеру (`opt-level = "s"`) для компактного бинарника и быстрого старта.

### 📥 Начало работы

Первый релиз уже доступен! Скачайте здесь:

👉 **[Скачать релиз v0.1.0](https://github.com/rpaff/LlamaLaunch/releases/tag/v0.1.0)**

**Быстрый старт:**
1. Скачайте последний релиз по ссылке выше.
2. Распакуйте архив в любую папку.
3. Откройте **Llama Launch**, нажмите **Settings**.
4. Укажите путь к вашему `llama-server.exe`.
5. Добавьте конфигурации моделей (имя, описание, аргументы запуска вроде `--model models/mymodel.gguf --ctx-size 4096`).
6. Нажмите **Start** на любой карточке — и наслаждайтесь!

### 🛠️ Технологии

| Компонент | Технология |
|-----------|------------|
| Язык | [Rust](https://www.rust-lang.org/) (Edition 2021) |
| GUI Фреймворк | [`eframe` / `egui`](https://github.com/emilk/egui) |
| Диалог файлов | [`rfd`](https://crates.io/crates/rfd) |
| Сериализация | [`serde` + `serde_json`](https://crates.io/crates/serde) |

### 📄 Лицензия

Проект распространяется под лицензией **MIT License**. Подробности в файле [LICENSE](LICENSE).  
Сделано с ❤️ пользователем [rpaff](https://github.com/rpaff).
