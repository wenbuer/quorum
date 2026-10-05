//! AAIL host -- M0.
//!
//! One window, two states:
//!   capsule -- a 320px strip docked to the right edge (44px open, 6px sliver shut)
//!   panel   -- a centred 680x460 work surface summoned with Alt+Space
//!
//! Scheduler, role registry and permission gate land in M1/M2. Everything here is
//! geometry plus the two things only the OS can give us: a global hotkey and a tray.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewWindow,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

// The panel is a fraction of the screen, clamped so it stays usable on both a small
// laptop panel and a large desktop display. Fixed pixels looked cramped on 1440p.
const PANEL_W_RATIO: f64 = 0.40;
const PANEL_H_RATIO: f64 = 0.55;
const PANEL_W_MIN: f64 = 760.0;
const PANEL_W_MAX: f64 = 1120.0;
const PANEL_H_MIN: f64 = 520.0;
const PANEL_H_MAX: f64 = 800.0;
const CAPSULE_W: f64 = 320.0;
const CAPSULE_OPEN_H: f64 = 44.0;
// Collapsed state is a dock you can actually aim at; the old 6px sliver was unhittable.
const DOCK_W: f64 = 148.0;
const DOCK_H: f64 = 44.0;
// Anything wider than this is the panel, not the capsule.
const PANEL_MIN_W: f64 = 500.0;

/// Ordered fallbacks. The first one the OS grants wins; everything else still works
/// from the tray when none of them can be had.
const HOTKEY_CANDIDATES: [(Modifiers, Code, &str); 4] = [
    // Default. Alt+Space is deliberately NOT first: it is the single most contested
    // shortcut on Windows (uTools, IMEs and launchers all grab it).
    (Modifiers::ALT, Code::Backquote, "Alt+Backquote"),
    (Modifiers::ALT, Code::Space, "Alt+Space"),
    (
        Modifiers::CONTROL.union(Modifiers::ALT),
        Code::Space,
        "Ctrl+Alt+Space",
    ),
    (
        Modifiers::CONTROL.union(Modifiers::SHIFT),
        Code::Space,
        "Ctrl+Shift+Space",
    ),
];

const POPUP_LABEL: &str = "popup";
const CAPSULE_CLOSE: &str = "capsule-close";

/// Raised for the whole lifetime of the level-2 window. The main window must not read
/// "the popup just took focus" as "the user walked away", or opening the editor hides its
/// own parent. Checked before the window is registered too, closing that race.
static POPUP_OPEN: AtomicBool = AtomicBool::new(false);
const POPUP_W: f64 = 440.0;
const POPUP_H: f64 = 560.0;

// Monitor geometry in logical pixels: (x, y, width, height).
fn monitor_box(window: &WebviewWindow) -> (f64, f64, f64, f64) {
    match window.current_monitor() {
        Ok(Some(m)) => {
            let s = m.scale_factor();
            let size = m.size();
            let pos = m.position();
            (
                pos.x as f64 / s,
                pos.y as f64 / s,
                size.width as f64 / s,
                size.height as f64 / s,
            )
        }
        _ => (0.0, 0.0, 1920.0, 1080.0),
    }
}

fn is_panel(window: &WebviewWindow) -> bool {
    match window.outer_size() {
        Ok(size) => {
            let s = window.scale_factor().unwrap_or(1.0);
            (size.width as f64 / s) > PANEL_MIN_W
        }
        Err(_) => false,
    }
}

/// Where the user last dragged each window, in logical pixels. Once they move a window it
/// stays put; a summon only re-centres a window they never moved.
static CAPSULE_POS: Mutex<Option<(f64, f64)>> = Mutex::new(None);
static PANEL_POS: Mutex<Option<(f64, f64)>> = Mutex::new(None);
/// Some(side) while the capsule is parked against a screen edge. Only then does it collapse
/// to the slim form -- floating free, or mid-drag, it stays open.
static EDGE_SIDE: Mutex<Option<&'static str>> = Mutex::new(None);
/// Whether the capsule may turn into the floating icon at all.
static FLOATING_ICON: AtomicBool = AtomicBool::new(true);
static CURRENT_MODE: Mutex<&'static str> = Mutex::new("capsule");
/// The hotkey the OS actually granted, so the UI can show it instead of guessing. The
/// startup event is easy to miss: the webview may not have a listener yet.
static ACTIVE_HOTKEY: Mutex<String> = Mutex::new(String::new());
/// The last position we set ourselves. A Moved event landing exactly there is our own doing,
/// not a drag, so it must not overwrite what the user chose.
static LAST_PLACED: Mutex<Option<(f64, f64)>> = Mutex::new(None);
/// Last time the window moved on its own (a hand drag); drives settle-after-drag.
static LAST_MOVE: AtomicU64 = AtomicU64::new(0);
static WATCHING: AtomicBool = AtomicBool::new(false);

fn mark_placed(x: f64, y: f64) {
    if let Ok(mut last) = LAST_PLACED.lock() {
        *last = Some((x, y));
    }
}

fn was_placed_at(x: f64, y: f64) -> bool {
    LAST_PLACED
        .lock()
        .ok()
        .and_then(|last| *last)
        .map(|(px, py)| (px - x).abs() < 2.0 && (py - y).abs() < 2.0)
        .unwrap_or(false)
}

fn remember(spot: &Mutex<Option<(f64, f64)>>, x: f64, y: f64) {
    if let Ok(mut held) = spot.lock() {
        *held = Some((x, y));
    }
}

fn remembered(spot: &Mutex<Option<(f64, f64)>>) -> Option<(f64, f64)> {
    spot.lock().ok().and_then(|held| *held)
}

/// Keep a dragged window on screen: a window parked past the edge is unreachable.
fn clamp_to_screen(window: &WebviewWindow, x: f64, y: f64, w: f64, h: f64) -> (f64, f64) {
    let (mx, my, mw, mh) = monitor_box(window);
    let x = x.clamp(mx - w + 48.0, mx + mw - 48.0);
    let y = y.clamp(my - h + 32.0, my + mh - 32.0);
    (x, y)
}

/// How close to a screen edge counts as parking. Zero would demand pixel-perfect contact;
/// demanding that made the gesture feel broken, so give the pointer a little room.
const EDGE_NEAR: f64 = 12.0;

/// Panel geometry, derived from the monitor it will open on.
fn panel_size(window: &WebviewWindow) -> (f64, f64) {
    let (_, _, mw, mh) = monitor_box(window);
    (
        (mw * PANEL_W_RATIO).clamp(PANEL_W_MIN, PANEL_W_MAX),
        (mh * PANEL_H_RATIO).clamp(PANEL_H_MIN, PANEL_H_MAX),
    )
}

fn current_spot(window: &WebviewWindow) -> (f64, f64, f64, f64) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let pos = window
        .outer_position()
        .unwrap_or(tauri::PhysicalPosition::new(0, 0));
    let size = window
        .outer_size()
        .unwrap_or(tauri::PhysicalSize::new(1, 1));
    (
        pos.x as f64 / scale,
        pos.y as f64 / scale,
        size.width as f64 / scale,
        size.height as f64 / scale,
    )
}

/// Which screen edge the window is currently resting against, if any.
fn edge_near(window: &WebviewWindow) -> Option<&'static str> {
    let (x, y, w, h) = current_spot(window);
    let (mx, my, mw, mh) = monitor_box(window);
    let gaps = [
        ("left", x - mx),
        ("right", (mx + mw) - (x + w)),
        ("top", y - my),
        ("bottom", (my + mh) - (y + h)),
    ];
    gaps.into_iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .filter(|(_, gap)| *gap <= EDGE_NEAR)
        .map(|(side, _)| side)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Hug the nearest edge the moment the drag gets close, instead of waiting for the drag
/// to end. Windows clamps at the screen edge, so this mostly covers the magnet gap.
fn magnet(window: &WebviewWindow) {
    let Some(side) = edge_near(window) else { return };
    let (x, y, w, h) = current_spot(window);
    let (tx, ty) = snap_spot(window, side, w, h);
    if (tx - x).abs() < 0.5 && (ty - y).abs() < 0.5 {
        return;
    }
    mark_placed(tx, ty);
    let _ = window.set_position(LogicalPosition::new(tx, ty));
    if let Ok(mut held) = EDGE_SIDE.lock() {
        *held = Some(side);
    }
}

/// The OS drag loop is modal and never tells us the button came up, so a pause in the
/// move events is the drop. Without this the capsule only parks when the pointer later
/// leaves it -- which reads as "it snaps only after I walk away".
fn arm_settle(window: &WebviewWindow) {
    LAST_MOVE.store(now_ms(), Ordering::Relaxed);
    if WATCHING.swap(true, Ordering::Relaxed) {
        return;
    }
    let app = window.app_handle().clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(120));
        if now_ms().saturating_sub(LAST_MOVE.load(Ordering::Relaxed)) < 350 {
            continue;
        }
        WATCHING.store(false, Ordering::Relaxed);
        let _ = settle(&app);
        return;
    });
}

/// Flush against the given edge, keeping the position along that edge.
fn snap_spot(window: &WebviewWindow, side: &str, w: f64, h: f64) -> (f64, f64) {
    let (mx, my, mw, mh) = monitor_box(window);
    let (x, y, _, _) = current_spot(window);
    match side {
        "left" => (mx, y.clamp(my, my + mh - h)),
        "right" => (mx + mw - w, y.clamp(my, my + mh - h)),
        "top" => (x.clamp(mx, mx + mw - w), my),
        _ => (x.clamp(mx, mx + mw - w), my + mh - h),
    }
}

fn place_docked(window: &WebviewWindow, width: f64, height: f64) -> tauri::Result<()> {
    let (mx, my, mw, mh) = monitor_box(window);
    let held = EDGE_SIDE.lock().ok().and_then(|side| *side);
    window.set_size(LogicalSize::new(width, height))?;
    let (x, y) = match held {
        // Parked at an edge: both the slim form and the hovered-open form hug that edge.
        Some(side) => snap_spot(window, side, width, height),
        None => match remembered(&CAPSULE_POS) {
            Some((x, y)) => clamp_to_screen(window, x, y, width, height),
            // Inset from the edge on purpose: flush against it would read as "parked" and
            // collapse the moment the pointer leaves.
            None => (mx + mw - width - 48.0, my + mh * 0.30),
        },
    };
    mark_placed(x, y);
    window.set_position(LogicalPosition::new(x, y))?;
    Ok(())
}

fn place_panel(window: &WebviewWindow) -> tauri::Result<()> {
    let (mx, my, mw, mh) = monitor_box(window);
    let (pw, ph) = panel_size(window);
    window.set_size(LogicalSize::new(pw, ph))?;
    let (x, y) = match remembered(&PANEL_POS) {
        Some((x, y)) => clamp_to_screen(window, x, y, pw, ph),
        None => (mx + (mw - pw) / 2.0, my + (mh - ph) / 2.4),
    };
    mark_placed(x, y);
    window.set_position(LogicalPosition::new(x, y))?;
    Ok(())
}

/// Decide where the capsule belongs after a drag or a pointer-out: hugged to an edge it
/// collapses to the slim form, anywhere else it stays open where it was dropped.
/// Called when the panel closes: put the capsule back exactly where it was, instead of
/// treating the panel's own position as the user's latest choice.
fn restore(app: &AppHandle) -> tauri::Result<()> {
    let parked = EDGE_SIDE.lock().ok().and_then(|side| *side);
    match parked {
        Some(_) => to_dock(app, "edge"),
        None => to_dock(app, "capsule"),
    }
}

fn settle(app: &AppHandle) -> tauri::Result<()> {
    let window = main_window(app)?;
    if !FLOATING_ICON.load(Ordering::Relaxed) {
        if let Ok(mut side) = EDGE_SIDE.lock() {
            *side = None;
        }
        let (x, y, _, _) = current_spot(&window);
        remember(&CAPSULE_POS, x, y);
        return to_dock(app, "capsule");
    }
    let near = edge_near(&window);
    let (x, y, _, _) = current_spot(&window);
    if let Ok(mut side) = EDGE_SIDE.lock() {
        *side = near;
    }
    if near.is_none() {
        remember(&CAPSULE_POS, x, y);
    }
    match near {
        Some(_) => to_dock(app, "edge"),
        None => to_dock(app, "capsule"),
    }
}

#[derive(serde::Serialize)]
struct DirEntry {
    name: String,
    path: String,
    is_dir: bool,
    size: u64,
}

/// One directory level at a time. Lazy on purpose: indexing a whole tree up front is slow,
/// and the browser only ever shows what the user opened.
#[tauri::command]
fn list_dir(path: String) -> Result<Vec<DirEntry>, String> {
    let entries = std::fs::read_dir(&path).map_err(|e| e.to_string())?;
    let mut out: Vec<DirEntry> = entries
        .flatten()
        .map(|entry| {
            let meta = entry.metadata().ok();
            DirEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false),
                size: meta.as_ref().map(|m| m.len()).unwrap_or(0),
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    out.truncate(500);
    Ok(out)
}

#[tauri::command]
fn set_floating_icon(enabled: bool) {
    FLOATING_ICON.store(enabled, Ordering::Relaxed);
}

#[tauri::command]
fn active_hotkey() -> String {
    ACTIVE_HOTKEY.lock().map(|held| held.clone()).unwrap_or_default()
}

fn set_current_mode(mode: &'static str) {
    if let Ok(mut current) = CURRENT_MODE.lock() {
        *current = mode;
    }
}

fn announce(window: &WebviewWindow, mode: &'static str) {
    set_current_mode(mode);
    // Emit the bare string. Wrapping it in a struct once cost us a whole debugging round:
    // the frontend compared the payload to "panel" and got an object instead.
    let _ = window.emit("aail://mode", mode);
}

fn main_window(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    app.get_webview_window("main")
        .ok_or(tauri::Error::WindowNotFound)
}

// Summon the panel and give it keyboard focus.
fn open_panel(app: &AppHandle) -> tauri::Result<()> {
    let window = main_window(app)?;
    set_current_mode("panel");
    place_panel(&window)?;
    window.show()?;
    window.set_focus()?;
    announce(&window, "panel");
    Ok(())
}

// Dock to the edge without stealing focus: either the open capsule or the compact dock.
fn to_dock(app: &AppHandle, mode: &'static str) -> tauri::Result<()> {
    let window = main_window(app)?;
    set_current_mode(mode);
    // Opening up again only hugs the edge if we are still against it. Without this check a
    // parked capsule could never be moved away: the hover-expand that starts a drag snapped
    // it straight back to the edge.
    if mode != "edge" && edge_near(&window).is_none() {
        if let Ok(mut side) = EDGE_SIDE.lock() {
            *side = None;
        }
    }
    let (width, height) = match mode {
        "edge" | "dock" => (DOCK_W, DOCK_H),
        _ => (CAPSULE_W, CAPSULE_OPEN_H),
    };
    place_docked(&window, width, height)?;
    if !window.is_visible()? {
        window.show()?;
    }
    announce(&window, mode);
    Ok(())
}

fn toggle_panel(app: &AppHandle) -> tauri::Result<()> {
    let window = main_window(app)?;
    if is_panel(&window) && window.is_visible()? {
        to_dock(app, "capsule")
    } else {
        open_panel(app)
    }
}

// Diagnostics go to a file: stdout is buffered through the tauri CLI and cannot be trusted
// to arrive in order (or at all) while we are debugging window creation.
// Forward slashes on purpose -- a backslash path here was eaten by layered escaping once.
fn trace(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("D:/aail-dev/aail-trace.log")
    {
        let _ = writeln!(f, "{msg}");
    }
}

// Level 2 is a real detached window, and the whole app allows exactly one of them.
// Asking for a second one raises the existing window instead of building another.
// Must be async. A synchronous command runs on the main thread, and build() needs that
// same thread to pump the event loop -- the window got created but never finished
// initialising, leaving an empty transparent shell with no WebView2 renderer.
#[tauri::command]
async fn open_popup(app: AppHandle) -> Result<(), String> {
    POPUP_OPEN.store(true, Ordering::Relaxed);
    if let Some(existing) = app.get_webview_window(POPUP_LABEL) {
        trace("  existing popup -> raise only");
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }

    // A window created at runtime does NOT inherit the main window's URL resolution the way
    // the config-defined window does: WebviewUrl::App alone left the popup's WebView2 with no
    // renderer at all. In dev we point at the dev server explicitly; release uses the bundled
    // assets.
    let url = if cfg!(debug_assertions) {
        match app.config().build.dev_url.clone() {
            Some(dev) => tauri::WebviewUrl::External(dev),
            None => tauri::WebviewUrl::App("index.html".into()),
        }
    } else {
        tauri::WebviewUrl::App("index.html".into())
    };
    trace(&format!("  dev_url={:?}", app.config().build.dev_url));
    trace(&format!("  using url={url:?}"));

    tauri::WebviewWindowBuilder::new(&app, POPUP_LABEL, url)
        .title("AAIL")
        .inner_size(POPUP_W, POPUP_H)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .center()
        .build()
        .map(|w| {
            w.on_window_event(|event| {
                if let tauri::WindowEvent::Destroyed = event {
                    POPUP_OPEN.store(false, Ordering::Relaxed);
                }
            });
        })
        .map_err(|e| {
            POPUP_OPEN.store(false, Ordering::Relaxed);
            e.to_string()
        })
}

// The capsule is 44px tall, so its menu has to be a native popup -- an in-page one has
// nowhere to live.
#[tauri::command]
async fn capsule_menu(window: WebviewWindow, app: AppHandle) -> Result<(), String> {
    let close = MenuItem::with_id(&app, CAPSULE_CLOSE, "关闭", true, None::<&str>).map_err(|e| e.to_string())?;
    let menu = Menu::with_items(&app, &[&close]).map_err(|e| e.to_string())?;
    window.popup_menu(&menu).map_err(|e| e.to_string())
}

// The frontend drives capsule states on hover; the kernel only owns geometry.
#[tauri::command]
fn set_mode(app: AppHandle, mode: String) -> Result<(), String> {
    let result = match mode.as_str() {
        "panel" => open_panel(&app),
        "capsule" => to_dock(&app, "capsule"),
        "restore" => restore(&app),
        "edge" => settle(&app),
        "hide" => main_window(&app).and_then(|w| w.hide()),
        other => return Err(format!("unknown mode: {}", other)),
    };
    result.map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            set_mode,
            set_floating_icon,
            list_dir,
            active_hotkey,
            open_popup,
            capsule_menu
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            let panel_item = MenuItem::with_id(app, "panel", "打开面板", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&panel_item, &quit_item])?;

            let mut tray = TrayIconBuilder::new().tooltip("AAIL");
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "panel" => {
                        let _ = open_panel(app);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    let _ = toggle_panel(tray.app_handle());
                }
            })
            .build(app)?;

            // Losing focus closes the panel; the frontend decides what that means.
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    // Remember wherever the user drags this window, so the next summon
                    // returns to their spot instead of re-centring under their cursor.
                    if let tauri::WindowEvent::Moved(pos) = event {
                        let scale = w.scale_factor().unwrap_or(1.0);
                        let spot = (pos.x as f64 / scale, pos.y as f64 / scale);
                        if was_placed_at(spot.0, spot.1) {
                            return;
                        }
                        match CURRENT_MODE.lock().map(|m| *m).unwrap_or("capsule") {
                            "panel" => remember(&PANEL_POS, spot.0, spot.1),
                            // A parked window sits where the edge puts it; that is not a choice.
                            "edge" => {}
                            _ => {
                                // Only a hand drag reaches here: our own placements are
                                // filtered out by was_placed_at above.
                                magnet(&w);
                                arm_settle(&w);
                                remember(&CAPSULE_POS, spot.0, spot.1);
                            }
                        }
                    }
                    if let tauri::WindowEvent::Focused(false) = event {
                        // A detached level-2 window taking focus must not collapse the panel
                        // underneath it -- otherwise opening the editor hides the editor's parent.
                        if POPUP_OPEN.load(Ordering::Relaxed)
                            || w.app_handle().get_webview_window(POPUP_LABEL).is_some()
                        {
                            return;
                        }
                        // Windows bounces focus around (foreground lock, IME switches, the
                        // alt-key release of our own hotkey). Believe the blur only if it
                        // still holds a moment later, or the panel flashes open and vanishes.
                        let win = w.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(280));
                            if let Ok(false) = win.is_focused() {
                                let _ = win.emit("aail://focus-lost", ());
                            }
                        });
                    }
                });
                window.on_menu_event(|window, event| {
                    if event.id.as_ref() == CAPSULE_CLOSE {
                        let _ = window.hide();
                    }
                });
            }

            // A taken hotkey must never kill the assistant: RegisterHotKey is exclusive and
            // the first choice is often already held by an IME or another launcher. Try the
            // candidates in order and run with whichever one the OS gives us.
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |_app, _shortcut, event| {
                        if event.state() == ShortcutState::Pressed {
                            let _ = toggle_panel(&handle);
                        }
                    })
                    .build(),
            )?;

            let active = HOTKEY_CANDIDATES
                .iter()
                .find_map(|(mods, code, label)| {
                    let shortcut = Shortcut::new(Some(*mods), *code);
                    app.global_shortcut()
                        .register(shortcut)
                        .ok()
                        .map(|_| *label)
                });

            if let Some(label) = active {
                if let Ok(mut held) = ACTIVE_HOTKEY.lock() {
                    *held = label.to_string();
                }
            }
            match active {
                Some(label) => println!("[AAIL] global hotkey: {label}"),
                None => eprintln!("[AAIL] no global hotkey could be registered; use the tray icon"),
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("aail://hotkey", active.unwrap_or(""));
            }

            let _ = to_dock(app.handle(), "capsule");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running AAIL");
}
