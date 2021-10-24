use chrono::prelude::DateTime;
use rss::{Channel, Item};
use std::error::Error;
use std::io::{stdout, Write};
use std::process::Command;

#[derive(serde::Deserialize, Debug)]
struct Config {
    urls: Vec<String>,
    cmd: Option<String>,
    args: Option<Vec<String>>,
    keywords: Option<Vec<String>>,
    timestamp: i64,
}

fn run_command(item: &Item, cmd: String, args: Option<Vec<String>>) {
    let mut process = Command::new(cmd);
    if args.is_some() {
        process.args(
            args.unwrap()
                .iter()
                .map(|arg| {
                    arg.replace("#TITLE", item.title().unwrap_or(""))
                        .replace("#LINK", item.link().unwrap_or(""))
                })
                .collect::<Vec<String>>(),
        );
    }

    stdout()
        .write_all(&process.output().expect("Error executing command").stdout)
        .ok();
}

async fn get_channel(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

async fn get_ch(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    return Ok(Channel::read_from(&content[..])?);
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = &mut envy::from_env::<Config>().unwrap();

    futures::future::join_all(config.urls.iter().map(|url| get_channel(url.as_str())))
        .await
        .iter()
        .for_each(|x| match x {
            Ok(channel) => {
                channel
                    .items()
                    .iter()
                    .filter(|&item| {
                        DateTime::parse_from_rfc2822(item.pub_date().unwrap())
                            .unwrap()
                            .timestamp()
                            > config.timestamp
                    })
                    .filter(|&item| {
                        if config.keywords.is_some() {
                            config.keywords.as_ref().unwrap().iter().any(|key| {
                                item.title()
                                    .unwrap()
                                    .to_lowercase()
                                    .contains(key.to_lowercase().as_str())
                            })
                        } else {
                            true
                        }
                    })
                    .for_each(|item| {
                        if config.cmd.is_some() {
                            run_command(
                                item,
                                config.cmd.as_ref().unwrap().to_string(),
                                config.args.as_ref().cloned(),
                            );
                        }
                    });
            }
            Err(e) => println!("Error: {}", e),
        });
}
