use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub nsec: String,
    pub relay_name: String,
    pub relay_url: String,
    pub custom_db_name: String,
}

pub fn load(file_path: &str) -> Result<Config> {
    let file_content = fs::read_to_string(file_path)
        .with_context(|| format!("設定ファイル {} の読み込みに失敗しました", file_path))?;
    let config: Config = serde_json::from_str(&file_content)
        .with_context(|| format!("設定ファイル {} の展開に失敗しました", file_path))?;
    Ok(config)
}
