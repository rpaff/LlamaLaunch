use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Структура всей конфигурации.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Путь к llama-server.exe (абсолютный или относительный к exe).
    pub llama_server_path: String,
    /// Список моделей.
    pub models: Vec<crate::models::ModelConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            llama_server_path: "./llama-server.exe".to_string(),
            models: Vec::new(),
        }
    }
}

impl Config {
    /// Возвращает путь к config.json рядом с exe.
    pub fn config_path() -> Result<PathBuf> {
        let exe_dir =
            std::env::current_exe().context("Не удалось определить путь к исполняемому файлу")?;
        Ok(exe_dir
            .parent()
            .unwrap_or(&exe_dir)
            .to_path_buf()
            .join("config.json"))
    }

    /// Загружает конфигурацию из файла. Если файл не существует — создаёт дефолтную.
    pub fn load() -> Result<Config> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Не удалось прочитать файл: {}", path.display()))?;
            let config: Config = serde_json::from_str(&content)
                .with_context(|| format!("Не удалось распарсить JSON: {}", path.display()))?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Сохраняет конфигурацию в файл.
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content =
            serde_json::to_string_pretty(self).context("Не удалось сериализовать конфигурацию")?;
        fs::write(&path, &content)
            .with_context(|| format!("Не удалось записать файл: {}", path.display()))?;
        Ok(())
    }

    /// Путь к llama-server.exe, относительно директории exe если относительный.
    pub fn resolved_server_path(&self) -> Result<PathBuf> {
        let exe_dir =
            std::env::current_exe().context("Не удалось определить путь к исполняемому файлу")?;
        let exe_dir = exe_dir.parent().unwrap_or(&exe_dir);

        let path = Path::new(&self.llama_server_path);
        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            Ok(exe_dir.join(path))
        }
    }
}
