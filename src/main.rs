use dialoguer::Input;
use dialoguer::Select;
use dotenv::dotenv;
use reqwest::blocking::Client;
use reqwest::Url;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env;

use ph2mapdl::{ItemSelectorStyle, WorkshopItem, APP_ID, BASE_URL};

fn main() {
    dotenv().ok();
    let client = Client::new();
    let key = env::var("STEAM_API_KEY").expect("Steam API key not present");
    let installdir = env::var("INSTALLDIR").expect("No installation directory specified");
    let mut search = input_search(String::new());
    let mut params = HashMap::from([
        ("key", key.clone()),
        // ("query_type", String::from("0")),
        ("numperpage", String::from("5")),
        ("appid", APP_ID.to_string()),
        ("search_text", search.clone()),
        ("cursor", String::from("*")),
        ("return_metadata", String::from("true")),
    ]);
    loop {
        let response = &get_search_results(&client, params.clone())["response"];
        println!("{:?}", response["next_cursor"]);
        let maybe_entries = response.get("publishedfiledetails");
        if maybe_entries.is_none() {
            println!("No results found.");
            search = input_search(search);
            params.insert("search_text", search.clone());
            continue;
        }
        let entries = maybe_entries.unwrap().as_array().unwrap().to_owned();
        // println!("{:?}", entries);
        let creator_ids = entries
            .iter()
            .map(|e| e["creator"].as_str().unwrap().parse().unwrap())
            .collect::<HashSet<_>>();
        let creators = get_creators(&client, &key, creator_ids);
        // println!("{:?}", creators);
        let items: Vec<WorkshopItem> = entries
            .iter()
            .map(|e| WorkshopItem::new(&installdir, &creators, e))
            .collect();

        let selection = Select::with_theme(&ItemSelectorStyle)
            // .with_prompt("Select an item")
            .max_length(1)
            .default(0) // important so you can hit enter on the first one
            .items(&items)
            .interact_opt()
            .unwrap();

        if let Some(index) = selection {
            process_selection(&items[index]);
            break;
        } else {
            println!("Nothing selected");
        }
    }
    println!("Done. Exiting.");
}

fn process_selection(item: &WorkshopItem) {
    println!("Selected: {}", item.id);
}

fn input_search(old: String) -> String {
    println!("\x1b[1m\x1b[4m");
    let answer: String = Input::new()
        .with_prompt("Search")
        .with_initial_text(old)
        .interact_text()
        .unwrap();
    print!("\x1b[0m");
    answer
}

fn get_search_results(client: &Client, params: HashMap<&str, String>) -> Value {
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
