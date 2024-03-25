use clap::{arg, Command};
use colored::*;
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use std::{fs, io::Read};

#[derive(Debug, Deserialize)]
struct AppConfig {
    api_key: String,
    server_url: String,
}

fn cli() -> Command {
    Command::new("fallo")
        .about("client for the fallo server - a redirect service")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("new")
                .about("Adds a new redirect")
                .arg(arg!(<LINK> "full link to redirect to"))
                .arg(arg!(<SHORT> "short part after the URL")),
        )
        .subcommand(Command::new("list").about("Gets a list of all redirects"))
        .subcommand(
            Command::new("delete")
                .alias("rm")
                .about("Removes a redirect")
                .arg(arg!(<SHORT> "short part after the URL")),
        )
}

fn main() {
    let home = home::home_dir()
        .expect("Unable to find your home directory")
        .to_str()
        .unwrap()
        .to_owned();

    let Ok(config_text) = fs::read_to_string(format!("{home}/.config/fallo/config.toml")) else {
        println!(
            "Unable to read config file! Create a new one at `$HOME/.config/fallo/config.toml`"
        );
        return;
    };

    let Ok(config) = toml::from_str::<AppConfig>(&config_text) else {
        println!("Unable to parse config file! Make sure it contains everything it should.");
        return;
    };

    let AppConfig {
        api_key,
        server_url,
    } = config;

    let matches = cli().get_matches();
    let client = Client::new();

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let link = sub_matches
                .get_one::<String>("LINK")
                .expect("required")
                .to_owned();
            let short = sub_matches.get_one::<String>("SHORT").expect("required");

            let response = client
                .post(format!("{server_url}/{short}"))
                .header("x-api-key", api_key)
                .body(link)
                .send()
                .expect("Unable to send request")
                .status();

            match response {
                StatusCode::OK => println!("OK!"),
                _ => println!(":("),
            }
        }
        Some(("list", _)) => {
            let Ok(mut response) = client
                .get(format!("{server_url}/"))
                .header("x-api-key", api_key)
                .send()
            else {
                println!(":(");
                return;
            };

            let mut json_str = String::new();
            response.read_to_string(&mut json_str).unwrap();

            let json: Value = serde_json::from_str(&json_str).unwrap();

            if let Value::Object(map) = json {
                for (key, value) in map.iter() {
                    println!(
                        "{} -> {}",
                        key.green(),
                        value.to_string().trim_matches('"').yellow()
                    );
                }
            } else {
                println!("Invalid JSON format. Expected JSON object.");
            }
        }
        Some(("delete", sub_matches)) => {
            let short = sub_matches.get_one::<String>("SHORT").expect("required");

            let response = client
                .delete(format!("{server_url}/{short}"))
                .header("x-api-key", api_key)
                .send()
                .expect("Unable to send request")
                .status();

            match response {
                StatusCode::OK => println!("OK!"),
                _ => println!(":("),
            }
        }
        _ => unreachable!(),
    };
}
