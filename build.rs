// build.rs

use std::env;

fn main() -> anyhow::Result<()> {
    // Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
    // see also https://github.com/rust-lang/cargo/issues/9554

    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")?;

    let wifi_ssid = env::var("WIFI_SSID").unwrap_or_else(|_| "internet".into());
    let wifi_pass = env::var("WIFI_PASS").unwrap_or_else(|_| "password".into());
    let api_port = env::var("API_PORT").unwrap_or_else(|_| "8080".into());
    let irc_server = env::var("IRC_SERVER").unwrap_or_else(|_| "openirc.snt.utwente.nl".into());
    let irc_channel = env::var("IRC_CHANNEL").unwrap_or_else(|_| "#esp32bot".into());
    let irc_nick = env::var("IRC_NICK").unwrap_or_else(|_| "esp32bot".into());
    let irc_owner_nick = env::var("IRC_OWNER_NICK").unwrap_or_else(|_| "bot_owner_nick".into());

    println!("cargo:rustc-env=WIFI_SSID={wifi_ssid}");
    println!("cargo:rustc-env=WIFI_PASS={wifi_pass}");
    println!("cargo:rustc-env=API_PORT={api_port}");
    println!("cargo:rustc-env=IRC_SERVER={irc_server}");
    println!("cargo:rustc-env=IRC_CHANNEL={irc_channel}");
    println!("cargo:rustc-env=IRC_NICK={irc_nick}");
    println!("cargo:rustc-env=IRC_OWNER_NICK={irc_owner_nick}");

    Ok(())
}

// EOF
