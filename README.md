# 🚀 LlamaLaunch

**[English](#-english)** | [Русский](#-русский)

---

## 🇬🇧 English

A sleek, cross-platform graphical launcher built in Rust for **[llama.cpp](https://github.com/ggerganov/llama.cpp)**. Say goodbye to typing terminal commands — manage and launch your local LLM models from a clean, intuitive interface.

### Why LlamaLaunch?

LlamaLaunch was built with one principle: **nothing between you and your model**. No web panels, no cloud dependencies, no telemetry or tracking of any kind. It is just a small, native Rust binary that does one job extremely well — launching llama.cpp servers the way you want them launched. The interface strips away everything unnecessary so you see only your models and the Start/Stop controls. Yet this minimalism never means limited functionality: every model configuration accepts fully custom launch arguments, so you can fine-tune `--ctx-size`, `--gpu-layers`, `--tensor-split`, temperature, or any other parameter that llama.cpp supports. One config per model, all stored in a single `config.json` next to the executable. Portable on a USB stick, silent background launches with no flashing console windows, and a live log panel that streams output in real time. It is minimal where it matters, powerful where you need it.

### ✨ Features

- **One-click model launching** — Start any configured model instantly. No more typing long command-line arguments.
- **Model library management** — Add, edit, and remove multiple model configurations with custom names, descriptions, and launch parameters.
- **Real-time log streaming** — Watch stdout/stderr in a live log panel with auto-scroll support. Every line arrives the moment it's written.
- **Zero-console process spawning** — Launches llama-server.exe silently (no flashing terminal window) thanks to Windows `CREATE_NO_WINDOW` flag.
- **Portable config** — All settings are stored in a `config.json` file alongside the executable. Just copy the folder and go.
- **Beautiful native UI** — Powered by [`egui`](https://github.com/emilk/egui), the UI is lightweight, fast, and feels right at home on any desktop.
- **Optimized build** — Compiled with LTO and size optimization (`opt-level = "s"`) for small binaries and fast startup.

### 📥 Get Started

The latest release is already out! Grab it here:

👉 **[Download v0.1.6 Release](https://github.com/rpaff/LlamaLaunch/releases/tag/v0.1.6)**

**Quick start:**
1. Download the latest release from the link above.
2. Extract the archive to any folder.
3. Open **Llama Launch**, click **Settings**.
4. Set the path to your `llama-server.exe`.
5. Add your model configs (name, description, launch args like `--model models/mymodel.gguf --ctx-size 4096`).
6. Click **Start** on any card and enjoy!

### 🛠️ Tech Stack

|| Component | Technology |
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

### Почему LlamaLaunch?

LlamaLaunch создан по одному принципу: **ничего лишнего между вами и моделью**. Никаких веб-панелей, облачных зависимостей, телеметрии или слежки. Это компактный нативный бинарник на Rust, который делает одну вещь — но делает отлично: запускает серверы llama.cpp именно так, как вам нужно. Интерфейс отбрасывает всё ненужное, чтобы вы видели только свои модели и кнопки Start/Stop. Но этот минимализм никогда не означает ограничений: каждая конфигурация модели принимает полностью произвольные аргументы запуска — настраивайте `--ctx-size`, `--gpu-layers`, `--tensor-split`, температуру или любой другой параметр, который поддерживает llama.cpp. Один конфиг на модель, всё хранится в одном `config.json` рядом с исполняемым файлом. Лаунчер помещается на флешку, запускает сервера без консольных окон и всплывающих терминалов, а панель логов транслирует вывод в реальном времени. Минимализм там, где это важно — мощь там, где она нужна.

### ✨ Возможности

- **Запуск моделей в один клик** — Запускайте любую настроенную модель мгновенно, без набора длинных аргументов командной строки.
- **Управление библиотекой моделей** — Добавляйте, редактируйте и удаляйте конфигурации моделей с произвольными именами, описаниями и параметрами запуска.
- **Трансляция логов в реальном времени** — Наблюдайте за stdout/stderr в панели логов с автопрокруткой. Каждая строка появляется мгновенно.
- **Запуск без консольного окна** — llama-server.exe запускается тихим способом (без выскакивающего окна терминала) благодаря флагу `CREATE_NO_WINDOW` в Windows.
- **Портативная конфигурация** — Все настройки хранятся в файле `config.json` рядом с исполняемым файлом. Просто скопируйте папку и перенесите.
- **Красивый нативный интерфейс** — Основан на [`egui`](https://github.com/emilk/egui), легковесном и быстром GUI-фреймворке, который ощущается родным на любом десктопе.
- **Оптимизированная сборка** — Компилируется с LTO и оптимизацией по размеру (`opt-level = "s"`) для компактного бинарника и быстрого старта.

### 📥 Начало работы

Последний релиз уже доступен! Скачайте здесь:

👉 **[Скачать релиз v0.1.6](https://github.com/rpaff/LlamaLaunch/releases/tag/v0.1.6)**

**Быстрый старт:**
1. Скачайте последний релиз по ссылке выше.
2. Распакуйте архив в любую папку.
3. Откройте **Llama Launch**, нажмите **Settings**.
4. Укажите путь к вашему `llama-server.exe`.
5. Добавьте конфигурации моделей (имя, описание, аргументы запуска вроде `--model models/mymodel.gguf --ctx-size 4096`).
6. Нажмите **Start** на любой карточке — и наслаждайтесь!

### 🛠️ Технологии

|| Компонент | Технология |
|-----------|------------|
| Язык | [Rust](https://www.rust-lang.org/) (Edition 2021) |
| GUI Фреймворк | [`eframe` / `egui`](https://github.com/emilk/egui) |
| Диалог файлов | [`rfd`](https://crates.io/crates/rfd) |
| Сериализация | [`serde` + `serde_json`](https://crates.io/crates/serde) |

### 📄 Лицензия

Проект распространяется под лицензией **MIT License**. Подробности в файле [LICENSE](LICENSE).  
Сделано с ❤️ пользователем [rpaff](https://github.com/rpaff).
