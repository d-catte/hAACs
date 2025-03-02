use std::thread;
use std::time::Duration;
use ::tts::Tts;
use slint::slint;

slint!(
);

fn main() {
    println!("Hello, world!");

    let mut tts = Tts::default().unwrap();
    tts.speak("This is an example line of text", true).unwrap();
    thread::sleep(Duration::from_secs(3));
    tts.speak("Here are my available voices", false).unwrap();
    for voice in Tts::voices(&tts).unwrap() {
        tts.set_voice(&voice).unwrap();
        tts.speak(voice.name(), false).unwrap();
        thread::sleep(Duration::from_secs(3));
    };
}
