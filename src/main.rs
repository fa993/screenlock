pub(crate) mod controller;

use std::{
    io::{stdout, Stdout, Write},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, size, Clear, ClearType},
};
use rdev::{grab, Button, Event as REvent, EventType, Key};

const EVENTS_TO_BLOCK: [EventType; 10] = [
    EventType::KeyPress(Key::CapsLock),
    EventType::KeyRelease(Key::CapsLock),
    EventType::KeyPress(Key::Tab),
    EventType::KeyPress(Key::MetaLeft),
    EventType::KeyPress(Key::MetaRight),
    EventType::KeyPress(Key::ControlLeft),
    EventType::KeyPress(Key::ControlRight),
    EventType::KeyPress(Key::KeyC),
    EventType::KeyPress(Key::Escape),
    EventType::ButtonPress(Button::Left),
];

fn capture_control() {
    let callback = |event: REvent| -> Option<REvent> {
        if EVENTS_TO_BLOCK.contains(&event.event_type) {
            println!("Consuming and cancelling event: {:?}", event.event_type);
            None // CapsLock is now effectively disabled
        } else {
            // println!("Event: {:?}", event);
            Some(event)
        }
    };
    // This will block.
    if let Err(error) = grab(callback) {
        println!("Error: {:?}", error)
    }
}

const PROMPT: &str = "Enter password: ";
const GITHUB_LINK: &str = "https://github.com/your/repo"; // <-- change this
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut out = stdout();
    let mut guard = CleanupGuard { out };

    let correct_password =
        std::env::var("LOCK_PASSWORD").unwrap_or_else(|_| "password".to_string());

    // Setup terminal
    terminal::enable_raw_mode()?;
    execute!(guard.get_out(), Clear(ClearType::All))?;

    // Static UI (title + explanation)
    execute!(
        guard.get_out(),
        MoveTo(0, 1),
        Print("🔒 This is a simple screen lock demo."),
        MoveTo(0, 2),
        Print(format!("💖 Send love to: {GITHUB_LINK}")),
        MoveTo(0, 4),
        Print(PROMPT)
    )?;
    guard.get_out().flush()?;

    let prompt_col = PROMPT.len() as u16;
    let total = Duration::from_secs(30);
    let start = Instant::now();
    let mut password = String::new();

    loop {
        let elapsed = start.elapsed();
        if elapsed >= total {
            break;
        }

        // print countdown at top-left
        let remaining = total - elapsed;
        print_countdown(guard.get_out(), remaining)?;

        // non-blocking input
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char(c) => {
                        password.push(c);
                        execute!(guard.get_out(), Print("*"))?;
                        guard.get_out().flush()?;
                    }
                    KeyCode::Backspace => {
                        if password.pop().is_some() {
                            let x = prompt_col + password.len() as u16;
                            execute!(guard.get_out(), MoveTo(x, 4), Print(" "), MoveTo(x, 4))?;
                            guard.get_out().flush()?;
                        }
                    }
                    KeyCode::Enter => {
                        if password == correct_password {
                            terminal::disable_raw_mode()?;
                            println!("\n✅ Correct password! Unlocked.");
                            return Ok(());
                        } else {
                            // wrong password → feedback on line 5
                            execute!(
                                guard.get_out(),
                                MoveTo(0, 5),
                                Clear(ClearType::CurrentLine),
                                MoveTo(0, 5),
                                SetForegroundColor(Color::Red),
                                Print("❌ Wrong password, try again."),
                                ResetColor
                            )?;
                            guard.get_out().flush()?;

                            // reset prompt
                            password.clear();
                            execute!(
                                guard.get_out(),
                                MoveTo(0, 4),
                                Clear(ClearType::CurrentLine),
                                MoveTo(0, 4),
                                Print(PROMPT),
                                MoveTo(prompt_col, 4)
                            )?;
                            guard.get_out().flush()?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn print_countdown(
    out: &mut Stdout,
    remaining: Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let secs = remaining.as_secs();
    let minutes = secs / 60;
    let seconds = secs % 60;
    let text = format!("{:02}:{:02}", minutes, seconds);

    execute!(
        out,
        SavePosition,
        MoveTo(0, 0),
        Clear(ClearType::CurrentLine),
        SetForegroundColor(Color::Red),
        Print(&text),
        ResetColor,
        RestorePosition
    )?;
    out.flush()?;

    Ok(())
}

/// Cleanup: disables raw mode and clears the screen
fn cleanup(out: &mut Stdout) {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(out, Clear(ClearType::All), MoveTo(0, 0));
}

/// RAII guard that owns stdout and always cleans up
struct CleanupGuard {
    out: Stdout,
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        cleanup(&mut self.out);
    }
}

impl CleanupGuard {
    fn get_out(&mut self) -> &mut Stdout {
        &mut self.out
    }
}
