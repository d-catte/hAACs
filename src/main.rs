mod word_model;

use std::ops::{AddAssign};
use ::tts::Tts;
use slint::{ModelRc, SharedString};
use crate::word_model::TextCompletion;

slint::include_modules!();
fn main() {
    /*
    println!("Hello, world!");

    let mut tts = Tts::default().unwrap();
    tts.speak("This is an example line of text", true).unwrap();
    thread::sleep(Duration::from_secs(3));
    tts.speak("Here are my available voices", false).unwrap();
    for voice in Tts::voices(&tts).unwrap() {
        tts.set_voice(&voice).unwrap();
        tts.speak(voice.name(), false).unwrap();
        thread::sleep(Duration::from_secs(3));
    }

    let _ = Keyboard::new();

     */
    let app = App::new().unwrap();

    let mut tts = Tts::default().unwrap();
    let text_complete = TextCompletion::new();

    let app_weak = app.as_weak();

    // Type Character
    app.on_character_typed( {
        let app_clone = app_weak.clone().unwrap();
        move |character| {
            println!("{:?}", character);
            let text = app_clone.get_text_value();
            let index = app_clone.get_cursor_index();
            match character.as_str() {
                "Delete" => {
                    if text.is_empty() {
                        return;
                    }
                    app_clone.set_text_value(remove_character_at_index(&text, index as usize));
                    app_clone.set_cursor_index(index - 1);
                }
                "Space" => {
                    app_clone.set_text_value(insert_character_at_index(&text, index as usize, ' '));
                    app_clone.set_cursor_index(index + 1);
                }
                _ => {
                    if should_be_capital(&text) {
                        app_clone.set_text_value(insert_character_at_index(&text, index as usize, character.chars().next().unwrap()));
                    } else {
                        let mut char = character.chars().next().unwrap();
                        if char >= 'A' || char <= 'Z' {
                            char = char.to_ascii_lowercase();
                        }
                        app_clone.set_text_value(insert_character_at_index(&text, index as usize, char));
                    }
                    app_clone.set_cursor_index(index + 1);
                }
            }
            // Suggest new word
            app_clone.set_auto_complete(ModelRc::from(text_complete.suggest(get_last_word(&app_clone.get_text_value()))));

            app_clone.invoke_cursor_moved();
        }
    });

    // Do Text To Speech
    app.on_tts( move |text| {
        tts.speak(text.as_str(), true).unwrap();
        println!("TTS");
    });

    app.on_cursor_moved( {
        let app_clone = app_weak.clone().unwrap();
        move || {
            let text = app_clone.get_text_value();
            let index = app_clone.get_cursor_index();

            app_clone.set_visible_cursor_text(insert_character_at_index(&text, index as usize, '|'));
            app_clone.set_invisible_cursor_text(insert_character_at_index(&text, index as usize, ' '));
        }
    });

    app.on_autocomplete({
        let app_clone = app_weak.clone().unwrap();
        move |mut string| {
            let mut text = &mut app_clone.get_text_value();
            let last_word = get_last_word(text);
            let new_length = text.len() - last_word.len();
            let mut binding = SharedString::from(&text.as_str()[..new_length]);
            text = &mut binding;
            if should_be_capital(&text) {
                let first_char = string.chars().next().unwrap().to_ascii_uppercase();
                string = replace_first_character(&string, first_char);
            }
            text.add_assign(string.as_str());
            text.add_assign(" ");
            let new_index = text.len();
            app_clone.set_cursor_index(new_index as i32);
            app_clone.set_text_value(text.clone());
            app_clone.set_invisible_cursor_text(text.clone());
            text.add_assign("|");
            app_clone.set_visible_cursor_text(text.clone());
            app_clone.set_auto_complete(ModelRc::default());
        }
    });

    app.run().unwrap();
}

fn insert_character_at_index(shared_string: &SharedString, index: usize, character: char) -> SharedString {
    let mut string = shared_string.to_string();
    string.insert(index, character);
    SharedString::from(string)
}

fn remove_character_at_index(shared_string: &SharedString, index: usize) -> SharedString {
    let mut string = shared_string.to_string();
    string.remove(index-1);
    SharedString::from(string)
}

fn get_last_word(text: &SharedString) -> &str {
    let text_str = text.as_str();

    if let Some(last_space_index) = text_str.rfind(' ') {
        &text_str[last_space_index + 1..]
    } else {
        text_str
    }
}

fn should_be_capital(text: &SharedString) -> bool {
    let text_str = text.as_str();

    if text_str.is_empty() {
        return true;
    }

    text_str.ends_with(". ")
}

fn replace_first_character(text: &SharedString, new_char: char) -> SharedString {
    let mut string = text.to_string();
    string.replace_range(0..1, &new_char.to_string());
    SharedString::from(string)
}
