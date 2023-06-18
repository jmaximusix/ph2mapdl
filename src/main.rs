use colored::Colorize;
use dialoguer::theme::Theme;
use dialoguer::Input;
use dialoguer::Select;
use dotenv::dotenv;
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::Url;
use serde_json::json;
use serde_json::Value;
// use std::collections::hash_map::RawEntryBuilder;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fmt::{Display, Formatter};
use std::vec;

const APP_ID: u32 = 1521580;
const BASE_URL: &str = "https://api.steampowered.com/";
const WORKSHOP_URL: &str = "https://steamcommunity.com/sharedfiles/filedetails/";
const MAX_LINE_LENGTH: usize = 100;

fn main() {
    dotenv().ok();
    let client = Client::new();
    let key = env::var("STEAM_API_KEY").expect("Steam API key not present");
    let installdir = env::var("INSTALLDIR").expect("No installation directory specified");
    let mut search = input_search(String::new());
    let mut params = HashMap::from([
        ("key", key.clone()),
        ("query_type", String::from("0")),
        ("numperpage", String::from("10")),
        ("appid", APP_ID.to_string()),
        ("search_text", search.clone()),
        ("cursor", String::from("*")),
        ("return_metadata", String::from("true")),
    ]);
    let new_search = true;
    loop {
        let response = &get_search_results(&client, params.clone())["response"];
        let maybe_entries = response.get("publishedfiledetails");
        if maybe_entries.is_none() {
            println!("No results found. Exiting.");
            search = input_search(search);
            continue;
        }
        let mut entries = maybe_entries.unwrap().as_array().unwrap().to_owned();
        // println!("{:?}", entries);
        let creator_ids = entries
            .iter()
            .map(|e| e["creator"].as_str().unwrap().parse().unwrap())
            .collect::<HashSet<_>>();
        let creators = get_creators(&client, &key, creator_ids);
        // println!("{:?}", creators);
        for entry in &mut entries {
            let creator_id = entry["creator"].as_str().unwrap().parse().unwrap();
            entry["creator_name"] = json!(&creators[&creator_id]);
        }
        let items: Vec<WorkshopItem> = entries.iter().map(WorkshopItem::new).collect();

        let selection = Select::with_theme(&ItemSelectorStyle)
            // .with_prompt("Select an item")
            .max_length(1)
            .default(0)
            .items(&items)
            .interact()
            .unwrap();
        // println!("{}", selection);
    }
}

struct ItemSelectorStyle;

impl Theme for ItemSelectorStyle {
    fn format_select_prompt_item(
        &self,
        f: &mut dyn std::fmt::Write,
        text: &str,
        _active: bool,
    ) -> std::fmt::Result {
        write!(f, "{text}")
    }
}

struct WorkshopItem {
    id: u64,
    title: String,
    creator: String,
    description: ItemDescription,
    favorited: u16,
    subscribed: u16,
}

impl WorkshopItem {
    fn new(entry: &Value) -> Self {
        // println!("{:?}", entry);
        Self {
            id: entry["publishedfileid"].as_str().unwrap().parse().unwrap(),
            title: entry["title"].as_str().unwrap().to_string(),
            creator: entry["creator_name"].as_str().unwrap().to_string(),
            description: ItemDescription {
                raw: entry["file_description"].as_str().unwrap().to_string(),
            },
            favorited: entry["favorited"].as_u64().unwrap() as u16,
            subscribed: entry["subscriptions"].as_u64().unwrap() as u16,
        }
    }
}

struct ItemDescription {
    raw: String,
}

impl Display for ItemDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let doublenl = Regex::new(r"\r?\n\r?\n").unwrap();
        let tags = Regex::new(r"\[/?[a-z0-9]+\]").unwrap();
        let leading_whitespace = Regex::new(r"^\s+").unwrap();
        let bullet_points = Regex::new(r"\[\*\]").unwrap();
        let linebreak_magic =
            Regex::new(&format!("(.{{1,{}}})(?:\\s+|$)", MAX_LINE_LENGTH)).unwrap();

        let first_two_paragraphs = doublenl
            .split(&self.raw)
            .take(2)
            .collect::<Vec<&str>>()
            .join("\n");

        let clean_text = tags.replace_all(&first_two_paragraphs, "");
        let clean_text = leading_whitespace.replace_all(&clean_text, "");
        let clean_text = bullet_points.replace_all(&clean_text, "  - ");
        let added_linebreaks = linebreak_magic.replace_all(&clean_text, "$1\n");
        write!(f, "{}", added_linebreaks.trim())
    }
}

impl Display for WorkshopItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let l = "\n----------------------------------------\n";
        let link = format!("{WORKSHOP_URL}?id={}", self.id);
        let hyperlink = format!(
            "\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
            link,
            self.title.blue().bold().underline()
        );
        write!(
            f,
            "{} {} {}{l}{}{l}📥 {}, ⭐ {}",
            hyperlink,
            String::from("by").cyan(),
            self.creator.cyan(),
            self.description,
            // "Hallo",
            self.subscribed,
            self.favorited,
        )
    }
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
        vec![("key", key), ("steamids", &ids_concat)],
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
