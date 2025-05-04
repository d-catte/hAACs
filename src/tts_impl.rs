use crate::check_internet_connection;
use rodio::{Decoder, OutputStream, Sink};
use std::io::BufReader;
#[cfg(unix)]
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::thread;

static THREAD_LOCK: AtomicBool = AtomicBool::new(false);

pub fn tts_speak(
    text: String,
    voice: &String,
    volume: i8,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the thread is already running
    if THREAD_LOCK.load(std::sync::atomic::Ordering::Relaxed) {
        return Ok(());
    }

    let shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));

    let file_extension;

    if check_internet_connection() {
        file_extension = "mp3";
        let voice_final = format!("en-US-{}", voice);

        let corrected_volume: i8 = if volume == 0 {
            return Ok(());
        } else {
            volume - 50
        };
        let command = format!(
            "edge-tts -v {} --text \"{}\" --volume={}% --write-media /tmp/output.mp3",
            voice_final, text, corrected_volume
        );

        //let display = std::env::var("DISPLAY").unwrap_or_else(|_| String::from(":0.0"));
        #[cfg(unix)]
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));

        #[cfg(unix)]
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
    } else {
        file_extension = "wav";
        let command = format!(
            "echo '{text}' | \
  ./piper --model en_US-ryan-low.onnx --output_file /tmp/output.wav"
        );

        #[cfg(unix)]
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));

        #[cfg(unix)]
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
    }

    // Play file
    #[cfg(unix)]
    thread::spawn(move || {
        THREAD_LOCK.store(true, std::sync::atomic::Ordering::Relaxed);
        // Play file
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let file = std::fs::File::open("/tmp/output.".to_owned() + file_extension).unwrap();
        sink.append(Decoder::new(BufReader::new(file)).unwrap());

        sink.sleep_until_end();

        // Delete the file
        std::fs::remove_file("/tmp/output.".to_owned() + file_extension).unwrap();
        THREAD_LOCK.store(false, std::sync::atomic::Ordering::Relaxed);
    });

    Ok(())
}
