use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{
  menu::{CheckMenuItem, MenuBuilder, MenuItem, MenuItemBuilder, SubmenuBuilder},
  tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
  window::Color,
  Emitter, Manager, State, WindowEvent,
};
use tokio::time::sleep;

mod timer;

use timer::{TimerEngine, TimerPhase, TimerPrefs, TimerState};

type AppMenuItem = MenuItem<tauri::Wry>;
type AppCheckMenuItem = CheckMenuItem<tauri::Wry>;

struct AppState(Arc<Mutex<TimerEngine>>);
struct TrayState(tauri::tray::TrayIcon);
struct MenuState {
  status_item: AppMenuItem,
  start_pause_item: AppMenuItem,
  auto_start_item: AppCheckMenuItem,
  focus_value_item: AppMenuItem,
  short_value_item: AppMenuItem,
  long_value_item: AppMenuItem,
  cycles_value_item: AppMenuItem,
}

#[tauri::command]
fn get_timer_state(state: State<AppState>) -> TimerState {
  with_engine(&state, |engine| engine.snapshot())
}

#[tauri::command]
fn start_timer(state: State<AppState>) -> TimerState {
  with_engine(&state, |engine| {
    engine.start();
    engine.snapshot()
  })
}

#[tauri::command]
fn pause_timer(state: State<AppState>) -> TimerState {
  with_engine(&state, |engine| {
    engine.pause();
    engine.snapshot()
  })
}

#[tauri::command]
fn reset_timer(state: State<AppState>) -> TimerState {
  with_engine(&state, |engine| {
    engine.reset();
    engine.snapshot()
  })
}

#[tauri::command]
fn skip_timer(state: State<AppState>) -> TimerState {
  with_engine(&state, |engine| {
    engine.skip();
    engine.snapshot()
  })
}

#[tauri::command]
fn set_prefs(app: tauri::AppHandle, state: State<AppState>, prefs: TimerPrefs) -> TimerState {
  let prefs = normalize_prefs(prefs);
  let snapshot = with_engine(&state, |engine| {
    engine.set_prefs(prefs.clone());
    engine.snapshot()
  });
  save_prefs(&app, &prefs);
  snapshot
}

fn with_engine<F, R>(state: &State<AppState>, f: F) -> R
where
  F: FnOnce(&mut TimerEngine) -> R,
{
  let mut engine = state.0.lock().unwrap_or_else(|e| e.into_inner());
  f(&mut engine)
}

fn spawn_timer(app: tauri::AppHandle, engine: Arc<Mutex<TimerEngine>>) {
  tauri::async_runtime::spawn(async move {
    loop {
      sleep(Duration::from_millis(500)).await;
      let snapshot = {
        let mut guard = engine.lock().unwrap_or_else(|e| e.into_inner());
        guard.tick()
      };
      let _ = app.emit("timer:tick", snapshot.clone());
      update_tray_title(&app, &snapshot);
    }
  });
}

fn format_remaining(ms: u64) -> String {
  let total_seconds = (ms + 999) / 1000;
  let minutes = total_seconds / 60;
  let seconds = total_seconds % 60;
  format!("{:02}:{:02}", minutes, seconds)
}

fn format_minutes_value(label: &str, minutes: u64) -> String {
  format!("{}: {} min", label, minutes)
}

fn format_cycles_value(cycles: u64) -> String {
  format!("Current: {} cycles", cycles)
}

fn phase_label(phase: TimerPhase) -> &'static str {
  match phase {
    TimerPhase::Focus => "Focus",
    TimerPhase::ShortBreak => "Short Break",
    TimerPhase::LongBreak => "Long Break",
  }
}

fn update_menu(app: &tauri::AppHandle, snapshot: &TimerState) {
  let menu_state = app.state::<MenuState>();
  let status = format!(
    "{} {}",
    phase_label(snapshot.phase),
    format_remaining(snapshot.remaining_ms)
  );
  let _ = menu_state.status_item.set_text(status);
  let _ = menu_state.start_pause_item.set_text(if snapshot.is_running {
    "Pause"
  } else {
    "Start"
  });
  let prefs = &snapshot.prefs;
  let _ = menu_state.auto_start_item.set_checked(prefs.auto_start);
  let _ = menu_state
    .focus_value_item
    .set_text(format_minutes_value("Current", prefs.focus_minutes));
  let _ = menu_state
    .short_value_item
    .set_text(format_minutes_value("Current", prefs.short_break_minutes));
  let _ = menu_state
    .long_value_item
    .set_text(format_minutes_value("Current", prefs.long_break_minutes));
  let _ = menu_state
    .cycles_value_item
    .set_text(format_cycles_value(prefs.cycles));
  let title = format_remaining(snapshot.remaining_ms);
  let _ = app.state::<TrayState>().0.set_title(Some(title));
}

fn update_tray_title(app: &tauri::AppHandle, snapshot: &TimerState) {
  let title = format_remaining(snapshot.remaining_ms);
  let _ = app.state::<TrayState>().0.set_title(Some(title));
}

fn open_preferences_window(app: &tauri::AppHandle) {
  let width = 420.0;
  let height = 560.0;

  if let Some(window) = app.get_webview_window("preferences") {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    return;
  }

  let mut builder = tauri::WebviewWindowBuilder::new(
    app,
    "preferences",
    tauri::WebviewUrl::App("preferences".into()),
  )
  .title("偏好设置")
  .inner_size(width, height)
  .resizable(false)
  .skip_taskbar(true)
  .background_color(Color(245, 239, 230, 255))
  .center()
  .prevent_overflow()
  ;

  #[cfg(target_os = "macos")]
  {
    builder = builder.title_bar_style(tauri::TitleBarStyle::Visible);
  }

  let window = builder.build();

  let Ok(window) = window else {
    return;
  };

  let window_clone = window.clone();
  window.on_window_event(move |event| {
    if let WindowEvent::CloseRequested { api, .. } = event {
      api.prevent_close();
      let _ = window_clone.hide();
    }
  });

  let _ = window.show();
  let _ = window.set_focus();
}

fn clamp_u64(value: u64, min: u64, max: u64) -> u64 {
  value.max(min).min(max)
}

fn normalize_prefs(mut prefs: TimerPrefs) -> TimerPrefs {
  prefs.focus_minutes = clamp_u64(prefs.focus_minutes, 1, 180);
  prefs.short_break_minutes = clamp_u64(prefs.short_break_minutes, 1, 30);
  prefs.long_break_minutes = clamp_u64(prefs.long_break_minutes, 1, 90);
  prefs.cycles = clamp_u64(prefs.cycles, 1, 12);
  prefs
}

fn prefs_path(app: &tauri::AppHandle) -> Option<PathBuf> {
  app
    .path()
    .app_config_dir()
    .ok()
    .map(|dir| dir.join("prefs.json"))
}

fn load_prefs(app: &tauri::AppHandle) -> Option<TimerPrefs> {
  let path = prefs_path(app)?;
  let data = fs::read_to_string(path).ok()?;
  serde_json::from_str(&data).ok().map(normalize_prefs)
}

fn save_prefs(app: &tauri::AppHandle, prefs: &TimerPrefs) {
  let Some(path) = prefs_path(app) else {
    return;
  };
  if let Some(parent) = path.parent() {
    let _ = fs::create_dir_all(parent);
  }
  if let Ok(payload) = serde_json::to_string_pretty(prefs) {
    let _ = fs::write(path, payload);
  }
}

fn update_prefs(
  app: &tauri::AppHandle,
  update: impl FnOnce(&mut TimerPrefs),
) -> TimerState {
  let state = app.state::<AppState>();
  let (prefs, snapshot) = with_engine(&state, |engine| {
    let mut prefs = engine.snapshot().prefs;
    update(&mut prefs);
    let prefs = normalize_prefs(prefs);
    engine.set_prefs(prefs.clone());
    let snapshot = engine.snapshot();
    (prefs, snapshot)
  });
  save_prefs(app, &prefs);
  snapshot
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  let engine = Arc::new(Mutex::new(TimerEngine::new()));
  tauri::Builder::default()
    .manage(AppState(engine.clone()))
    .setup(move |app| {
      #[cfg(target_os = "macos")]
      {
        let handle = app.handle();
        handle.set_activation_policy(tauri::ActivationPolicy::Accessory)?;
        handle.set_dock_visibility(false)?;
      }
      if let Some(prefs) = load_prefs(app.handle()) {
        let state = app.state::<AppState>();
        with_engine(&state, |engine| {
          engine.set_prefs(prefs);
        });
      }
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      let status_item = MenuItemBuilder::with_id("status", "Focus 25:00")
        .enabled(false)
        .build(app)?;
      let start_pause_item = MenuItemBuilder::with_id("toggle_run", "Start").build(app)?;
      let reset_item = MenuItemBuilder::with_id("reset", "Reset Timer").build(app)?;
      let skip_item = MenuItemBuilder::with_id("skip", "Skip Phase").build(app)?;

      let initial_snapshot = with_engine(&app.state::<AppState>(), |engine| engine.snapshot());
      let prefs = &initial_snapshot.prefs;
      let auto_start_item = CheckMenuItem::with_id(
        app,
        "pref:auto_start",
        "Auto Start Next Phase",
        true,
        prefs.auto_start,
        None::<&str>,
      )?;

      let focus_value_item =
        MenuItemBuilder::with_id("pref:focus:value", format_minutes_value("Current", prefs.focus_minutes))
          .enabled(false)
          .build(app)?;
      let focus_inc_item = MenuItemBuilder::with_id("pref:focus:inc", "Increase 5 min").build(app)?;
      let focus_dec_item = MenuItemBuilder::with_id("pref:focus:dec", "Decrease 5 min").build(app)?;
      let focus_menu = SubmenuBuilder::new(app, "Focus Length")
        .item(&focus_value_item)
        .separator()
        .items(&[&focus_inc_item, &focus_dec_item])
        .build()?;

      let short_value_item =
        MenuItemBuilder::with_id("pref:short:value", format_minutes_value("Current", prefs.short_break_minutes))
          .enabled(false)
          .build(app)?;
      let short_inc_item = MenuItemBuilder::with_id("pref:short:inc", "Increase 1 min").build(app)?;
      let short_dec_item = MenuItemBuilder::with_id("pref:short:dec", "Decrease 1 min").build(app)?;
      let short_menu = SubmenuBuilder::new(app, "Short Break")
        .item(&short_value_item)
        .separator()
        .items(&[&short_inc_item, &short_dec_item])
        .build()?;

      let long_value_item =
        MenuItemBuilder::with_id("pref:long:value", format_minutes_value("Current", prefs.long_break_minutes))
          .enabled(false)
          .build(app)?;
      let long_inc_item = MenuItemBuilder::with_id("pref:long:inc", "Increase 5 min").build(app)?;
      let long_dec_item = MenuItemBuilder::with_id("pref:long:dec", "Decrease 5 min").build(app)?;
      let long_menu = SubmenuBuilder::new(app, "Long Break")
        .item(&long_value_item)
        .separator()
        .items(&[&long_inc_item, &long_dec_item])
        .build()?;

      let cycles_value_item =
        MenuItemBuilder::with_id("pref:cycles:value", format_cycles_value(prefs.cycles))
          .enabled(false)
          .build(app)?;
      let cycles_inc_item = MenuItemBuilder::with_id("pref:cycles:inc", "Increase 1").build(app)?;
      let cycles_dec_item = MenuItemBuilder::with_id("pref:cycles:dec", "Decrease 1").build(app)?;
      let cycles_menu = SubmenuBuilder::new(app, "Cycles")
        .item(&cycles_value_item)
        .separator()
        .items(&[&cycles_inc_item, &cycles_dec_item])
        .build()?;

      let open_prefs_item = MenuItemBuilder::with_id("open_prefs", "Preferences...").build(app)?;
      let prefs_menu = SubmenuBuilder::new(app, "Preferences")
        .item(&open_prefs_item)
        .separator()
        .item(&auto_start_item)
        .separator()
        .item(&focus_menu)
        .item(&short_menu)
        .item(&long_menu)
        .item(&cycles_menu)
        .build()?;
      let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
      let menu = MenuBuilder::new(app)
        .item(&status_item)
        .separator()
        .items(&[&start_pause_item, &reset_item, &skip_item])
        .separator()
        .item(&prefs_menu)
        .separator()
        .item(&quit_item)
        .build()?;
      let tray = TrayIconBuilder::with_id("tray")
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(false)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_tray_icon_event(|tray, event| {
          if let TrayIconEvent::Click { button, .. } = event {
            if button == MouseButton::Left {
              let app = tray.app_handle();
              let snapshot =
                with_engine(&app.state::<AppState>(), |engine| engine.snapshot());
              update_menu(app, &snapshot);
            }
          }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
          "toggle_run" => {
            let state = app.state::<AppState>();
            let snapshot = with_engine(&state, |engine| {
              if engine.snapshot().is_running {
                engine.pause();
              } else {
                engine.start();
              }
              engine.snapshot()
            });
            update_menu(app, &snapshot);
          }
          "reset" => {
            let state = app.state::<AppState>();
            let snapshot = with_engine(&state, |engine| {
              engine.reset();
              engine.snapshot()
            });
            update_menu(app, &snapshot);
          }
          "skip" => {
            let state = app.state::<AppState>();
            let snapshot = with_engine(&state, |engine| {
              engine.skip();
              engine.snapshot()
            });
            update_menu(app, &snapshot);
          }
          "pref:auto_start" => {
            let snapshot = update_prefs(app, |prefs| {
              prefs.auto_start = !prefs.auto_start;
            });
            update_menu(app, &snapshot);
          }
          "pref:focus:inc" => {
            let snapshot = update_prefs(app, |prefs| {
              prefs.focus_minutes = clamp_u64(prefs.focus_minutes + 5, 1, 180);
            });
            update_menu(app, &snapshot);
          }
          "pref:focus:dec" => {
            let snapshot = update_prefs(app, |prefs| {
              prefs.focus_minutes = clamp_u64(prefs.focus_minutes.saturating_sub(5), 1, 180);
            });
            update_menu(app, &snapshot);
          }
          "pref:short:inc" => {
            let snapshot = update_prefs(app, |prefs| {
              prefs.short_break_minutes = clamp_u64(prefs.short_break_minutes + 1, 1, 30);
            });
            update_menu(app, &snapshot);
          }
          "pref:short:dec" => {
            let snapshot = update_prefs(app, |prefs| {
              prefs.short_break_minutes =
                clamp_u64(prefs.short_break_minutes.saturating_sub(1), 1, 30);
            });
            update_menu(app, &snapshot);
          }
          "pref:long:inc" => {
            let snapshot = update_prefs(app, |prefs| {
              prefs.long_break_minutes = clamp_u64(prefs.long_break_minutes + 5, 1, 90);
            });
            update_menu(app, &snapshot);
          }
          "pref:long:dec" => {
            let snapshot = update_prefs(app, |prefs| {
              prefs.long_break_minutes =
                clamp_u64(prefs.long_break_minutes.saturating_sub(5), 1, 90);
            });
            update_menu(app, &snapshot);
          }
          "pref:cycles:inc" => {
            let snapshot = update_prefs(app, |prefs| {
              prefs.cycles = clamp_u64(prefs.cycles + 1, 1, 12);
            });
            update_menu(app, &snapshot);
          }
          "pref:cycles:dec" => {
            let snapshot = update_prefs(app, |prefs| {
              prefs.cycles = clamp_u64(prefs.cycles.saturating_sub(1), 1, 12);
            });
            update_menu(app, &snapshot);
          }
          "open_prefs" => {
            open_preferences_window(app);
          }
          "quit" => {
            app.exit(0);
          }
          _ => {}
        })
        .build(app)?;
      app.manage(TrayState(tray));
      app.manage(MenuState {
        status_item: status_item.clone(),
        start_pause_item: start_pause_item.clone(),
        auto_start_item: auto_start_item.clone(),
        focus_value_item: focus_value_item.clone(),
        short_value_item: short_value_item.clone(),
        long_value_item: long_value_item.clone(),
        cycles_value_item: cycles_value_item.clone(),
      });

      update_menu(app.handle(), &initial_snapshot);

      spawn_timer(app.handle().clone(), engine.clone());

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_timer_state,
      start_timer,
      pause_timer,
      reset_timer,
      skip_timer,
      set_prefs
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
