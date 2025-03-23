use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use slint::SharedString;

pub struct TextCompletion {
    words: Vec<String>,
    matcher: SkimMatcherV2,
}

impl TextCompletion {
    pub fn new() -> TextCompletion {
        let words = read_lines_from_file("common_english_words.txt");
        TextCompletion {
            words,
            matcher: SkimMatcherV2::default().ignore_case(),
        }
    }

    pub fn suggest(&self, input: &str) -> [SharedString; 3] {
        let mut suggestions = [SharedString::new(), SharedString::new(), SharedString::new()];
        let mut index = 0;
        for iter in 0..2 {
            for word in &self.words {
                if index == 3 {
                    return suggestions;
                }
                if ((iter == 0 && word.starts_with(&input.to_lowercase())) || (iter == 1 && self.matcher.fuzzy_match(word, input).is_some())) && !word.eq(&input.to_lowercase()) && !suggestions.contains(&SharedString::from(word.clone())) {
                    suggestions[index] = SharedString::from(word.clone());
                    index += 1;
                }
            }
        }
        suggestions
    }
}

fn read_lines_from_file(file_path: &str) -> Vec<String> {
    let path = Path::new(file_path);
    let file = File::open(&path).expect("File not found");
    let reader = io::BufReader::new(file);

    reader.lines().filter_map(Result::ok).collect()
}