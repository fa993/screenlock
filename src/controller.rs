use std::{
    collections::HashMap,
    io::{self, stdout, Stdout},
    time::Duration,
};

use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    event::{self, Event},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

// This system doesn't account for inter-entity comm
// it is considered too complex for our applications
// even though there may be a performance hit due to rc

pub struct DrawContext {
    pub out: Stdout,
}

impl Drop for DrawContext {
    fn drop(&mut self) {
        cleanup(&mut self.out);
    }
}

fn cleanup(out: &mut Stdout) {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(out, Clear(ClearType::All), MoveTo(0, 0));
}

pub trait Named {
    fn get_name(&self) -> &str;
}

pub trait HasProperties {
    fn get_property(&self, key: &str) -> Option<String>;
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
    fn update(&mut self) -> bool; // returns whether it is focused
    fn handle_event(&mut self, _: EventContext) -> bool {
        false
    }
}

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
    fn get_property(&self, key: &str) -> Option<String> {
        self.properties.get(key).cloned()
    }

    fn set_property(&mut self, key: &str, value: &str) -> bool {
        self.properties.insert(key.to_string(), value.to_string());
        true
    }
}

impl<T: Entity> Visible for BaseEntity<T> {}

impl<T: Entity> Entity for BaseEntity<T> {
    fn draw(&self, draw_context: &mut DrawContext) -> anyhow::Result<()> {
        if self.properties.get("visible") == Some(&"false".to_string()) {
            // skip if not visible
            return Ok(());
        }

        self.delegate_entity.draw(draw_context)
    }

    fn update(&mut self) -> bool {
        self.delegate_entity.update()
    }

    fn handle_event(&mut self, event: EventContext) -> bool {
        self.delegate_entity.handle_event(event)
    }
}

impl<T: Entity> BaseEntity<T> {
    pub fn new(name: &str, delegate_entity: T) -> Self {
        let mut properties = std::collections::HashMap::new();
        properties.insert("visible".to_string(), "true".to_string());
        BaseEntity {
            name: name.to_string(),
            properties,
            delegate_entity,
        }
    }
}

pub struct EventContext<'a> {
    pub event: &'a Event,
}

pub struct ControllerContext<'a> {
    entity_map: HashMap<String, &'a mut Box<dyn FullEntity>>,
}

pub trait FullEntity: Entity + Named + Visible {}

pub struct Controller {
    entities: Vec<Box<dyn FullEntity>>,
    poll_interval: Duration,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            entities: Vec::new(),
            poll_interval: Duration::from_millis(50),
        }
    }

    pub fn add_entity(&mut self, entity: Box<dyn FullEntity>) {
        self.entities.push(entity);
    }

    fn update_and_draw_entity(
        entity: &mut Box<dyn FullEntity>,
        context: &mut DrawContext,
    ) -> io::Result<()> {
        let focused = entity.update();
        if !focused {
            execute!(context.out, SavePosition)?;
        }
        entity.draw(context);
        if !focused {
            execute!(context.out, RestorePosition)?
        }
        Ok(())
    }

    pub fn execute(&mut self) -> anyhow::Result<()> {
        let mut context = DrawContext { out: stdout() };

        loop {
            for entity in self.entities.iter_mut() {
                Self::update_and_draw_entity(entity, &mut context)?;
            }
            if event::poll(self.poll_interval)? {
                let event = event::read()?;
                for entity in self.entities.iter_mut() {
                    let acted = entity.handle_event(EventContext { event: &event });
                    if acted {
                        Self::update_and_draw_entity(entity, &mut context)?;
                    }
                }
            }
        }
    }
}

pub struct StaticTextEntity {
    id: String,
    lines: [String; 3],
}

impl StaticTextEntity {
    pub fn new(id: &str, lines: [String; 3]) -> Self {
        StaticTextEntity {
            id: format!("StaticTextEntity-{id}"),
            lines,
        }
    }
}
// "🔒 This is a simple screen lock demo."
// format!("💖 Send love to: {}")
// prompt
impl Entity for StaticTextEntity {
    fn draw(&self, draw_context: &mut DrawContext) -> anyhow::Result<()> {
        // Static UI (title + explanation)
        execute!(
            draw_context.out,
            MoveTo(0, 1),
            Print(self.lines[0].as_str()),
            MoveTo(0, 2),
            Print(self.lines[1].as_str()),
            MoveTo(0, 4),
            Print(self.lines[2].as_str()),
        )?;
        Ok(())
    }

    fn update(&mut self) -> bool {
        false
    }
}

pub struct CountDownEntity {
    id: String,
    total: Duration,
    start: std::time::Instant,
    print_text: String,
}

impl CountDownEntity {
    pub fn new(id: &str, total: Duration) -> Self {
        CountDownEntity {
            id: format!("CountDownEntity-{id}"),
            total,
            start: std::time::Instant::now(),
            print_text: String::new(),
        }
    }
}

impl Entity for CountDownEntity {
    fn draw(&self, draw_context: &mut DrawContext) -> anyhow::Result<()> {
        execute!(
            draw_context.out,
            SavePosition,
            MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::Red),
            Print(&self.print_text),
            ResetColor,
            RestorePosition
        )?;
        Ok(())
    }

    fn update(&mut self) -> bool {
        let elapsed = self.start.elapsed();
        let remaining = if elapsed >= self.total {
            Duration::from_secs(0)
        } else {
            self.total - elapsed
        };
        let secs = remaining.as_secs();
        let minutes = secs / 60;
        let seconds = secs % 60;
        self.print_text = format!("{:02}:{:02}", minutes, seconds);

        false
    }
}
