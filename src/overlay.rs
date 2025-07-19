use std::{cell::Cell, rc::Rc, time::Duration};

use gtk4::{
    gdk::Key, gio::spawn_blocking, glib::{clone, spawn_future_local, timeout_add, timeout_add_local, timeout_future, ControlFlow, Propagation}, prelude::{BoxExt, GtkWindowExt, WidgetExt}, Align, Application, ApplicationWindow, Box as gBox, EventControllerKey, Label, Orientation
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use std::process::Command;

use crate::manager::status;

static NAMESPACE: &str = "z3phyrl::batman";

static WIDTH: i32 = 1920;
static HEIGHT: i32 = 1080;

static _1ms: Duration = Duration::from_millis(1);

pub fn warn(
    app: &Application,
    title: Option<&str>,
    subtitle: Option<&str>,
    detail: Option<&str>,
    margin: Option<i32>,
) {
    let window = warning(app, title, subtitle, detail, margin);

    let key = EventControllerKey::new();

    key.connect_key_pressed(clone!(
        #[strong]
        window,
        move |_, key, _, _modifier| {
            match key {
                Key::Escape => window.set_visible(false),
                _ => {}
            }
            Propagation::Proceed
        }
    ));

    window.add_controller(key);

    window.present();
}

pub fn open(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(WIDTH)
        .default_height(HEIGHT)
        .build();

    window.init_layer_shell();
    window.set_namespace(Some(NAMESPACE));
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_keyboard_mode(KeyboardMode::Exclusive);

    window
}

fn warning(
    app: &Application,
    title: Option<&str>,
    subtitle: Option<&str>,
    detail: Option<&str>,
    margin: Option<i32>,
) -> ApplicationWindow {
    let window = open(app);

    let container = gBox::new(Orientation::Vertical, 25);

    let title = Label::new(title);
    title.add_css_class("title");
    let subtitle = Label::new(subtitle);
    subtitle.add_css_class("subtitle");
    let detail = Label::new(detail);
    detail.add_css_class("detail");

    title.set_valign(Align::Start);
    title.set_margin_top(margin.unwrap_or(125));

    container.append(&title);
    container.append(&subtitle);
    container.append(&detail);

    window.set_child(Some(&container));
    window.set_css_classes(&["warning"]);
    window
}

fn countdown_ui(app: &Application) -> (ApplicationWindow, Label, Label) {
    let window = ApplicationWindow::builder()
        .application(app)
        .css_classes(["countdown-window"])
        // .default_height(360)
        .default_width(480)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Overlay);

    let container = gBox::default();

    let battery = Label::new(Some("󰁺"));
    battery.add_css_class("battery");

    let countdown = Label::new(Some("60.000"));
    countdown.add_css_class("countdown");

    container.append(&battery);
    container.append(&countdown);

    window.set_child(Some(&container));

    (window, battery, countdown)
}

pub fn countdown(app: &Application, mut duration: Duration) {
    let warning = warning(app, Some("POWER OFF"), Some("low battery"), None, None);
    let (countdown_window, battery, countdown) = countdown_ui(app);
    let key = EventControllerKey::new();

    key.connect_key_pressed(clone!(
        #[strong]
        warning,
        #[strong]
        countdown_window,
        move |_, key, _, _modifier| {
            match key {
                Key::Escape => {
                    warning.set_visible(false);
                    countdown_window.set_anchor(Edge::Bottom, true);
                    countdown_window.set_anchor(Edge::Right, true);
                }
                _ => {}
            }
            Propagation::Proceed
        }
    ));

    warning.add_controller(key);
    warning.present();
    countdown_window.present();

    let is_charging = Rc::new(Cell::new(false));
    let check = timeout_add_local(Duration::from_millis(50), clone!(#[strong] is_charging, move || {
        is_charging.set(status().unwrap_or_default().trim() == "Charging");
        ControlFlow::Continue
    }));

    spawn_future_local(async move {
        let red = duration.mul_f32(0.2);
        while duration > _1ms && !is_charging.get() {
            timeout_future(_1ms).await;
            duration -= _1ms;
            if duration < red {
                battery.add_css_class("red");
                countdown.add_css_class("red");
            }
            countdown.set_text(&format!("{:06}", duration.as_millis()));
        }
        warning.set_visible(false);
        countdown_window.set_visible(false);
        check.remove();
        if !is_charging.get() {
            let _ = spawn_blocking(|| Command::new("shutdown").args(&["-h", "now"]).spawn()).await;
        }
    });
}
