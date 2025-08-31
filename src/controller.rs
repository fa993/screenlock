use std::{
    collections::HashMap,
    io::{stdout, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{KeyCode, KeyEvent},
    terminal::disable_raw_mode,
};

use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    event::{self, Event},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{enable_raw_mode, Clear, ClearType},
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
    let _ = disable_raw_mode();
    let _ = execute!(out, Clear(ClearType::All), MoveTo(0, 0));
}

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

pub struct ControlEvent {
    name: String,
    property_key: String,
    property_value: String,
}

pub struct UpdateResult {
    pub kill: bool,
    pub focused: bool,
    pub events: Vec<ControlEvent>,
}

impl UpdateResult {
    #[allow(unused)]
    pub fn new(kill: bool, focused: bool, events: Vec<ControlEvent>) -> Self {
        UpdateResult {
            kill,
            focused,
            events,
        }
    }

    pub fn kill() -> Self {
        UpdateResult {
            kill: true,
            focused: false,
            events: Vec::new(),
        }
    }

    pub fn focus() -> Self {
        UpdateResult {
            kill: false,
            focused: true,
            events: Vec::new(),
        }
    }

    pub fn nop() -> Self {
        UpdateResult {
            kill: false,
            focused: false,
            events: Vec::new(),
        }
    }
}

pub struct EventContext<'a> {
    pub event: &'a Event,
}

pub trait FullEntity: Entity + Named + HasProperties {}

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

    pub fn add_entity<U: FullEntity + 'static>(&mut self, entity: U) {
        self.entities.push(Box::new(entity));
    }

    fn update_and_draw_entity(
        entity: &mut Box<dyn FullEntity>,
        context: &mut DrawContext,
    ) -> anyhow::Result<UpdateResult> {
        let result = entity.update();
        if !result.focused {
            execute!(context.out, SavePosition)?;
        }
        entity.draw(context)?;
        if !result.focused {
            execute!(context.out, RestorePosition)?
        }
        Ok(result)
    }

    fn execute_entity_events(&mut self, events: &mut Vec<ControlEvent>) {
        for event in events.drain(..) {
            for entity in self.entities.iter_mut() {
                if entity.get_name() == event.name {
                    entity.set_property(&event.property_key, &event.property_value);
                    break;
                }
            }
        }
    }

    fn work_loop(&mut self, context: &mut DrawContext) -> anyhow::Result<()> {
        loop {
            let mut events_to_process = Vec::new();
            for entity in self.entities.iter_mut() {
                let result = Self::update_and_draw_entity(entity, context)?;
                if result.kill {
                    return Ok(());
                }
                events_to_process.extend(result.events);
            }
            self.execute_entity_events(&mut events_to_process);
            if event::poll(self.poll_interval)? {
                let event = event::read()?;
                for entity in self.entities.iter_mut() {
                    let acted = entity.handle_event(EventContext { event: &event });
                    if acted {
                        let result = Self::update_and_draw_entity(entity, context)?;
                        if result.kill {
                            return Ok(());
                        }
                        events_to_process.extend(result.events);
                    }
                }
                self.execute_entity_events(&mut events_to_process);
            }
        }
    }

    pub fn execute(&mut self) -> anyhow::Result<()> {
        let mut context = DrawContext { out: stdout() };

        enable_raw_mode()?;
        execute!(context.out, Clear(ClearType::All), MoveTo(0, 0))?;

        self.work_loop(&mut context)?;

        Ok(())
    }
}

pub struct StaticTextEntity {
    id: String,
    lines: [String; 2],
}

impl StaticTextEntity {
    pub fn new(id: &str, lines: [String; 2]) -> Self {
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
            Print(self.lines[1].as_str())
        )?;
        Ok(())
    }
}

impl Named for StaticTextEntity {
    fn get_name(&self) -> &str {
        self.id.as_str()
    }
}

pub struct CountDownEntity {
    id: String,
    total: Duration,
    start: Instant,
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

impl Named for CountDownEntity {
    fn get_name(&self) -> &str {
        self.id.as_str()
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

    fn update(&mut self) -> UpdateResult {
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
        let over = remaining.as_secs() <= 0;
        if over {
            UpdateResult::kill()
        } else {
            UpdateResult::nop()
        }
    }
}

pub struct PasswordPromptEntity {
    id: String,
    prompt: String,
    correct_password: String,
    password: String,
    dirty: bool,
    linked_feedback: String,
}

impl PasswordPromptEntity {
    pub fn new(id: &str, prompt: &str, correct_password: &str, linked_feedback_name: &str) -> Self {
        PasswordPromptEntity {
            id: format!("PasswordPromptEntity-{id}"),
            prompt: prompt.to_string(),
            correct_password: correct_password.to_string(),
            password: String::new(),
            dirty: true,
            linked_feedback: linked_feedback_name.to_string(),
        }
    }
}

impl Named for PasswordPromptEntity {
    fn get_name(&self) -> &str {
        self.id.as_str()
    }
}

impl Entity for PasswordPromptEntity {
    fn draw(&self, draw_context: &mut DrawContext) -> anyhow::Result<()> {
        let prompt_col = self.prompt.len() as u16;
        execute!(
            draw_context.out,
            MoveTo(0, 4),
            Clear(ClearType::CurrentLine),
            MoveTo(0, 4),
            Print(format!("{}{}", self.prompt, "*".repeat(self.password.len())).as_str()),
            MoveTo(prompt_col + self.password.len() as u16, 4)
        )?;
        Ok(())
    }

    fn update(&mut self) -> UpdateResult {
        if self.password == self.correct_password && !self.dirty {
            return UpdateResult::kill();
        }
        if !self.dirty {
            self.dirty = true;
            return UpdateResult {
                kill: false,
                focused: true,
                events: vec![ControlEvent {
                    name: self.linked_feedback.clone(),
                    property_key: "visible".to_string(),
                    property_value: "true".to_string(),
                }],
            };
        }
        UpdateResult::focus()
    }

    fn handle_event(&mut self, event: EventContext) -> bool {
        match event.event {
            Event::Key(KeyEvent { code, .. }) => {
                match code {
                    KeyCode::Char(c) => {
                        self.password.push(*c);
                        self.dirty = true;
                        true
                    }
                    KeyCode::Backspace => {
                        self.password.pop();
                        self.dirty = true;
                        true
                    }
                    KeyCode::Enter => {
                        self.dirty = false;
                        if self.password == self.correct_password {
                            return true; // signal to kill
                        } else {
                            self.password.clear();
                        }
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

pub struct FeedbackEntity {
    id: String,
    message: String,
    last_shown: Option<Instant>,
    max_show_duration: Duration,
    properties: std::collections::HashMap<String, String>,
}

impl FeedbackEntity {
    pub fn new(id: &str, message: &str, max_shown_duration: Duration) -> Self {
        FeedbackEntity {
            id: format!("FeedbackEntity-{id}"),
            message: message.to_string(),
            last_shown: None,
            max_show_duration: max_shown_duration,
            properties: {
                let mut map = HashMap::new();
                map.insert("visible".to_string(), "true".to_string());
                map
            },
        }
    }
}

impl Named for FeedbackEntity {
    fn get_name(&self) -> &str {
        self.id.as_str()
    }
}

impl HasProperties for FeedbackEntity {
    fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }

    fn set_property(&mut self, key: &str, value: &str) -> bool {
        self.properties.insert(key.to_string(), value.to_string());
        true
    }
}

impl Visible for FeedbackEntity {}

impl FullEntity for FeedbackEntity {}

impl Entity for FeedbackEntity {
    fn draw(&self, draw_context: &mut DrawContext) -> anyhow::Result<()> {
        if !self.is_visible() {
            execute!(
                draw_context.out,
                MoveTo(0, 5),
                Clear(ClearType::CurrentLine),
            )?;
        } else {
            execute!(
                draw_context.out,
                MoveTo(0, 5),
                Clear(ClearType::CurrentLine),
                MoveTo(0, 5),
                SetForegroundColor(Color::Red),
                Print(self.message.as_str()),
                ResetColor
            )?;
        }

        Ok(())
    }

    fn update(&mut self) -> UpdateResult {
        if self.is_visible() && self.last_shown.is_none() {
            self.last_shown = Some(Instant::now());
        }
        let cond = self
            .last_shown
            .map(|t| t.elapsed() >= self.max_show_duration)
            .unwrap_or_default();
        if self.is_visible() && cond {
            self.set_visible(false);
            self.last_shown = None;
        }
        UpdateResult::nop()
    }
}
