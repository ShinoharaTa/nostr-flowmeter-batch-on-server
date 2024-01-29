use anyhow::{Context, Ok, Result};
use chrono::{Timelike, Utc};
use nostr_sdk::{prelude::*, Timestamp};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct Config {
    nsec: String,
    relay_name: String,
    relay_url: String,
    custom_db_name: String,
}

fn load_config(file_path: &str) -> Result<Config> {
    let file_content = fs::read_to_string(file_path)
        .with_context(|| format!("設定ファイル {} の読み込みに失敗しました", file_path))?;
    let config: Config = serde_json::from_str(&file_content)
        .with_context(|| format!("設定ファイル {} の展開に失敗しました", file_path))?;
    Ok(config)
}

async fn count(client: &Client) -> Result<Number> {
    let now = Utc::now();
    let until = now.with_second(0).with_context(|| format!("失敗"))?;
    let minutes = Duration::from_secs(60);
    let since = until - minutes;
    let filters = Filter::new()
        .kind(Kind::TextNote)
        .since(Timestamp::from(since.timestamp() as u64))
        .until(Timestamp::from(until.timestamp() as u64))
        .limit(10000);
    let events = client
        .get_events_of(vec![filters], Some(Duration::from_secs(20)))
        .await
        .with_context(|| format!("イベントの取得に失敗しました。"))?;
    println!("events: {:?}", events.len());
    Ok(Number::from(events.len()))
}

async fn get_app_specific_data(
    client: &Client,
    keys: &Keys,
    table_name: String,
) -> Result<Option<Event>> {
    let filters = Filter::new()
        .author(keys.public_key())
        .kind(Kind::ApplicationSpecificData)
        .custom_tag(Alphabet::D, [table_name])
        .limit(10);
    let events = client
        .get_events_of(vec![filters], Some(Duration::from_secs(20)))
        .await
        .with_context(|| format!("イベントの取得に失敗しました。"))?;
    if let Some(event) = events.get(0) {
        return Ok(Some(event.clone()));
    } else {
        return Ok(None);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = "./config.json";
    let config = load_config(config_path)?;
    let client = Client::default();
    client.add_relay(config.relay_url).await?;
    // .with_context(|| format!("リレーの接続に失敗しました: {}", config.relay_url))?;
    client.connect().await;
    let keys = Keys::from_sk_str(&config.nsec)
        .with_context(|| format!("次のキーは正常に使用できませんでした: {}", config.nsec))?;

    // 集計処理実施
    let count = count(&client).await?;
    // match get_app_specific_data(&client, &keys, config.custom_db_name).await {
    //     Ok(Some(event)) => {
    //         // event あり
    //         println!("{:?}", event);
    //     }
    //     Ok(None) => {
    //         // event なし
    //         println!("イベントなし");
    //     }
    // }
    if let Some(event) = get_app_specific_data(&client, &keys, config.custom_db_name).await? {
        println!("event = {:?}", event);
    } else {
        println!("none??")
    }

    Ok(())
}
