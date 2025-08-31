use std::collections::HashMap;

use crate::{
    controller::{DrawContext, EventContext, UpdateResult},
    entity::{Entity, FullEntity, HasProperties, Named},
};

pub struct BaseEntity<T: Entity> {
    name: String,
    properties: std::collections::HashMap<String, String>,
    delegate_entity: T,
}

impl<T: Entity> Named for BaseEntity<T> {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

impl<T: Entity> HasProperties for BaseEntity<T> {
    fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }

    fn set_property(&mut self, key: &str, value: &str) -> bool {
        self.properties.insert(key.to_string(), value.to_string());
        true
    }
}

impl<T: Entity> Entity for BaseEntity<T> {
    fn draw(&self, draw_context: &mut DrawContext) -> anyhow::Result<()> {
        self.delegate_entity.draw(draw_context)
    }

    fn update(&mut self) -> UpdateResult {
        self.delegate_entity.update()
    }

    fn handle_event(&mut self, event: EventContext) -> bool {
        self.delegate_entity.handle_event(event)
    }
}

impl<T: Entity + Named> BaseEntity<T> {
    pub fn new(delegate_entity: T) -> Self {
        BaseEntity {
            name: format!("BaseEntity-{}", delegate_entity.get_name()),
            properties: HashMap::new(),
            delegate_entity,
        }
    }
}

impl<T: Entity> FullEntity for BaseEntity<T> {}
