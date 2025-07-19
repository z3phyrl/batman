use anyhow::Result;
use gtk4::{
    gdk::Display, glib::{spawn_future_local, ExitCode}, prelude::{ApplicationExt, ApplicationExtManual}, style_context_add_provider_for_display, Application, CssProvider, STYLE_PROVIDER_PRIORITY_USER
};
use sass_rs::{Options, compile_string};

mod overlay;
mod manager;

static APP_ID: &str = "com.z3phyrl.batman";

fn load_css(css: &str) {
    let provider = CssProvider::new();
    provider.load_from_string(css);

    style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        STYLE_PROVIDER_PRIORITY_USER,
    );
}

fn main() -> Result<ExitCode> {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        load_css(&compile_string(include_str!("style.scss"), Options::default()).unwrap())
    });
    app.connect_activate(manager::run);

    Ok(app.run())
}
