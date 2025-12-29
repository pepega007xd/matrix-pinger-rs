/**
 * google gemini helped me write most of this
 */

use anyhow::Result;
use matrix_sdk::{
    config::SyncSettings,
    room::Room,
    ruma::events::room::message::{
        OriginalSyncRoomMessageEvent,
        RoomMessageEventContent,
        MessageType,
        ForwardThread,
        AddMentions
    },
    Client,
};
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;

#[derive(Debug, Deserialize)]
struct Config {
    username: String,
    homeserver: String,
    store_path: String,
    device_id: String,
    device_display_name: String,
}

fn get_reply_text(msg: String) -> String {
    let msg_lower = msg.to_lowercase().trim().to_string();

    let mut output = "";

    if msg_lower == "ping" { output = "pong"; }

    if msg_lower.starts_with("!echo ") {
        output = msg.strip_prefix("!echo ").unwrap_or("")
    }

    return output.to_string()
}

fn get_config() -> Config {
    let file = File::open("config.json")
        .expect("failed to open config.json");
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Config`.
    let config = serde_json::from_reader(reader)
        .expect("failed to read config.json values");

    return config;
}

fn get_password() -> String {
    let password = fs::read_to_string("password.txt")
        .expect("couldn't read password.txt");

    return password.trim().to_string();
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Define credentials
    let config = get_config();
    let password = get_password();

    println!("Starting bot...");

    // 2. Build the Client with Persistence (SQLite)
    let client = Client::builder()
        .homeserver_url(config.homeserver)
        .sqlite_store(config.store_path, None) // <--- This saves the session to disk!
        .build()
        .await?;

    // 3. Log in ONLY if we don't have a saved session
    // If we restore from disk, client.logged_in() returns true.
    if client.session().is_none() {
        println!("No session found. Logging in...");
        client.matrix_auth()
            .login_username(config.username, &password)
            .initial_device_display_name(&config.device_display_name)
            .device_id(&config.device_id)
            .send()
            .await?;
        println!("Logged in successfully!");
    } else {
        println!("Restored previous session from disk.");
    }

    // 4. Register Handler & Sync
    client.add_event_handler(on_room_message);

    println!("Bot is running...");

    // We use sync(settings) to start.
    // Note: With persistence, the bot automatically remembers where it left off!
    client.sync(SyncSettings::default()).await?;

    Ok(())
}

async fn on_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    if let MessageType::Text(text_content) = &event.content.msgtype {

        let response = get_reply_text(text_content.body.clone());

        if response != "" {
            println!(
                "Received {:?} in room: {:?}",
                text_content.body,
                room.room_id()
            );

            let content = RoomMessageEventContent::text_plain(response)
                .make_reply_to(
                    &event,
                    ForwardThread::Yes,
                    AddMentions::Yes
                );

            room.send(content).await.ok();
        }
    }
}
