use rand::Rng;
use serde::{Deserialize, Serialize};
use std::process::Child;

/// Конфигурация одной модели (без процесса).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Уникальный идентификатор.
    pub id: String,
    /// Отображаемое название.
    pub name: String,
    /// Описание для отображения на карточке.
    pub description: String,
    /// Inline-строка параметров запуска.
    pub args: String,
}

impl ModelConfig {
    /// Генерирует новый уникальный ID.
    pub fn new_unique_id() -> String {
        let mut rng = rand::thread_rng();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        let random: u64 = rng.gen();
        format!("{:x}{:x}", timestamp, random)
    }
}

/// Обёртка над запущенным процессом.
pub struct RunningModel {
    /// Конфигурация модели.
    pub config: ModelConfig,
    /// Ссылка на дочерний процесс.
    pub child: Child,
}

impl RunningModel {
    /// Останавливает процесс.
    pub fn stop(&mut self) -> Result<(), std::io::Error> {
        self.child.kill()?;
        match self.child.try_wait() {
            Ok(Some(status)) => {
                eprintln!(
                    "Процесс {} завершён со статусом: {}",
                    self.config.name, status
                );
            }
            Ok(None) => {
                eprintln!(
                    "Не удалось завершить процесс {}, принудительное завершение.",
                    self.config.name
                );
            }
            Err(e) => {
                eprintln!(
                    "Ошибка при попытке завершить процесс {}: {}",
                    self.config.name, e
                );
            }
        }
        Ok(())
    }
}
