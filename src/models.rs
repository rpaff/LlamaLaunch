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

/// Запись лога с временной меткой.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Timestamp вида "HH:MM:SS".
    pub timestamp: String,
    /// Текст сообщения из процесса.
    pub message: String,
}

impl LogEntry {
    /// Создаёт запись с текущим локальным временем.
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            timestamp: local_hhmmss(),
            message,
        }
    }

    /// Полностью отформатированная строка для рендера в UI.
    pub fn display_line(&self) -> String {
        format!("[{}] {}", self.timestamp, self.message)
    }
}

#[cfg(target_os = "windows")]
fn local_hhmmss() -> String {
    #[repr(C)]
    struct SystemTime {
        w_year: u16,
        w_month: u16,
        w_day_of_week: u16,
        w_day: u16,
        w_hour: u16,
        w_minute: u16,
        w_second: u16,
        w_wMilliseconds: u16,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetLocalTime(lpSystemTime: *mut SystemTime);
    }

    let mut st: SystemTime = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut st) };
    format!(
        "{:02}:{:02}:{:02}",
        st.w_hour, st.w_minute, st.w_second
    )
}

#[cfg(not(target_os = "windows"))]
fn local_hhmmss() -> String {
    let now_utc = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now_utc.as_secs();
    let hours = (secs / 3600) % 24;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    format!("{hours:02}:{mins:02}:{secs:02}")
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
