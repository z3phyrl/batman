use std::{panic, time::Duration};

use gtk4::{
    Application,
    glib::{spawn_future_local, timeout_future_seconds},
};
use notify_rust::Notification;

use crate::overlay::{countdown, open, warn};

static NOTIFY: u8 = 30; // notify low batt
static WARN: u8 = 20; // full screen warning
static FORCE: u8 = 10; // full screen 1 minute countdown to shutdown

fn percent() -> Option<u8> {
    if let Ok(percent) = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
        percent.trim().parse::<u8>().ok()
    } else {
        None
    }
}
pub fn status() -> Option<String> {
    std::fs::read_to_string("/sys/class/power_supply/BAT0/status").ok()
}

pub fn run(app: &Application) {
    println!("Hello, World!");

    let _window = open(app);

    let app = app.to_owned();
    spawn_future_local(async move {
        let mut notified = false;
        let mut warned = false;
        let mut forced = false;
        let Some(mut current) = percent() else {
            panic!("Please install some battery.");
        };

        loop {
            timeout_future_seconds(30).await;
            // println!("loop");
            let Some(percent) = percent() else { continue };
            // println!("{percent}");
            if percent == current {
                continue;
            } else {
                current = percent;
            }

            let Some(status) = status() else { continue };
            if status.trim() == "Charging" {
                if notified && percent > NOTIFY {
                    notified = false;
                }
                if warned && percent > WARN {
                    warned = false;
                }
                if forced && percent > FORCE {
                    forced = false;
                }
                continue;
            }
            // println!("{status}");
            if percent <= NOTIFY && !notified {
                let _ = Notification::new()
                    .summary("LOW BATTERY")
                    .body("A lil low battery. PLUG ME IN!!!")
                    .show_async()
                    .await;
                notified = true;
            } else if percent <= WARN && !warned {
                warn(&app, Some("WARNING"), Some("low battery"), None, Some(256));
                warned = true;
            } else if percent <= FORCE && !forced {
                countdown(&app, Duration::from_secs(10));
                forced = true;
            }
        }
    });
}
