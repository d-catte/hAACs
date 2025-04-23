mod word_model;
mod bluetooth;
mod speedy_spellers;

use crate::bluetooth::BluetoothDevices;
use crate::speedy_spellers::SpeedySpeller;
use crate::word_model::{read_lines_from_file, TextCompletion};
use slint::{Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::cell::RefCell;
use std::ops::AddAssign;
use std::rc::Rc;
use ::tts::Tts;
use tts::Voice;

slint::include_modules!();
fn main() {
    let app = App::new().unwrap();

    let tts = Tts::default().unwrap();
    app.set_voices(get_voices(&tts));

    let autocomplete_words = Rc::new(read_lines_from_file("common_english_words.txt"));
    let text_complete = TextCompletion::new(Rc::clone(&autocomplete_words));

    let app_weak = app.as_weak();
    //let bluetooth_interface = Rc::new(BluetoothDevices::new());
    let bluetooth_interface = Rc::new(RefCell::new(BluetoothDevices::new()));


    let mut speedy_spellers = SpeedySpeller::new(app_weak.clone().unwrap(), autocomplete_words);
    speedy_spellers.register_callbacks();

    // Type Character
    app.on_character_typed({
        let app_clone = app_weak.clone().unwrap();
        move |character| {
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
                        app_clone.set_text_value(insert_character_at_index(
                            &text,
                            index as usize,
                            character.chars().next().unwrap(),
                        ));
                    } else {
                        let mut char = character.chars().next().unwrap();
                        if char >= 'A' || char <= 'Z' {
                            char = char.to_ascii_lowercase();
                        }
                        app_clone.set_text_value(insert_character_at_index(
                            &text,
                            index as usize,
                            char,
                        ));
                    }
                    app_clone.set_cursor_index(index + 1);
                }
            }
            // Suggest new word
            app_clone.set_auto_complete(ModelRc::from(
                text_complete.suggest(get_last_word(&app_clone.get_text_value())),
            ));

            app_clone.invoke_cursor_moved();
        }
    });

    // Do Text To Speech
    app.on_tts({
        let mut tts_clone = tts.clone();
        move |text| {
            tts_clone.speak(text.as_str(), true).unwrap();
        }
    });

    app.on_cursor_moved({
        let app_clone = app_weak.clone().unwrap();
        move || {
            let text = app_clone.get_text_value();
            let index = app_clone.get_cursor_index();

            app_clone.set_visible_cursor_text(insert_character_at_index(
                &text,
                index as usize,
                '|',
            ));
            app_clone.set_invisible_cursor_text(insert_character_at_index(
                &text,
                index as usize,
                ' ',
            ));
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
            if should_be_capital(text) {
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

    app.on_set_voice({
        let app_clone = app_weak.clone().unwrap();
        let mut tts_clone = tts.clone();
        move |string| {
            tts_clone.set_voice(&get_voice_from_name(&string, &tts_clone, &app_clone)).unwrap();
        }
    });
    
    app.on_set_bluetooth_audio({
        let bluetooth_interface = Rc::clone(&bluetooth_interface); // Clone Rc for this closure
        move |device_str| {
            for device in bluetooth_interface.borrow().devices.lock().unwrap().iter() {
                if device.get_device_name().to_shared_string().eq(&device_str) {
                    device.connect();
                    break;
                }
                if device.mac_address.to_shared_string().eq(&device_str) {
                    device.connect();
                    break
                }
                if device.alias.to_shared_string().eq(&device_str) {
                    device.connect();
                    break;
                }
            }
        }
    });

    app.on_refresh_bluetooth({
        let bluetooth_interface = Rc::clone(&bluetooth_interface);
        let app_clone = app_weak.clone().unwrap();
        move || {
            #[cfg(unix)]
            bluetooth_interface.borrow_mut().refresh_bluetooth();
            // TODO Refresh bluetooth name list
            let mut device_names: Vec<SharedString> = Vec::new();
            device_names.push(SharedString::from("Speakers"));
            for device in bluetooth_interface.borrow().devices.lock().unwrap().iter() {
                device_names.push(SharedString::from(device.get_device_name()));
            }
            let vec_model = VecModel::from(device_names);
            let rc_model = Rc::new(vec_model);
            app_clone.set_bluetooth_devices(ModelRc::from(rc_model));
        }
    });


    app.run().unwrap();
}

fn insert_character_at_index(
    shared_string: &SharedString,
    index: usize,
    character: char,
) -> SharedString {
    let mut string = shared_string.to_string();
    string.insert(index, character);
    SharedString::from(string)
}

fn remove_character_at_index(shared_string: &SharedString, index: usize) -> SharedString {
    let mut string = shared_string.to_string();
    string.remove(index - 1);
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

fn get_voices(tts: &Tts) -> ModelRc<SharedString> {
    let voices = tts.voices().unwrap();
    let voice_names: Vec<SharedString> = voices.iter()
        .filter(|voice| voice.name().starts_with("English (America)"))
        .map(|voice| SharedString::from(voice.name()))
        .collect();


    let vec_model = VecModel::from(voice_names);
    let rc_model = Rc::new(vec_model);
    ModelRc::from(rc_model)
}

fn get_voice_from_name(name: &SharedString, tts: &Tts, app: &App) -> Voice {
    let voices = app.get_voices();
    let mut index = 0;
    for (i, voice) in voices.iter().enumerate() {
        if voice.eq(name) {
            index = i;
            break;
        }
    }
    tts.voices().unwrap()[index].clone()
}
