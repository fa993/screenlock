use crate::controller::{DrawContext, EventContext, UpdateResult};

pub trait Named {
    fn get_name(&self) -> &str;
}

pub trait HasProperties {
    fn get_property(&self, key: &str) -> Option<&str>;
    fn set_property(&mut self, key: &str, value: &str) -> bool;
}

pub trait Visible: HasProperties {
    fn is_visible(&self) -> bool {
        self.get_property("visible")
            .map(|v| v == "true")
            .unwrap_or(true)
    }

    fn set_visible(&mut self, visible: bool) {
        self.set_property("visible", if visible { "true" } else { "false" });
    }
}

pub trait Entity {
    fn draw(&self, draw_context: &mut DrawContext) -> anyhow::Result<()>;
    fn update(&mut self) -> UpdateResult {
        UpdateResult::nop()
    }
    fn handle_event(&mut self, _: EventContext) -> bool {
        false
    }
}

pub trait FullEntity: Entity + Named + HasProperties {}
