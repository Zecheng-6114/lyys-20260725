use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

const COMMANDS: &[&str] = &[
    "help",
    "h",
    "?",
    "status",
    "st",
    "inventory",
    "inv",
    "i",
    "equipment",
    "eq",
    "search",
    "find",
    "s",
    "inspect",
    "ex",
    "examine",
    "equip",
    "unequip",
    "swapbackpack",
    "swapbp",
    "buy",
    "b",
    "sell",
    "list",
    "ls",
    "l",
    "open",
    "close",
    "take",
    "put",
    "contents",
    "talk",
    "speak",
    "npcs",
    "quit",
    "q",
    "exit",
];

const EQUIP_SLOTS: &[&str] = &[
    "head",
    "头部",
    "头",
    "chest",
    "胸部",
    "胸",
    "legs",
    "腿部",
    "腿",
    "feet",
    "脚部",
    "脚",
    "mainhand",
    "主手",
    "offhand",
    "副手",
    "accessory1",
    "饰品1",
    "accessory2",
    "饰品2",
];

const ITEM_TYPES: &[&str] = &[
    "weapon",
    "武器",
    "armor",
    "防具",
    "consumable",
    "消耗品",
    "material",
    "材料",
    "container",
    "容器",
    "quest",
    "任务物品",
    "misc",
    "杂物",
];

const RARITIES: &[&str] = &[
    "common",
    "普通",
    "uncommon",
    "优秀",
    "rare",
    "稀有",
    "epic",
    "史诗",
    "legendary",
    "传说",
];

const DIALOGUE_EXITS: &[&str] = &["q", "quit", "exit"];

#[derive(Default)]
pub struct CompletionSnapshot {
    pub backpack_items: Vec<String>,
    pub shop_items: Vec<String>,
    pub container_ids: Vec<String>,
    pub container_items: HashMap<String, Vec<String>>,
    pub npc_names: Vec<String>,
    pub all_item_names: Vec<String>,
    pub dialogue_choice_count: Option<usize>,
}

pub struct GameHelper {
    pub snapshot: CompletionSnapshot,
}

impl GameHelper {
    pub fn new() -> Self {
        Self {
            snapshot: CompletionSnapshot::default(),
        }
    }

    pub fn set_snapshot(&mut self, snapshot: CompletionSnapshot) {
        self.snapshot = snapshot;
    }
}

impl Completer for GameHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>)> {
        let snap = &self.snapshot;

        if let Some(n) = snap.dialogue_choice_count {
            return Ok(complete_dialogue(line, pos, n));
        }

        let before = &line[..pos];
        let (token_start, token) = current_token(before);
        let words: Vec<&str> = before.split_whitespace().collect();
        let ends_with_space = before.ends_with(char::is_whitespace);

        if words.is_empty() || (words.len() == 1 && !ends_with_space) {
            return Ok((token_start, filter_pairs(COMMANDS.iter().copied(), token)));
        }

        let verb = words[0].to_lowercase();
        let arg_index = if ends_with_space {
            words.len()
        } else {
            words.len().saturating_sub(1)
        };

        let candidates: Vec<String> = match verb.as_str() {
            "inspect" | "ex" | "examine" if arg_index == 1 => {
                merge_unique([&snap.backpack_items[..], &snap.all_item_names[..]])
            }
            "equip" | "sell" if arg_index == 1 => snap.backpack_items.clone(),
            "swapbackpack" | "swapbp" if arg_index == 1 => snap.backpack_items.clone(),
            "buy" | "b" if arg_index == 1 => snap.shop_items.clone(),
            "unequip" if arg_index == 1 => EQUIP_SLOTS.iter().map(|s| (*s).to_string()).collect(),
            "open" | "contents" if arg_index == 1 => snap.container_ids.clone(),
            "talk" | "speak" if arg_index == 1 => snap.npc_names.clone(),
            "take" if arg_index == 1 => snap.container_ids.clone(),
            "take" if arg_index == 2 => resolve_container_items(snap, words.get(1).copied().unwrap_or("")),
            "put" if arg_index == 1 => snap.container_ids.clone(),
            "put" if arg_index == 2 => snap.backpack_items.clone(),
            "search" | "find" | "s" => complete_search_args(),
            _ => Vec::new(),
        };

        Ok((
            token_start,
            filter_pairs(candidates.iter().map(|s| s.as_str()), token),
        ))
    }
}

impl Hinter for GameHelper {
    type Hint = String;
}

impl Highlighter for GameHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }
}

impl Validator for GameHelper {}

impl Helper for GameHelper {}

fn resolve_container_items(snap: &CompletionSnapshot, cid: &str) -> Vec<String> {
    if let Some(items) = snap.container_items.get(cid) {
        return items.clone();
    }
    snap.container_items
        .iter()
        .find(|(id, _)| id.eq_ignore_ascii_case(cid))
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

fn complete_dialogue(line: &str, pos: usize, choice_count: usize) -> (usize, Vec<Pair>) {
    let before = &line[..pos];
    let (token_start, token) = current_token(before);
    let mut cands: Vec<String> = (1..=choice_count).map(|i| i.to_string()).collect();
    cands.extend(DIALOGUE_EXITS.iter().map(|s| (*s).to_string()));
    (
        token_start,
        filter_pairs(cands.iter().map(|s| s.as_str()), token),
    )
}

fn complete_search_args() -> Vec<String> {
    let mut out = Vec::new();
    for t in ITEM_TYPES {
        out.push((*t).to_string());
        out.push(format!("type:{}", t));
    }
    for r in RARITIES {
        out.push((*r).to_string());
        out.push(format!("rarity:{}", r));
    }
    out
}

fn current_token(before: &str) -> (usize, &str) {
    let start = before
        .rfind(|c: char| c.is_whitespace())
        .map(|i| i + 1)
        .unwrap_or(0);
    (start, &before[start..])
}

fn filter_pairs<'a, I>(candidates: I, prefix: &str) -> Vec<Pair>
where
    I: Iterator<Item = &'a str>,
{
    let lower = prefix.to_lowercase();
    let mut seen = HashSet::new();
    let mut pairs = Vec::new();
    for c in candidates {
        if !c.to_lowercase().starts_with(&lower) {
            continue;
        }
        if !seen.insert(c.to_string()) {
            continue;
        }
        pairs.push(Pair {
            display: c.to_string(),
            replacement: c.to_string(),
        });
    }
    pairs.sort_by(|a, b| a.display.cmp(&b.display));
    pairs
}

fn merge_unique(slices: [&[String]; 2]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for slice in slices {
        for s in slice {
            if seen.insert(s.clone()) {
                out.push(s.clone());
            }
        }
    }
    out
}
