mod util;

extern crate job_scheduler;

use anyhow::{Context, Result};
use chrono::{Timelike, Utc};
use core::result::Result::Ok;
use job_scheduler::{Job, JobScheduler};
use nostr_sdk::{prelude::*, Timestamp};
use std::time::Duration;

const MAX_LENGTH: usize = 1440;

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
    let config = util::config::load(config_path)?;
    let client = Client::default();
    client.add_relay(config.relay_url).await?;
    client.connect().await;
    let keys = Keys::from_sk_str(&config.nsec)
        .with_context(|| format!("次のキーは正常に使用できませんでした: {}", config.nsec))?;

    let mut sched = JobScheduler::new();

    // 集計処理実施
    let count = count(&client).await?;
    let db_tag = Tag::Generic(TagKind::D, vec![config.custom_db_name.clone()]);
    let content: String;
    if let Some(event) =
        get_app_specific_data(&client, &keys, config.custom_db_name.clone()).await?
    {
        let get_content = event.content.clone();
        let parsed_content = serde_json::from_str(&get_content);
        let chart = match parsed_content {
            Ok(Value::Array(arr)) => {
                let mut arr = arr;
                arr.push(json!(count));
                if arr.len() > MAX_LENGTH {
                    arr.drain(0..(arr.len() - MAX_LENGTH));
                }
                arr
            }
            _ => vec![json!(count)],
        };
        content = serde_json::to_string(&chart)?;
        println!("event = {:?}", event);
    } else {
        content = String::from("[]");
        println!("event none");
    }
    let new_event =
        EventBuilder::new(Kind::ApplicationSpecificData, content, [db_tag]).to_event(&keys)?;
    println!("{:?}", new_event);
    let _result = client.send_event(new_event).await?;

    match "0 * * * * *".parse() {
        Ok(cron) => {
            sched.add(Job::new(cron, || {
                println!("1 min!");
            }));
        }
        Err(e) => {
            eprintln!("failed: {}", e);
        }
    }

    loop {
        sched.tick();
        std::thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}
