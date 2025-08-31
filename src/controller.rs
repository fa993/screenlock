use std::{
    io::{stdout, Stdout},
    time::Duration,
};

use crossterm::terminal::disable_raw_mode;

use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    event::{self, Event},
    execute,
    terminal::{enable_raw_mode, Clear, ClearType},
};

use crate::entity::FullEntity;

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

pub struct ControlEvent {
    pub name: String,
    pub property_key: String,
    pub property_value: String,
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
