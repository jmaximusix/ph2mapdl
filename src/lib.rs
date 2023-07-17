use colored::Colorize;
use dialoguer::theme::Theme;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub const APP_ID: u32 = 1521580;
pub const BASE_URL: &str = "https://api.steampowered.com/";
const WORKSHOP_URL: &str = "https://steamcommunity.com/sharedfiles/filedetails/";
const MAX_LINE_LENGTH: usize = 100;

pub struct ItemSelectorStyle;

impl Theme for ItemSelectorStyle {
    fn format_select_prompt_item(
        &self,
        f: &mut dyn std::fmt::Write,
        text: &str,
        _active: bool,
    ) -> std::fmt::Result {
        write!(
            f,
            "{}\n\n{text}",
            "Select Map using arrow keys. Enter to select. Esc for more options".bright_black()
        )
    }
}

pub struct WorkshopItem {
    pub id: u64,
    pub title: String,
    pub installed: bool,
    creator: String,
    description: ItemDescription,
    favorited: u16,
    subscribed: u16,
}

impl WorkshopItem {
    pub fn new(installdir: &str, creators: &HashMap<u64, String>, entry: &Value) -> Self {
        let creator_name = &creators[&entry["creator"].as_str().unwrap().parse().unwrap()];
        let mid = entry["publishedfileid"].as_str().unwrap().parse().unwrap();
        let path = format!("{}/steamapps/workshop/content/{APP_ID}/{}", installdir, mid);
        Self {
            id: mid,
            title: entry["title"].as_str().unwrap().to_string(),
            creator: creator_name.clone(),
            description: ItemDescription {
                raw: entry["file_description"].as_str().unwrap().to_string(),
            },
            favorited: entry["favorited"].as_u64().unwrap() as u16,
            subscribed: entry["subscriptions"].as_u64().unwrap() as u16,
            installed: std::path::Path::new(&path).exists(),
        }
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
        let installed_status = String::from(if self.installed { " [installed]" } else { "" });
        write!(
            f,
            "{} {} {}{}{l}{}{l}📥 {}, ⭐ {}",
            hyperlink,
            String::from("by").cyan(),
            self.creator.cyan(),
            installed_status.bright_green(),
            self.description,
            self.subscribed,
            self.favorited,
        )
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
