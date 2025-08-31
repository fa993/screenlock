use std::time::{Duration, Instant};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use crate::{
    controller::{DrawContext, UpdateResult},
    entity::{Entity, Named},
};

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
            MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(Color::Red),
            Print(&self.print_text),
            ResetColor,
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
