mod bluetooth;
mod speedy_spellers;
mod tts_impl;
mod word_model;
mod wifi;

use crate::bluetooth::BluetoothDevices;
use crate::speedy_spellers::SpeedySpeller;
use crate::tts_impl::tts_speak;
use crate::word_model::{read_lines_from_file, read_lines_from_file_keep_spaces, TextCompletion};
use slint::{ModelRc, SharedString, ToSharedString, VecModel};
use std::cell::RefCell;
use std::ops::AddAssign;
use std::rc::Rc;
use crate::wifi::scan_wifi;

slint::include_modules!();
fn main() {
    let app = App::new().unwrap();

    let autocomplete_words = Rc::new(read_lines_from_file("common_english_words.txt"));
    let text_complete = TextCompletion::new(Rc::clone(&autocomplete_words));

    let app_weak = app.as_weak();

    let bluetooth_interface = Rc::new(RefCell::new(BluetoothDevices::new()));

    let selected_voice = Rc::new(RefCell::new(String::from("AnaNeural")));
    let voices = vec![
        "AnaNeural",
        "AndrewNeural",
        "AriaNeural",
        "AvaNeural",
        "BrianNeural",
        "ChristopherNeural",
        "EmmaNeural",
        "EricNeural",
        "GuyNeural",
        "JennyNeural",
        "MichelleNeural",
        "RogerNeural",
        "SteffanNeural",
    ];

    let mut speedy_spellers = SpeedySpeller::new(app_weak.clone().unwrap(), autocomplete_words);
    speedy_spellers.register_callbacks();

    // Type Character
    app.global::<KeyboardCallback>().on_character_typed({
        let app_clone = app_weak.clone().unwrap();
        move |character| {
            let text = app_clone.global::<AACCallback>().get_text();
            let index = app_clone.global::<AACCallback>().get_cursor_index();
            match character.as_str() {
                "Delete" => {
                    if text.is_empty() {
                        return;
                    }
                    app_clone.global::<AACCallback>().set_text(remove_character_at_index(&text, index as usize));
                    app_clone.global::<AACCallback>().set_cursor_index(index - 1);
                }
                "Space" => {
                    app_clone.global::<AACCallback>().set_text(insert_character_at_index(&text, index as usize, ' '));
                    app_clone.global::<AACCallback>().set_cursor_index(index + 1);
                }
                _ => {
                    if should_be_capital(&text) {
                        app_clone.global::<AACCallback>().set_text(insert_character_at_index(
                            &text,
                            index as usize,
                            character.chars().next().unwrap(),
                        ));
                    } else {
                        let mut char = character.chars().next().unwrap();
                        if char >= 'A' || char <= 'Z' {
                            char = char.to_ascii_lowercase();
                        }
                        app_clone.global::<AACCallback>().set_text(insert_character_at_index(
                            &text,
                            index as usize,
                            char,
                        ));
                    }
                    app_clone.global::<AACCallback>().set_cursor_index(index + 1);
                }
            }
            // Suggest new word
            app_clone.global::<KeyboardCallback>().set_auto_complete(ModelRc::from(
                text_complete.suggest(get_last_word(&app_clone.global::<AACCallback>().get_text())),
            ));

            app_clone.global::<AACCallback>().invoke_cursor_moved();
        }
    });
    
    // Recheck internet connection
    app.global::<SettingsData>().on_refresh_internet({
        let app_clone = app_weak.clone().unwrap();
        move || {
            app_clone.global::<SettingsData>().set_online(check_internet_connection());
            app_clone.global::<SettingsData>().set_wifi_names(scan_wifi());
        }
    });
    
    app.global::<SettingsData>().on_connect_to_wifi({
        move |username, password| {
            // TODO Connect to WiFi
        }
    });

    // Do Text To Speech
    app.global::<AACCallback>().on_tts({
        let selected_voice_clone = Rc::clone(&selected_voice);
        let app_clone = app_weak.clone().unwrap();
        move |text| {
            tts_speak(text.to_string(), &selected_voice_clone.borrow(), app_clone.global::<SettingsData>().get_volume() as i8).unwrap();
        }
    });

    app.global::<AACCallback>().set_voices(vec_to_rc(voices));
    let quick_button_options = read_lines_from_file_keep_spaces("quick_buttons.txt");
    let quick_button_options_model: VecModel<SharedString> = VecModel::from(quick_button_options.into_iter().map(SharedString::from).collect::<Vec<SharedString>>());
    app.global::<SettingsData>().set_quick_button_options(ModelRc::from(Rc::new(quick_button_options_model)));

    #[cfg(unix)]
    app.global::<SettingsData>().set_production_env(true);
    #[cfg(not(unix))]
    app.global::<SettingsData>().set_production_env(false);

    app.global::<SettingsData>().set_online(check_internet_connection());
    
    app.global::<SettingsData>().set_wifi_names(scan_wifi());

    app.global::<AACCallback>().on_cursor_moved({
        let app_clone = app_weak.clone().unwrap();
        move || {
            let text = app_clone.global::<AACCallback>().get_text();
            let index = app_clone.global::<AACCallback>().get_cursor_index();

            app_clone.global::<AACCallback>().set_visible_cursor_text(insert_character_at_index(
                &text,
                index as usize,
                '|',
            ));
            app_clone.global::<AACCallback>().set_invisible_cursor_text(insert_character_at_index(
                &text,
                index as usize,
                ' ',
            ));
        }
    });

    app.global::<KeyboardCallback>().on_autocomplete({
        let app_clone = app_weak.clone().unwrap();
        move |mut string| {
            let mut text = &mut app_clone.global::<AACCallback>().get_text();
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
            app_clone.global::<AACCallback>().set_cursor_index(new_index as i32);
            app_clone.global::<AACCallback>().set_text(text.clone());
            app_clone.global::<AACCallback>().set_invisible_cursor_text(text.clone());
            text.add_assign("|");
            app_clone.global::<AACCallback>().set_visible_cursor_text(text.clone());
            app_clone.global::<KeyboardCallback>().set_auto_complete(ModelRc::default());
        }
    });

    app.global::<AACCallback>().on_set_voice({
        let selected_voice_clone = Rc::clone(&selected_voice);
        move |string| {
            let mut selected_voice_borrow = selected_voice_clone.borrow_mut();
            let len = selected_voice_borrow.len();
            selected_voice_borrow.replace_range(0..len, string.as_str());
        }
    });

    app.global::<SettingsData>().on_set_bluetooth_audio_device({
        let bluetooth_interface = Rc::clone(&bluetooth_interface); // Clone Rc for this closure
        move |device_str| {
            if device_str.eq("Speaker") {
                // TODO Disconnect BT and switch to GPIO output
                return;
            }
            #[cfg(unix)]
            for device in bluetooth_interface.borrow().devices.lock().unwrap().iter() {
                if device.get_device_name().to_shared_string().eq(&device_str) {
                    device.connect();
                    break;
                }
                if device.mac_address.to_shared_string().eq(&device_str) {
                    device.connect();
                    break;
                }
                if device.alias.to_shared_string().eq(&device_str) {
                    device.connect();
                    break;
                }
            }
        }
    });

    app.global::<SettingsData>().on_refresh_bluetooth({
        let bluetooth_interface = Rc::clone(&bluetooth_interface);
        let app_clone = app_weak.clone().unwrap();
        move || {
            #[cfg(unix)]
            bluetooth_interface.borrow_mut().refresh_bluetooth();
            let mut device_names: Vec<SharedString> = Vec::new();
            device_names.push(SharedString::from("Speakers"));
            #[cfg(unix)]
            for device in bluetooth_interface.borrow().devices.lock().unwrap().iter() {
                device_names.push(SharedString::from(device.get_device_name()));
            }
            let vec_model = VecModel::from(device_names);
            let rc_model = Rc::new(vec_model);
            app_clone.global::<SettingsData>().set_bluetooth_devices(ModelRc::from(rc_model));
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

fn vec_to_rc(voices: Vec<&str>) -> ModelRc<SharedString> {
    let shared_voices: Vec<SharedString> = voices.into_iter().map(SharedString::from).collect();
    let vec_model = VecModel::from(shared_voices);
    ModelRc::new(vec_model)
}

pub fn check_internet_connection() -> bool {
    // Attempt to connect to Google's public DNS server
    match std::net::TcpStream::connect("8.8.8.8:53") {
        Ok(_) => true,  // Connection successful
        Err(_) => false, // Connection failed
    }
}
