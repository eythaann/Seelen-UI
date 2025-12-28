use std::sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock,
};

use discord_rich_presence::{
    activity::{Activity, Assets, Button, Party, Timestamps},
    DiscordIpc, DiscordIpcClient,
};

use crate::{
    error::Result, is_local_dev, state::application::FULL_STATE, utils::now_timestamp_as_millis,
    windows_api::event_window::IS_INTERACTIVE_SESSION,
};

use super::spawn_named_thread;

static DISCORD_IPC_CONNECTED: AtomicBool = AtomicBool::new(false);
static START_TIME: LazyLock<i64> = LazyLock::new(|| now_timestamp_as_millis() as i64);

static DETAILS: [&str; 22] = [
    "Personalizing my desktop",
    "Tweaking every pixel!",
    "Crafting the ultimate workspace",
    "Transforming my PC into art",
    "Running the smoothest UI ever",
    "Designing a futuristic interface",
    "Building a digital paradise",
    "Redefining desktop aesthetics",
    "Experimenting with UI alchemy",
    "Engineering the perfect theme",
    "Unleashing desktop superpowers",
    "Mixing colors, shapes, and textures",
    "Pushing customization limits",
    "Developing visual harmony",
    "Architecting the perfect layout",
    "Breathing life into pixels",
    "Curating icon perfection",
    "Revolutionizing my desktop experience",
    "Creating a digital masterpiece",
    "Yeap, this is my desktop",
    "Art is my passion",
    "Omg I love this",
];

static STATUSES: [&str; 20] = [
    "Brilliant",
    "Smart",
    "Powerful",
    "Creative",
    "Futuristic",
    "Pro Mode",
    "SLU",
    "Glowy",
    "Ravens",
    "Unicorns",
    "Seelen",
    "Next-Gen",
    "Ethereal",
    "Cyberpunk",
    "Minimalist",
    "Productive",
    "Magical",
    "Pixels",
    "Neon",
    "+Aura",
];

pub fn start_discord_rpc() -> Result<()> {
    if !FULL_STATE.load().settings.drpc {
        return Ok(());
    }

    spawn_named_thread("Discord IPC", || {
        let retry_conection_time = if is_local_dev() {
            std::time::Duration::from_secs(5)
        } else {
            std::time::Duration::from_secs(5 * 60) // 5 minutes
        };

        loop {
            log::trace!("Trying to connect to Discord IPC");
            let mut client = DiscordIpcClient::new("1384275226652704928").unwrap(); // never fails

            if client.connect().is_err() {
                log::trace!(
                    "Discord RPC not connected, retrying in {} seconds",
                    retry_conection_time.as_secs()
                );
                std::thread::sleep(retry_conection_time);
                continue;
            }

            log::info!("Discord IPC successfully connected");
            DISCORD_IPC_CONNECTED.store(true, Ordering::SeqCst);

            while DISCORD_IPC_CONNECTED.load(Ordering::SeqCst) {
                // Pause when session is not interactive to reduce CPU usage
                if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
                    std::thread::sleep(std::time::Duration::from_secs(60));
                    continue;
                }

                match client.set_activity(get_activity()) {
                    Ok(_) => {
                        log::trace!("Discord RPC activity updated");
                        // refresh every 10 minutes
                        std::thread::sleep(std::time::Duration::from_secs(600));
                    }
                    Err(_err) => {
                        DISCORD_IPC_CONNECTED.store(false, Ordering::SeqCst);
                        let _ = client.close();
                    }
                }
            }

            log::info!("Discord IPC disconnected");
            std::thread::sleep(retry_conection_time);
        }
    });
    Ok(())
}

pub fn get_activity() -> Activity<'static> {
    let random_detail = DETAILS[rand::random_range(0..DETAILS.len())];
    let random_status = STATUSES[rand::random_range(0..STATUSES.len())];

    Activity::new()
        .state(random_status)
        .details(random_detail)
        .assets(
            Assets::new()
                .large_image("app_logo")
                .large_text("Seelen UI")
                .small_image("seelen_corp_logo2")
                .small_text("Made by Seelen Corp."),
        )
        .timestamps(Timestamps::new().start(*START_TIME))
        .party(Party::new().id("seelen-ui-party").size([10, 10]))
        .buttons(vec![
            Button::new("🚀 Download Now!", "https://seelen.io/apps/seelen-ui"),
            Button::new(
                "🐦‍⬛ Seelen Network",
                "https://discord.com/invite/seelen-network-751144791596597561",
            ),
        ])
}
