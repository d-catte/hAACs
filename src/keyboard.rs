use crate::AdaptiveKeyboard;
use slint::platform::Key;
use slint::{ComponentHandle, SharedString, Weak};

#[allow(dead_code)]
pub struct Keyboard {
    active_keyboard: KeyboardType,
    swipe_keyboard: AdaptiveKeyboard,
}

impl Keyboard {
    pub fn new() -> Keyboard {
        let swipe_keyboard = AdaptiveKeyboard::new().unwrap();
        let weak = &swipe_keyboard.as_weak();
        swipe_keyboard.on_character_typed({
            let weak = weak.clone();
            move |string| {
                type_char(&string, &weak);
            }
        });
        swipe_keyboard.on_autocomplete(move |string| {
            finish_word(&string);
        });

        swipe_keyboard.run().unwrap(); // TODO Figure out a better way to save on resources

        Keyboard {
            active_keyboard: KeyboardType::Swipe, // TODO Change to Standard
            swipe_keyboard,
        }
    }

    #[allow(dead_code)]
    pub fn show_keyboard(&mut self) {
        match self.active_keyboard {
            KeyboardType::Swipe => {
                self.swipe_keyboard.show().unwrap();
            }
            KeyboardType::Standard => {}
        }
    }

    #[allow(dead_code)]
    pub fn hide_keyboard(&mut self) {
        match self.active_keyboard {
            KeyboardType::Swipe => {
                self.swipe_keyboard.hide().unwrap();
            }
            KeyboardType::Standard => {}
        }
    }

    #[allow(dead_code)]
    pub fn set_word_suggestion(&mut self, index: usize, word: &str) {
        match self.active_keyboard {
            KeyboardType::Swipe => {
                match index {
                    1 => {
                        self.swipe_keyboard
                            .set_word_suggestion_one(SharedString::from(word));
                    }
                    2 => {
                        self.swipe_keyboard
                            .set_word_suggestion_two(SharedString::from(word));
                    }
                    _ => {
                        // 3
                        self.swipe_keyboard
                            .set_word_suggestion_three(SharedString::from(word));
                    }
                }
            }
            KeyboardType::Standard => {}
        }
    }
}

// TODO Implement "typing"
fn type_char(character: &str, weak: &Weak<AdaptiveKeyboard>) {
    println!("Typed {}", character);
    if character.len() > 1 {
        let key = match character {
            "Delete" => Key::Delete,
            "Space" => Key::Space,
            _ => Key::Alt, // Useless key
        };
        weak.unwrap()
            .window()
            .dispatch_event(slint::platform::WindowEvent::KeyPressed { text: key.into() });
        weak.unwrap()
            .window()
            .dispatch_event(slint::platform::WindowEvent::KeyReleased { text: key.into() });
    } else {
        weak.unwrap()
            .window()
            .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                text: SharedString::from(character),
            });
        weak.unwrap()
            .window()
            .dispatch_event(slint::platform::WindowEvent::KeyReleased {
                text: SharedString::from(character),
            });
    }
}

// TODO Implement word completion
#[allow(unused_variables)]
fn finish_word(word: &str) {}

#[allow(dead_code)]
enum KeyboardType {
    Swipe,
    Standard,
}
