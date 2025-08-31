use crossterm::{
    cursor::MoveTo,
    event::{Event, KeyCode, KeyEvent},
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};

use crate::{
    controller::{ControlEvent, DrawContext, EventContext, UpdateResult},
    entity::{Entity, Named},
    PROMPT_Y,
};

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
            MoveTo(0, PROMPT_Y),
            Clear(ClearType::CurrentLine),
            MoveTo(0, PROMPT_Y),
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
