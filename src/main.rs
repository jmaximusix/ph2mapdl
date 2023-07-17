use clap::Parser;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use dotenv::dotenv;
use std::fs;
// use ini::Ini;
use reqwest::blocking::Client;
use reqwest::Url;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env;
use std::process::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    start_search: Vec<String>,
    #[arg(long, short, default_value_t = 5)]
    page_size: usize,
    #[arg(long)]
    no_action: bool,
    #[arg(long)]
    update: bool,
}

use ph2mapdl::{ItemSelectorStyle, WorkshopItem, APP_ID, BASE_URL};

type Params = HashMap<&'static str, String>;

extern "C" fn handler() {
    println!("{}", "\n\x1b[0mReceived Ctrl-C. Exiting.".bright_red());
    std::process::exit(1);
}

fn main() {
    dotenv().ok();
    let cli = Cli::parse();
    #[cfg(not(windows))]
    unsafe {
        libc::signal(libc::SIGINT, handler as usize)
    };
    #[cfg(windows)]
    unsafe {
        libc::signal(libc::CTRL_C_EVENT, handler as usize)
    };

    let client = Client::new();
    let key = env::var("STEAM_API_KEY").expect("Steam API key not present");
    let installdir = env::var("INSTALLDIR").expect("No installation directory specified");
    let mut params: Params = HashMap::from([
        ("key", key.clone()),
        // ("query_type", String::from("0")),
        ("numperpage", cli.page_size.to_string()),
        ("appid", APP_ID.to_string()),
        ("search_text", String::new()),
        ("cursor", String::from("*")),
        ("return_metadata", String::from("true")),
    ]);
    if cli.start_search.is_empty() {
        new_search(&mut params);
    } else {
        let start_search = cli.start_search.join(" ");
        params.insert("search_text", start_search);
    }
    loop {
        let Some(items) = get_search_results(&client, &key, &installdir, &mut params) else {
            println!("No results found. Adjust your search:");
            new_search(&mut params);
            continue;
        };
        let selection = Select::with_theme(&ItemSelectorStyle)
            //.with_prompt("\n")
            .max_length(1)
            .default(0) // important so you can hit enter on the first one
            .items(&items)
            .interact_opt()
            .unwrap();

        if let Some(index) = selection {
            process_selection(&items[index], &installdir, &cli);
            break;
        } else {
            match what_to_do() {
                0 => break,
                1 => continue,
                2 => new_search(&mut params),
                _ => unreachable!(),
            }
        }
    }
    println!("{}", "Done!\n".cyan());
}

fn what_to_do() -> usize {
    let choices = &["Quit", "Load more results", "Modify search"];
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What do you want to do?")
        .default(0)
        .items(choices)
        .interact_opt()
        .unwrap()
        .unwrap_or(0)
}

fn get_search_results(
    client: &Client,
    key: &String,
    install_dir: &str,
    params: &mut Params,
) -> Option<Vec<WorkshopItem>> {
    let response = &search_workshop(client, params)["response"];
    let next_cursor = response["next_cursor"].as_str().unwrap().to_string();
    params.insert("cursor", next_cursor);
    let entries = response
        .get("publishedfiledetails")?
        .as_array()
        .unwrap()
        .to_owned();
    let creator_ids = entries
        .iter()
        .map(|e| e["creator"].as_str().unwrap().parse().unwrap())
        .collect::<HashSet<_>>();
    let creators = get_creators(client, key, creator_ids);
    let items: Vec<WorkshopItem> = entries
        .iter()
        .map(|e| WorkshopItem::new(install_dir, &creators, e))
        .collect();
    Some(items)
}

fn process_selection(item: &WorkshopItem, install_dir: &str, cli: &Cli) {
    println!(
        "{item}\n\n{} {}",
        format!("Selected Map: {}", item.title).green(),
        format!("({})", item.id).black()
    );
    if cli.no_action {
        println!("{}", "No action taken. Exiting.".yellow());
        std::process::exit(0);
    }
    if item.installed && !(cli.update) {
        println!("{}", "Map already installed.".green().dimmed());
    } else {
        if !(item.installed) {
            println!("{}", "Map not installed. Downloading now...".purple());
        } else {
            println!("{}", "Reinstalling/Updating Map...".purple());
        }
        download_map(item.id, install_dir)
    }
    select_map(item.id, install_dir);
}

fn download_map(id: u64, install_dir: &str) {
    Command::new("steamcmd")
        .args([
            "+force_install_dir",
            install_dir,
            "+login anonymous",
            "+workshop_download_item",
            APP_ID.to_string().as_str(),
            id.to_string().as_str(),
            "+quit",
        ])
        .status()
        .unwrap();
    println!("Map downloaded successfully.");
}

fn select_map(id: u64, install_dir: &str) {
    println!("Updating config to selected map...");
    let config_path = format!("{install_dir}/PerfectHeist2/Saved/Config/LinuxServer/Game.ini");

    let file_content = fs::read_to_string(&config_path).expect("Could not read Config file");
    let updated_lines: Vec<String> = file_content
        .lines()
        .map(|line| {
            if let Some((key, _value)) = line.split_once('=') {
                let trimmed_key = key.trim();
                match trimmed_key {
                    "WorkshopMapID" => format!("{}={}", trimmed_key, id),
                    "WorkshopFolderFullPath" => format!(
                        "{}={}",
                        trimmed_key,
                        format!("{}/steamapps/workshop/content/{APP_ID}/{}", install_dir, id)
                    ),
                    _ => line.to_string(),
                }
            } else {
                line.to_string()
            }
        })
        .collect();
    fs::write(config_path, updated_lines.join("\n")).unwrap();
    //
    // Not using ini crate because it doesn't preserve comments
    //
    // let mut config = Ini::load_from_file(&config_path).unwrap();
    // config.set_to(
    //     Some("Advanced"),
    //     "WorkshopMapID".to_string(),
    //     id.to_string(),
    // );
    // config.set_to(
    //     Some("Advanced"),
    //     "WorkshopFolderFullPath".to_string(),
    //     format!("{}/steamapps/workshop/content/{APP_ID}/{}", install_dir, id),
    // );
    // config.write_to_file(config_path).unwrap();
}

fn new_search(params: &mut Params) {
    let search = input_search(params.get("search_text").unwrap());
    params.insert("search_text", search);
    params.insert("cursor", String::from("*"));
}

fn input_search(old: &String) -> String {
    println!("\x1b[1m\x1b[4m");
    let answer: String = Input::new()
        .with_prompt("Search")
        .with_initial_text(old)
        .interact_text()
        .unwrap();
    println!("\x1b[0m");
    answer
}

fn search_workshop(client: &Client, params: &Params) -> Value {
    let url = Url::parse_with_params(
        &format!("{BASE_URL}IPublishedFileService/QueryFiles/v1/"),
        params,
    )
    .unwrap();
    client.get(url).send().unwrap().json::<Value>().unwrap()
}

fn get_creators(client: &Client, key: &String, ids: HashSet<u64>) -> HashMap<u64, String> {
    let ids_concat = ids
        .iter()
        .map(u64::to_string)
        .collect::<Vec<String>>()
        .join(",");

    let url = Url::parse_with_params(
        &format!("{BASE_URL}ISteamUser/GetPlayerSummaries/v2/"),
        [("key", key), ("steamids", &ids_concat)],
    )
    .unwrap();
    let response = &client.get(url).send().unwrap().json::<Value>().unwrap()["response"];
    response
        .get("players")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["steamid"].as_str().unwrap().parse().unwrap(),
                e["personaname"].as_str().unwrap().to_string(),
            )
        })
        .collect::<HashMap<u64, String>>()
}
