use crate::{
    controller::DrawContext,
    entity::{Entity, Named},
    Lines, TITLE_Y,
};
use crossterm::QueueableCommand;
use crossterm::{cursor::MoveTo, style::Print};
use std::io::Write;

pub struct StaticTextEntity {
    id: String,
    lines: Lines,
}

impl StaticTextEntity {
    pub fn new(id: &str, lines: Lines) -> Self {
        StaticTextEntity {
            id: format!("StaticTextEntity-{id}"),
            lines,
        }
    }
}

impl Entity for StaticTextEntity {
    fn draw(&self, draw_context: &mut DrawContext) -> anyhow::Result<()> {
        // Static UI (title + explanation)
        for (idx, line) in self.lines.iter().enumerate() {
            draw_context.out.queue(MoveTo(0, TITLE_Y + idx as u16))?;
            draw_context.out.queue(Print(line))?;
        }
        draw_context.out.flush()?;
        Ok(())
    }
}

impl Named for StaticTextEntity {
    fn get_name(&self) -> &str {
        self.id.as_str()
    }
}
