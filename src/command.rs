use crate::data::GameData;
use crate::item::equipment::EquipmentSlot;
use crate::item::{ItemType, Rarity};

#[derive(Debug)]
pub enum Command {
    Help,
    Status,
    Inventory,
    Equipment,
    Search {
        name: Option<String>,
        item_type: Option<ItemType>,
        rarity: Option<Rarity>,
    },
    Inspect(String),
    Equip(String),
    Unequip(EquipmentSlot),
    SwapBackpack(String),
    Buy {
        item: String,
        quantity: u32,
    },
    Sell {
        item: String,
        quantity: u32,
    },
    ListShop,
    Open(String),
    Close,
    Take {
        container: String,
        item: String,
    },
    Put {
        container: String,
        item: String,
    },
    ContainerContents(String),
    Quit,
}

impl Command {
    pub fn parse(input: &str, data: &GameData) -> Result<Command, String> {
        let input = input.trim();
        if input.is_empty() {
            return Err(data.err("empty_input").to_string());
        }

        let parts: Vec<&str> = input.splitn(2, char::is_whitespace).collect();
        let verb = parts[0].to_lowercase();
        let rest = parts.get(1).map(|s| s.trim()).filter(|s| !s.is_empty());

        match verb.as_str() {
            "help" | "h" | "?" => Ok(Command::Help),
            "status" | "st" => Ok(Command::Status),
            "inventory" | "inv" | "i" => Ok(Command::Inventory),
            "equipment" | "eq" => Ok(Command::Equipment),
            "search" | "find" | "s" => {
                let query = Self::parse_search(rest);
                Ok(Command::Search {
                    name: query.0,
                    item_type: query.1,
                    rarity: query.2,
                })
            }
            "inspect" | "ex" | "examine" => {
                let name = rest.ok_or_else(|| data.err("inspect_what").to_string())?;
                Ok(Command::Inspect(name.to_string()))
            }
            "equip" => {
                let name = rest.ok_or_else(|| data.err("equip_what").to_string())?;
                Ok(Command::Equip(name.to_string()))
            }
            "unequip" => {
                let slot_str = rest.ok_or_else(|| data.err("unequip_what").to_string())?;
                let slot = EquipmentSlot::parse(slot_str)
                    .ok_or_else(|| data.err("unknown_slot").replace("{slot}", slot_str))?;
                Ok(Command::Unequip(slot))
            }
            "swapbackpack" | "swapbp" => {
                let name = rest.ok_or_else(|| data.err("swap_what").to_string())?;
                Ok(Command::SwapBackpack(name.to_string()))
            }
            "buy" | "b" => {
                let (item, qty) = Self::parse_item_quantity(rest, data)?;
                Ok(Command::Buy { item, quantity: qty })
            }
            "sell" => {
                let (item, qty) = Self::parse_item_quantity(rest, data)?;
                Ok(Command::Sell { item, quantity: qty })
            }
            "list" | "ls" | "l" => Ok(Command::ListShop),
            "open" => {
                let name = rest.ok_or_else(|| data.err("open_what").to_string())?;
                Ok(Command::Open(name.to_string()))
            }
            "close" => Ok(Command::Close),
            "take" => {
                let (container, item) = Self::parse_container_item(rest, data)?;
                Ok(Command::Take { container, item })
            }
            "put" => {
                let (container, item) = Self::parse_container_item(rest, data)?;
                Ok(Command::Put { container, item })
            }
            "contents" => {
                let name = rest.ok_or_else(|| data.err("contents_what").to_string())?;
                Ok(Command::ContainerContents(name.to_string()))
            }
            "quit" | "q" | "exit" => Ok(Command::Quit),
            _ => Err(data.err("unknown_command").replace("{cmd}", &verb)),
        }
    }

    fn parse_item_quantity(
        rest: Option<&str>,
        data: &GameData,
    ) -> Result<(String, u32), String> {
        let rest = rest.ok_or_else(|| data.err("specify_item").to_string())?;

        let tokens: Vec<&str> = rest.rsplitn(2, char::is_whitespace).collect();
        if tokens.len() == 2 {
            if let Ok(qty) = tokens[0].parse::<u32>() {
                return Ok((tokens[1].trim().to_string(), qty));
            }
        }

        Ok((rest.to_string(), 1))
    }

    fn parse_container_item(
        rest: Option<&str>,
        data: &GameData,
    ) -> Result<(String, String), String> {
        let rest = rest.ok_or_else(|| data.err("take_format").to_string())?;

        let parts: Vec<&str> = rest.splitn(2, char::is_whitespace).collect();
        if parts.len() < 2 {
            return Err(data.err("take_format").to_string());
        }

        Ok((parts[0].to_string(), parts[1].trim().to_string()))
    }

    fn parse_search(
        rest: Option<&str>,
    ) -> (Option<String>, Option<ItemType>, Option<Rarity>) {
        let rest = match rest {
            Some(r) => r,
            None => return (None, None, None),
        };

        let mut name_parts = Vec::new();
        let mut item_type = None;
        let mut rarity = None;

        for token in rest.split_whitespace() {
            if let Some(val) = token.strip_prefix("type:") {
                if let Some(t) = ItemType::parse(val) {
                    item_type = Some(t);
                    continue;
                }
            }
            if let Some(val) = token.strip_prefix("rarity:") {
                if let Some(r) = Rarity::parse(val) {
                    rarity = Some(r);
                    continue;
                }
            }

            if let Some(t) = ItemType::parse(token) {
                item_type = Some(t);
                continue;
            }
            if let Some(r) = Rarity::parse(token) {
                rarity = Some(r);
                continue;
            }

            name_parts.push(token);
        }

        let name = if name_parts.is_empty() {
            None
        } else {
            Some(name_parts.join(" "))
        };

        (name, item_type, rarity)
    }
}
