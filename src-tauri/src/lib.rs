use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::{self, RecvTimeoutError, Sender},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use enigo::{Button, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use serde::Deserialize;
use tauri::{Emitter, State};

#[derive(Default)]
struct ClickerState {
    running: AtomicBool,
    next_generation: AtomicU64,
    active_generation: AtomicU64,
    stop_sender: Mutex<Option<(u64, Sender<()>)>>,
    gate: Mutex<()>,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AutomationConfig {
    mode: String,
    key: Option<String>,
    button: String,
    click_type: String,
    interval_secs: f64,
}

fn parse_key(value: &str) -> Result<Key, String> {
    let mut chars = value.chars();
    if let (Some(character), None) = (chars.next(), chars.next()) {
        return Ok(Key::Unicode(character));
    }

    let key = match value {
        "Enter" => Key::Return,
        " " => Key::Space,
        "Escape" => Key::Escape,
        "Backspace" => Key::Backspace,
        "Tab" => Key::Tab,
        "Delete" => Key::Delete,
        "Insert" => Key::Insert,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "ArrowLeft" => Key::LeftArrow,
        "ArrowRight" => Key::RightArrow,
        "ArrowUp" => Key::UpArrow,
        "ArrowDown" => Key::DownArrow,
        "Shift" => Key::Shift,
        "ShiftLeft" => Key::LShift,
        "ShiftRight" => Key::RShift,
        "Control" => Key::Control,
        "ControlLeft" => Key::LControl,
        "ControlRight" => Key::RControl,
        "Alt" => Key::Alt,
        "AltLeft" => Key::LMenu,
        "AltRight" => Key::RMenu,
        "Meta" | "MetaLeft" => Key::LWin,
        "MetaRight" => Key::RWin,
        "CapsLock" => Key::CapsLock,
        "NumLock" => Key::Numlock,
        "ScrollLock" => Key::Scroll,
        "Pause" => Key::Pause,
        "PrintScreen" => Key::PrintScr,
        "ContextMenu" => Key::Select,
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        "F13" => Key::F13,
        "F14" => Key::F14,
        "F15" => Key::F15,
        "F16" => Key::F16,
        "F17" => Key::F17,
        "F18" => Key::F18,
        "F19" => Key::F19,
        "F20" => Key::F20,
        "F21" => Key::F21,
        "F22" => Key::F22,
        "F23" => Key::F23,
        "F24" => Key::F24,
        "Numpad0" => Key::Numpad0,
        "Numpad1" => Key::Numpad1,
        "Numpad2" => Key::Numpad2,
        "Numpad3" => Key::Numpad3,
        "Numpad4" => Key::Numpad4,
        "Numpad5" => Key::Numpad5,
        "Numpad6" => Key::Numpad6,
        "Numpad7" => Key::Numpad7,
        "Numpad8" => Key::Numpad8,
        "Numpad9" => Key::Numpad9,
        "NumpadAdd" => Key::Add,
        "NumpadSubtract" => Key::Subtract,
        "NumpadMultiply" => Key::Multiply,
        "NumpadDivide" => Key::Divide,
        "NumpadDecimal" => Key::Decimal,
        "NumpadEnter" => Key::Return,
        "VolumeUp" | "AudioVolumeUp" => Key::VolumeUp,
        "VolumeDown" | "AudioVolumeDown" => Key::VolumeDown,
        "VolumeMute" | "AudioVolumeMute" => Key::VolumeMute,
        "MediaPlayPause" => Key::MediaPlayPause,
        "MediaTrackNext" => Key::MediaNextTrack,
        "MediaTrackPrevious" => Key::MediaPrevTrack,
        _ => return Err(format!("The key “{value}” is not supported on this keyboard.")),
    };
    Ok(key)
}

fn validate_config(config: &AutomationConfig) -> Result<Option<Key>, String> {
    if !config.interval_secs.is_finite()
        || !(0.01..=86400.0).contains(&config.interval_secs)
    {
        return Err("Enter an interval between 0.01 and 86,400 seconds.".into());
    }

    match config.mode.as_str() {
        "keyboard" => {
            let key = config
                .key
                .as_deref()
                .ok_or_else(|| "Choose a keyboard key first.".to_string())?;
            if key == "F6" {
                return Err("F6 is reserved for Start / Stop. Choose a different key.".into());
            }
            parse_key(key).map(Some)
        }
        "mouse" => {
            if !matches!(config.button.as_str(), "left" | "right" | "middle") {
                return Err("Choose a left, right, or middle mouse button.".into());
            }
            if !matches!(config.click_type.as_str(), "single" | "double") {
                return Err("Choose a single or double click.".into());
            }
            Ok(None)
        }
        _ => Err("Choose keyboard or mouse mode.".into()),
    }
}

fn perform_action(
    enigo: &mut Enigo,
    config: &AutomationConfig,
    key: Option<Key>,
    stop_receiver: &mpsc::Receiver<()>,
) -> Result<(), String> {
    match config.mode.as_str() {
        "keyboard" => enigo
            .key(key.ok_or_else(|| "No keyboard key was selected.".to_string())?, Direction::Click)
            .map_err(|error| error.to_string()),
        "mouse" => {
            let button = match config.button.as_str() {
                "left" => Button::Left,
                "right" => Button::Right,
                "middle" => Button::Middle,
                _ => return Err("Choose a left, right, or middle mouse button.".into()),
            };
            enigo.button(button, Direction::Click).map_err(|error| error.to_string())?;
            if config.click_type == "double" {
                thread::sleep(Duration::from_millis(60));
                if stop_receiver.try_recv().is_ok() { return Ok(()); }
                enigo.button(button, Direction::Click).map_err(|error| error.to_string())?;
            }
            Ok(())
        }
        _ => Err("Choose keyboard or mouse mode.".into()),
    }
}

fn finish_worker(app: &tauri::AppHandle, state: &ClickerState, generation: u64, error: Option<String>) {
    if state.active_generation.load(Ordering::SeqCst) != generation {
        return;
    }
    state.running.store(false, Ordering::SeqCst);
    if let Ok(mut sender) = state.stop_sender.lock() {
        if sender.as_ref().is_some_and(|(current, _)| *current == generation) {
            *sender = None;
        }
    }
    let _ = app.emit("clicker-state", false);
    if let Some(message) = error {
        let _ = app.emit("automation-error", message);
    }
}

fn start(state: Arc<ClickerState>, app: tauri::AppHandle, config: AutomationConfig) -> Result<bool, String> {
    let key = validate_config(&config)?;
    let generation;
    let stop_receiver;
    {
        let _guard = state.gate.lock().map_err(|_| "The clicker is unavailable.".to_string())?;
        if state.running.load(Ordering::SeqCst) {
            return Ok(false);
        }
        generation = state.next_generation.fetch_add(1, Ordering::SeqCst) + 1;
        state.active_generation.store(generation, Ordering::SeqCst);
        state.running.store(true, Ordering::SeqCst);
        let (stop_sender, receiver) = mpsc::channel();
        stop_receiver = receiver;
        *state.stop_sender.lock().map_err(|_| "The clicker is unavailable.".to_string())? =
            Some((generation, stop_sender));
    }

    let worker_state = Arc::clone(&state);
    let worker_app = app.clone();
    let worker_config = config.clone();
    let (ready_sender, ready_receiver) = mpsc::sync_channel::<Result<(), String>>(1);
    thread::spawn(move || {
        let mut enigo = match Enigo::new(&Settings::default()) {
            Ok(enigo) => {
                let _ = ready_sender.send(Ok(()));
                enigo
            }
            Err(error) => {
                let message = format!("Could not start keyboard and mouse input: {error}");
                let _ = ready_sender.send(Err(message.clone()));
                finish_worker(&worker_app, &worker_state, generation, Some(message));
                return;
            }
        };

        let interval = Duration::from_secs_f64(worker_config.interval_secs);
        let mut worker_error = None;
        loop {
            if stop_receiver.try_recv().is_ok() {
                break;
            }
            if let Err(error) = perform_action(&mut enigo, &worker_config, key, &stop_receiver) {
                worker_error = Some(format!("Input failed: {error}"));
                break;
            }
            match stop_receiver.recv_timeout(interval) {
                Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
        finish_worker(&worker_app, &worker_state, generation, worker_error);
    });

    let _ = app.emit("clicker-state", true);
    match ready_receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(())) => Ok(true),
        Ok(Err(message)) => Err(message),
        Err(_) => {
            stop(&state, &app);
            Err("Starting the input controller timed out. Try again.".into())
        }
    }
}

fn stop(state: &ClickerState, app: &tauri::AppHandle) -> bool {
    let _guard = match state.gate.lock() {
        Ok(guard) => guard,
        Err(_) => return false,
    };
    if !state.running.swap(false, Ordering::SeqCst) {
        return false;
    }
    let generation = state.active_generation.load(Ordering::SeqCst);
    let sender = state
        .stop_sender
        .lock()
        .ok()
        .and_then(|mut slot| match slot.take() {
            Some((current, sender)) if current == generation => Some(sender),
            Some(other) => {
                *slot = Some(other);
                None
            }
            None => None,
        });
    if let Some(sender) = sender {
        let _ = sender.send(());
    }
    let _ = app.emit("clicker-state", false);
    true
}

#[tauri::command]
fn start_clicker(
    app: tauri::AppHandle,
    state: State<'_, Arc<ClickerState>>,
    config: AutomationConfig,
) -> Result<bool, String> {
    start(Arc::clone(&state), app, config)
}

#[tauri::command]
fn stop_clicker(app: tauri::AppHandle, state: State<'_, Arc<ClickerState>>) -> bool {
    stop(&state, &app)
}

#[tauri::command]
fn toggle_clicker(
    app: tauri::AppHandle,
    state: State<'_, Arc<ClickerState>>,
    config: AutomationConfig,
) -> Result<bool, String> {
    if state.running.load(Ordering::SeqCst) {
        stop(&state, &app);
        Ok(false)
    } else {
        start(Arc::clone(&state), app, config)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shortcut_plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_shortcut("F6")
        .expect("F6 is a valid global shortcut")
        .with_handler(|app, _shortcut, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                let _ = app.emit("hotkey-toggle", ());
            }
        })
        .build();

    tauri::Builder::default()
        .plugin(shortcut_plugin)
        .manage(Arc::new(ClickerState::default()))
        .invoke_handler(tauri::generate_handler![start_clicker, stop_clicker, toggle_clicker])
        .run(tauri::generate_context!())
        .expect("error while running Professor Soly × Codex Auto Clicker");
}
