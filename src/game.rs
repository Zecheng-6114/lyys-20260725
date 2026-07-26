use std::collections::HashMap;

use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{Config, Editor};

use crate::command::Command;
use crate::completion::{CompletionSnapshot, GameHelper};
use crate::container::{Chest, Container};
use crate::data::GameData;
use crate::dialogue::{ActiveDialogue, DialogueDef, DialogueEffect, NpcDef};
use crate::error::GameError;
use crate::item::equipment::{EquipmentSlot, SetEffect};
use crate::item::{ItemInstance, ItemRegistry, ItemType, Rarity};
use crate::player::Player;
use crate::shop::Shop;

pub struct Game {
    pub player: Player,
    pub registry: ItemRegistry,
    pub current_shop: Option<Shop>,
    pub containers: Vec<Chest>,
    pub set_effects: Vec<SetEffect>,
    pub open_container: Option<String>,
    pub npcs: Vec<NpcDef>,
    pub dialogues: Vec<DialogueDef>,
    pub active_dialogue: Option<ActiveDialogue>,
    pub data: GameData,
}

impl Game {
    pub fn new(data: GameData) -> Self {
        let mut registry = ItemRegistry::new();

        for def in data.build_item_defs() {
            registry.register(def);
        }

        let player_name = data.msg("player_name");
        let player = Player::new(player_name, &registry);

        let current_shop = data.raw.shops.first().map(|raw_shop| {
            let mut shop = Shop::new(&raw_shop.name, raw_shop.buy_modifier, raw_shop.sell_modifier);
            for (def_id, &stock) in &raw_shop.stock {
                shop.add_listing(def_id, Some(stock), None);
            }
            shop
        });

        let containers = data
            .raw
            .containers
            .iter()
            .map(|rc| Chest::new(&rc.id, &rc.name, rc.capacity))
            .collect();

        let npcs = data.build_npc_defs();
        let dialogues = data.build_dialogue_defs();

        Self {
            player,
            registry,
            current_shop,
            containers,
            set_effects: Vec::new(),
            open_container: None,
            npcs,
            dialogues,
            active_dialogue: None,
            data,
        }
    }

    pub fn run(&mut self) {
        println!("=== {} ===", self.data.msg("game_title"));
        println!("{}\n", self.data.msg("game_help_hint"));

        let config = Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();
        let mut rl: Editor<GameHelper, DefaultHistory> =
            Editor::with_config(config).expect("初始化行编辑器失败");
        rl.set_helper(Some(GameHelper::new()));

        let prompt = format!("{} ", self.data.msg("prompt"));

        loop {
            self.refresh_completion(&mut rl);

            let line = match rl.readline(&prompt) {
                Ok(l) => l,
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => break,
                Err(_) => break,
            };

            let input = line.trim().to_string();
            if input.is_empty() {
                continue;
            }
            let _ = rl.add_history_entry(input.as_str());

            if self.active_dialogue.is_some() {
                if self.handle_dialogue_input(&input) {
                    break;
                }
                continue;
            }

            match Command::parse(&input, &self.data) {
                Ok(Command::Quit) => {
                    println!("{}", self.data.msg("goodbye"));
                    break;
                }
                Ok(cmd) => self.handle_command(cmd),
                Err(msg) => println!("{}", msg),
            }
        }
    }

    fn refresh_completion(&self, rl: &mut Editor<GameHelper, DefaultHistory>) {
        let snapshot = self.build_completion_snapshot();
        if let Some(helper) = rl.helper_mut() {
            helper.set_snapshot(snapshot);
        }
    }

    fn build_completion_snapshot(&self) -> CompletionSnapshot {
        let mut backpack_items = Vec::new();
        if let Some(bp) = &self.player.backpack {
            for item in bp.items() {
                if let Some(def) = self.registry.get(&item.def_id) {
                    if !backpack_items.iter().any(|n| n == &def.name) {
                        backpack_items.push(def.name.clone());
                    }
                }
            }
        }

        let mut shop_items = Vec::new();
        if let Some(shop) = &self.current_shop {
            for listing in &shop.listings {
                if let Some(def) = self.registry.get(&listing.def_id) {
                    shop_items.push(def.name.clone());
                }
            }
        }

        let mut container_ids = Vec::new();
        let mut container_items = HashMap::new();
        for c in &self.containers {
            container_ids.push(c.id.clone());
            let mut names = Vec::new();
            for item in c.items() {
                if let Some(def) = self.registry.get(&item.def_id) {
                    if !names.iter().any(|n| n == &def.name) {
                        names.push(def.name.clone());
                    }
                }
            }
            container_items.insert(c.id.clone(), names);
        }

        let npc_names = self.npcs.iter().map(|n| n.name.clone()).collect();

        let mut all_item_names: Vec<String> = self
            .data
            .raw
            .items
            .iter()
            .map(|i| i.name.clone())
            .collect();
        all_item_names.sort();
        all_item_names.dedup();

        let dialogue_choice_count = self.active_dialogue.as_ref().and_then(|active| {
            let dialogue = self.dialogues.iter().find(|d| d.id == active.dialogue_id)?;
            let node = dialogue
                .nodes
                .iter()
                .find(|n| n.id == active.current_node_id)?;
            Some(node.choices.len())
        });

        CompletionSnapshot {
            backpack_items,
            shop_items,
            container_ids,
            container_items,
            npc_names,
            all_item_names,
            dialogue_choice_count,
        }
    }

    fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::Help => self.cmd_help(),
            Command::Status => self.cmd_status(),
            Command::Inventory => self.cmd_inventory(),
            Command::Equipment => self.cmd_equipment(),
            Command::Search { name, item_type, rarity } => {
                self.cmd_search(name, item_type, rarity);
            }
            Command::Inspect(name) => self.cmd_inspect(&name),
            Command::Equip(name) => self.cmd_equip(&name),
            Command::Unequip(slot) => self.cmd_unequip(&slot),
            Command::SwapBackpack(name) => self.cmd_swap_backpack(&name),
            Command::Buy { item, quantity } => self.cmd_buy(&item, quantity),
            Command::Sell { item, quantity } => self.cmd_sell(&item, quantity),
            Command::ListShop => self.cmd_list_shop(),
            Command::Open(name) => self.cmd_open(&name),
            Command::Close => self.cmd_close(),
            Command::Take { container, item } => self.cmd_take(&container, &item),
            Command::Put { container, item } => self.cmd_put(&container, &item),
            Command::ContainerContents(name) => self.cmd_container_contents(&name),
            Command::Talk(name) => self.cmd_talk(&name),
            Command::ListNpcs => self.cmd_list_npcs(),
            Command::Quit => unreachable!(),
        }
    }

    fn format_error(&self, err: &GameError) -> String {
        let d = &self.data;
        match err {
            GameError::ContainerFull => d.err("container_full").to_string(),
            GameError::ItemNotFound(name) => d.err("item_not_found").replace("{name}", name),
            GameError::InvalidCommand(msg) => d.err("invalid_command").replace("{msg}", msg),
            GameError::NotEnoughGold { required, available } => {
                d.err("not_enough_gold")
                    .replace("{required}", &required.to_string())
                    .replace("{available}", &available.to_string())
            }
            GameError::ItemNotEquippable(name) => {
                d.err("not_equippable").replace("{name}", name)
            }
            GameError::InvalidTarget(msg) => {
                d.err("invalid_target").replace("{msg}", msg)
            }
            GameError::OutOfStock(name) => {
                if name.contains('(') {
                    let parts: Vec<&str> = name.splitn(2, '(').collect();
                    let n = parts[0].trim();
                    d.err("out_of_stock_with")
                        .replace("{name}", n)
                        .replace("{stock}", parts.get(1).unwrap_or(&""))
                } else {
                    d.err("out_of_stock").replace("{name}", name)
                }
            }
            GameError::ContainerNotFound(name) => {
                d.err("container_not_found").replace("{name}", name)
            }
            GameError::NpcNotFound(name) => {
                d.err("npc_not_found").replace("{name}", name)
            }
        }
    }

    fn cmd_help(&self) {
        println!("{}", self.data.raw.help.title);
        for line in &self.data.raw.help.lines {
            println!("{}", line);
        }
    }

    fn cmd_status(&self) {
        let d = &self.data;
        println!("=== {} ===", self.player.name);
        println!(
            "{}",
            d.raw.status.get("level_gold").unwrap()
                .replace("{level}", &self.player.level.to_string())
                .replace("{gold}", &self.player.gold.to_string())
        );
        println!(
            "{}",
            d.raw.status.get("hp").unwrap()
                .replace("{hp}", &self.player.hp.to_string())
                .replace("{max_hp}", &self.player.max_hp.to_string())
        );

        let stats = self.player.effective_stats(&self.registry, &self.set_effects);
        if !stats.is_empty() {
            println!("{}", d.raw.status.get("stats").unwrap().replace("{stats}", &stats.to_string()));
        }

        if let Some((used, cap)) = self.player.backpack_info() {
            println!(
                "{}",
                d.raw.status.get("backpack").unwrap()
                    .replace("{used}", &used.to_string())
                    .replace("{cap}", &cap.to_string())
            );
        } else {
            println!("{}", d.raw.status.get("no_backpack").unwrap());
        }
    }

    fn cmd_inventory(&self) {
        let d = &self.data;
        match &self.player.backpack {
            Some(bp) => {
                println!(
                    "=== {} ({}/{}) ===",
                    bp.name(),
                    bp.used_space(),
                    bp.capacity()
                );
                if bp.items().is_empty() {
                    println!("{}", d.raw.backpack_ui.get("empty").unwrap());
                } else {
                    let dur_fmt = d.raw.backpack_ui.get("durability").unwrap();
                    for item in bp.items() {
                        if let Some(def) = self.registry.get(&item.def_id) {
                            let durability_str = if def.max_durability > 0 {
                                dur_fmt
                                    .replace("{cur}", &item.durability.to_string())
                                    .replace("{max}", &def.max_durability.to_string())
                            } else {
                                String::new()
                            };
                            println!(
                                "  [{}] {} ({}){} - {} {}",
                                item.instance_id, def.name, def.rarity, durability_str, def.base_value,
                                d.msg("currency_unit")
                            );
                        }
                    }
                }
            }
            None => println!("{}", d.raw.backpack_ui.get("no_backpack").unwrap()),
        }

        if !self.player.loose_items.is_empty() {
            println!("\n{}", d.raw.backpack_ui.get("loose_title").unwrap());
            for item in &self.player.loose_items {
                if let Some(def) = self.registry.get(&item.def_id) {
                    println!("  [{}] {} - {}", item.instance_id, def.name, def.rarity);
                }
            }
        }
    }

    fn cmd_equipment(&self) {
        let d = &self.data;
        println!("{}", d.raw.equipment_ui.get("title").unwrap());
        let slots = [
            EquipmentSlot::Head,
            EquipmentSlot::Chest,
            EquipmentSlot::Legs,
            EquipmentSlot::Feet,
            EquipmentSlot::MainHand,
            EquipmentSlot::OffHand,
            EquipmentSlot::Accessory1,
            EquipmentSlot::Accessory2,
        ];

        let empty_str = d.raw.equipment_ui.get("empty").unwrap();
        for slot in &slots {
            let item_str = match self.player.equipment.get(slot) {
                Some(item) => {
                    if let Some(def) = self.registry.get(&item.def_id) {
                        format!("{} ({})", def.name, def.rarity)
                    } else {
                        "???".to_string()
                    }
                }
                None => empty_str.to_string(),
            };
            println!("  {}: {}", slot, item_str);
        }
    }

    fn cmd_search(
        &self,
        name: Option<String>,
        item_type: Option<ItemType>,
        rarity: Option<Rarity>,
    ) {
        let d = &self.data;
        let bp = match &self.player.backpack {
            Some(bp) => bp,
            None => {
                println!("{}", d.raw.search_ui.get("no_backpack").unwrap());
                return;
            }
        };

        let results: Vec<&ItemInstance> = bp
            .items()
            .iter()
            .filter(|item| {
                if let Some(def) = self.registry.get(&item.def_id) {
                    let name_match = name
                        .as_ref()
                        .map(|n| def.name.to_lowercase().contains(&n.to_lowercase()))
                        .unwrap_or(true);
                    let type_match = item_type.map(|t| def.item_type == t).unwrap_or(true);
                    let rarity_match = rarity.map(|r| def.rarity == r).unwrap_or(true);
                    name_match && type_match && rarity_match
                } else {
                    false
                }
            })
            .collect();

        if results.is_empty() {
            println!("{}", d.raw.search_ui.get("no_results").unwrap());
        } else {
            println!(
                "{}",
                d.raw.search_ui.get("result_header").unwrap().replace("{count}", &results.len().to_string())
            );
            for item in results {
                if let Some(def) = self.registry.get(&item.def_id) {
                    println!(
                        "  [{}] {} ({}, {}) - {} {}",
                        item.instance_id, def.name, def.item_type, def.rarity, def.base_value,
                        d.msg("currency_unit")
                    );
                }
            }
        }
    }

    fn cmd_inspect(&self, name: &str) {
        let matches = self.player.find_in_backpack_by_name(name, &self.registry);
        if let Some(item) = matches.first() {
            self.print_item_detail(item);
            return;
        }

        for (_, item) in self.player.equipment.all_equipped() {
            if let Some(def) = self.registry.get(&item.def_id) {
                if def.name.to_lowercase().contains(&name.to_lowercase()) {
                    self.print_item_detail(item);
                    return;
                }
            }
        }

        let defs = self.registry.find_by_name(name);
        if let Some(def) = defs.first() {
            self.print_def_detail(def);
        } else {
            println!("{}", self.data.raw.inspect_ui.get("not_found").unwrap().replace("{name}", name));
        }
    }

    fn print_item_detail(&self, item: &ItemInstance) {
        if let Some(def) = self.registry.get(&item.def_id) {
            let d = &self.data.raw.inspect_ui;
            println!("=== {} ===", def.name);
            println!("{}", def.description);
            println!("{}", d.get("type_rarity").unwrap()
                .replace("{type}", &def.item_type.to_string())
                .replace("{rarity}", &def.rarity.to_string()));
            println!("{}", d.get("instance_id").unwrap().replace("{id}", &item.instance_id.to_string()));
            println!("{}", d.get("base_price").unwrap()
                .replace("{price}", &def.base_value.to_string())
                .replace("{currency}", self.data.msg("currency_unit")));
            if def.max_durability > 0 {
                println!("{}", d.get("durability").unwrap()
                    .replace("{cur}", &item.durability.to_string())
                    .replace("{max}", &def.max_durability.to_string()));
            }
            if !def.stats.is_empty() {
                println!("{}", d.get("stats").unwrap().replace("{stats}", &def.stats.to_string()));
            }
            if let Some(equip) = &def.equip {
                println!("{}", d.get("equip_slot").unwrap()
                    .replace("{slot}", &equip.slot.to_string())
                    .replace("{level}", &equip.required_level.to_string()));
            }
            if let Some(container) = &def.container {
                println!("{}", d.get("capacity").unwrap()
                    .replace("{cap}", &container.capacity.to_string()));
            }
            if let Some(effect) = &def.effect {
                println!("{}", d.get("effect").unwrap().replace("{desc}", &effect.description));
            }
            if let Some(ref set_id) = def.set_id {
                println!("{}", d.get("set").unwrap().replace("{id}", set_id));
            }
        }
    }

    fn print_def_detail(&self, def: &crate::item::ItemDef) {
        let d = &self.data.raw.inspect_ui;
        println!("=== {} ===", def.name);
        println!("{}", def.description);
        println!("{}", d.get("type_rarity").unwrap()
            .replace("{type}", &def.item_type.to_string())
            .replace("{rarity}", &def.rarity.to_string()));
        println!("{}", d.get("base_price").unwrap()
            .replace("{price}", &def.base_value.to_string())
            .replace("{currency}", self.data.msg("currency_unit")));
        if !def.stats.is_empty() {
            println!("{}", d.get("stats").unwrap().replace("{stats}", &def.stats.to_string()));
        }
        if let Some(equip) = &def.equip {
            println!("{}", d.get("equip_slot").unwrap()
                .replace("{slot}", &equip.slot.to_string())
                .replace("{level}", &equip.required_level.to_string()));
        }
        if let Some(container) = &def.container {
            println!("{}", d.get("capacity").unwrap()
                .replace("{cap}", &container.capacity.to_string()));
        }
        if let Some(effect) = &def.effect {
            println!("{}", d.get("effect").unwrap().replace("{desc}", &effect.description));
        }
    }

    fn cmd_equip(&mut self, name: &str) {
        let d = &self.data;
        let matches = self.player.find_in_backpack_by_name(name, &self.registry);
        if matches.is_empty() {
            println!("{}", d.raw.equip_ui.get("not_found").unwrap().replace("{name}", name));
            return;
        }

        let instance_id = matches[0].instance_id;
        let def_id = matches[0].def_id.clone();

        if let Some(def) = self.registry.get(&def_id) {
            if def.equip.is_none() {
                println!("{}", d.raw.equip_ui.get("not_equippable").unwrap().replace("{name}", &def.name));
                return;
            }
            let equip_data = def.equip.as_ref().unwrap();
            if equip_data.slot == EquipmentSlot::Backpack {
                println!("{}", d.raw.equip_ui.get("use_swapbackpack").unwrap());
                return;
            }
            if self.player.level < equip_data.required_level {
                println!("{}", d.raw.equip_ui.get("need_level").unwrap()
                    .replace("{level}", &equip_data.required_level.to_string())
                    .replace("{name}", &def.name));
                return;
            }
        }

        let item = self.player.remove_from_backpack(instance_id).unwrap();
        let item_name = self.registry.get(&def_id).map(|d| d.name.clone()).unwrap_or_default();

        match self.player.equip_item(item, &self.registry) {
            Ok(Some(prev)) => {
                if let Some(def) = self.registry.get(&prev.def_id) {
                    println!("{} {}", d.raw.equip_ui.get("unequipped").unwrap(), def.name);
                }
                if let Err(_e) = self.player.add_to_backpack(prev) {
                    println!("{}", d.raw.equip_ui.get("old_backpack_fail").unwrap().replace("{err}", ""));
                }
            }
            Ok(None) => {}
            Err(_e) => {}
        }

        if !item_name.is_empty() {
            println!("{} {}", d.raw.equip_ui.get("equipped").unwrap(), item_name);
        }
    }

    fn cmd_unequip(&mut self, slot: &EquipmentSlot) {
        match self.player.unequip_item(slot) {
            Ok(item) => {
                let name = self.registry.get(&item.def_id).map(|d| d.name.as_str()).unwrap_or("???");
                println!("{} {}", self.data.raw.equip_ui.get("unequipped").unwrap(), name);
                if let Err(_e) = self.player.add_to_backpack(item) {
                    // Item dropped on ground
                }
            }
            Err(_e) => {
                println!("{}", self.data.raw.equip_ui.get("no_slot").unwrap());
            }
        }
    }

    fn cmd_swap_backpack(&mut self, name: &str) {
        let d = &self.data;
        let matches = self.player.find_in_backpack_by_name(name, &self.registry);
        let backpack_match = matches.iter().find(|item| {
            self.registry
                .get(&item.def_id)
                .map(|def| def.container.is_some())
                .unwrap_or(false)
        });

        let instance_id = match backpack_match {
            Some(item) => item.instance_id,
            None => {
                println!("{}", d.raw.swap_ui.get("not_found").unwrap().replace("{name}", name));
                return;
            }
        };

        let new_bp_item = self.player.remove_from_backpack(instance_id).unwrap();

        match self.player.swap_backpack(new_bp_item, &self.registry) {
            Ok(result) => {
                let (used, cap) = self.player.backpack_info().unwrap_or((0, 0));
                println!("{}", d.raw.swap_ui.get("replaced").unwrap()
                    .replace("{cap}", &cap.to_string())
                    .replace("{used}", &used.to_string()));

                if !result.overflow_items.is_empty() {
                    println!("{}", d.raw.swap_ui.get("overflow_header").unwrap());
                    for item in &result.overflow_items {
                        if let Some(def) = self.registry.get(&item.def_id) {
                            println!("  - {}", def.name);
                        }
                    }
                    self.player.loose_items.extend(result.overflow_items);
                }
            }
            Err(_e) => {}
        }
    }

    fn cmd_buy(&mut self, item_name: &str, quantity: u32) {
        if let Some(ref mut shop) = self.current_shop {
            match shop.buy(item_name, quantity, &mut self.player, &self.registry) {
                Ok(result) => {
                    let msg = if result.actual_qty < quantity {
                        self.data.raw.shop_ui.get("buy_partial").unwrap()
                    } else {
                        self.data.raw.shop_ui.get("buy_success").unwrap()
                    };
                    println!("{}", msg
                        .replace("{qty}", &result.actual_qty.to_string())
                        .replace("{name}", &result.item_name)
                        .replace("{cost}", &result.total_cost.to_string()));
                }
                Err(e) => println!("{}", self.format_error(&e)),
            }
        } else {
            println!("{}", self.data.err("not_in_shop"));
        }
    }

    fn cmd_sell(&mut self, item_name: &str, quantity: u32) {
        if let Some(ref mut shop) = self.current_shop {
            match shop.sell(item_name, quantity, &mut self.player, &self.registry) {
                Ok(result) => {
                    println!("{}", self.data.raw.shop_ui.get("sell_success").unwrap()
                        .replace("{qty}", &result.qty_sold.to_string())
                        .replace("{names}", &result.names.join(", "))
                        .replace("{gold}", &result.total_gold.to_string()));
                }
                Err(e) => println!("{}", self.format_error(&e)),
            }
        } else {
            println!("{}", self.data.err("not_in_shop"));
        }
    }

    fn cmd_list_shop(&self) {
        if let Some(ref shop) = self.current_shop {
            let d = &self.data;
            println!("=== {} ===", shop.name);
            println!("{}", d.raw.shop_ui.get("price_header").unwrap()
                .replace("{buy}", &format!("{:.0}", (shop.buy_modifier - 1.0) * 100.0))
                .replace("{sell}", &format!("{:.0}", (1.0 - shop.sell_modifier) * 100.0)));
            let items = shop.list_items(&self.registry);
            if items.is_empty() {
                println!("{}", d.raw.shop_ui.get("shop_empty").unwrap());
            } else {
                for (def, price, stock) in items {
                    let stock_str = match stock {
                        Some(s) => d.raw.shop_ui.get("stock_display").unwrap().replace("{stock}", &s.to_string()),
                        None => d.raw.shop_ui.get("stock_unlimited").unwrap().to_string(),
                    };
                    println!(
                        "  {} ({}) - {} {} [{}]",
                        def.name, def.rarity, price, d.msg("currency_unit"), stock_str
                    );
                }
            }
        } else {
            println!("{}", self.data.err("not_in_shop"));
        }
    }

    fn cmd_open(&mut self, name: &str) {
        let container = self.containers.iter().find(|c| c.id == name);
        match container {
            Some(c) => {
                let cname = c.container_name.clone();
                println!("{}", self.data.raw.container_ui.get("opened").unwrap().replace("{name}", &cname));
                self.open_container = Some(c.id.clone());
                self.cmd_container_contents(name);
            }
            None => println!("{}", self.format_error(&GameError::ContainerNotFound(name.to_string()))),
        }
    }

    fn cmd_close(&mut self) {
        if self.open_container.is_some() {
            println!("{}", self.data.raw.container_ui.get("closed").unwrap());
        } else {
            println!("{}", self.data.raw.container_ui.get("not_open").unwrap());
        }
        self.open_container = None;
    }

    fn cmd_take(&mut self, container_name: &str, item_name: &str) {
        let container = self.containers.iter_mut().find(|c| c.id == container_name);
        let container = match container {
            Some(c) => c,
            None => {
                println!("{}", self.format_error(&GameError::ContainerNotFound(container_name.to_string())));
                return;
            }
        };

        let matches = container.find_by_name(item_name, &self.registry);
        if matches.is_empty() {
            println!("{}", self.data.raw.container_ui.get("take_fail").unwrap().replace("{name}", item_name));
            return;
        }

        let instance_id = matches[0].instance_id;
        let def_id = matches[0].def_id.clone();
        drop(matches);

        let item = container.remove_by_instance_id(instance_id).unwrap();
        let def_name = self.registry.get(&item.def_id).map(|d| d.name.clone()).unwrap_or_default();

        match self.player.add_to_backpack(item) {
            Ok(()) => println!("{} {}", self.data.raw.container_ui.get("taken").unwrap(), def_name),
            Err(_e) => {
                let _ = container.add(ItemInstance::new(&def_id, 0));
            }
        }
    }

    fn cmd_put(&mut self, container_name: &str, item_name: &str) {
        let matches = self.player.find_in_backpack_by_name(item_name, &self.registry);
        if matches.is_empty() {
            println!("{}", self.data.raw.container_ui.get("put_fail").unwrap().replace("{name}", item_name));
            return;
        }

        let instance_id = matches[0].instance_id;
        let item = self.player.remove_from_backpack(instance_id).unwrap();
        let def_name = self.registry.get(&item.def_id).map(|d| d.name.clone()).unwrap_or_default();

        let container = self.containers.iter_mut().find(|c| c.id == container_name);
        match container {
            Some(c) => {
                if c.is_full() {
                    println!("{}", self.data.raw.container_ui.get("full").unwrap());
                    let _ = self.player.add_to_backpack(item);
                } else {
                    let cname = c.container_name.clone();
                    c.add(item).unwrap();
                    println!("{}", self.data.raw.container_ui.get("put_success").unwrap()
                        .replace("{name}", &def_name).replace("{container}", &cname));
                }
            }
            None => {
                println!("{}", self.format_error(&GameError::ContainerNotFound(container_name.to_string())));
                let _ = self.player.add_to_backpack(item);
            }
        }
    }

    fn cmd_container_contents(&self, name: &str) {
        let container = self.containers.iter().find(|c| c.id == name);
        match container {
            Some(c) => {
                println!(
                    "=== {} ({}/{}) ===",
                    c.container_name,
                    c.used_space(),
                    c.capacity()
                );
                if c.items().is_empty() {
                    println!("{}", self.data.raw.container_ui.get("empty").unwrap());
                } else {
                    for item in c.items() {
                        if let Some(def) = self.registry.get(&item.def_id) {
                            println!(
                                "  [{}] {} ({}) - {} {}",
                                item.instance_id, def.name, def.rarity, def.base_value,
                                self.data.msg("currency_unit")
                            );
                        }
                    }
                }
            }
            None => println!("{}", self.format_error(&GameError::ContainerNotFound(name.to_string()))),
        }
    }

    fn cmd_list_npcs(&self) {
        let d = &self.data;
        if self.npcs.is_empty() {
            println!("{}", d.raw.dialogue_ui.get("no_npcs").unwrap());
            return;
        }
        println!("{}", d.raw.dialogue_ui.get("npc_list_title").unwrap());
        for npc in &self.npcs {
            println!("  {} ({})", npc.name, npc.id);
        }
    }

    fn cmd_talk(&mut self, name: &str) {
        let lower = name.to_lowercase();
        let npc = match self.npcs.iter().find(|n| n.name.to_lowercase().contains(&lower)) {
            Some(n) => n,
            None => {
                println!("{}", self.format_error(&GameError::NpcNotFound(name.to_string())));
                return;
            }
        };

        let dialogue_id = npc.dialogue_id.clone();

        self.active_dialogue = Some(ActiveDialogue {
            dialogue_id,
            current_node_id: "start".to_string(),
        });

        self.display_dialogue_node();
    }

    fn display_dialogue_node(&self) {
        let active = self.active_dialogue.as_ref().unwrap();
        let dialogue = match self.dialogues.iter().find(|d| d.id == active.dialogue_id) {
            Some(d) => d,
            None => return,
        };

        let node = match dialogue.nodes.iter().find(|n| n.id == active.current_node_id) {
            Some(n) => n,
            None => return,
        };

        let d = &self.data;
        println!("\n=== {} ===", dialogue.npc_name);
        println!("\"{}\"", node.text);

        if node.choices.is_empty() {
            println!("{}", d.raw.dialogue_ui.get("no_choices").unwrap());
            return;
        }

        for (i, choice) in node.choices.iter().enumerate() {
            println!("  {}. {}", i + 1, choice.text);
        }
    }

    fn handle_dialogue_input(&mut self, input: &str) -> bool {
        let input = input.trim();

        if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            println!("{}", self.data.raw.dialogue_ui.get("dialogue_end").unwrap());
            self.active_dialogue = None;
            return false;
        }

        let choice_idx = match input.parse::<usize>() {
            Ok(n) => n,
            Err(_) => {
                println!("{}", self.data.raw.dialogue_ui.get("invalid_input").unwrap());
                return false;
            }
        };

        if choice_idx == 0 {
            println!("{}", self.data.raw.dialogue_ui.get("invalid_input").unwrap());
            return false;
        }

        let active = self.active_dialogue.as_ref().unwrap();
        let dialogue_id = active.dialogue_id.clone();
        let node_id = active.current_node_id.clone();

        let dialogue = match self.dialogues.iter().find(|d| d.id == dialogue_id) {
            Some(d) => d,
            None => {
                self.active_dialogue = None;
                return false;
            }
        };

        let node = match dialogue.nodes.iter().find(|n| n.id == node_id) {
            Some(n) => n,
            None => {
                self.active_dialogue = None;
                return false;
            }
        };

        if choice_idx > node.choices.len() {
            println!("{}", self.data.raw.dialogue_ui.get("invalid_input").unwrap());
            return false;
        }

        let choice = &node.choices[choice_idx - 1];
        let effects = choice.effects.clone();
        let next = choice.next.clone();

        self.apply_dialogue_effects(&effects);

        match next {
            Some(next_id) => {
                self.active_dialogue.as_mut().unwrap().current_node_id = next_id.clone();
                self.display_dialogue_node();

                let dialogue = self.dialogues.iter().find(|d| d.id == dialogue_id).unwrap();
                let next_node = dialogue.nodes.iter().find(|n| n.id == next_id).unwrap();
                if next_node.choices.is_empty() {
                    println!("{}", self.data.raw.dialogue_ui.get("dialogue_end").unwrap());
                    self.active_dialogue = None;
                }
            }
            None => {
                println!("{}", self.data.raw.dialogue_ui.get("dialogue_end").unwrap());
                self.active_dialogue = None;
            }
        }

        false
    }

    fn apply_dialogue_effects(&mut self, effects: &[DialogueEffect]) {
        for effect in effects {
            match effect {
                DialogueEffect::GiveGold(amount) => {
                    self.player.gold += amount;
                    println!(
                        "{}",
                        self.data.raw.dialogue_ui.get("received_gold").unwrap()
                            .replace("{amount}", &amount.to_string())
                    );
                }
                DialogueEffect::GiveItem { def_id, quantity } => {
                    for _ in 0..*quantity {
                        if let Some(instance) = self.registry.create_instance(def_id) {
                            let name = self.registry.get(def_id).map(|d| d.name.clone()).unwrap_or_default();
                            if let Err(_) = self.player.add_to_backpack(instance) {
                                println!(
                                    "{}",
                                    self.data.raw.dialogue_ui.get("backpack_full").unwrap()
                                );
                                break;
                            }
                            println!(
                                "{}",
                                self.data.raw.dialogue_ui.get("received_item").unwrap()
                                    .replace("{name}", &name)
                            );
                        }
                    }
                }
                DialogueEffect::TakeGold(amount) => {
                    if self.player.gold >= *amount {
                        self.player.gold -= amount;
                    }
                }
                DialogueEffect::TakeItem { def_id, quantity } => {
                    let matches = self.player.find_in_backpack_by_name(def_id, &self.registry);
                    let to_take = (*quantity as usize).min(matches.len());
                    let ids: Vec<u64> = matches.iter().take(to_take).map(|i| i.instance_id).collect();
                    for id in ids {
                        self.player.remove_from_backpack(id);
                    }
                }
            }
        }
    }
}
