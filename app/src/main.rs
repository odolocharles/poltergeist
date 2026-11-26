//! Poltergeist desktop app: pure Rust (egui), no webview, no network.
//! The engine runs on a background thread; every setting applies live.

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use poltergeist_backends::{
    dry_run::{DryRunInput, DryRunPower},
    NativeInput, NativePower,
};
use poltergeist_core::{
    state::new_state, Button, Config, Engine, InputBackend, Mode, MovementStyle, Paths,
    PowerBackend, SharedConfig, SharedState,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

/// Linux desktops (notably GNOME on Wayland) often give toolkit windows no usable
/// title bar, so on Linux the app draws its own window controls. Windows and macOS
/// keep their native frame.
const CUSTOM_CHROME: bool = cfg!(target_os = "linux");

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Poltergeist")
        .with_app_id("poltergeist")
        .with_decorations(!CUSTOM_CHROME)
        .with_inner_size([1040.0, 760.0])
        .with_min_inner_size([820.0, 600.0]);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../../assets/poltergeist.png")) {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    eframe::run_native(
        "Poltergeist",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(App::new()))
        }),
    )
}

/// A frame-local copy of the engine telemetry, so the UI never holds the lock while drawing.
struct Snap {
    running: bool,
    pos: (f32, f32),
    pressed: bool,
    clicks: Vec<(f32, f32, f32)>, // x, y, age in seconds
    click_count: u64,
    move_count: u64,
    uptime: Option<Duration>,
    log: Vec<String>,
    error: Option<String>,
}

struct App {
    cfg: SharedConfig,
    shared: SharedState,
    stop: Option<Arc<AtomicBool>>,
    worker: Option<JoinHandle<()>>,
    simulate: bool,
    dirty_since: Option<Instant>,
    notice: String,
    paths: Option<Paths>,
    test_clicks: u32,
    test_flash: Option<Instant>,
    test_pad: Option<Rect>,
}

impl App {
    fn new() -> Self {
        let cfg = Config::load_or_create().unwrap_or_default();
        // Create Poltergeist/screenshots and Poltergeist/screen-recordings on first run.
        let paths = Paths::init(cfg.output_root.clone()).ok();
        Self {
            cfg: cfg.shared(),
            shared: new_state(),
            stop: None,
            worker: None,
            simulate: false,
            dirty_since: None,
            notice: String::new(),
            paths,
            test_clicks: 0,
            test_flash: None,
            test_pad: None,
        }
    }

    fn running(&self) -> bool {
        self.worker.is_some()
    }

    fn snapshot(&self) -> Snap {
        let s = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        Snap {
            running: s.running,
            pos: s.pos,
            pressed: s.pressed,
            clicks: s
                .recent_clicks
                .iter()
                .map(|c| (c.x, c.y, now.duration_since(c.at).as_secs_f32()))
                .collect(),
            click_count: s.click_count,
            move_count: s.move_count,
            uptime: s.started_at.filter(|_| s.running).map(|t| t.elapsed()),
            log: s.log.iter().rev().take(60).cloned().collect(),
            error: s.error.clone(),
        }
    }

    fn save_now(&mut self) {
        let cfg = self.cfg.read().unwrap_or_else(|e| e.into_inner()).clone();
        if let Err(e) = cfg.save() {
            self.notice = format!("Could not save settings: {e}");
        }
        self.dirty_since = None;
    }

    fn make_backends(&mut self) -> (Box<dyn InputBackend + Send>, Box<dyn PowerBackend + Send>) {
        if !self.simulate {
            match NativeInput::new() {
                Ok(i) => return (Box::new(i), Box::new(NativePower::default())),
                Err(e) => {
                    self.notice = format!("Real input is unavailable ({e}). Running in simulation instead.");
                    self.simulate = true;
                }
            }
        }
        (Box::new(DryRunInput::quiet()), Box::new(DryRunPower::quiet()))
    }

    fn start_engine(&mut self) {
        self.save_now();
        self.notice.clear();
        let (input, power) = self.make_backends();
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
        self.stop = Some(stop);
        self.worker = Some(handle);
    }

    fn stop_engine(&mut self) {
        if let Some(s) = self.stop.take() {
            s.store(true, Ordering::Relaxed);
        }
        if let Some(h) = self.worker.take() {
            let _ = h.join();
        }
    }

    // ---------------------------------------------------------------- settings
    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        let cfg_arc = self.cfg.clone();
        let mut cfg = cfg_arc.write().unwrap_or_else(|e| e.into_inner());
        let before = cfg.clone();

        ui.add_space(4.0);
        ui.label(RichText::new("What should it do?").strong());
        ui.horizontal(|ui| {
            ui.selectable_value(&mut cfg.mode, Mode::KeepAwake, "Keep awake");
            ui.selectable_value(&mut cfg.mode, Mode::Cursor, "Move cursor");
            ui.selectable_value(&mut cfg.mode, Mode::Both, "Both");
        });

        ui.add_space(8.0);
        ui.label(RichText::new("Cursor movement").strong());
        ui.horizontal(|ui| {
            ui.selectable_value(&mut cfg.style, MovementStyle::Random, "Random");
            ui.selectable_value(&mut cfg.style, MovementStyle::Jitter, "Jitter");
            ui.selectable_value(&mut cfg.style, MovementStyle::Circle, "Circle");
        });
        ui.add(egui::Slider::new(&mut cfg.min_interval_s, 0.2..=120.0).logarithmic(true).text("min wait").suffix(" s"));
        ui.add(egui::Slider::new(&mut cfg.max_interval_s, 0.2..=120.0).logarithmic(true).text("max wait").suffix(" s"));
        ui.add(egui::Slider::new(&mut cfg.radius, 0.01..=0.5).text("range"));
        ui.checkbox(&mut cfg.humanize, "Human-like curved paths");
        ui.add(egui::Slider::new(&mut cfg.idle_key_chance, 0.0..=1.0).text("idle key tap chance"));

        ui.add_space(10.0);
        ui.separator();
        ui.label(RichText::new("Auto-click").strong());
        ui.checkbox(&mut cfg.click.enabled, "Click automatically");
        ui.add_enabled_ui(cfg.click.enabled, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut cfg.click.button, Button::Left, "Left");
                ui.selectable_value(&mut cfg.click.button, Button::Right, "Right");
                ui.selectable_value(&mut cfg.click.button, Button::Middle, "Middle");
            });
            let mut xp = cfg.click.x * 100.0;
            if ui.add(egui::Slider::new(&mut xp, 0.0..=100.0).text("spot X").suffix("%")).changed() {
                cfg.click.x = xp / 100.0;
            }
            let mut yp = cfg.click.y * 100.0;
            if ui.add(egui::Slider::new(&mut yp, 0.0..=100.0).text("spot Y").suffix("%")).changed() {
                cfg.click.y = yp / 100.0;
            }
            ui.label(RichText::new("Tip: click inside the preview to pick the spot.").weak().small());
            ui.add(egui::Slider::new(&mut cfg.click.min_interval_s, 0.2..=300.0).logarithmic(true).text("min wait").suffix(" s"));
            ui.add(egui::Slider::new(&mut cfg.click.max_interval_s, 0.2..=300.0).logarithmic(true).text("max wait").suffix(" s"));
            ui.add(egui::Slider::new(&mut cfg.click.hold_ms_min, 10..=400).text("hold min").suffix(" ms"));
            ui.add(egui::Slider::new(&mut cfg.click.hold_ms_max, 10..=400).text("hold max").suffix(" ms"));
            ui.add(egui::Slider::new(&mut cfg.click.scatter, 0.0..=0.05).text("scatter"));
            ui.add(egui::Slider::new(&mut cfg.click.max_clicks, 0..=1000).text("max clicks (0 = no limit)"));
        });

        ui.add_space(10.0);
        ui.separator();
        ui.label(RichText::new("Display and safety").strong());
        ui.checkbox(&mut cfg.show_overlay, "Show pointer overlay on screen");
        ui.checkbox(&mut self.simulate, "Simulation only (never touches your real mouse)");

        cfg.sanitize();
        if *cfg != before {
            self.dirty_since = Some(Instant::now());
        }
    }

    // ------------------------------------------------------------------- main
    fn main_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, snap: &Snap) {
        ui.add_space(4.0);
        ui.label(RichText::new("Live preview").strong());
        let width = ui.available_width().min(700.0);
        let (resp, painter) = ui.allocate_painter(Vec2::new(width, width * 9.0 / 16.0), Sense::click_and_drag());
        let rect = resp.rect;
        painter.rect_filled(rect, 6.0, Color32::from_rgb(18, 22, 30));
        for i in 1..10 {
            let f = i as f32 / 10.0;
            let c = Color32::from_rgba_unmultiplied(255, 255, 255, 10);
            painter.line_segment([Pos2::new(rect.left() + rect.width() * f, rect.top()), Pos2::new(rect.left() + rect.width() * f, rect.bottom())], Stroke::new(1.0, c));
            painter.line_segment([Pos2::new(rect.left(), rect.top() + rect.height() * f), Pos2::new(rect.right(), rect.top() + rect.height() * f)], Stroke::new(1.0, c));
        }
        let (target, enabled) = {
            let c = self.cfg.read().unwrap_or_else(|e| e.into_inner());
            ((c.click.x, c.click.y), c.click.enabled)
        };
        if enabled {
            let p = to_screen(rect, target);
            let col = Color32::from_rgb(255, 200, 60);
            painter.circle_stroke(p, 10.0, Stroke::new(1.5, col));
            painter.line_segment([p - Vec2::new(16.0, 0.0), p + Vec2::new(16.0, 0.0)], Stroke::new(1.5, col));
            painter.line_segment([p - Vec2::new(0.0, 16.0), p + Vec2::new(0.0, 16.0)], Stroke::new(1.5, col));
        }
        paint_pointer(&painter, rect, snap, 1.0);
        if resp.clicked() || resp.dragged() {
            if let Some(p) = resp.interact_pointer_pos() {
                let mut c = self.cfg.write().unwrap_or_else(|e| e.into_inner());
                c.click.x = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
                c.click.y = ((p.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
                self.dirty_since = Some(Instant::now());
            }
        }
        painter.text(rect.left_top() + Vec2::new(8.0, 6.0), Align2::LEFT_TOP, if self.simulate { "SIMULATION" } else { "LIVE" }, FontId::proportional(11.0), Color32::from_gray(140));

        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label(RichText::new("Test pad: proves clicks are real").strong());
            ui.label(RichText::new("This button counts clicks the operating system delivers to it. Aim the auto-click here and watch the counter rise.").weak().small());
            let flashing = self.test_flash.map(|t| t.elapsed() < Duration::from_millis(300)).unwrap_or(false);
            let fill = if flashing { Color32::from_rgb(40, 160, 90) } else { Color32::from_rgb(45, 55, 75) };
            let btn = egui::Button::new(RichText::new(format!("🎯  {} clicks received", self.test_clicks)).size(22.0)).fill(fill).min_size(Vec2::new(300.0, 80.0));
            let r = ui.add(btn);
            if r.clicked() {
                self.test_clicks += 1;
                self.test_flash = Some(Instant::now());
            }
            self.test_pad = Some(r.rect);
            ui.horizontal(|ui| {
                if ui.button("Aim auto-click at test pad").clicked() {
                    self.aim_at_test_pad(ctx);
                }
                if ui.button("Reset counter").clicked() {
                    self.test_clicks = 0;
                }
            });
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(format!("Moves: {}", snap.move_count));
            ui.separator();
            ui.label(format!("Auto-clicks: {}", snap.click_count));
            ui.separator();
            ui.label(match snap.uptime {
                Some(d) => format!("Running for {}s", d.as_secs()),
                None => "Stopped".to_string(),
            });
        });
        if let Some(p) = &self.paths {
            ui.label(RichText::new(format!("Screenshots: {}", p.screenshots.display())).weak().small());
            ui.label(RichText::new(format!("Recordings:  {}", p.recordings.display())).weak().small());
        }
        ui.add_space(6.0);
        egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
            for line in &snap.log {
                ui.label(RichText::new(line).monospace().small());
            }
        });
    }

    fn aim_at_test_pad(&mut self, ctx: &egui::Context) {
        let Some(pad) = self.test_pad else { return };
        let vp = ctx.input(|i| i.viewport().clone());
        match (vp.inner_rect, vp.monitor_size) {
            (Some(inner), Some(mon)) if mon.x > 0.0 && mon.y > 0.0 => {
                let c = inner.min + pad.center().to_vec2();
                let mut cfg = self.cfg.write().unwrap_or_else(|e| e.into_inner());
                cfg.click.x = (c.x / mon.x).clamp(0.0, 1.0);
                cfg.click.y = (c.y / mon.y).clamp(0.0, 1.0);
                cfg.click.enabled = true;
                self.notice = "Aimed at the test pad (assumes one monitor). Press Start.".into();
                self.dirty_since = Some(Instant::now());
            }
            _ => {
                self.notice = "This desktop hides window positions (common on Wayland). Click the spot in the preview instead.".into();
            }
        }
    }

    fn overlay(&self, ctx: &egui::Context, snap: &Snap) {
        let (pos, pressed, clicks) = (snap.pos, snap.pressed, snap.clicks.clone());
        let snap2 = Snap {
            running: snap.running,
            pos,
            pressed,
            clicks,
            click_count: 0,
            move_count: 0,
            uptime: None,
            log: vec![],
            error: None,
        };
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("poltergeist_overlay"),
            egui::ViewportBuilder::default()
                .with_title("Poltergeist overlay")
                .with_transparent(true)
                .with_decorations(false)
                .with_always_on_top()
                .with_mouse_passthrough(true)
                .with_fullscreen(true)
                .with_taskbar(false),
            move |ctx, _class| {
                egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
                    let rect = ui.max_rect();
                    paint_pointer(ui.painter(), rect, &snap2, 2.0);
                });
                ctx.request_repaint_after(Duration::from_millis(16));
            },
        );
    }
}

fn toggle_maximize(ctx: &egui::Context) {
    let maxed = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maxed));
}

enum WinBtn {
    Minimize,
    Maximize,
    Restore,
    Close,
}

/// A window control drawn with shapes, so it needs no special fonts.
fn window_button(ui: &mut egui::Ui, kind: WinBtn) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(36.0, 26.0), Sense::click());
    let hot = resp.hovered();
    let bg = match (&kind, hot) {
        (WinBtn::Close, true) => Color32::from_rgb(200, 50, 50),
        (_, true) => Color32::from_gray(65),
        _ => Color32::TRANSPARENT,
    };
    let p = ui.painter();
    p.rect_filled(rect, 4.0, bg);
    let c = rect.center();
    let st = Stroke::new(1.6, Color32::from_gray(230));
    match kind {
        WinBtn::Minimize => {
            p.line_segment([c + Vec2::new(-5.0, 3.0), c + Vec2::new(5.0, 3.0)], st);
        }
        WinBtn::Maximize => {
            p.rect_stroke(Rect::from_center_size(c, Vec2::splat(10.0)), 1.0, st, egui::StrokeKind::Inside);
        }
        WinBtn::Restore => {
            p.rect_stroke(Rect::from_center_size(c + Vec2::new(1.5, -1.5), Vec2::splat(8.0)), 1.0, st, egui::StrokeKind::Inside);
            p.rect_stroke(Rect::from_center_size(c + Vec2::new(-1.5, 1.5), Vec2::splat(8.0)), 1.0, st, egui::StrokeKind::Inside);
        }
        WinBtn::Close => {
            p.line_segment([c + Vec2::new(-5.0, -5.0), c + Vec2::new(5.0, 5.0)], st);
            p.line_segment([c + Vec2::new(-5.0, 5.0), c + Vec2::new(5.0, -5.0)], st);
        }
    }
    resp
}

/// With our own title bar the OS draws no resize border, so handle the window edges.
fn resize_edges(ctx: &egui::Context) {
    use egui::{viewport::ResizeDirection as D, CursorIcon as C};
    let vp = ctx.input(|i| i.viewport().clone());
    if vp.maximized.unwrap_or(false) || vp.fullscreen.unwrap_or(false) {
        return;
    }
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else { return };
    let r = ctx.content_rect();
    let m = 5.0;
    let (l, rt) = (pos.x < r.left() + m, pos.x > r.right() - m);
    let (t, b) = (pos.y < r.top() + m, pos.y > r.bottom() - m);
    let hit = match (l, rt, t, b) {
        (true, _, true, _) => Some((D::NorthWest, C::ResizeNorthWest)),
        (_, true, true, _) => Some((D::NorthEast, C::ResizeNorthEast)),
        (true, _, _, true) => Some((D::SouthWest, C::ResizeSouthWest)),
        (_, true, _, true) => Some((D::SouthEast, C::ResizeSouthEast)),
        (true, ..) => Some((D::West, C::ResizeHorizontal)),
        (_, true, ..) => Some((D::East, C::ResizeHorizontal)),
        (_, _, true, _) => Some((D::North, C::ResizeVertical)),
        (_, _, _, true) => Some((D::South, C::ResizeVertical)),
        _ => None,
    };
    if let Some((dir, cursor)) = hit {
        ctx.set_cursor_icon(cursor);
        if ctx.input(|i| i.pointer.primary_pressed()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
        }
    }
}

fn to_screen(rect: Rect, p: (f32, f32)) -> Pos2 {
    Pos2::new(rect.left() + rect.width() * p.0, rect.top() + rect.height() * p.1)
}

/// Draws the pointer ring and click ripples. `scale` enlarges them for the on-screen overlay.
fn paint_pointer(painter: &egui::Painter, rect: Rect, snap: &Snap, scale: f32) {
    for (x, y, age) in &snap.clicks {
        if *age < 0.9 {
            let a = ((1.0 - age / 0.9) * 220.0) as u8;
            painter.circle_stroke(to_screen(rect, (*x, *y)), (8.0 + age * 60.0) * scale, Stroke::new(3.0, Color32::from_rgba_unmultiplied(80, 200, 255, a)));
        }
    }
    if !snap.running {
        return; // no pointer to show while stopped
    }
    let p = to_screen(rect, snap.pos);
    if snap.pressed {
        painter.circle_filled(p, 22.0 * scale, Color32::from_rgba_unmultiplied(255, 150, 40, 120));
    }
    painter.circle_stroke(p, 14.0 * scale, Stroke::new(3.0, Color32::from_rgb(255, 80, 80)));
    painter.circle_filled(p, 4.0 * scale, Color32::from_rgb(255, 80, 80));
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Reap a worker that ended on its own (error, or it was stopped elsewhere).
        if self.worker.as_ref().map(|h| h.is_finished()).unwrap_or(false) {
            if let Some(h) = self.worker.take() {
                let _ = h.join();
            }
            self.stop = None;
        }
        // Debounced auto-save of settings.
        if let Some(t) = self.dirty_since {
            if t.elapsed() > Duration::from_millis(600) {
                self.save_now();
            } else {
                ctx.request_repaint_after(Duration::from_millis(650));
            }
        }

        // F11 toggles full screen; Esc always leaves it. A guaranteed way back to a window.
        let fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
        } else if fullscreen && ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        }
        if CUSTOM_CHROME {
            resize_edges(ctx);
        }

        let snap = self.snapshot();
        if let Some(e) = &snap.error {
            if self.notice.is_empty() {
                self.notice = format!("Engine stopped: {e}");
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            if CUSTOM_CHROME {
                // Whole bar is a drag handle; widgets added after this take priority over it.
                let bar = ui.max_rect();
                let drag = ui.interact(bar, ui.id().with("titlebar"), Sense::click_and_drag());
                if drag.drag_started() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if drag.double_clicked() {
                    toggle_maximize(ctx);
                }
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading("👻 Poltergeist");
                ui.separator();
                if self.running() {
                    if ui.add(egui::Button::new(RichText::new("⏹  Stop").size(16.0)).fill(Color32::from_rgb(170, 50, 50))).clicked() {
                        self.stop_engine();
                    }
                } else if ui.add(egui::Button::new(RichText::new("▶  Start").size(16.0)).fill(Color32::from_rgb(40, 140, 80))).clicked() {
                    self.start_engine();
                }
                ui.label(if self.running() { if self.simulate { "● running (simulation)" } else { "● running" } } else { "○ stopped" });
                ui.label(RichText::new("offline · local only").weak());
                if CUSTOM_CHROME {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if window_button(ui, WinBtn::Close).on_hover_text("Close").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        let maxed = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                        let (kind, tip) = if maxed { (WinBtn::Restore, "Restore to window") } else { (WinBtn::Maximize, "Maximize") };
                        if window_button(ui, kind).on_hover_text(tip).clicked() {
                            toggle_maximize(ctx);
                        }
                        if window_button(ui, WinBtn::Minimize).on_hover_text("Minimize").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    });
                }
            });
            ui.add_space(4.0);
        });

        // Notices live in a fixed bottom bar so they never shift the layout.
        egui::TopBottomPanel::bottom("status").exact_height(24.0).show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                if self.notice.is_empty() {
                    ui.label(RichText::new("Ready").weak());
                } else {
                    ui.label(RichText::new(&self.notice).color(Color32::from_rgb(255, 200, 60)));
                }
            });
        });

        egui::SidePanel::left("settings").min_width(310.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| self.settings_ui(ui));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| self.main_ui(ctx, ui, &snap));
        });

        let overlay_on = self.cfg.read().unwrap_or_else(|e| e.into_inner()).show_overlay;
        if overlay_on {
            self.overlay(ctx, &snap);
        }

        // Keep animating while running or while ripples are fading.
        if snap.running || snap.clicks.iter().any(|c| c.2 < 1.0) {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop_engine();
        self.save_now();
    }
}
