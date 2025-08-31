use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use crate::{
    controller::{DrawContext, UpdateResult},
    entity::{Entity, FullEntity, HasProperties, Named, Visible},
    FEEDBACK_Y,
};

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
                MoveTo(0, FEEDBACK_Y),
                Clear(ClearType::CurrentLine),
            )?;
        } else {
            execute!(
                draw_context.out,
                MoveTo(0, FEEDBACK_Y),
                Clear(ClearType::CurrentLine),
                MoveTo(0, FEEDBACK_Y),
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
