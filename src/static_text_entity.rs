use crossterm::{cursor::MoveTo, execute, style::Print};

use crate::{
    controller::DrawContext,
    entity::{Entity, Named},
    Lines, TITLE_Y,
};

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
        execute!(
            draw_context.out,
            MoveTo(0, TITLE_Y),
            Print(self.lines[0]),
            MoveTo(0, TITLE_Y + 1),
            Print(self.lines[1])
        )?;
        Ok(())
    }
}

impl Named for StaticTextEntity {
    fn get_name(&self) -> &str {
        self.id.as_str()
    }
}
