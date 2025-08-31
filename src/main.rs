pub(crate) mod controller;

use std::{
    thread::{self},
    time::Duration,
};

use rdev::{grab, Button, Event as REvent, EventType, Key};

use crate::controller::{
    BaseEntity, Controller, CountDownEntity, FeedbackEntity, Named, PasswordPromptEntity,
    StaticTextEntity, Visible,
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
const GITHUB_LINK: &str = "https://github.com/your/repo"; // <-- change this
fn main() -> anyhow::Result<()> {
    let correct_password =
        std::env::var("LOCK_PASSWORD").unwrap_or_else(|_| "password".to_string());

    let mut controller = Controller::new();

    controller.add_entity(BaseEntity::new(StaticTextEntity::new(
        "title",
        [
            format!("🔒 This is a simple screen lock demo."),
            format!("💖 Send love to: {GITHUB_LINK}"),
        ],
    )));

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
