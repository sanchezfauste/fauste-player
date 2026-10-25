//! The IME bridge. winit 0.30 drops GameActivity's text events, so the
//! keyboard's text is read from the activity each frame and the difference
//! with the last one becomes egui events (mobile spec §11).

use egui::{Event, Key, Modifiers};

/// The last text the keyboard held for the focused field.
#[derive(Debug, Default)]
pub struct ImeBridge {
    last: String,
}

impl ImeBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// The events that turn the last text into `now`: one Backspace press
    /// and release per character after the common prefix (the cursor is at
    /// the end), then the rest of `now` as one text event. Characters, not
    /// bytes, are counted.
    pub fn update(&mut self, now: &str) -> Vec<Event> {
        if now == self.last {
            return Vec::new();
        }
        let common = self
            .last
            .chars()
            .zip(now.chars())
            .take_while(|(a, b)| a == b)
            .count();
        let removed = self.last.chars().count().saturating_sub(common);
        let mut events = Vec::with_capacity(removed * 2 + 1);
        for _ in 0..removed {
            for pressed in [true, false] {
                events.push(Event::Key {
                    key: Key::Backspace,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                });
            }
        }
        let added: String = now.chars().skip(common).collect();
        if !added.is_empty() {
            events.push(Event::Text(added));
        }
        self.last = now.to_owned();
        events
    }

    /// Another field took the focus and the keyboard holds `now`: nothing
    /// is typed.
    pub fn reset(&mut self, now: &str) {
        now.clone_into(&mut self.last);
    }
}
