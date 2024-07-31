use chrono::Local;
use colored::*;

pub enum LogCategory {
    Mining,
    Transaction,
    BlockCreation,
    ChainValidation,
    General,
    Error,
}

pub struct Logger;

impl Logger {
    pub fn log(category: LogCategory, message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let (category_str, color) = match category {
            LogCategory::Mining => ("MINING", Color::Magenta),
            LogCategory::Transaction => ("TRANSACTION", Color::Green),
            LogCategory::BlockCreation => ("BLOCK", Color::Cyan),
            LogCategory::ChainValidation => ("VALIDATION", Color::Yellow),
            LogCategory::General => ("INFO", Color::White),
            LogCategory::Error => ("ERROR", Color::Red),
        };

        println!(
            "{} [{}] {}",
            timestamp.color(Color::Blue),
            category_str.color(color).bold(),
            message.color(color)
        );
    }

    pub fn mining(message: &str) {
        Self::log(LogCategory::Mining, message);
    }

    pub fn transaction(message: &str) {
        Self::log(LogCategory::Transaction, message);
    }

    pub fn block(message: &str) {
        Self::log(LogCategory::BlockCreation, message);
    }

    pub fn validation(message: &str) {
        Self::log(LogCategory::ChainValidation, message);
    }

    pub fn info(message: &str) {
        Self::log(LogCategory::General, message);
    }

    pub fn error(message: &str) {
        Self::log(LogCategory::Error, message);
    }
}