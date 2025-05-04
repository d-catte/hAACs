use crate::word_model::read_lines_from_file;
use crate::{AACCallback, App, LetterGameData};
use rand::prelude::SliceRandom;
use rand::{rng, Rng};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::rc::Rc;

pub struct SpeedySpeller {
    app: App,
    words_level_1: Rc<Vec<String>>,
    words_level_2: Rc<Vec<String>>,
    words_level_3: Rc<Vec<String>>,
}

impl SpeedySpeller {
    pub fn new(app: App, level_3_words: Rc<Vec<String>>) -> Self {
        Self {
            app,
            words_level_1: Rc::new(read_lines_from_file("speedy_speller_1.txt")),
            words_level_2: Rc::new(read_lines_from_file("speedy_speller_2.txt")),
            words_level_3: level_3_words,
        }
    }

    pub fn register_callbacks(&mut self) {
        let app_weak = self.app.as_weak();

        let words_level_1 = Rc::clone(&self.words_level_1);
        let words_level_2 = Rc::clone(&self.words_level_2);
        let words_level_3 = Rc::clone(&self.words_level_3);

        self.app.global::<LetterGameData>().on_game_start({
            let app_clone = app_weak.clone().unwrap();

            move || {
                app_clone.global::<LetterGameData>().set_score(SharedString::from("-1"));
                app_clone.global::<LetterGameData>().invoke_match_win();
            }
        });

        self.app.global::<LetterGameData>().on_letter_pressed({
            let app_clone = app_weak.clone().unwrap();
            move |string| {
                let mut current_letters = app_clone.global::<LetterGameData>().get_current_letters();
                let word = app_clone.global::<LetterGameData>().get_current_word();

                let new_letters = format!("{}{}", current_letters, string).to_lowercase();

                if word.starts_with(&new_letters) {
                    current_letters.push_str(&string);
                    app_clone.global::<LetterGameData>().set_current_letters(current_letters);
                }

                println!("{} -> {}", word, new_letters);

                if word.eq(&new_letters) {
                    app_clone.global::<LetterGameData>().invoke_match_win();
                }
            }
        });

        self.app.global::<LetterGameData>().on_match_win({
            let app_clone = app_weak.clone().unwrap();
            move || {
                let mut score: i8 = app_clone.global::<LetterGameData>().get_score().parse::<i8>().unwrap();
                score += 1;
                app_clone.global::<LetterGameData>().set_score(SharedString::from(&score.to_string()));
                app_clone.global::<LetterGameData>().set_current_letters(SharedString::default());

                // Pick next match word
                app_clone.global::<LetterGameData>().set_chars(scramble(app_clone.global::<LetterGameData>().get_chars()));
                let diff = app_clone.global::<LetterGameData>().get_difficulty() as u8;
                println!("{}", diff);
                let current_words = match diff {
                    1 => Rc::clone(&words_level_1),
                    2 => Rc::clone(&words_level_2),
                    _ => Rc::clone(&words_level_3),
                };
                let new_word = random_word(Rc::clone(&current_words));
                app_clone.global::<LetterGameData>().set_current_word(new_word.clone());
                app_clone.global::<AACCallback>().invoke_tts(new_word);
            }
        });

        self.app.global::<LetterGameData>().on_game_over({
            let app_clone = app_weak.clone().unwrap();
            move || {
                app_clone.global::<LetterGameData>().set_game_started(false);
            }
        })
    }
}

fn scramble(chars: ModelRc<SharedString>) -> ModelRc<SharedString> {
    let mut chars_vec: Vec<SharedString> = chars.iter().collect();

    chars_vec.shuffle(&mut rng());

    let scrambled_vec_model = Rc::new(VecModel::from(chars_vec));
    scrambled_vec_model.into()
}

fn random_word(words: Rc<Vec<String>>) -> SharedString {
    let word = SharedString::from(words[rng().random_range(0..words.len())].clone());
    println!("Random word: {}", word);
    word
}
