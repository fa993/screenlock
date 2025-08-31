pub(crate) mod base_entity;
pub(crate) mod controller;
pub(crate) mod count_down_entity;
pub(crate) mod entity;
pub(crate) mod feedback_entity;
pub(crate) mod password_prompt_entity;
pub(crate) mod static_text_entity;

use std::{
    thread::{self},
    time::Duration,
};

use rdev::{grab, Button, Event as REvent, EventType, Key};

use crate::{
    base_entity::BaseEntity,
    controller::Controller,
    count_down_entity::CountDownEntity,
    entity::{Named, Visible},
    feedback_entity::FeedbackEntity,
    password_prompt_entity::PasswordPromptEntity,
    static_text_entity::StaticTextEntity,
};

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

pub(crate) type Lines = [&'static str; 2];

const LINES: Lines = [
    "🔒 This is a simple screen lock demo.",
    "💖 Send love to: https://github.com/your/repo",
];

pub const COUNTDOWN_Y: u16 = 0;
pub const TITLE_Y: u16 = COUNTDOWN_Y + 1;
pub const PROMPT_Y: u16 = TITLE_Y + LINES.len() as u16 + 1; // titles length + 1 line gap
pub const FEEDBACK_Y: u16 = PROMPT_Y + 1;

fn capture_control() {
    let callback = |event: REvent| -> Option<REvent> {
        if EVENTS_TO_BLOCK.contains(&event.event_type) {
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
fn main() -> anyhow::Result<()> {
    let mut correct_password =
        std::env::var("LOCK_PASSWORD").unwrap_or_else(|_| "password".to_string());

    correct_password = correct_password.trim().to_lowercase();

    let mut controller = Controller::new();

    controller.add_entity(BaseEntity::new(StaticTextEntity::new("title", LINES)));

    controller.add_entity(BaseEntity::new(CountDownEntity::new(
        "countdown",
        Duration::from_secs(30),
    )));

    let mut f_entity = FeedbackEntity::new(
        "feedback",
        "❌ Wrong password, try again.",
        Duration::from_secs(2),
    );

    let p_entity = BaseEntity::new(PasswordPromptEntity::new(
        "password",
        PROMPT,
        correct_password.as_str(),
        f_entity.get_name(),
    ));

    f_entity.set_visible(false);

    controller.add_entity(p_entity);

    controller.add_entity(f_entity);

    let handle = thread::spawn(|| {
        capture_control();
    });

    controller.execute()?;
    drop(handle);

    Ok(())
}
