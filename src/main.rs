/**
 * google gemini helped me write most of this
 */
use anyhow::Result;
use matrix_sdk::{
    Client, RoomState,
    config::SyncSettings,
    room::Room,
    ruma::events::room::{
        member::StrippedRoomMemberEvent,
        message::{
            AddMentions, ForwardThread, MessageType, OriginalSyncRoomMessageEvent,
            RoomMessageEventContent,
        },
    },
};
use serde::Deserialize;
use std::fs::{self, File};
use std::io::BufReader;

use std::sync::LazyLock;
use std::time::Instant;

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

const HELP_STRING: &str = r#"
ping - invoke pong
?echo CONTENT - echoes CONTENT
?uptime - reports current uptime
?help - shows this help message
https://github.com/okurka12/matrix-pinger-rs"#;

#[derive(Debug, Deserialize)]
struct Config {
    username: String,
    homeserver: String,
    store_path: String,
    device_id: String,
    device_display_name: String,
}

fn get_uptime() -> String {
    let total_seconds = START_TIME.elapsed().as_secs();

    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
}

fn get_reply_text(msg: &str) -> Option<String> {
    if let Some(text) = msg.strip_prefix("?echo ") {
        Some(text.to_string())
    } else {
        match msg {
            "ping" => Some("pong".to_string()),
            "?uptime" => Some(get_uptime()),
            "?help" => Some(HELP_STRING.to_string()),
            _ => None,
        }
    }
}

fn get_config() -> Config {
    let file = File::open("config.json").expect("failed to open config.json");
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Config`.
    let config = serde_json::from_reader(reader).expect("failed to read config.json values");

    return config;
}

fn get_password() -> String {
    let password = fs::read_to_string("password.txt").expect("couldn't read password.txt");

    return password.trim().to_string();
}

#[tokio::main]
async fn main() -> Result<()> {
    /* initialize uptime */
    LazyLock::force(&START_TIME);

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
        client
            .matrix_auth()
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
    client.add_event_handler(handle_invitation);

    println!("Bot is running...");

    // We use sync(settings) to start.
    // Note: With persistence, the bot automatically remembers where it left off!
    client.sync(SyncSettings::default()).await?;

    Ok(())
}

async fn on_room_message(event: OriginalSyncRoomMessageEvent, room: Room) {
    if let MessageType::Text(text_content) = &event.content.msgtype {
        if room.client().user_id() == Some(&event.sender) {
            return;
        }

        if let Some(response) = get_reply_text(text_content.body.as_str()) {
            println!(
                "Received {:?} in room: {:?}",
                text_content.body,
                room.room_id()
            );

            let content = RoomMessageEventContent::text_plain(response).make_reply_to(
                &event,
                ForwardThread::Yes,
                AddMentions::Yes,
            );

            room.send(content).await.ok();
        }
    }
}

pub async fn handle_invitation(ev: StrippedRoomMemberEvent, room: Room, client: Client) {
    // 1. Check if the room state is 'Invited'
    // In 0.16, we check the state() method
    if room.state() != RoomState::Invited {
        return;
    }

    // 2. Ensure the invite is for the bot
    let Some(user_id) = client.user_id() else {
        return;
    };
    if ev.state_key != user_id.to_string() {
        return;
    }

    // 3. Since we confirmed it's an invited room, we can use the
    // join method directly on the room object or cast it.
    println!("Accepting invite to room: {}", room.room_id());

    // In recent versions, Room provides a direct join method if it's invited
    match room.join().await {
        Ok(_) => println!("Successfully joined!"),
        Err(err) => {
            eprintln!("Failed to join room {}: {err}", room.room_id());
        }
    }
}
