#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    Head,
    Chest,
    Legs,
    Feet,
    MainHand,
    OffHand,
    Backpack,
    Accessory1,
    Accessory2,
}

impl std::fmt::Display for EquipmentSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EquipmentSlot::Head => "头部",
            EquipmentSlot::Chest => "胸部",
            EquipmentSlot::Legs => "腿部",
            EquipmentSlot::Feet => "脚部",
            EquipmentSlot::MainHand => "主手",
            EquipmentSlot::OffHand => "副手",
            EquipmentSlot::Backpack => "背包",
            EquipmentSlot::Accessory1 => "饰品1",
            EquipmentSlot::Accessory2 => "饰品2",
        };
        write!(f, "{}", s)
    }
}

impl EquipmentSlot {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "head" | "头部" | "头" => Some(EquipmentSlot::Head),
            "chest" | "胸部" | "胸" => Some(EquipmentSlot::Chest),
            "legs" | "腿部" | "腿" => Some(EquipmentSlot::Legs),
            "feet" | "脚部" | "脚" => Some(EquipmentSlot::Feet),
            "mainhand" | "主手" => Some(EquipmentSlot::MainHand),
            "offhand" | "副手" => Some(EquipmentSlot::OffHand),
            "backpack" | "背包" => Some(EquipmentSlot::Backpack),
            "accessory1" | "饰品1" => Some(EquipmentSlot::Accessory1),
            "accessory2" | "饰品2" => Some(EquipmentSlot::Accessory2),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub max_hp_bonus: i32,
}

impl Stats {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn add(&self, other: &Stats) -> Stats {
        Stats {
            attack: self.attack + other.attack,
            defense: self.defense + other.defense,
            speed: self.speed + other.speed,
            max_hp_bonus: self.max_hp_bonus + other.max_hp_bonus,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.attack == 0 && self.defense == 0 && self.speed == 0 && self.max_hp_bonus == 0
    }
}

impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if self.attack != 0 {
            parts.push(format!("攻击{:+}", self.attack));
        }
        if self.defense != 0 {
            parts.push(format!("防御{:+}", self.defense));
        }
        if self.speed != 0 {
            parts.push(format!("速度{:+}", self.speed));
        }
        if self.max_hp_bonus != 0 {
            parts.push(format!("生命{:+}", self.max_hp_bonus));
        }
        if parts.is_empty() {
            write!(f, "无")
        } else {
            write!(f, "{}", parts.join(" "))
        }
    }
}

#[derive(Debug, Clone)]
pub struct EquipData {
    pub slot: EquipmentSlot,
    pub required_level: u32,
}

#[derive(Debug, Clone)]
pub struct SetEffect {
    pub piece_def_ids: Vec<String>,
    pub thresholds: Vec<(usize, Stats)>,
}
