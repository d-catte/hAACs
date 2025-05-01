use std::io::BufReader;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::thread;
use rodio::{Decoder, OutputStream, Sink};

static THREAD_LOCK: AtomicBool = AtomicBool::new(false);

pub fn tts_speak(text: String, voice: &String) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the thread is already running
    if THREAD_LOCK.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }
    
    let shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));

    let voice_final = format!("en-US-{}", voice);

    let command = format!("edge-tts -v {} --text \"{}\" --write-media /tmp/output.mp3", voice_final, text);
    println!("Running command: {}", command);

    //let display = std::env::var("DISPLAY").unwrap_or_else(|_| String::from(":0.0"));
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));

    Command::new(&shell)
        .args(["-i", "-c", &command])
        .env_clear()
        .env("TERM", "xterm-256color")
        .env("PATH", std::env::var("PATH")?)
        .env("HOME", std::env::var("HOME")?)
        .env("USER", std::env::var("USER")?)
        .env("SHELL", &shell)
        //.env("DISPLAY", display)
        .env("XDG_RUNTIME_DIR", &runtime_dir)
        .env("PULSE_SERVER", format!("unix:{}/pulse/native", runtime_dir))
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path={}/bus", runtime_dir),
        )
        .env("XDG_SESSION_TYPE", "wayland")
        .spawn()?
        .wait()?;
    
    // Play file
    thread::spawn(move || {
        THREAD_LOCK.store(true, std::sync::atomic::Ordering::Relaxed);
        // Play file
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let file = std::fs::File::open("/tmp/output.mp3").unwrap();
        sink.append(Decoder::new(BufReader::new(file)).unwrap());

        sink.sleep_until_end();

        // Delete the file
        std::fs::remove_file("/tmp/output.mp3").unwrap();
        THREAD_LOCK.store(false, std::sync::atomic::Ordering::Relaxed);
    });

    Ok(())
}
