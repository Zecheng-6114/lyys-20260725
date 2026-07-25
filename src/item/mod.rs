pub mod equipment;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use equipment::{EquipData, Stats};

static NEXT_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Rarity::Common => "普通",
            Rarity::Uncommon => "优秀",
            Rarity::Rare => "稀有",
            Rarity::Epic => "史诗",
            Rarity::Legendary => "传说",
        };
        write!(f, "{}", s)
    }
}

impl Rarity {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "common" | "普通" => Some(Rarity::Common),
            "uncommon" | "优秀" => Some(Rarity::Uncommon),
            "rare" | "稀有" => Some(Rarity::Rare),
            "epic" | "史诗" => Some(Rarity::Epic),
            "legendary" | "传说" => Some(Rarity::Legendary),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Weapon,
    Armor,
    Consumable,
    Material,
    Container,
    Quest,
    Misc,
}

impl std::fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ItemType::Weapon => "武器",
            ItemType::Armor => "防具",
            ItemType::Consumable => "消耗品",
            ItemType::Material => "材料",
            ItemType::Container => "容器",
            ItemType::Quest => "任务物品",
            ItemType::Misc => "杂物",
        };
        write!(f, "{}", s)
    }
}

impl ItemType {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "weapon" | "武器" => Some(ItemType::Weapon),
            "armor" | "防具" => Some(ItemType::Armor),
            "consumable" | "消耗品" => Some(ItemType::Consumable),
            "material" | "材料" => Some(ItemType::Material),
            "container" | "容器" => Some(ItemType::Container),
            "quest" | "任务物品" => Some(ItemType::Quest),
            "misc" | "杂物" => Some(ItemType::Misc),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ContainerDef {
    pub capacity: usize,
}

#[derive(Debug, Clone)]
pub struct ItemEffect {
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub item_type: ItemType,
    pub rarity: Rarity,
    pub base_value: u32,
    pub max_durability: u32,
    pub stats: Stats,
    pub equip: Option<EquipData>,
    pub container: Option<ContainerDef>,
    pub effect: Option<ItemEffect>,
    pub set_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ItemInstance {
    pub instance_id: u64,
    pub def_id: String,
    pub durability: u32,
}

impl ItemInstance {
    pub fn new(def_id: &str, max_durability: u32) -> Self {
        Self {
            instance_id: NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed),
            def_id: def_id.to_string(),
            durability: max_durability,
        }
    }
}

pub struct ItemRegistry {
    defs: HashMap<String, ItemDef>,
}

impl ItemRegistry {
    pub fn new() -> Self {
        Self {
            defs: HashMap::new(),
        }
    }

    pub fn register(&mut self, def: ItemDef) {
        self.defs.insert(def.id.clone(), def);
    }

    pub fn get(&self, def_id: &str) -> Option<&ItemDef> {
        self.defs.get(def_id)
    }

    pub fn create_instance(&self, def_id: &str) -> Option<ItemInstance> {
        self.defs.get(def_id).map(|def| {
            ItemInstance::new(def_id, def.max_durability)
        })
    }

    pub fn find_by_name(&self, query: &str) -> Vec<&ItemDef> {
        let lower = query.to_lowercase();
        self.defs
            .values()
            .filter(|def| def.name.to_lowercase().contains(&lower))
            .collect()
    }
}
