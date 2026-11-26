//! Throwaway S2 checks for map question Q42. Never merged.
//!
//! The frontend runs the checks. This module gives it the parts that the page
//! cannot do: real keyboard input through the display server, a different
//! application on the clipboard, a pixel snapshot from WebKitGTK, the
//! `webkit://gpu` report and the font match.
//!
//! The session backend selects the tools:
//! - X11 (`GDK_BACKEND=x11`, so XWayland on this host): xdotool and xclip.
//! - Wayland: ydotool (kernel uinput, so the compositor sees a real device)
//!   and wl-clipboard.

use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use javascriptcore::ValueExt;
use webkit2gtk::{InputMethodContextExt, SnapshotOptions, SnapshotRegion, WebViewExt};

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    X11,
    Wayland,
}

pub fn backend() -> Backend {
    match std::env::var("GDK_BACKEND").as_deref() {
        Ok("x11") => Backend::X11,
        _ if std::env::var_os("WAYLAND_DISPLAY").is_some() => Backend::Wayland,
        _ => Backend::X11,
    }
}

#[tauri::command]
pub fn s2_config() -> Option<serde_json::Value> {
    let option = std::env::var("DIFFY_S2_CHECK").ok()?;
    let label = std::env::var("DIFFY_S2_LABEL").unwrap_or_else(|_| format!("q42-{option}"));
    Some(serde_json::json!({ "option": option, "label": label, "backend": backend() }))
}

fn run(cmd: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("cannot run {cmd}: {e}"))?;
    if !out.status.success() {
        return Err(format!("{cmd} {args:?} failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// The Linux input event code of a key, for ydotool.
fn keycode(name: &str) -> Result<u32, String> {
    Ok(match name {
        "ctrl" => 29,
        "shift" => 42,
        "alt" => 56,
        "Escape" => 1,
        "Tab" => 15,
        "Return" => 28,
        "space" => 57,
        "apostrophe" => 40,
        "Home" => 102,
        "End" => 107,
        "Next" => 109,
        "Prior" => 104,
        "BackSpace" => 14,
        "a" => 30, "c" => 46, "e" => 18, "f" => 33, "m" => 50, "u" => 22, "v" => 47, "z" => 44,
        "9" => 10,
        _ => return Err(format!("no keycode for {name}")),
    })
}

/// One chord such as "ctrl+shift+u" as ydotool key events: press all, release in reverse.
fn ydotool_chord(chord: &str) -> Result<Vec<String>, String> {
    let codes: Vec<u32> = chord.split('+').map(keycode).collect::<Result<_, _>>()?;
    let mut ev: Vec<String> = codes.iter().map(|c| format!("{c}:1")).collect();
    ev.extend(codes.iter().rev().map(|c| format!("{c}:0")));
    Ok(ev)
}

/// One input step: `key <chord> [<chord> ...]`, `repeat <n> <chord>` or `type <text>`.
/// Chords use the xdotool key names.
#[tauri::command]
pub async fn input(step: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || input_blocking(&step))
        .await
        .map_err(|e| e.to_string())?
}

fn input_blocking(step: &str) -> Result<(), String> {
    let (verb, rest) = step.split_once(' ').ok_or("bad step")?;
    match (backend(), verb) {
        (Backend::X11, "key") => {
            let mut args = vec!["key", "--clearmodifiers", "--delay", "40"];
            args.extend(rest.split(' '));
            run("xdotool", &args).map(|_| ())
        }
        (Backend::X11, "repeat") => {
            let (n, chord) = rest.split_once(' ').ok_or("bad repeat")?;
            run("xdotool", &["key", "--repeat", n, "--delay", "15", chord]).map(|_| ())
        }
        (Backend::X11, "type") => run("xdotool", &["type", "--delay", "60", rest]).map(|_| ()),
        (Backend::Wayland, "key") => {
            let mut ev = vec!["key".to_string(), "-d".to_string(), "40".to_string()];
            for chord in rest.split(' ') {
                ev.extend(ydotool_chord(chord)?);
            }
            let args: Vec<&str> = ev.iter().map(String::as_str).collect();
            run("ydotool", &args).map(|_| ())
        }
        (Backend::Wayland, "repeat") => {
            let (n, chord) = rest.split_once(' ').ok_or("bad repeat")?;
            let n: usize = n.parse().map_err(|_| "bad count")?;
            let mut ev = vec!["key".to_string(), "-d".to_string(), "15".to_string()];
            for _ in 0..n {
                ev.extend(ydotool_chord(chord)?);
            }
            let args: Vec<&str> = ev.iter().map(String::as_str).collect();
            run("ydotool", &args).map(|_| ())
        }
        (Backend::Wayland, "type") => run("ydotool", &["type", "-d", "60", rest]).map(|_| ()),
        _ => Err(format!("unknown step {step}")),
    }
}

/// Write text to the clipboard from a different application.
#[tauri::command]
pub async fn ext_clipboard_write(text: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let (cmd, args): (&str, &[&str]) = match backend() {
            Backend::X11 => ("xclip", &["-selection", "clipboard", "-i"]),
            Backend::Wayland => ("wl-copy", &[]),
        };
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot run {cmd}: {e}"))?;
        use std::io::Write;
        child.stdin.take().unwrap().write_all(text.as_bytes()).map_err(|e| e.to_string())?;
        // xclip and wl-copy fork and stay to serve the selection. Wait for the parent only.
        child.wait().map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Read the clipboard from a different application. `via` is "x11" (xclip) or "wayland" (wl-paste).
#[tauri::command]
pub async fn ext_clipboard_read(via: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || match via.as_str() {
        "x11" => run("timeout", &["3", "xclip", "-selection", "clipboard", "-o"]),
        "wayland" => run("timeout", &["3", "wl-paste", "-n"]),
        _ => Err("bad via".into()),
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Select the global ibus engine, or return the current one when `name` is empty.
#[tauri::command]
pub fn ime_engine(name: String) -> Result<String, String> {
    if !name.is_empty() {
        // `ibus engine <name>` can exit with 1 after it sets the engine. Check the result below.
        let _ = Command::new("ibus").args(["engine", &name]).output();
    }
    run("ibus", &["engine"]).map(|s| s.trim().to_string())
}

/// wry 0.57 turns off the preedit of the WebKitGTK input method context.
/// This turns it on again, as a candidate workaround.
#[tauri::command]
pub fn set_preedit(app: AppHandle, enable: bool) -> Result<(), String> {
    let win = app.get_webview_window("main").ok_or("no main window")?;
    win.with_webview(move |pw| {
        if let Some(ctx) = pw.inner().input_method_context() {
            ctx.set_enable_preedit(enable);
        }
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn focus_main(app: AppHandle) -> Result<(), String> {
    let w = app.get_webview_window("main").ok_or("no main window")?;
    w.set_focus().map_err(|e| e.to_string())
}

/// The fontconfig match for each family.
#[tauri::command]
pub fn font_match(families: Vec<String>) -> serde_json::Value {
    let list: Vec<_> = families
        .iter()
        .map(|f| {
            let m = run("fc-match", &[f.as_str()]).map(|s| s.trim().to_string()).unwrap_or_else(|e| e);
            serde_json::json!({ "family": f, "fc_match": m })
        })
        .collect();
    serde_json::Value::Array(list)
}

/// Open `webkit://gpu` in a second window and return its text.
#[tauri::command]
pub async fn gpu_report(app: AppHandle) -> Result<String, String> {
    let url = WebviewUrl::External("webkit://gpu".parse().map_err(|e| format!("{e}"))?);
    let win = WebviewWindowBuilder::new(&app, "gpu", url)
        .title("Q42 GPU report")
        .inner_size(700.0, 500.0)
        .build()
        .map_err(|e| e.to_string())?;
    let mut text = String::new();
    for _ in 0..20 {
        tokio_sleep(250).await;
        let (tx, rx) = mpsc::channel();
        win.with_webview(move |pw| {
            pw.inner().evaluate_javascript(
                "document.body ? document.body.innerText : ''",
                None,
                None,
                None::<&gio::Cancellable>,
                move |r| {
                    let _ = tx.send(r.map(|v| v.to_str().to_string()).map_err(|e| e.to_string()));
                },
            );
        })
        .map_err(|e| e.to_string())?;
        text = wait(rx, 3000)??;
        if text.len() > 100 {
            break;
        }
    }
    let _ = win.close();
    Ok(text)
}

async fn tokio_sleep(ms: u64) {
    let _ = tauri::async_runtime::spawn_blocking(move || std::thread::sleep(Duration::from_millis(ms))).await;
}

fn wait<T: Send + 'static>(rx: mpsc::Receiver<T>, ms: u64) -> Result<T, String> {
    rx.recv_timeout(Duration::from_millis(ms)).map_err(|e| format!("no reply from the main thread: {e}"))
}

/// One rendered line from the DOM, in CSS pixels in the viewport.
#[derive(Deserialize)]
pub struct LineBox {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    /// The line has text, so its row must have ink.
    text: bool,
}

/// An editor text area, in CSS pixels in the viewport.
#[derive(Deserialize)]
pub struct Region {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Serialize, Default)]
pub struct SnapshotCheck {
    /// The longest run of pixel rows with no ink in a text area, in editor lines.
    /// It reads only the image, so it does not race with the DOM.
    longest_blank_run_lines: f64,
    width: i32,
    height: i32,
    scale: f64,
    text_lines: u32,
    text_lines_without_ink: u32,
    empty_lines_with_ink: u32,
    distinct_colors_sampled: u32,
    saved: Option<String>,
}

/// Take a WebKitGTK snapshot of the visible page. Check that each DOM line with
/// text has ink in its row. Save the PNG when `save` names a file.
#[tauri::command]
pub async fn snapshot_check(
    app: AppHandle,
    lines: Vec<LineBox>,
    regions: Vec<Region>,
    line_height: f64,
    css_width: f64,
    save: Option<String>,
) -> Result<SnapshotCheck, String> {
    let win = app.get_webview_window("main").ok_or("no main window")?;
    let (tx, rx) = mpsc::channel();
    win.with_webview(move |pw| {
        pw.inner().snapshot(
            SnapshotRegion::Visible,
            SnapshotOptions::NONE,
            None::<&gio::Cancellable>,
            move |r| {
                let _ = tx.send(r.map_err(|e| e.to_string()).and_then(|s| analyse(s, &lines, &regions, line_height, css_width, save)));
            },
        );
    })
    .map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || wait(rx, 5000))
        .await
        .map_err(|e| e.to_string())??
}

fn analyse(
    surface: cairo::Surface,
    lines: &[LineBox],
    regions: &[Region],
    line_height: f64,
    css_width: f64,
    save: Option<String>,
) -> Result<SnapshotCheck, String> {
    let mut img = cairo::ImageSurface::try_from(surface).map_err(|_| "snapshot is not an image surface")?;
    let (w, h, stride) = (img.width(), img.height(), img.stride() as usize);
    let scale = w as f64 / css_width;
    let mut out = SnapshotCheck { width: w, height: h, scale, ..Default::default() };
    if let Some(name) = save.filter(|n| n.chars().all(|c| c.is_ascii_alphanumeric() || "-_".contains(c))) {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../bench-results/q42");
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("{name}.png"));
        let mut f = std::fs::File::create(&path).map_err(|e| e.to_string())?;
        img.write_to_png(&mut f).map_err(|e| e.to_string())?;
        out.saved = Some(path.canonicalize().unwrap_or(path).display().to_string());
    }
    img.flush();
    let data = img.data().map_err(|e| e.to_string())?;
    let px = |x: i32, y: i32| -> [u8; 3] {
        let i = y as usize * stride + x as usize * 4;
        [data[i], data[i + 1], data[i + 2]]
    };
    let mut colors = std::collections::HashSet::new();
    for l in lines {
        let x0 = ((l.x * scale) as i32).clamp(0, w - 1);
        let x1 = (((l.x + l.w) * scale) as i32).clamp(0, w);
        // Leave out the top and bottom pixel rows, where a neighbour line can bleed in.
        let y0 = (((l.y + 1.0) * scale) as i32).clamp(0, h - 1);
        let y1 = (((l.y + l.h - 1.0) * scale) as i32).clamp(0, h);
        if x1 <= x0 || y1 <= y0 {
            continue;
        }
        // The background of the row is the most frequent colour in it.
        let mut freq = std::collections::HashMap::new();
        for y in (y0..y1).step_by(2) {
            for x in (x0..x1).step_by(3) {
                let c = px(x, y);
                *freq.entry(c).or_insert(0u32) += 1;
                if colors.len() < 4096 {
                    colors.insert(c);
                }
            }
        }
        let bg = freq.into_iter().max_by_key(|&(_, n)| n).map(|(c, _)| c).unwrap_or([0; 3]);
        let mut ink = 0u32;
        for y in y0..y1 {
            for x in x0..x1 {
                let c = px(x, y);
                let d = (0..3).map(|k| (c[k] as i32 - bg[k] as i32).abs()).max().unwrap();
                if d > 60 {
                    ink += 1;
                }
            }
        }
        let has_ink = ink >= 3;
        if l.text {
            out.text_lines += 1;
            if !has_ink {
                out.text_lines_without_ink += 1;
            }
        } else if has_ink {
            out.empty_lines_with_ink += 1;
        }
    }
    out.distinct_colors_sampled = colors.len() as u32;
    let mut longest = 0i32;
    for r in regions {
        let x0 = ((r.x * scale) as i32).clamp(0, w - 1);
        let x1 = (((r.x + r.w) * scale) as i32).clamp(0, w);
        let y0 = ((r.y * scale) as i32).clamp(0, h - 1);
        let y1 = (((r.y + r.h) * scale) as i32).clamp(0, h);
        let mut run = 0i32;
        for y in y0..y1 {
            let mut freq = std::collections::HashMap::new();
            for x in (x0..x1).step_by(2) {
                *freq.entry(px(x, y)).or_insert(0u32) += 1;
            }
            let bg = freq.into_iter().max_by_key(|&(_, n)| n).map(|(c, _)| c).unwrap_or([0; 3]);
            let ink = (x0..x1)
                .filter(|&x| {
                    let c = px(x, y);
                    (0..3).map(|k| (c[k] as i32 - bg[k] as i32).abs()).max().unwrap() > 60
                })
                .take(2)
                .count();
            if ink >= 2 {
                run = 0;
            } else {
                run += 1;
                longest = longest.max(run);
            }
        }
    }
    out.longest_blank_run_lines = longest as f64 / (line_height * scale);
    Ok(out)
}

// Q78: fcitx5 with the preedit on. Throwaway code. Never merge.

#[tauri::command]
pub fn q78_config() -> Option<serde_json::Value> {
    let label = std::env::var("DIFFY_Q78_LABEL").ok()?;
    Some(serde_json::json!({ "label": label, "backend": backend() }))
}

/// Switch fcitx5 to an input method and return the current one.
#[tauri::command]
pub fn q78_im(name: String) -> Result<String, String> {
    if !name.is_empty() {
        run("fcitx5-remote", &["-s", &name])?;
        std::thread::sleep(Duration::from_millis(300));
    }
    run("fcitx5-remote", &["-n"]).map(|s| s.trim().to_string())
}

/// The KWin window list as JSON, from the command in Q78_GEO.
#[tauri::command]
pub async fn q78_geometry() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let cmd = std::env::var("Q78_GEO").map_err(|_| "Q78_GEO is not set".to_string())?;
        run("python3", &["-I", &cmd])
    })
    .await
    .map_err(|e| e.to_string())?
}

// Q85: what the AT-SPI tree gives for option B. Throwaway code. Never merge.

#[tauri::command]
pub fn q85_config() -> Option<serde_json::Value> {
    let label = std::env::var("DIFFY_Q85_LABEL").ok()?;
    let sr = std::env::var("DIFFY_Q85_SR").unwrap_or_else(|_| "auto".into());
    let target = std::env::var("DIFFY_Q85_TARGET").unwrap_or_else(|_| "right".into());
    let fix = std::env::var("DIFFY_Q85_FIX").is_ok();
    Some(serde_json::json!({ "label": label, "sr": sr, "target": target, "fix": fix, "fixAttr": std::env::var("DIFFY_Q85_FIX_ATTR").ok() }))
}
