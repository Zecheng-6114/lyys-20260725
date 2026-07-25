use std::collections::HashMap;

use crate::container::{Backpack, Container};
use crate::error::GameError;
use crate::item::equipment::{EquipmentSlot, SetEffect, Stats};
use crate::item::{ItemInstance, ItemRegistry};

pub struct Equipment {
    slots: HashMap<EquipmentSlot, ItemInstance>,
}

impl Equipment {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    pub fn equip(&mut self, slot: EquipmentSlot, item: ItemInstance) -> Option<ItemInstance> {
        self.slots.insert(slot, item)
    }

    pub fn unequip(&mut self, slot: &EquipmentSlot) -> Option<ItemInstance> {
        self.slots.remove(slot)
    }

    pub fn get(&self, slot: &EquipmentSlot) -> Option<&ItemInstance> {
        self.slots.get(slot)
    }

    pub fn all_equipped(&self) -> impl Iterator<Item = (&EquipmentSlot, &ItemInstance)> {
        self.slots.iter()
    }
}

pub struct SwapResult {
    pub overflow_items: Vec<ItemInstance>,
}

pub struct Player {
    pub name: String,
    pub gold: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub level: u32,
    pub equipment: Equipment,
    pub backpack: Option<Backpack>,
    pub loose_items: Vec<ItemInstance>,
}

impl Player {
    pub fn new(name: &str, registry: &ItemRegistry) -> Self {
        let mut player = Self {
            name: name.to_string(),
            gold: 100,
            hp: 100,
            max_hp: 100,
            level: 1,
            equipment: Equipment::new(),
            backpack: None,
            loose_items: Vec::new(),
        };

        // Give player a starter backpack
        if let Some(instance) = registry.create_instance("small_backpack") {
            player.backpack = Some(Backpack::new(instance, registry));
        }

        player
    }

    pub fn add_to_backpack(&mut self, item: ItemInstance) -> Result<(), GameError> {
        match &mut self.backpack {
            Some(bp) => bp.add(item),
            None => {
                self.loose_items.push(item);
                Ok(())
            }
        }
    }

    pub fn remove_from_backpack(&mut self, instance_id: u64) -> Option<ItemInstance> {
        if let Some(bp) = &mut self.backpack {
            bp.remove_by_instance_id(instance_id)
        } else {
            None
        }
    }

    pub fn find_in_backpack_by_name<'a>(
        &'a self,
        name: &str,
        registry: &'a ItemRegistry,
    ) -> Vec<&'a ItemInstance> {
        match &self.backpack {
            Some(bp) => bp.find_by_name(name, registry),
            None => vec![],
        }
    }

    pub fn swap_backpack(
        &mut self,
        new_backpack_item: ItemInstance,
        registry: &ItemRegistry,
    ) -> Result<SwapResult, GameError> {
        let def = registry
            .get(&new_backpack_item.def_id)
            .ok_or_else(|| GameError::ItemNotFound(new_backpack_item.def_id.clone()))?;

        if def.container.is_none() {
            return Err(GameError::InvalidTarget(format!(
                "{} 不是容器",
                def.name
            )));
        }

        let mut new_bp = Backpack::new(new_backpack_item, registry);
        let mut overflow = Vec::new();

        if let Some(old_bp) = self.backpack.take() {
            let old_item = old_bp.item.clone();

            for item in old_bp.into_contents() {
                if new_bp.free_space() > 0 {
                    new_bp.add(item).unwrap();
                } else {
                    overflow.push(item);
                }
            }

            // Try to put old backpack item into new backpack
            if new_bp.free_space() > 0 {
                new_bp.add(old_item).unwrap();
            } else {
                overflow.push(old_item);
            }
        }

        self.backpack = Some(new_bp);
        Ok(SwapResult {
            overflow_items: overflow,
        })
    }

    pub fn equip_item(
        &mut self,
        item: ItemInstance,
        registry: &ItemRegistry,
    ) -> Result<Option<ItemInstance>, GameError> {
        let def = registry
            .get(&item.def_id)
            .ok_or_else(|| GameError::ItemNotFound(item.def_id.clone()))?;

        let equip_data = def
            .equip
            .as_ref()
            .ok_or_else(|| GameError::ItemNotEquippable(def.name.clone()))?;

        if equip_data.slot == EquipmentSlot::Backpack {
            return Err(GameError::InvalidCommand(
                "使用 'swapbackpack' 来更换背包".to_string(),
            ));
        }

        if self.level < equip_data.required_level {
            return Err(GameError::InvalidTarget(format!(
                "需要等级 {} 才能装备 {}",
                equip_data.required_level, def.name
            )));
        }

        let prev = self.equipment.equip(equip_data.slot, item);
        Ok(prev)
    }

    pub fn unequip_item(&mut self, slot: &EquipmentSlot) -> Result<ItemInstance, GameError> {
        self.equipment
            .unequip(slot)
            .ok_or_else(|| GameError::ItemNotFound(format!("该槽位没有装备")))
    }

    pub fn effective_stats(&self, registry: &ItemRegistry, set_effects: &[SetEffect]) -> Stats {
        let mut total = Stats::zero();

        for (_, item) in self.equipment.all_equipped() {
            if let Some(def) = registry.get(&item.def_id) {
                total = total.add(&def.stats);
            }
        }

        // Apply set bonuses
        for set in set_effects {
            let equipped_count = self
                .equipment
                .all_equipped()
                .filter(|(_, item)| set.piece_def_ids.contains(&item.def_id))
                .count();
            for (threshold, bonus) in &set.thresholds {
                if equipped_count >= *threshold {
                    total = total.add(bonus);
                }
            }
        }

        total
    }

    pub fn backpack_info(&self) -> Option<(usize, usize)> {
        self.backpack.as_ref().map(|bp| (bp.used_space(), bp.capacity()))
    }
}
