use adw::prelude::*;
use gtk::{gio, glib};
use poltergeist_backends::{
    dry_run::{DryRunInput, DryRunPower},
    NativeInput, NativePower,
};
use poltergeist_core::{
    state::new_state, Button, Config, Engine, InputBackend, Mode, MovementStyle, Paths,
    PowerBackend, SharedConfig, SharedState,
};
use std::{
    cell::{Cell, RefCell},
    f64::consts::PI,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const APP_ID: &str = "org.sorabbyngo.Poltergeist";

pub fn run() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

// ------------------------------------------------------------------ controller
/// Owns the config, the live telemetry and the engine thread.
struct Controller {
    cfg: SharedConfig,
    shared: SharedState,
    stop: RefCell<Option<Arc<AtomicBool>>>,
    worker: RefCell<Option<JoinHandle<()>>>,
    simulate: Cell<bool>,
    dirty_since: Cell<Option<Instant>>,
}

type Backends = (Box<dyn InputBackend + Send>, Box<dyn PowerBackend + Send>);

fn dry() -> Backends {
    (Box::new(DryRunInput::quiet()), Box::new(DryRunPower::quiet()))
}

impl Controller {
    fn new() -> Rc<Self> {
        let cfg = Config::load_or_create().unwrap_or_default();
        Rc::new(Self {
            cfg: cfg.shared(),
            shared: new_state(),
            stop: RefCell::new(None),
            worker: RefCell::new(None),
            simulate: Cell::new(false),
            dirty_since: Cell::new(None),
        })
    }

    fn running(&self) -> bool {
        self.worker.borrow().is_some()
    }

    fn read(&self) -> Config {
        self.cfg.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    fn edit(&self, f: impl FnOnce(&mut Config)) {
        let mut c = self.cfg.write().unwrap_or_else(|e| e.into_inner());
        let before = c.clone();
        f(&mut c);
        c.sanitize();
        if *c != before {
            self.dirty_since.set(Some(Instant::now()));
        }
    }

    fn save_now(&self) {
        let _ = self.read().save();
        self.dirty_since.set(None);
    }

    /// Returns a notice to show the user if real input was unavailable.
    fn start(&self) -> Option<String> {
        self.save_now();
        let mut notice = None;
        let (input, power) = if self.simulate.get() {
            dry()
        } else {
            match NativeInput::new() {
                Ok(i) => (Box::new(i) as Box<dyn InputBackend + Send>, Box::new(NativePower::default()) as Box<dyn PowerBackend + Send>),
                Err(e) => {
                    notice = Some(format!("Real input is unavailable ({e}). Running in simulation."));
                    self.simulate.set(true);
                    dry()
                }
            }
        };
        let stop = Arc::new(AtomicBool::new(false));
        let (cfg, state, s2) = (self.cfg.clone(), self.shared.clone(), stop.clone());
        let handle = thread::spawn(move || {
            let mut engine = Engine::new(cfg, input, power, s2, state.clone());
            if let Err(e) = engine.run() {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.error = Some(e.to_string());
                s.running = false;
            }
        });
        *self.stop.borrow_mut() = Some(stop);
        *self.worker.borrow_mut() = Some(handle);
        notice
    }

    fn stop(&self) {
        if let Some(s) = self.stop.borrow_mut().take() {
            s.store(true, Ordering::Relaxed);
        }
        if let Some(h) = self.worker.borrow_mut().take() {
            let _ = h.join();
        }
    }

    /// Cleans up a worker that ended by itself (for example after an error).
    fn reap(&self) -> Option<String> {
        let done = self.worker.borrow().as_ref().map(|h| h.is_finished()).unwrap_or(false);
        if !done {
            return None;
        }
        if let Some(h) = self.worker.borrow_mut().take() {
            let _ = h.join();
        }
        *self.stop.borrow_mut() = None;
        self.shared.lock().unwrap_or_else(|e| e.into_inner()).error.take().map(|e| format!("Engine stopped: {e}"))
    }
}

// --------------------------------------------------------------- row helpers
type Syncer = Box<dyn Fn()>;

fn bind_spin(
    ctl: &Rc<Controller>,
    group: &adw::PreferencesGroup,
    title: &str,
    (min, max, step): (f64, f64, f64),
    digits: u32,
    scale: f64,
    get: fn(&Config) -> f64,
    set: fn(&mut Config, f64),
    syncers: &mut Vec<Syncer>,
) -> adw::SpinRow {
    let row = adw::SpinRow::with_range(min, max, step);
    row.set_title(title);
    row.set_digits(digits);
    row.set_value(get(&ctl.read()) * scale);
    let c = ctl.clone();
    row.connect_value_notify(move |r| {
        let v = r.value() / scale;
        c.edit(|cfg| set(cfg, v));
    });
    let (c, r2) = (ctl.clone(), row.clone());
    syncers.push(Box::new(move || {
        let want = get(&c.read()) * scale;
        if (r2.value() - want).abs() > 1e-6 {
            r2.set_value(want);
        }
    }));
    group.add(&row);
    row
}

fn bind_switch(
    ctl: &Rc<Controller>,
    group: &adw::PreferencesGroup,
    title: &str,
    get: fn(&Config) -> bool,
    set: fn(&mut Config, bool),
    syncers: &mut Vec<Syncer>,
) {
    let row = adw::SwitchRow::new();
    row.set_title(title);
    row.set_active(get(&ctl.read()));
    let c = ctl.clone();
    row.connect_active_notify(move |r| {
        let v = r.is_active();
        c.edit(|cfg| set(cfg, v));
    });
    let (c, r2) = (ctl.clone(), row.clone());
    syncers.push(Box::new(move || {
        let want = get(&c.read());
        if r2.is_active() != want {
            r2.set_active(want);
        }
    }));
    group.add(&row);
}

fn bind_combo(
    ctl: &Rc<Controller>,
    group: &adw::PreferencesGroup,
    title: &str,
    options: &[&str],
    get: fn(&Config) -> u32,
    set: fn(&mut Config, u32),
) {
    let row = adw::ComboRow::new();
    row.set_title(title);
    row.set_model(Some(&gtk::StringList::new(options)));
    row.set_selected(get(&ctl.read()));
    let c = ctl.clone();
    row.connect_selected_notify(move |r| {
        let v = r.selected();
        c.edit(|cfg| set(cfg, v));
    });
    group.add(&row);
}

// ------------------------------------------------------------------------ UI
fn build_ui(app: &adw::Application) {
    let ctl = Controller::new();
    let cfg0 = ctl.read();
    let paths = Paths::init(cfg0.output_root.clone()).ok(); // creates Poltergeist/screenshots + screen-recordings
    let mut syncers: Vec<Syncer> = Vec::new();

    // ---- settings (left)
    let page = adw::PreferencesPage::new();
    page.set_hexpand(false);
    page.set_width_request(430);

    let g = adw::PreferencesGroup::new();
    g.set_title("What should it do?");
    bind_combo(&ctl, &g, "Mode", &["Keep awake", "Move cursor", "Both"],
        |c| match c.mode { Mode::KeepAwake => 0, Mode::Cursor => 1, Mode::Both => 2 },
        |c, v| c.mode = match v { 0 => Mode::KeepAwake, 1 => Mode::Cursor, _ => Mode::Both });
    page.add(&g);

    let g = adw::PreferencesGroup::new();
    g.set_title("Cursor movement");
    bind_combo(&ctl, &g, "Style", &["Random", "Jitter", "Circle"],
        |c| match c.style { MovementStyle::Random => 0, MovementStyle::Jitter => 1, MovementStyle::Circle => 2 },
        |c, v| c.style = match v { 0 => MovementStyle::Random, 1 => MovementStyle::Jitter, _ => MovementStyle::Circle });
    bind_spin(&ctl, &g, "Minimum wait (s)", (0.2, 120.0, 0.5), 1, 1.0, |c| c.min_interval_s, |c, v| c.min_interval_s = v, &mut syncers);
    bind_spin(&ctl, &g, "Maximum wait (s)", (0.2, 120.0, 0.5), 1, 1.0, |c| c.max_interval_s, |c, v| c.max_interval_s = v, &mut syncers);
    bind_spin(&ctl, &g, "Range (% of screen)", (1.0, 50.0, 1.0), 0, 100.0, |c| c.radius as f64, |c, v| c.radius = v as f32, &mut syncers);
    bind_switch(&ctl, &g, "Human-like curved paths", |c| c.humanize, |c, v| c.humanize = v, &mut syncers);
    page.add(&g);

    let g = adw::PreferencesGroup::new();
    g.set_title("Auto-click");
    bind_switch(&ctl, &g, "Click automatically", |c| c.click.enabled, |c, v| c.click.enabled = v, &mut syncers);
    bind_combo(&ctl, &g, "Button", &["Left", "Right", "Middle"],
        |c| match c.click.button { Button::Left => 0, Button::Right => 1, Button::Middle => 2 },
        |c, v| c.click.button = match v { 0 => Button::Left, 1 => Button::Right, _ => Button::Middle });
    let spot_x = bind_spin(&ctl, &g, "Spot X (%)", (0.0, 100.0, 1.0), 1, 100.0, |c| c.click.x as f64, |c, v| c.click.x = v as f32, &mut syncers);
    let spot_y = bind_spin(&ctl, &g, "Spot Y (%)", (0.0, 100.0, 1.0), 1, 100.0, |c| c.click.y as f64, |c, v| c.click.y = v as f32, &mut syncers);
    bind_spin(&ctl, &g, "Minimum wait (s)", (0.2, 300.0, 0.5), 1, 1.0, |c| c.click.min_interval_s, |c, v| c.click.min_interval_s = v, &mut syncers);
    bind_spin(&ctl, &g, "Maximum wait (s)", (0.2, 300.0, 0.5), 1, 1.0, |c| c.click.max_interval_s, |c, v| c.click.max_interval_s = v, &mut syncers);
    bind_spin(&ctl, &g, "Hold at least (ms)", (10.0, 400.0, 5.0), 0, 1.0, |c| c.click.hold_ms_min as f64, |c, v| c.click.hold_ms_min = v as u64, &mut syncers);
    bind_spin(&ctl, &g, "Hold at most (ms)", (10.0, 400.0, 5.0), 0, 1.0, |c| c.click.hold_ms_max as f64, |c, v| c.click.hold_ms_max = v as u64, &mut syncers);
    bind_spin(&ctl, &g, "Stop after clicks (0 = never)", (0.0, 1000.0, 1.0), 0, 1.0, |c| c.click.max_clicks as f64, |c, v| c.click.max_clicks = v as u32, &mut syncers);
    page.add(&g);

    // The simulation switch lives in the controller, not the config.
    let g = adw::PreferencesGroup::new();
    g.set_title("Safety");
    let sim_row = adw::SwitchRow::new();
    sim_row.set_title("Simulation only");
    sim_row.set_subtitle("Never touches your real mouse");
    sim_row.set_active(ctl.simulate.get());
    {
        let c = ctl.clone();
        sim_row.connect_active_notify(move |r| c.simulate.set(r.is_active()));
    }
    g.add(&sim_row);
    page.add(&g);

    // ---- preview + test pad + stats (right)
    let right = gtk::Box::new(gtk::Orientation::Vertical, 14);
    right.set_margin_top(18);
    right.set_margin_bottom(18);
    right.set_margin_start(18);
    right.set_margin_end(18);
    right.set_hexpand(true);

    let title = gtk::Label::new(Some("Live preview"));
    title.add_css_class("heading");
    title.set_xalign(0.0);
    right.append(&title);

    let area = gtk::DrawingArea::new();
    area.set_content_width(480);
    area.set_content_height(270);
    area.set_hexpand(true);
    {
        let (cfg, shared) = (ctl.cfg.clone(), ctl.shared.clone());
        area.set_draw_func(move |_, cr, w, h| {
            let (w, h) = (w as f64, h as f64);
            cr.set_source_rgb(0.07, 0.09, 0.12);
            cr.rectangle(0.0, 0.0, w, h);
            let _ = cr.fill();
            cr.set_source_rgba(1.0, 1.0, 1.0, 0.06);
            cr.set_line_width(1.0);
            for i in 1..10 {
                let f = i as f64 / 10.0;
                cr.move_to(w * f, 0.0);
                cr.line_to(w * f, h);
                cr.move_to(0.0, h * f);
                cr.line_to(w, h * f);
            }
            let _ = cr.stroke();

            let (target, enabled) = {
                let c = cfg.read().unwrap_or_else(|e| e.into_inner());
                ((c.click.x as f64, c.click.y as f64), c.click.enabled)
            };
            let (pos, pressed, running, clicks) = {
                let s = shared.lock().unwrap_or_else(|e| e.into_inner());
                let clicks: Vec<(f64, f64, f64)> = s
                    .recent_clicks
                    .iter()
                    .map(|c| (c.x as f64, c.y as f64, c.at.elapsed().as_secs_f64()))
                    .collect();
                ((s.pos.0 as f64, s.pos.1 as f64), s.pressed, s.running, clicks)
            };

            if enabled {
                let (tx, ty) = (target.0 * w, target.1 * h);
                cr.set_source_rgb(1.0, 0.78, 0.24);
                cr.set_line_width(1.5);
                cr.arc(tx, ty, 10.0, 0.0, 2.0 * PI);
                cr.move_to(tx - 16.0, ty);
                cr.line_to(tx + 16.0, ty);
                cr.move_to(tx, ty - 16.0);
                cr.line_to(tx, ty + 16.0);
                let _ = cr.stroke();
            }
            for (x, y, age) in clicks.iter().filter(|c| c.2 < 0.9) {
                cr.set_source_rgba(0.31, 0.78, 1.0, 1.0 - age / 0.9);
                cr.set_line_width(3.0);
                cr.arc(x * w, y * h, 8.0 + age * 60.0, 0.0, 2.0 * PI);
                let _ = cr.stroke();
            }
            if running {
                let (px, py) = (pos.0 * w, pos.1 * h);
                if pressed {
                    cr.set_source_rgba(1.0, 0.59, 0.16, 0.5);
                    cr.arc(px, py, 22.0, 0.0, 2.0 * PI);
                    let _ = cr.fill();
                }
                cr.set_source_rgb(1.0, 0.31, 0.31);
                cr.set_line_width(3.0);
                cr.arc(px, py, 14.0, 0.0, 2.0 * PI);
                let _ = cr.stroke();
                cr.arc(px, py, 4.0, 0.0, 2.0 * PI);
                let _ = cr.fill();
            }
        });
    }
    // Click or drag in the preview to choose where auto-click lands.
    let set_target: Rc<dyn Fn(f64, f64)> = {
        let (ctl, area, sx, sy) = (ctl.clone(), area.clone(), spot_x.clone(), spot_y.clone());
        Rc::new(move |x, y| {
            let (w, h) = (area.width().max(1) as f64, area.height().max(1) as f64);
            let (fx, fy) = ((x / w).clamp(0.0, 1.0), (y / h).clamp(0.0, 1.0));
            ctl.edit(|c| {
                c.click.x = fx as f32;
                c.click.y = fy as f32;
            });
            sx.set_value(fx * 100.0);
            sy.set_value(fy * 100.0);
        })
    };
    let press = gtk::GestureClick::new();
    {
        let st = set_target.clone();
        press.connect_pressed(move |_, _, x, y| st(x, y));
    }
    area.add_controller(press);
    let drag = gtk::GestureDrag::new();
    {
        let st = set_target.clone();
        drag.connect_drag_update(move |g, ox, oy| {
            if let Some((sx, sy)) = g.start_point() {
                st(sx + ox, sy + oy);
            }
        });
    }
    area.add_controller(drag);
    let frame = gtk::AspectFrame::new(0.5, 0.5, 16.0 / 9.0, false);
    frame.set_child(Some(&area));
    right.append(&frame);
    let hint = gtk::Label::new(Some("Click or drag inside the preview to choose where auto-click lands."));
    hint.add_css_class("dim-label");
    hint.add_css_class("caption");
    hint.set_xalign(0.0);
    right.append(&hint);

    // Test pad: proves the operating system really receives the clicks.
    let pad_title = gtk::Label::new(Some("Test pad: proves clicks are real"));
    pad_title.add_css_class("heading");
    pad_title.set_xalign(0.0);
    right.append(&pad_title);
    let pad_count = Rc::new(Cell::new(0u32));
    let pad = gtk::Button::with_label("🎯  0 clicks received");
    pad.add_css_class("pill");
    pad.set_height_request(72);
    {
        let n = pad_count.clone();
        pad.connect_clicked(move |b| {
            n.set(n.get() + 1);
            b.set_label(&format!("🎯  {} clicks received", n.get()));
            b.add_css_class("success");
            let b2 = b.clone();
            glib::timeout_add_local_once(Duration::from_millis(300), move || b2.remove_css_class("success"));
        });
    }
    right.append(&pad);
    let pad_hint = gtk::Label::new(Some(
        "This button counts clicks the operating system delivers to it. Set the spot over this button and watch the counter rise.",
    ));
    pad_hint.add_css_class("dim-label");
    pad_hint.add_css_class("caption");
    pad_hint.set_wrap(true);
    pad_hint.set_xalign(0.0);
    right.append(&pad_hint);

    let stats = gtk::Label::new(None);
    stats.set_xalign(0.0);
    right.append(&stats);
    if let Some(p) = &paths {
        let l = gtk::Label::new(Some(&format!("Screenshots: {}\nRecordings: {}", p.screenshots.display(), p.recordings.display())));
        l.add_css_class("dim-label");
        l.add_css_class("caption");
        l.set_xalign(0.0);
        l.set_selectable(true);
        right.append(&l);
    }
    let log = gtk::Label::new(None);
    log.add_css_class("monospace");
    log.add_css_class("caption");
    log.set_xalign(0.0);
    right.append(&log);

    let right_scroll = gtk::ScrolledWindow::new();
    right_scroll.set_child(Some(&right));
    right_scroll.set_hexpand(true);
    right_scroll.set_hscrollbar_policy(gtk::PolicyType::Never);

    let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    root.append(&page);
    root.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    root.append(&right_scroll);

    // ---- window with a native header bar (real minimize / maximize / close)
    let start = gtk::Button::with_label("Start");
    start.add_css_class("suggested-action");
    let header = adw::HeaderBar::new();
    header.pack_start(&start);
    header.set_title_widget(Some(&adw::WindowTitle::new("Poltergeist", "offline · local only")));

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&root));
    let toasts = adw::ToastOverlay::new();
    toasts.set_child(Some(&toolbar));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Poltergeist")
        .default_width(1060)
        .default_height(780)
        .content(&toasts)
        .build();
    gtk::Window::set_default_icon_name(APP_ID);

    {
        let (ctl, toasts) = (ctl.clone(), toasts.clone());
        start.connect_clicked(move |_| {
            if ctl.running() {
                ctl.stop();
            } else if let Some(msg) = ctl.start() {
                toasts.add_toast(adw::Toast::new(&msg));
            }
        });
    }

    // F11 toggles full screen. The native header bar always offers a way back too.
    let fs = gio::SimpleAction::new("fullscreen", None);
    {
        let w = window.clone();
        fs.connect_activate(move |_, _| w.set_fullscreened(!w.is_fullscreen()));
    }
    window.add_action(&fs);
    app.set_accels_for_action("win.fullscreen", &["F11"]);

    // ---- one timer drives redraws, stats, button state and debounced saving
    {
        let (ctl, area, start, stats, log, toasts, sim_row) =
            (ctl.clone(), area.clone(), start.clone(), stats.clone(), log.clone(), toasts.clone(), sim_row.clone());
        glib::timeout_add_local(Duration::from_millis(33), move || {
            if let Some(msg) = ctl.reap() {
                toasts.add_toast(adw::Toast::new(&msg));
            }
            for s in &syncers {
                s();
            }
            if sim_row.is_active() != ctl.simulate.get() {
                sim_row.set_active(ctl.simulate.get());
            }
            if ctl.running() {
                start.set_label("Stop");
                start.remove_css_class("suggested-action");
                start.add_css_class("destructive-action");
            } else {
                start.set_label("Start");
                start.remove_css_class("destructive-action");
                start.add_css_class("suggested-action");
            }
            {
                let s = ctl.shared.lock().unwrap_or_else(|e| e.into_inner());
                let up = match s.started_at.filter(|_| s.running) {
                    Some(t) => format!("running for {}s{}", t.elapsed().as_secs(), if ctl.simulate.get() { " (simulation)" } else { "" }),
                    None => "stopped".to_string(),
                };
                stats.set_text(&format!("Moves: {}   ·   Auto-clicks: {}   ·   {}", s.move_count, s.click_count, up));
                log.set_text(&s.log.iter().rev().take(8).cloned().collect::<Vec<_>>().join("\n"));
            }
            area.queue_draw();
            if let Some(t) = ctl.dirty_since.get() {
                if t.elapsed() > Duration::from_millis(600) {
                    ctl.save_now();
                }
            }
            glib::ControlFlow::Continue
        });
    }

    {
        let ctl = ctl.clone();
        window.connect_close_request(move |_| {
            ctl.stop();
            ctl.save_now();
            glib::Propagation::Proceed
        });
    }
    window.present();
}
