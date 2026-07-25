use std::collections::HashMap;

use serde::Deserialize;

use crate::dialogue::{
    DialogueChoice, DialogueDef, DialogueEffect, DialogueNode, NpcDef,
};
use crate::item::equipment::{EquipData, EquipmentSlot, Stats};
use crate::item::{ContainerDef, ItemDef, ItemEffect, ItemType, Rarity};

#[derive(Debug, Deserialize)]
pub struct RawData {
    pub messages: HashMap<String, String>,
    pub help: HelpData,
    pub status: HashMap<String, String>,
    pub backpack_ui: HashMap<String, String>,
    pub equipment_ui: HashMap<String, String>,
    pub search_ui: HashMap<String, String>,
    pub shop_ui: HashMap<String, String>,
    pub container_ui: HashMap<String, String>,
    pub equip_ui: HashMap<String, String>,
    pub swap_ui: HashMap<String, String>,
    pub inspect_ui: HashMap<String, String>,
    pub dialogue_ui: HashMap<String, String>,
    pub errors: HashMap<String, String>,
    pub items: Vec<RawItemDef>,
    pub shops: Vec<RawShopDef>,
    pub containers: Vec<RawContainerDef>,
    pub npcs: Vec<RawNpcDef>,
    pub dialogues: Vec<RawDialogueDef>,
}

#[derive(Debug, Deserialize)]
pub struct HelpData {
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawItemDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub item_type: String,
    pub rarity: String,
    pub base_value: u32,
    pub max_durability: u32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub max_hp_bonus: i32,
    pub equip_slot: Option<String>,
    pub required_level: Option<u32>,
    pub container_capacity: Option<usize>,
    pub heal_hp: Option<i32>,
    pub effect_desc: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawShopDef {
    pub name: String,
    pub buy_modifier: f32,
    pub sell_modifier: f32,
    pub stock: HashMap<String, u32>,
}

#[derive(Debug, Deserialize)]
pub struct RawContainerDef {
    pub id: String,
    pub name: String,
    pub capacity: usize,
}

#[derive(Debug, Deserialize)]
pub struct RawNpcDef {
    pub id: String,
    pub name: String,
    pub dialogue: String,
}

#[derive(Debug, Deserialize)]
pub struct RawDialogueDef {
    pub id: String,
    pub npc_name: String,
    pub nodes: Vec<RawDialogueNode>,
}

#[derive(Debug, Deserialize)]
pub struct RawDialogueNode {
    pub id: String,
    pub text: String,
    pub choices: Vec<RawDialogueChoice>,
}

#[derive(Debug, Deserialize)]
pub struct RawDialogueChoice {
    pub text: String,
    pub next: Option<String>,
    #[serde(default)]
    pub effects: Vec<RawDialogueEffect>,
}

#[derive(Debug, Deserialize)]
pub struct RawDialogueEffect {
    pub effect_type: String,
    pub value: Option<u32>,
    pub item_id: Option<String>,
    pub quantity: Option<u32>,
}

pub struct GameData {
    pub raw: RawData,
}

impl GameData {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("无法读取数据文件 '{}': {}", path, e))?;
        let raw: RawData =
            toml::from_str(&content).map_err(|e| format!("数据文件解析错误: {}", e))?;
        Ok(Self { raw })
    }

    pub fn msg<'a>(&'a self, key: &'a str) -> &'a str {
        self.raw
            .messages
            .get(key)
            .map(|s| s.as_str())
            .unwrap_or(key)
    }

    pub fn err<'a>(&'a self, key: &'a str) -> &'a str {
        self.raw
            .errors
            .get(key)
            .map(|s| s.as_str())
            .unwrap_or(key)
    }

    pub fn build_item_defs(&self) -> Vec<ItemDef> {
        self.raw
            .items
            .iter()
            .map(|raw| {
                let item_type = match raw.item_type.as_str() {
                    "weapon" => ItemType::Weapon,
                    "armor" => ItemType::Armor,
                    "consumable" => ItemType::Consumable,
                    "material" => ItemType::Material,
                    "container" => ItemType::Container,
                    "quest" => ItemType::Quest,
                    _ => ItemType::Misc,
                };

                let rarity = match raw.rarity.as_str() {
                    "common" => Rarity::Common,
                    "uncommon" => Rarity::Uncommon,
                    "rare" => Rarity::Rare,
                    "epic" => Rarity::Epic,
                    "legendary" => Rarity::Legendary,
                    _ => Rarity::Common,
                };

                let stats = Stats {
                    attack: raw.attack,
                    defense: raw.defense,
                    speed: raw.speed,
                    max_hp_bonus: raw.max_hp_bonus,
                };

                let equip = raw.equip_slot.as_ref().and_then(|slot_str| {
                    let slot = match slot_str.as_str() {
                        "head" => EquipmentSlot::Head,
                        "chest" => EquipmentSlot::Chest,
                        "legs" => EquipmentSlot::Legs,
                        "feet" => EquipmentSlot::Feet,
                        "mainhand" => EquipmentSlot::MainHand,
                        "offhand" => EquipmentSlot::OffHand,
                        "backpack" => EquipmentSlot::Backpack,
                        "accessory1" => EquipmentSlot::Accessory1,
                        "accessory2" => EquipmentSlot::Accessory2,
                        _ => return None,
                    };
                    Some(EquipData {
                        slot,
                        required_level: raw.required_level.unwrap_or(1),
                    })
                });

                let container = raw
                    .container_capacity
                    .map(|cap| ContainerDef { capacity: cap });

                let effect = raw.heal_hp.map(|hp| ItemEffect {
                    description: raw
                        .effect_desc
                        .clone()
                        .unwrap_or_else(|| format!("恢复 {} HP", hp)),
                });

                ItemDef {
                    id: raw.id.clone(),
                    name: raw.name.clone(),
                    description: raw.description.clone(),
                    item_type,
                    rarity,
                    base_value: raw.base_value,
                    max_durability: raw.max_durability,
                    stats,
                    equip,
                    container,
                    effect,
                    set_id: None,
                }
            })
            .collect()
    }

    pub fn build_npc_defs(&self) -> Vec<NpcDef> {
        self.raw
            .npcs
            .iter()
            .map(|raw| NpcDef {
                id: raw.id.clone(),
                name: raw.name.clone(),
                dialogue_id: raw.dialogue.clone(),
            })
            .collect()
    }

    pub fn build_dialogue_defs(&self) -> Vec<DialogueDef> {
        self.raw
            .dialogues
            .iter()
            .map(|raw| {
                let nodes = raw
                    .nodes
                    .iter()
                    .map(|raw_node| {
                        let choices = raw_node
                            .choices
                            .iter()
                            .map(|raw_choice| {
                                let effects = raw_choice
                                    .effects
                                    .iter()
                                    .filter_map(|raw_effect| {
                                        match raw_effect.effect_type.as_str() {
                                            "give_gold" => {
                                                Some(DialogueEffect::GiveGold(raw_effect.value.unwrap_or(0)))
                                            }
                                            "give_item" => {
                                                Some(DialogueEffect::GiveItem {
                                                    def_id: raw_effect.item_id.clone().unwrap_or_default(),
                                                    quantity: raw_effect.quantity.unwrap_or(1),
                                                })
                                            }
                                            "take_gold" => {
                                                Some(DialogueEffect::TakeGold(raw_effect.value.unwrap_or(0)))
                                            }
                                            "take_item" => {
                                                Some(DialogueEffect::TakeItem {
                                                    def_id: raw_effect.item_id.clone().unwrap_or_default(),
                                                    quantity: raw_effect.quantity.unwrap_or(1),
                                                })
                                            }
                                            _ => None,
                                        }
                                    })
                                    .collect();
                                DialogueChoice {
                                    text: raw_choice.text.clone(),
                                    next: raw_choice.next.clone(),
                                    effects,
                                }
                            })
                            .collect();
                        DialogueNode {
                            id: raw_node.id.clone(),
                            text: raw_node.text.clone(),
                            choices,
                        }
                    })
                    .collect();
                DialogueDef {
                    id: raw.id.clone(),
                    npc_name: raw.npc_name.clone(),
                    nodes,
                }
            })
            .collect()
    }
}
