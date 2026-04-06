use fmtsize::{Conventional, FmtSize};
use leptos::{html::{Input, Div}, *};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::config::LeptosOptions;
use leptos_meta::*;
use leptos_router::*;
use leptos_router::components::{Router, Route, Routes};
#[cfg(feature = "ssr")]
use ammonia::clean;

/// Helper to create a ServerFnError with the default NoCustomError type parameter.
#[cfg(feature = "ssr")]
fn sfn_err(msg: impl Into<String>) -> ServerFnError {
    let s: String = msg.into();
    ServerFnError::ServerError(s)
}

fn encode_uri_component(s: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut result = String::with_capacity(s.len() * 3);
    for &byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push('%');
                result.push(HEX[(byte >> 4) as usize] as char);
                result.push(HEX[(byte & 0x0f) as usize] as char);
            }
        }
    }
    result
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn format_uptime(seconds: u64) -> String {
    let d = seconds / 86400;
    let h = (seconds % 86400) / 3600;
    let m = (seconds % 3600) / 60;
    if d > 0 {
        format!("{}d {}h {}m", d, h, m)
    } else if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}

fn is_image_file(name: &str) -> bool {
    let l = name.to_lowercase();
    l.ends_with(".jpg")
        || l.ends_with(".jpeg")
        || l.ends_with(".png")
        || l.ends_with(".gif")
        || l.ends_with(".webp")
        || l.ends_with(".svg")
        || l.ends_with(".bmp")
}

#[cfg(not(feature = "ssr"))]
mod notification {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        type AudioContext;
        #[wasm_bindgen(catch, constructor)]
        fn new() -> Result<AudioContext, JsValue>;
        #[wasm_bindgen(method, js_name = "createOscillator")]
        fn create_oscillator(this: &AudioContext) -> OscillatorNode;
        #[wasm_bindgen(method, js_name = "createGain")]
        fn create_gain(this: &AudioContext) -> GainNode;
        #[wasm_bindgen(method, getter)]
        fn destination(this: &AudioContext) -> JsValue;
        #[wasm_bindgen(method, getter, js_name = "currentTime")]
        fn current_time(this: &AudioContext) -> f64;

        type OscillatorNode;
        #[wasm_bindgen(method)]
        fn connect(this: &OscillatorNode, dest: &JsValue) -> JsValue;
        #[wasm_bindgen(method)]
        fn start(this: &OscillatorNode);
        #[wasm_bindgen(method)]
        fn stop(this: &OscillatorNode, when: f64);
        #[wasm_bindgen(method, getter)]
        fn frequency(this: &OscillatorNode) -> AudioParam;
        #[wasm_bindgen(method, setter, js_name = "type")]
        fn set_type(this: &OscillatorNode, value: &str);

        type GainNode;
        #[wasm_bindgen(method)]
        fn connect(this: &GainNode, dest: &JsValue) -> JsValue;
        #[wasm_bindgen(method, getter)]
        fn gain(this: &GainNode) -> AudioParam;

        type AudioParam;
        #[wasm_bindgen(method, setter)]
        fn set_value(this: &AudioParam, val: f32);
    }

    pub fn play() {
        let Some(ctx) = AudioContext::new().ok() else {
            return;
        };
        let osc = ctx.create_oscillator();
        let gain_node = ctx.create_gain();
        osc.set_type("sine");
        osc.frequency().set_value(880.0);
        gain_node.gain().set_value(0.3);
        osc.connect(&gain_node);
        gain_node.connect(&ctx.destination());
        let t = ctx.current_time();
        osc.start();
        osc.stop(t + 0.12);
    }
}

#[cfg(not(feature = "ssr"))]
mod random_name {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = Math)]
        fn random() -> f64;
    }

    const ADJECTIVES: &[&str] = &[
        "Happy", "Sleepy", "Clever", "Swift", "Gentle", "Brave", "Calm",
        "Bright", "Bold", "Fuzzy", "Jolly", "Lucky", "Mighty", "Noble",
        "Perky", "Quiet", "Silly", "Witty", "Zany", "Cozy",
        "Fluffy", "Sneaky", "Wobbly", "Sassy", "Dapper", "Grumpy", "Zippy",
        "Peppy", "Cheeky", "Mellow", "Frisky", "Giddy", "Lively", "Nimble",
        "Plucky", "Rowdy", "Sparkly", "Wacky", "Bouncy", "Cuddly",
    ];

    const ANIMALS: &[&str] = &[
        "Panda", "Fox", "Owl", "Cat", "Dog", "Bear", "Wolf",
        "Hawk", "Deer", "Otter", "Bunny", "Tiger", "Eagle", "Dolphin",
        "Koala", "Lynx", "Moose", "Parrot", "Raccoon", "Sloth",
        "Badger", "Beaver", "Bison", "Camel", "Cobra", "Crane", "Crow",
        "Ferret", "Gecko", "Goose", "Hippo", "Hyena", "Ibis", "Iguana",
        "Jackal", "Jaguar", "Lemur", "Llama", "Lobster", "Macaw",
        "Mantis", "Meerkat", "Monkey", "Newt", "Ocelot", "Octopus",
        "Osprey", "Panther", "Pelican", "Penguin", "Pigeon", "Puma",
        "Quail", "Raven", "Salmon", "Seal", "Shark", "Snail",
        "Sparrow", "Squid", "Stork", "Swan", "Tapir", "Toucan",
        "Turkey", "Turtle", "Viper", "Vulture", "Walrus", "Weasel",
        "Whale", "Wombat", "Yak", "Zebra", "Alpaca", "Axolotl",
        "Chinchilla", "Coyote", "Donkey", "Falcon", "Flamingo", "Gibbon",
        "Gopher", "Hedgehog", "Heron", "Hornet", "Kiwi", "Leopard",
        "Manatee", "Narwhal", "Opossum", "Oriole", "Peacock", "Porcupine",
        "Rooster", "Seahorse", "Starling", "Stingray", "Warthog", "Woodpecker",
    ];

    pub fn generate() -> String {
        let adj_idx = (random() * ADJECTIVES.len() as f64) as usize;
        let animal_idx = (random() * ANIMALS.len() as f64) as usize;
        format!(
            "{} {}",
            ADJECTIVES[adj_idx.min(ADJECTIVES.len() - 1)],
            ANIMALS[animal_idx.min(ANIMALS.len() - 1)]
        )
    }
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StatsData {
    pub total_connections: u64,
    pub total_uploads: u64,
    pub total_upload_bytes: u64,
    pub total_downloads: u64,
    pub total_download_bytes: u64,
    pub total_chat_messages: u64,
    pub top_downloads: Vec<(String, u64)>,
    pub uptime_seconds: u64,
}

#[cfg(feature = "ssr")]
fn resolve_safe_path(
    base: &std::path::Path,
    user_path: &str,
) -> Result<std::path::PathBuf, ServerFnError> {
    let target = base.join(user_path);
    let canonical = if target.exists() {
        target.canonicalize()
    } else if let Some(parent) = target.parent() {
        if parent.exists() {
            parent
                .canonicalize()
                .map(|p| p.join(target.file_name().unwrap_or_default()))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Parent not found",
            ))
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Invalid path",
        ))
    }
    .map_err(|e| sfn_err(format!("Path error: {}", e)))?;

    let canonical_base = base
        .canonicalize()
        .map_err(|e| sfn_err(format!("Base path error: {}", e)))?;

    if !canonical.starts_with(&canonical_base) {
        return Err(sfn_err("Access denied".to_string()));
    }
    Ok(canonical)
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/shareboxx.css"/>
        <Title text="ShareBoxx"/>
        <Router>
            <main>
                <Routes fallback=HomePage>
                    <Route path=path!("") view=HomePage/>
                    <Route path=path!("stats") view=StatsPage/>
                    <Route path=path!("/*any") view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ActiveTab {
    Files,
    Chat,
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let (path, set_path) = signal("".to_string());
    let (active_tab, set_active_tab) = signal(ActiveTab::Files);
    let (user_count, set_user_count) = signal(0u32);
    let (chat_version, set_chat_version) = signal(0u32);
    let (has_unread, set_has_unread) = signal(false);
    let (file_list_version, set_file_list_version) = signal(0u32);

    // SSE connection for live chat updates and user count
    #[cfg(not(feature = "ssr"))]
    {
        use futures::StreamExt;
        spawn_local(async move {
            let mut source = gloo_net::eventsource::futures::EventSource::new("/ws")
                .expect("couldn't connect to SSE stream");

            let mut chat_stream = source.subscribe("chat").unwrap();
            let mut users_stream = source.subscribe("users").unwrap();

            // Handle chat updates in a separate task
            spawn_local(async move {
                while let Some(_) = chat_stream.next().await {
                    set_chat_version.update(|v| *v += 1);
                    if active_tab.get_untracked() != ActiveTab::Chat {
                        set_has_unread.set(true);
                        notification::play();
                    }
                }
            });

            // Handle user count updates (keeps EventSource alive)
            while let Some(Ok((_event_type, msg))) = users_stream.next().await {
                if let Some(data_str) = msg.data().as_string() {
                    if let Ok(count) = data_str.parse::<u32>() {
                        set_user_count.set(count);
                    }
                }
            }
        });
    }

    view! {
        <div class="app">
            <header class="app-header">
                <h1 class="logo">"ShareBoxx"</h1>
                <div class="header-right">
                    <div class="status-badge">
                        <span class="online-dot"></span>
                        {move || user_count.get()}
                        " online"
                    </div>
                    <a href="/stats" rel="external" class="header-link">"Stats"</a>
                </div>
            </header>

            <nav class="tab-bar">
                <button
                    class="tab-btn"
                    class:active=move || active_tab.get() == ActiveTab::Files
                    on:click=move |_| set_active_tab.set(ActiveTab::Files)
                >
                    "Files"
                </button>
                <button
                    class="tab-btn"
                    class:active=move || active_tab.get() == ActiveTab::Chat
                    on:click=move |_| {
                        set_active_tab.set(ActiveTab::Chat);
                        set_has_unread.set(false);
                    }
                >
                    "Chat"
                    <Show when=move || has_unread.get() fallback=|| ()>
                        <span class="unread-dot"></span>
                    </Show>
                </button>
            </nav>

            <div class="app-content">
                <div class="panel panel-files" class:active=move || active_tab.get() == ActiveTab::Files>
                    <div class="card welcome-card">
                        <div class="card-body">
                            <h2>"Welcome to ShareBoxx"</h2>
                            <p>"A free offline file sharing service. Upload files and share them with anyone on this network."</p>
                            <p class="text-muted">"This is a local, anonymous service with no internet connection and no accounts. Note that executables are not checked for malware, so be careful what you download."</p>
                        </div>
                    </div>

                    <FileUploadComponent path=path set_file_list_version=set_file_list_version file_list_version=file_list_version/>

                    <div class="card">
                        <div class="card-header">
                            <h2>"Download Files"</h2>
                        </div>
                        <div class="card-body">
                            <FileListComponent path=path set_path=set_path file_list_version=file_list_version set_file_list_version=set_file_list_version/>
                        </div>
                    </div>
                </div>

                <div class="panel panel-chat" class:active=move || active_tab.get() == ActiveTab::Chat>
                    <ChatComponent chat_version=chat_version active_tab=active_tab/>
                </div>
            </div>
        </div>
    }.into_any()
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}

#[server(GetFileList)]
pub async fn get_file_list(path: String) -> Result<Vec<(String, String, u64)>, ServerFnError> {
    let base_path = std::env::current_dir()
        .map_err(|e| sfn_err(format!("Error getting current directory: {:?}", e)))?;
    let base = base_path.join("files");

    let safe_path = resolve_safe_path(&base, &path)?;
    logging::log!("Listing directory: {:?}", safe_path);

    let files = std::fs::read_dir(&safe_path)
        .map_err(|e| sfn_err(format!("Error reading directory: {:?}", e)))?;

    let file_entries: Vec<(String, String, u64)> = files
        .filter_map(|entry| match entry {
            Ok(entry) => {
                let name = entry.file_name().into_string().ok()?;
                if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    Some(("d".to_string(), name, 0))
                } else {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    Some(("f".to_string(), name, size))
                }
            }
            Err(_) => None,
        })
        .collect();

    // Sort file_entries by name, with directories first, then files.
    let mut file_entries = file_entries;
    file_entries.sort_by(|a, b| {
        if a.0 == "d" && b.0 == "f" {
            std::cmp::Ordering::Less
        } else if a.0 == "f" && b.0 == "d" {
            std::cmp::Ordering::Greater
        } else {
            a.1.to_lowercase().cmp(&b.1.to_lowercase())
        }
    });

    // If path is not empty, prepend ".." to the list of files
    if !path.is_empty() {
        let mut new_files = Vec::new();
        new_files.push(("d".to_string(), "..".to_string(), 0));
        new_files.extend(file_entries);
        return Ok(new_files);
    }

    Ok(file_entries)
}

#[server]
pub async fn get_disk_space() -> Result<(u64, u64), ServerFnError> {
    let output = std::process::Command::new("df")
        .args(["--output=size,avail", "-B1", "./files"])
        .output()
        .map_err(|e| sfn_err(format!("df failed: {}", e)))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout
        .lines()
        .nth(1)
        .ok_or_else(|| sfn_err("Failed to parse df".to_string()))?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(sfn_err(
            "Invalid df output".to_string(),
        ));
    }
    let total: u64 = parts[0].parse().unwrap_or(0);
    let avail: u64 = parts[1].parse().unwrap_or(0);
    Ok((total.saturating_sub(avail), total))
}

#[server]
pub async fn create_directory(path: String, name: String) -> Result<(), ServerFnError> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name == ".." || name == "."
    {
        return Err(sfn_err(
            "Invalid directory name".to_string(),
        ));
    }
    let base = std::env::current_dir()
        .map_err(|e| sfn_err(e.to_string()))?
        .join("files");
    let combined = format!("{}{}", path, name);
    resolve_safe_path(&base, &combined)?;
    let target = base.join(&path).join(&name);
    std::fs::create_dir_all(&target)
        .map_err(|e| sfn_err(format!("Failed: {}", e)))?;
    Ok(())
}

#[server]
pub async fn get_stats() -> Result<StatsData, ServerFnError> {
    use ssr_imports::*;
    let stats = STATS
        .read()
        .map_err(|e| sfn_err(e.to_string()))?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let mut top: Vec<(String, u64)> = stats
        .file_downloads
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    top.sort_by(|a, b| b.1.cmp(&a.1));
    top.truncate(10);
    Ok(StatsData {
        total_connections: stats.total_connections,
        total_uploads: stats.total_uploads,
        total_upload_bytes: stats.total_upload_bytes,
        total_downloads: stats.total_downloads,
        total_download_bytes: stats.total_download_bytes,
        total_chat_messages: stats.total_chat_messages,
        top_downloads: top,
        uptime_seconds: now.saturating_sub(stats.started_at),
    })
}

#[component]
pub fn FileUploadComponent(
    path: ReadSignal<String>,
    set_file_list_version: WriteSignal<u32>,
    file_list_version: ReadSignal<u32>,
) -> impl IntoView {
    let file_input_ref: NodeRef<Input> = NodeRef::new();
    let (has_file, set_has_file) = signal(false);
    let (progress, set_progress) = signal(-1.0f64); // -1 = idle, 0..1 = uploading
    let (upload_status, set_upload_status) = signal(String::new());

    let disk_space = Resource::new(
        move || file_list_version.get(),
        |_| get_disk_space(),
    );

    let on_upload_click = move |_| {
        #[cfg(not(feature = "ssr"))]
        {
            use wasm_bindgen::prelude::*;
            use wasm_bindgen::JsCast;

            let Some(input) = file_input_ref.get() else { return };
            let html_input: &web_sys::HtmlInputElement = input.unchecked_ref();
            let Some(files) = html_input.files() else { return };
            if files.length() == 0 { return; }

            let form_data = web_sys::FormData::new().unwrap();
            form_data.append_with_str("upload_path", &path.get_untracked()).unwrap();
            for i in 0..files.length() {
                let file = files.get(i).unwrap();
                form_data.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
            }

            let xhr = web_sys::XmlHttpRequest::new().unwrap();
            xhr.open("POST", "/upload").unwrap();

            // Progress handler
            {
                let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::ProgressEvent| {
                    if event.length_computable() && event.total() > 0.0 {
                        set_progress.set(event.loaded() / event.total());
                    }
                });
                xhr.upload().unwrap().set_onprogress(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }

            // Completion handler
            {
                let xhr2 = xhr.clone();
                let input_ref = file_input_ref;
                let closure = Closure::<dyn FnMut()>::new(move || {
                    if xhr2.ready_state() == 4 {
                        if xhr2.status().unwrap_or(0) == 200 {
                            set_upload_status.set("done".to_string());
                            set_file_list_version.update(|v| *v += 1);
                            if let Some(el) = input_ref.get() {
                                el.set_value("");
                            }
                            set_has_file.set(false);
                        } else {
                            set_upload_status.set("error".to_string());
                        }
                        set_progress.set(-1.0);
                    }
                });
                xhr.set_onreadystatechange(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }

            set_upload_status.set(String::new());
            set_progress.set(0.0);
            xhr.send_with_opt_form_data(Some(&form_data)).unwrap();
        }
    };

    view! {
        <div class="card upload-card">
            <div class="card-header">
                <h2>"Upload Files"</h2>
                <Suspense fallback=|| ()>
                    {move || disk_space.get().and_then(|r| r.ok()).map(|(used, total)| {
                        let pct = if total > 0 { (used as f64 / total as f64 * 100.0) as u32 } else { 0 };
                        view! {
                            <div class="disk-space">
                                <div class="disk-space-bar">
                                    <div class="disk-space-fill" style=format!("width: {}%", pct)></div>
                                </div>
                                <span class="disk-space-text">{format_bytes(used)} " / " {format_bytes(total)}</span>
                            </div>
                        }
                    })}
                </Suspense>
            </div>
            <div class="card-body">
                <div class="upload-controls">
                    <input type="file" multiple node_ref=file_input_ref
                        on:change=move |_| {
                            if let Some(input) = file_input_ref.get() {
                                set_has_file.set(!input.value().is_empty());
                                set_upload_status.set(String::new());
                            }
                        }
                    />
                    <button class="btn-primary" type="button"
                        disabled=move || !has_file.get() || (progress.get() >= 0.0)
                        on:click=on_upload_click
                    >
                        {move || if progress.get() >= 0.0 { "Uploading..." } else { "Upload" }}
                    </button>
                </div>
                <Show when={move || progress.get() >= 0.0} fallback=|| ()>
                    <div class="upload-progress">
                        <div class="progress-bar">
                            <div class="progress-fill" style=move || format!("width: {}%", (progress.get() * 100.0) as u32)></div>
                        </div>
                        <span class="progress-text">{move || format!("{}%", (progress.get() * 100.0) as u32)}</span>
                    </div>
                </Show>
                <Show when=move || upload_status.get() == "done" fallback=|| ()>
                    <div class="upload-success">"Upload complete!"</div>
                </Show>
                <Show when=move || upload_status.get() == "error" fallback=|| ()>
                    <div class="upload-error">"Upload failed. Please try again."</div>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn FileListComponent(
    path: ReadSignal<String>,
    set_path: WriteSignal<String>,
    file_list_version: ReadSignal<u32>,
    set_file_list_version: WriteSignal<u32>,
) -> impl IntoView {
    let folder_input_ref: NodeRef<Input> = NodeRef::new();
    let (folder_name, set_folder_name) = signal(String::new());

    let directory_listing = Resource::new(
        move || (path.get(), file_list_version.get()),
        |(p, _)| get_file_list(p),
    );

    view! {
        <div>
            <div class="current-dir">
                {move || {
                    let p = path.get();
                    if p.is_empty() { "/".to_string() } else { format!("/{}", p) }
                }}
            </div>

            <div class="new-folder-row">
                <input type="text" class="new-folder-input" placeholder="New folder name..."
                    node_ref=folder_input_ref
                    on:input=move |_| {
                        if let Some(input) = folder_input_ref.get() {
                            set_folder_name.set(input.value());
                        }
                    }
                />
                <button class="btn-primary" type="button"
                    disabled=move || folder_name.get().is_empty()
                    on:click=move |_| {
                        let name = folder_name.get_untracked();
                        let p = path.get_untracked();
                        spawn_local(async move {
                            if create_directory(p, name).await.is_ok() {
                                set_file_list_version.update(|v| *v += 1);
                            }
                        });
                        if let Some(input) = folder_input_ref.get() {
                            input.set_value("");
                        }
                        set_folder_name.set(String::new());
                    }
                >"Create"</button>
            </div>

            <Suspense fallback=|| view! { <p class="loading">"Loading..."</p> }>
                <Show
                    when=move || directory_listing.get()
                        .map(|res| res.is_ok())
                        .unwrap_or(false)
                    fallback=move || {
                        directory_listing.get()
                            .and_then(|r| r.err())
                            .map(|e| view! { <p>"ERROR: " {e.to_string()}</p> })
                    }
                >
                    <div class="file-list">
                        <For
                            each=move || directory_listing.get()
                                .and_then(|r| r.ok())
                                .unwrap_or_default()
                            key=|file| file.1.clone()
                            children=move |n| {
                                let (file_type, file_name, file_size) = n;
                                let link_target = if file_type == "f" {
                                    let p = path.get_untracked();
                                    let mut encoded = String::from("/files/");
                                    for seg in p.split('/') {
                                        if !seg.is_empty() {
                                            encoded.push_str(&encode_uri_component(seg));
                                            encoded.push('/');
                                        }
                                    }
                                    encoded.push_str(&encode_uri_component(&file_name));
                                    encoded
                                } else { "#".to_string() };

                                let preview_link = link_target.clone();
                                let is_img = is_image_file(&file_name) && file_type == "f";

                                view! {
                                    <a
                                        href=link_target
                                        rel="external"
                                        class="file-item"
                                        on:click=move |ev| {
                                            if file_name == ".." {
                                                ev.prevent_default();
                                                let current_path = path.get();
                                                let mut path_parts: Vec<&str> = current_path.trim_end_matches('/').split('/').collect();
                                                path_parts.pop();
                                                let new_path = path_parts.join("/");
                                                set_path.set(if new_path.is_empty() { "".to_string() } else { format!("{}/", new_path) });
                                            } else if file_type == "d" {
                                                ev.prevent_default();
                                                set_path.update(|p| {
                                                    p.push_str(&file_name);
                                                    p.push('/');
                                                });
                                            }
                                        }
                                    >
                                        <img
                                            src={if file_type == "d" { "/assets/folder.png" } else { "/assets/file.png" }}
                                            class="file-icon"
                                        />
                                        <span class="file-name">
                                            {if file_type == "d" { format!("{}/", file_name) } else { file_name.clone() }}
                                        </span>
                                        {if is_img {
                                            Some(view! { <img src=preview_link class="file-preview"/> })
                                        } else {
                                            None
                                        }}
                                        <span class="file-size">
                                            {if file_type == "f" { file_size.fmt_size(Conventional).to_string() } else { "".to_string() }}
                                        </span>
                                    </a>
                                }.into_any()
                            }
                        />
                    </div>
                </Show>
            </Suspense>
        </div>
    }.into_any()
}

#[server]
pub async fn get_chat_messages() -> Result<Vec<(String, String, u64)>, ServerFnError> {
    let base_path = std::env::current_dir()
        .map_err(|e| sfn_err(format!("Error getting current directory: {:?}", e)))?;
    let chat_file_path = base_path.join("chat.json");

    if !chat_file_path.exists() {
        return Ok(Vec::new());
    }

    let chat_file = std::fs::read_to_string(&chat_file_path)
        .map_err(|e| sfn_err(format!("Error reading chat file: {:?}", e)))?;

    let chat_messages: Vec<(String, String, u64)> = if !chat_file.is_empty() {
        serde_json::from_str(&chat_file).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Only return the last 50 chat messages
    let chat_messages = chat_messages.iter().rev().take(50).cloned().rev().collect();

    Ok(chat_messages)
}

#[server(SendChatMessage, "/api")]
pub async fn send_chat_message(
    chat_name: String,
    chat_message: String,
) -> Result<(), ServerFnError> {
    use crate::app::ssr_imports::*;

    logging::log!("Chat message received: {:?}", chat_message);
    let chat_name_clean = if chat_name.is_empty() {
        "Anonymous".to_string()
    } else {
        chat_name
    };

    let base_path = std::env::current_dir()
        .map_err(|e| sfn_err(format!("Error getting current directory: {:?}", e)))?;
    let chat_file_path = base_path.join("chat.json");
    let chat_tmp_path = base_path.join("chat.json.tmp");

    // Read existing messages or start fresh
    let mut chat_messages: Vec<(String, String, u64)> = if chat_file_path.exists() {
        let chat_file = std::fs::read_to_string(&chat_file_path)
            .map_err(|e| sfn_err(format!("Error reading chat file: {:?}", e)))?;
        if !chat_file.is_empty() {
            serde_json::from_str(&chat_file).unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Append the new chat message
    if !chat_message.is_empty() && chat_message.len() < 1000 {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        chat_messages.push((chat_name_clean, chat_message.clone(), timestamp));

        // Atomic write: write to tmp then rename
        let data = serde_json::to_string(&chat_messages)
            .map_err(|e| sfn_err(format!("Error serializing chat: {:?}", e)))?;
        std::fs::write(&chat_tmp_path, &data)
            .map_err(|e| sfn_err(format!("Error writing chat tmp: {:?}", e)))?;
        std::fs::rename(&chat_tmp_path, &chat_file_path)
            .map_err(|e| sfn_err(format!("Error renaming chat file: {:?}", e)))?;

        // Track stats
        if let Ok(mut stats) = STATS.write() {
            stats.total_chat_messages += 1;
        }
        save_stats();

        // Send signal to trigger SSE update
        _ = CHAT_CHANNEL.send(1);
    }
    Ok(())
}

#[component]
pub fn ChatComponent(
    chat_version: ReadSignal<u32>,
    active_tab: ReadSignal<ActiveTab>,
) -> impl IntoView {
    let chat_input_ref: NodeRef<Input> = NodeRef::new();
    let name_ref: NodeRef<Input> = NodeRef::new();
    let messages_ref: NodeRef<Div> = NodeRef::new();
    let send_chat_message = ServerAction::<SendChatMessage>::new();
    let (saved_name, set_saved_name) = signal(String::new());

    // Resource refetches when action completes or SSE chat update arrives
    let chat_messages_resource = Resource::new(
        move || (chat_version.get(), send_chat_message.version().get()),
        |_| get_chat_messages(),
    );

    // After action completes, clear message input and restore name
    Effect::new(move |prev: Option<usize>| {
        let v = send_chat_message.version().get();
        if prev.is_some() && v > 0 {
            if let Some(input) = chat_input_ref.get() {
                input.set_value("");
            }
            // Restore name in case ActionForm reset the form
            if let Some(input) = name_ref.get() {
                input.set_value(&saved_name.get_untracked());
            }
        }
        v
    });

    // Auto-scroll chat to bottom when messages update or tab switches to chat
    Effect::new(move |_: Option<()>| {
        let _tab = active_tab.get(); // Track tab changes so switching to Chat triggers scroll
        if let Some(Ok(_)) = chat_messages_resource.get() {
            #[cfg(not(feature = "ssr"))]
            {
                let div_ref = messages_ref;
                spawn_local(async move {
                    if let Some(div) = div_ref.get() {
                        div.set_scroll_top(div.scroll_height());
                    }
                });
            }
        }
    });

    // Assign a random name on first load (client only)
    #[cfg(not(feature = "ssr"))]
    {
        Effect::new(move |prev: Option<bool>| {
            if prev.unwrap_or(false) {
                return true;
            }
            if let Some(input) = name_ref.get() {
                let name = random_name::generate();
                input.set_value(&name);
                set_saved_name.set(name);
                return true;
            }
            false
        });
    }

    view! {
        <div class="card chat-card">
            <div class="card-header">
                <h2>"Chat"</h2>
            </div>
            <div class="chat-messages" node_ref=messages_ref>
                <Suspense fallback=|| view! { <p class="loading">"Loading..."</p> }>
                    <Show
                        when=move || chat_messages_resource.get()
                            .map(|res| res.is_ok())
                            .unwrap_or(false)
                        fallback=|| view! { <p class="chat-empty">"No messages yet. Start the conversation!"</p> }
                    >
                        <For
                            each=move || chat_messages_resource
                                .get()
                                .and_then(|res| res.ok())
                                .unwrap_or_default()
                            key=|msg| msg.2
                            children=move |(user, message, timestamp)| {
                                let time_str = {
                                    #[cfg(not(feature = "ssr"))]
                                    {
                                        let now = (js_sys::Date::now() / 1000.0) as u64;
                                        let diff = now.saturating_sub(timestamp);
                                        if diff < 60 {
                                            "just now".to_string()
                                        } else if diff < 3600 {
                                            format!("{}m ago", diff / 60)
                                        } else if diff < 86400 {
                                            format!("{}h ago", diff / 3600)
                                        } else {
                                            format!("{}d ago", diff / 86400)
                                        }
                                    }
                                    #[cfg(feature = "ssr")]
                                    {
                                        String::new()
                                    }
                                };
                                view! {
                                    <div class="chat-message">
                                        <div class="chat-message-header">
                                            <span class="chat-author">{user}</span>
                                            <span class="chat-time">{time_str}</span>
                                        </div>
                                        <span class="chat-text">{message}</span>
                                    </div>
                                }
                            }
                        />
                    </Show>
                </Suspense>
            </div>
            <div class="chat-input-form">
                <ActionForm action=send_chat_message>
                    <div class="chat-input-group">
                        <input type="text" class="name-input" placeholder="Name" name="chat_name"
                            node_ref=name_ref
                            on:input=move |_| {
                                if let Some(input) = name_ref.get() {
                                    set_saved_name.set(input.value());
                                }
                            }
                        />
                        <input
                            type="text"
                            placeholder="Type a message..."
                            name="chat_message"
                            node_ref=chat_input_ref
                        />
                        <button class="btn-send" type="submit">"Send"</button>
                    </div>
                </ActionForm>
            </div>
        </div>
    }.into_any()
}

#[component]
fn StatsPage() -> impl IntoView {
    let stats = Resource::new(|| (), |_| get_stats());

    view! {
        <div class="app">
            <header class="app-header">
                <a href="/" rel="external" class="logo">"ShareBoxx"</a>
                <a href="/stats" rel="external" class="header-link">"Stats"</a>
            </header>
            <div class="stats-page">
                <h2 class="stats-title">"Server Statistics"</h2>
                <Suspense fallback=|| view! { <p class="loading">"Loading..."</p> }>
                    {move || stats.get().and_then(|r| r.ok()).map(|s| {
                        let top_downloads = s.top_downloads.clone();
                        let has_downloads = !top_downloads.is_empty();
                        view! {
                            <div class="stats-grid">
                                <div class="stat-card">
                                    <div class="stat-value">{s.total_connections}</div>
                                    <div class="stat-label">"Connections"</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-value">{s.total_uploads}</div>
                                    <div class="stat-label">"Uploads"</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-value">{format_bytes(s.total_upload_bytes)}</div>
                                    <div class="stat-label">"Data Uploaded"</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-value">{s.total_downloads}</div>
                                    <div class="stat-label">"Downloads"</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-value">{format_bytes(s.total_download_bytes)}</div>
                                    <div class="stat-label">"Data Downloaded"</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-value">{s.total_chat_messages}</div>
                                    <div class="stat-label">"Chat Messages"</div>
                                </div>
                                <div class="stat-card">
                                    <div class="stat-value">{format_uptime(s.uptime_seconds)}</div>
                                    <div class="stat-label">"Uptime"</div>
                                </div>
                            </div>
                            <div class="card">
                                <div class="card-header"><h2>"Top Downloads"</h2></div>
                                <div class="card-body">
                                    {if has_downloads {
                                        Some(view! {
                                            <table class="top-downloads-table">
                                                <thead><tr><th>"File"</th><th>"Downloads"</th></tr></thead>
                                                <tbody>
                                                    {top_downloads.iter().map(|(name, count)| view! {
                                                        <tr><td>{name.clone()}</td><td>{*count}</td></tr>
                                                    }).collect::<Vec<_>>()}
                                                </tbody>
                                            </table>
                                        })
                                    } else {
                                        None
                                    }}
                                    {if !has_downloads {
                                        Some(view! { <p class="text-muted">"No downloads yet."</p> })
                                    } else {
                                        None
                                    }}
                                </div>
                            </div>
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }.into_any()
}

#[cfg(feature = "ssr")]
pub mod ssr_imports {
    pub use once_cell::sync::OnceCell;
    pub use std::collections::HashMap;
    pub use std::sync::atomic::{AtomicI32, Ordering};
    pub use std::sync::{Arc, RwLock};

    pub static CONNECTED_USERS: AtomicI32 = AtomicI32::new(0);

    #[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
    pub struct Stats {
        pub total_connections: u64,
        pub total_uploads: u64,
        pub total_upload_bytes: u64,
        pub total_downloads: u64,
        pub total_download_bytes: u64,
        pub total_chat_messages: u64,
        pub file_downloads: HashMap<String, u64>,
        pub started_at: u64,
    }

    lazy_static::lazy_static! {
        pub static ref CHAT_CHANNEL: tokio::sync::broadcast::Sender<i32> = {
            let (tx, _rx) = tokio::sync::broadcast::channel(16);
            tx
        };
        pub static ref USERS_CHANNEL: tokio::sync::broadcast::Sender<i32> = {
            let (tx, _rx) = tokio::sync::broadcast::channel(16);
            tx
        };
        pub static ref STATS: Arc<RwLock<Stats>> = {
            let stats = load_stats().unwrap_or_else(|| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                Stats {
                    started_at: now,
                    ..Default::default()
                }
            });
            Arc::new(RwLock::new(stats))
        };
    }

    fn load_stats() -> Option<Stats> {
        let data = std::fs::read_to_string("stats.json").ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save_stats() {
        if let Ok(stats) = STATS.read() {
            if let Ok(data) = serde_json::to_string_pretty(&*stats) {
                let _ = std::fs::write("stats.json.tmp", &data)
                    .and_then(|_| std::fs::rename("stats.json.tmp", "stats.json"));
            }
        }
    }

    static LOG_INIT: OnceCell<()> = OnceCell::new();

    pub fn init_logging() {
        LOG_INIT.get_or_init(|| {
            simple_logger::SimpleLogger::new().env().init().unwrap();
        });
    }
}
