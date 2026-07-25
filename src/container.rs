use crate::error::GameError;
use crate::item::{ItemInstance, ItemRegistry};

pub trait Container {
    fn name(&self) -> &str;
    fn capacity(&self) -> usize;
    fn items(&self) -> &[ItemInstance];
    fn items_mut(&mut self) -> &mut Vec<ItemInstance>;

    fn used_space(&self) -> usize {
        self.items().len()
    }

    fn free_space(&self) -> usize {
        self.capacity().saturating_sub(self.used_space())
    }

    fn is_full(&self) -> bool {
        self.free_space() == 0
    }

    fn add(&mut self, item: ItemInstance) -> Result<(), GameError> {
        if self.is_full() {
            return Err(GameError::ContainerFull);
        }
        self.items_mut().push(item);
        Ok(())
    }

    fn remove_by_instance_id(&mut self, id: u64) -> Option<ItemInstance> {
        let pos = self.items().iter().position(|i| i.instance_id == id)?;
        Some(self.items_mut().remove(pos))
    }

    fn find_by_name<'a>(
        &'a self,
        name_fragment: &str,
        registry: &'a ItemRegistry,
    ) -> Vec<&'a ItemInstance> {
        let lower = name_fragment.to_lowercase();
        self.items()
            .iter()
            .filter(|item| {
                registry
                    .get(&item.def_id)
                    .map(|def| def.name.to_lowercase().contains(&lower))
                    .unwrap_or(false)
            })
            .collect()
    }
}

pub struct Backpack {
    pub item: ItemInstance,
    capacity: usize,
    container_name: String,
    contents: Vec<ItemInstance>,
}

impl Backpack {
    pub fn new(item: ItemInstance, registry: &ItemRegistry) -> Self {
        let (capacity, name) = registry
            .get(&item.def_id)
            .and_then(|def| {
                def.container.as_ref().map(|c| {
                    (c.capacity, def.name.clone())
                })
            })
            .unwrap_or((10, "背包".to_string()));

        Self {
            item,
            capacity,
            container_name: name,
            contents: Vec::new(),
        }
    }

    pub fn into_contents(self) -> Vec<ItemInstance> {
        self.contents
    }
}

impl Container for Backpack {
    fn name(&self) -> &str {
        &self.container_name
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn items(&self) -> &[ItemInstance] {
        &self.contents
    }

    fn items_mut(&mut self) -> &mut Vec<ItemInstance> {
        &mut self.contents
    }
}

pub struct Chest {
    pub id: String,
    pub container_name: String,
    capacity: usize,
    contents: Vec<ItemInstance>,
}

impl Chest {
    pub fn new(id: &str, name: &str, capacity: usize) -> Self {
        Self {
            id: id.to_string(),
            container_name: name.to_string(),
            capacity,
            contents: Vec::new(),
        }
    }
}

impl Container for Chest {
    fn name(&self) -> &str {
        &self.container_name
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn items(&self) -> &[ItemInstance] {
        &self.contents
    }

    fn items_mut(&mut self) -> &mut Vec<ItemInstance> {
        &mut self.contents
    }
}

