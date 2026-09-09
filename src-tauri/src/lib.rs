use serde::{Deserialize, Serialize};
use serde_json::json;
use sherpa_rs::sense_voice::{SenseVoiceConfig, SenseVoiceRecognizer};
use sherpa_rs::silero_vad::{SileroVad, SileroVadConfig};
use std::fs::File;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Instant;
use std::{fs, time::Duration};
use tauri::{Emitter, Manager, Window};
use tokio::sync::oneshot;
use wasapi::{initialize_mta, DeviceEnumerator, Direction};

struct AppState(Mutex<Option<oneshot::Sender<()>>>);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct Config {
    vad_slience_ms: u128,
    vad_max_speech_ms: u64,
    provider: String,
    enable_cuda: bool,
    ai_api_base: String,
    ai_api_key: String,
    ai_model: String,
    target_lang: String,
}

fn create_config() -> Result<(), Box<dyn std::error::Error>> {
    let data = Config {
        vad_slience_ms: 800,
        vad_max_speech_ms: 8000,
        provider: "cpu".into(),
        enable_cuda: false,
        ai_api_base: "http://192.168.31.65:8080/v1".into(),
        ai_api_key: "sk-123".into(),
        ai_model: "Gamma4-E4B-it".into(),
        target_lang: "zh".into(),
    };
    let mut file = File::create_new("config.json")?;
    serde_json::to_writer_pretty(&mut file, &data)?;
    Ok(())
}

fn sensevoice_seg(
    seg: sherpa_rs::silero_vad::SpeechSegment,
    rec: &mut SenseVoiceRecognizer,
) -> Result<String, Box<dyn std::error::Error>> {
    let r = rec.transcribe(16000, &seg.samples);
    Ok(r.text)
}

// 翻译一句，失败自动重试 3 次，返回译文或错误信息
fn translate(http: &reqwest::blocking::Client, cfg: &Config, text: &str) -> String {
    let url = if cfg.ai_api_base.ends_with("/chat/completions") {
        cfg.ai_api_base.clone()
    } else {
        format!("{}/chat/completions", cfg.ai_api_base.trim_end_matches('/'))
    };
    let prompt = format!("你是字幕翻译助手。把下面这段字幕原文直接翻译成{}，只输出译文本身，不要任何解释、说明或多余的话：\n{}", cfg.target_lang, text);
    let body = json!({
        "model": &cfg.ai_model,
        "messages": [{"role": "user", "content": prompt}]
    });

    let mut last_err = String::new();
    for attempt in 1..=3 {
        let mut req = http.post(&url).json(&body);
        if !cfg.ai_api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", cfg.ai_api_key));
        }
        match req.send() {
            Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>() {
                Ok(v) => {
                    if let Some(t) = v["choices"][0]["message"]["content"].as_str() {
                        if !t.trim().is_empty() {
                            return t.to_string();
                        }
                    }
                    last_err = format!("空译文: {}", v);
                }
                Err(e) => last_err = format!("JSON 解析失败: {e}"),
            },
            Ok(resp) => last_err = format!("HTTP {}", resp.status()),
            Err(e) => last_err = e.to_string(),
        }
        if attempt < 3 {
            std::thread::sleep(Duration::from_millis(500 * attempt as u64));
        }
    }
    format!("[翻译失败] {}", last_err)
}

#[tauri::command]
fn start_loopback(window: Window, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let (stop_tx, mut stop_rx) = oneshot::channel();
    *state.0.lock().unwrap() = Some(stop_tx);

    let win = window.clone();
    std::thread::spawn(move || {
        let mut run = || -> Result<(), Box<dyn std::error::Error>> {
            let config_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config.json");
            if File::open(&config_path).is_err() {
                let _ = create_config();
            }
            let s = fs::read_to_string(&config_path)?;
            let cfg: Config = serde_json::from_str(&s)?;
            let vad_silence = cfg.vad_slience_ms;
            let vad_max = cfg.vad_max_speech_ms;
            let _ = initialize_mta();
            let enumerator = DeviceEnumerator::new()?;
            let dev = enumerator.get_default_device(&Direction::Render)?;
            let mut client = dev.get_iaudioclient()?;
            let format = client.get_mixformat()?;
            client.initialize_client(
                &format,
                &Direction::Capture,
                &&wasapi::StreamMode::PollingShared {
                    autoconvert: true,
                    buffer_duration_hns: 200_000,
                },
            )?;
            let block_align = format.get_blockalign() as usize;
            let mut vad = SileroVad::new(
                SileroVadConfig {
                    model: asset(&window,"silero_vad.onnx").to_str().unwrap().to_string(),
                    min_silence_duration: vad_silence as f32 / 1000.0,
                    min_speech_duration: 0.25,
                    max_speech_duration: vad_max as f32 / 1000.0,
                    threshold: 0.5,
                    sample_rate: 16000,
                    window_size: 512,
                    provider: if cfg.enable_cuda {
                        Some(cfg.provider.clone())
                    } else {
                        Some("cpu".into())
                    },
                    ..Default::default()
                },
                30.0,
            )?;
            let _base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
            let mut rec = SenseVoiceRecognizer::new(SenseVoiceConfig {
                model: asset(&window,"model.int8.onnx").to_str().unwrap().to_string(),
                tokens: asset(&window,"tokens.txt").to_str().unwrap().to_string(),
                provider: if cfg.enable_cuda {
                    Some(cfg.provider.clone())
                } else {
                    Some("cpu".into())
                },
                ..Default::default()
            })?;

            // 翻译线程：从 channel 拿 (id, text)，翻译后 emit，不阻塞识别循环
            let (tx, rx) = mpsc::channel::<(u64, String)>();
            let win_t = win.clone();
            let cfg_t = cfg.clone();
            std::thread::spawn(move || {
                let http = reqwest::blocking::Client::builder()
                    .no_proxy()
                    .timeout(Duration::from_secs(30))
                    .build()
                    .unwrap();
                for (id, text) in rx {
                    let trans = translate(&http, &cfg_t, &text);
                    let _ = win_t.emit("translation", json!({"id": id, "trans": trans}));
                }
            });

            let capture = client.get_audiocaptureclient()?;
            client.start_stream()?;
            let mut id: u64 = 0;
            let mut speech_start :Option<Instant>=None;
            let max_speech =Duration::from_millis(vad_max);
            loop {
                if stop_rx.try_recv().is_ok() {
                    println!("正在停止");
                    client.stop_stream()?;
                    break Ok(());
                }
                let frames = capture.get_next_packet_size()?.unwrap_or(0);
                if frames == 0 {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                let mut buf = vec![0u8; frames as usize * block_align];
                let (read_frames, _) = capture.read_from_device(&mut buf)?;
                let chunk48: Vec<f32> = buf[..read_frames as usize * block_align]
                    .chunks(8)
                    .map(|c| {
                        let l = f32::from_le_bytes([c[0], c[1], c[2], c[3]]);
                        let r = f32::from_le_bytes([c[4], c[5], c[6], c[7]]);
                        (l + r) / 2.0
                    })
                    .collect();
                let chunk16: Vec<f32> = chunk48.iter().step_by(3).copied().collect();
                vad.accept_waveform(chunk16);
                if vad.is_speech(){
                    if speech_start.is_none(){ speech_start =Some(Instant::now());}
                    if speech_start.unwrap().elapsed() >max_speech{

                        vad.flush();
                        speech_start=None;
                    }
                    else {
                        speech_start=None;
                    }
                }
                while !vad.is_empty() {
                    
                    let seg = vad.front();
                    if (seg.samples.len() as f32 /16000.0) < 0.3{ vad.pop(); continue;}
                    let text = sensevoice_seg(seg, &mut rec)?;
                    let _ = win.emit("subtitle", json!({"id": id, "text": text}));
                    vad.pop();
                    let _ = tx.send((id, text));
                    id += 1;
                }
            }
        };
        if let Err(e) = run() {
            eprintln!("loopback error: {e}");
        }
    });

    // 丢到后台线程跑，wasapi/silero 的 *const c_void 非 Send，不能跨 await

    Ok(())
}

#[tauri::command]
fn stop_loopback(_window: Window, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some(stop_tx) = state.0.lock().unwrap().take() {
        let _ = stop_tx.send(());
    }

    Ok(())
}
#[tauri::command]
fn get_config()->Result<Config,String>{
    let p =std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config.json");
    let s =fs::read_to_string(&p).map_err(|e| e.to_string())?;
    serde_json::from_str(&s).map_err(|e|e.to_string())
}
#[tauri::command]
fn save_config(cfg: Config)->Result<(),String>{
      let p =std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config.json");
    let f =File::create(&p).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(f, &cfg).map_err(|e|e.to_string())?;
    Ok(())
}
#[tauri::command]
fn set_decorations(window: Window,show:bool)->Result<(),String>
{
    window.set_decorations(show).map_err(|e|e.to_string())?;
    Ok(())
}
fn asset(window: &Window,name:&str)->std::path::PathBuf{
    if let Ok(r) =window.app_handle().path().resource_dir(){
        let p =r.join("model").join(name);
        if p.exists() {return p;}
    }
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("model").join(name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState(Mutex::new(None)))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_loopback,
            stop_loopback,
            get_config,
            save_config,
            set_decorations
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
