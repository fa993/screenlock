use crossterm::{cursor::MoveTo, execute, style::Print};

use crate::{
    controller::DrawContext,
    entity::{Entity, Named},
};

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
