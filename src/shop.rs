use crate::error::GameError;
use crate::item::{ItemDef, ItemRegistry};
use crate::player::Player;

pub struct ShopListing {
    pub def_id: String,
    pub stock: Option<u32>,
    pub price_override: Option<u32>,
}

pub struct BuyResult {
    pub item_name: String,
    pub actual_qty: u32,
    pub total_cost: u32,
}

pub struct SellResult {
    pub names: Vec<String>,
    pub qty_sold: usize,
    pub total_gold: u32,
}

pub struct Shop {
    pub name: String,
    pub listings: Vec<ShopListing>,
    pub buy_modifier: f32,
    pub sell_modifier: f32,
}

impl Shop {
    pub fn new(name: &str, buy_modifier: f32, sell_modifier: f32) -> Self {
        Self {
            name: name.to_string(),
            listings: Vec::new(),
            buy_modifier,
            sell_modifier,
        }
    }

    pub fn add_listing(&mut self, def_id: &str, stock: Option<u32>, price_override: Option<u32>) {
        self.listings.push(ShopListing {
            def_id: def_id.to_string(),
            stock,
            price_override,
        });
    }

    pub fn buy_price(&self, def: &ItemDef) -> u32 {
        let base = def.base_value as f32;
        (base * self.buy_modifier).ceil() as u32
    }

    pub fn sell_price(&self, def: &ItemDef) -> u32 {
        let base = def.base_value as f32;
        (base * self.sell_modifier).floor() as u32
    }

    pub fn list_items<'a>(&'a self, registry: &'a ItemRegistry) -> Vec<(&'a ItemDef, u32, Option<u32>)> {
        self.listings
            .iter()
            .filter_map(|listing| {
                let def = registry.get(&listing.def_id)?;
                let price = listing.price_override.unwrap_or_else(|| self.buy_price(def));
                Some((def, price, listing.stock))
            })
            .collect()
    }

    pub fn buy(
        &mut self,
        item_name: &str,
        quantity: u32,
        player: &mut Player,
        registry: &ItemRegistry,
    ) -> Result<BuyResult, GameError> {
        let lower = item_name.to_lowercase();

        let listing_idx = self
            .listings
            .iter()
            .position(|listing| {
                registry
                    .get(&listing.def_id)
                    .map(|def| def.name.to_lowercase().contains(&lower))
                    .unwrap_or(false)
            })
            .ok_or_else(|| GameError::ItemNotFound(item_name.to_string()))?;

        let listing = &self.listings[listing_idx];
        let def = registry
            .get(&listing.def_id)
            .ok_or_else(|| GameError::ItemNotFound(item_name.to_string()))?;

        if let Some(stock) = listing.stock {
            if stock == 0 {
                return Err(GameError::OutOfStock(def.name.clone()));
            }
            if stock < quantity {
                return Err(GameError::OutOfStock(format!(
                    "{} (库存: {})",
                    def.name, stock
                )));
            }
        }

        let unit_price = listing.price_override.unwrap_or_else(|| self.buy_price(def));
        let total_price = unit_price * quantity;

        if player.gold < total_price {
            return Err(GameError::NotEnoughGold {
                required: total_price,
                available: player.gold,
            });
        }

        let mut added = 0;
        for _ in 0..quantity {
            if let Some(instance) = registry.create_instance(&listing.def_id) {
                match player.add_to_backpack(instance) {
                    Ok(()) => added += 1,
                    Err(GameError::ContainerFull) => break,
                    Err(e) => return Err(e),
                }
            }
        }

        let actual_cost = unit_price * added;
        player.gold -= actual_cost;

        if let Some(ref mut stock) = self.listings[listing_idx].stock {
            *stock -= added as u32;
        }

        Ok(BuyResult {
            item_name: def.name.clone(),
            actual_qty: added,
            total_cost: actual_cost,
        })
    }

    pub fn sell(
        &mut self,
        item_name: &str,
        quantity: u32,
        player: &mut Player,
        registry: &ItemRegistry,
    ) -> Result<SellResult, GameError> {
        let lower = item_name.to_lowercase();

        let matches = player.find_in_backpack_by_name(&lower, registry);

        if matches.is_empty() {
            return Err(GameError::ItemNotFound(item_name.to_string()));
        }

        let qty_to_sell = (quantity as usize).min(matches.len());
        let instance_ids: Vec<u64> = matches.iter().take(qty_to_sell).map(|i| i.instance_id).collect();

        let mut total_gold = 0u32;
        let mut sold_names = Vec::new();

        for instance_id in instance_ids {
            if let Some(item) = player.remove_from_backpack(instance_id) {
                if let Some(def) = registry.get(&item.def_id) {
                    let price = self.sell_price(def);
                    total_gold += price;
                    if !sold_names.contains(&def.name) {
                        sold_names.push(def.name.clone());
                    }
                }
            }
        }

        player.gold += total_gold;

        Ok(SellResult {
            names: sold_names,
            qty_sold: qty_to_sell,
            total_gold,
        })
    }
}
