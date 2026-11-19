use std::fs;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::{Color, Webview, WebviewBuilder},
    LogicalPosition, LogicalSize, Manager, Url, WebviewUrl, WebviewWindow, WindowEvent,
};

const TITLEBAR_HEIGHT: f64 = 48.0;
const TRAY_ID: &str = "lumine-tray";
const WEBVIEW_BACKGROUND: Color = Color(0, 0, 0, 255);

const FADE_OUT_SCRIPT: &str =
    "{ const d = document.documentElement; d.style.transition = 'none'; d.style.opacity = '0'; }";
const FADE_IN_SCRIPT: &str = "{ const d = document.documentElement; requestAnimationFrame(() => { d.style.transition = 'opacity 180ms ease'; d.style.opacity = '1'; }); }";

const SITES: [(&str, &str); 2] = [
    ("portal", "https://portal.luminecloud.com/"),
    ("panel", "https://cp.luminecloud.com/"),
];

fn is_known_tab(tab: &str) -> bool {
    SITES.iter().any(|(label, _)| *label == tab)
}

fn state_path(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok().map(|dir| dir.join("state.json"))
}

fn load_last_tab(app: &tauri::AppHandle) -> &'static str {
    const KEY: &str = "\"active_tab\":\"";

    let Some(path) = state_path(app) else {
        return "portal";
    };
    let Ok(text) = fs::read_to_string(path) else {
        return "portal";
    };
    let Some(start) = text.find(KEY) else {
        return "portal";
    };
    let rest = &text[start + KEY.len()..];
    let Some(end) = rest.find('"') else {
        return "portal";
    };
    let tab = &rest[..end];
    SITES
        .iter()
        .find(|(label, _)| *label == tab)
        .map(|(label, _)| *label)
        .unwrap_or("portal")
}

fn save_last_tab(app: &tauri::AppHandle, tab: &str) {
    let Some(path) = state_path(app) else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, format!("{{\"active_tab\":\"{tab}\"}}"));
}

fn site_webview(manager: &Webview, tab: &str) -> Result<Webview, String> {
    if !is_known_tab(tab) {
        return Err(format!("unknown tab: {tab}"));
    }
    manager
        .get_webview(tab)
        .ok_or_else(|| format!("webview '{tab}' is not available"))
}

fn content_size(window: &WebviewWindow) -> tauri::Result<LogicalSize<f64>> {
    let scale = window.scale_factor()?;
    let inner = window.inner_size()?.to_logical::<f64>(scale);
    Ok(LogicalSize::new(
        inner.width,
        (inner.height - TITLEBAR_HEIGHT).max(1.0),
    ))
}

fn sync_content_bounds(window: &WebviewWindow) -> tauri::Result<()> {
    let size = content_size(window)?;
    for (label, _) in SITES {
        if let Some(view) = window.get_webview(label) {
            view.set_position(LogicalPosition::new(0.0, TITLEBAR_HEIGHT))?;
            view.set_size(size)?;
        }
    }
    Ok(())
}

fn create_site_webviews(window: &WebviewWindow, initial_tab: &str) -> tauri::Result<()> {
    let size = content_size(window)?;
    let host: &Webview = window.as_ref();
    let host_window = host.window();

    for (label, raw_url) in SITES {
        let target: Url = raw_url.parse().expect("site URLs are valid");
        let view = host_window.add_child(
            WebviewBuilder::new(label, WebviewUrl::External(target))
                .zoom_hotkeys_enabled(true)
                .disable_drag_drop_handler()
                .background_color(WEBVIEW_BACKGROUND),
            LogicalPosition::new(0.0, TITLEBAR_HEIGHT),
            size,
        )?;

        if label != initial_tab {
            view.hide()?;
        }
    }

    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn fade_in(view: &Webview) -> Result<(), String> {
    view.eval(FADE_OUT_SCRIPT)
        .map_err(|error| error.to_string())?;
    view.show().map_err(|error| error.to_string())?;
    view.set_focus().map_err(|error| error.to_string())?;
    view.eval(FADE_IN_SCRIPT)
        .map_err(|error| error.to_string())
}

fn persist_session_cookies(app: &tauri::AppHandle) {
    let expiry = time::OffsetDateTime::now_utc() + time::Duration::days(90);

    for (label, _) in SITES {
        let Some(view) = app.get_webview(label) else { continue };
        let Ok(cookies) = view.cookies() else { continue };

        for mut cookie in cookies {
            let Some(domain) = cookie.domain() else { continue };
            if !domain.ends_with("luminecloud.com") {
                continue;
            }
            if cookie.expires().is_some() {
                continue;
            }
            cookie.set_expires(Some(expiry));
            let _ = view.set_cookie(cookie);
        }
    }
}

fn create_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "Hiện cửa sổ Lumine", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Thoát", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    let icon = app
        .default_window_icon()
        .expect("app icon is bundled")
        .clone();

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .tooltip("Lumine Desktop")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
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
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

#[tauri::command]
fn initial_tab(app: tauri::AppHandle) -> String {
    load_last_tab(&app).to_string()
}

#[tauri::command]
fn activate_tab(app: tauri::AppHandle, webview: Webview, tab: String) -> Result<(), String> {
    if !is_known_tab(&tab) {
        return Err(format!("unknown tab: {tab}"));
    }

    for (label, _) in SITES {
        let view = site_webview(&webview, label)?;
        if label == tab.as_str() {
            fade_in(&view)?;
        } else {
            view.hide().map_err(|error| error.to_string())?;
        }
    }

    save_last_tab(&app, &tab);
    Ok(())
}

#[tauri::command]
fn reload_active_site(webview: Webview, tab: String) -> Result<(), String> {
    site_webview(&webview, &tab)?
        .reload()
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_active_site_in_browser(webview: Webview, tab: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    let url = site_webview(&webview, &tab)?
        .url()
        .map_err(|error| error.to_string())?;

    webview
        .opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|error| error.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let initial_tab = load_last_tab(&handle);

            let window = app
                .get_webview_window("main")
                .expect("main window is defined in tauri.conf.json");

            create_site_webviews(&window, initial_tab)?;
            create_tray(&handle)?;

            let cookie_handle = handle.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
                persist_session_cookies(&cookie_handle);
            });

            let event_window = window.clone();
            window.on_window_event(move |event| {
                if matches!(
                    event,
                    WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }
                ) {
                    let window = event_window.clone();
                    tauri::async_runtime::spawn(async move {
                        let _ = sync_content_bounds(&window);
                    });
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            initial_tab,
            activate_tab,
            reload_active_site,
            open_active_site_in_browser
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Lumine Desktop");
}
