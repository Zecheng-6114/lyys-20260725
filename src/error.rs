use std::fmt;

#[derive(Debug)]
pub enum GameError {
    ContainerFull,
    ItemNotFound(String),
    InvalidCommand(String),
    NotEnoughGold { required: u32, available: u32 },
    ItemNotEquippable(String),
    InvalidTarget(String),
    OutOfStock(String),
    ContainerNotFound(String),
    NpcNotFound(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::ContainerFull => write!(f, "容器已满"),
            GameError::ItemNotFound(name) => write!(f, "找不到物品: {}", name),
            GameError::InvalidCommand(msg) => write!(f, "无效命令: {}", msg),
            GameError::NotEnoughGold { required, available } => {
                write!(f, "金币不足 (需要 {}, 拥有 {})", required, available)
            }
            GameError::ItemNotEquippable(name) => write!(f, "{} 不可装备", name),
            GameError::InvalidTarget(msg) => write!(f, "无效目标: {}", msg),
            GameError::OutOfStock(name) => write!(f, "商品已售罄: {}", name),
            GameError::ContainerNotFound(name) => write!(f, "找不到容器: {}", name),
            GameError::NpcNotFound(name) => write!(f, "找不到NPC: {}", name),
        }
    }
}
