use std::process::Command;

pub fn tts_speak(text: String, voice: &String) -> Result<(), Box<dyn std::error::Error>> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));

    let voice_final = format!("en-US-{}", voice);

    let command = format!("edge-playback -v {} --text \"{}\"", voice_final, text);
    println!("Running command: {}", command);

    let display = std::env::var("DISPLAY").unwrap_or_else(|_| String::from(":0.0"));
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));

    Command::new(&shell)
        .args(&["-i", "-c", &command])
        .env_clear()
        .env("TERM", "xterm-256color")
        .env("PATH", std::env::var("PATH")?)
        .env("HOME", std::env::var("HOME")?)
        .env("USER", std::env::var("USER")?)
        .env("SHELL", &shell)
        .env("DISPLAY", display)
        .env("XDG_RUNTIME_DIR", &runtime_dir)
        .env("PULSE_SERVER", format!("unix:{}/pulse/native", runtime_dir))
        .env("DBUS_SESSION_BUS_ADDRESS", format!("unix:path={}/bus", runtime_dir))
        .env("XDG_SESSION_TYPE", "x11")  // or "wayland" if you're using Wayland
        .spawn()?
        .wait()?;

    Ok(())
}



