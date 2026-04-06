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
                    <Route path=path!("/*any") view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ActiveTab {
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
                <div class="status-badge">
                    <span class="online-dot"></span>
                    {move || user_count.get()}
                    " online"
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

                    <FileUploadComponent path=path/>

                    <div class="card">
                        <div class="card-header">
                            <h2>"Download Files"</h2>
                        </div>
                        <div class="card-body">
                            <FileListComponent path=path set_path=set_path/>
                        </div>
                    </div>
                </div>

                <div class="panel panel-chat" class:active=move || active_tab.get() == ActiveTab::Chat>
                    <ChatComponent chat_version=chat_version/>
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
pub async fn get_file_list(
    path : String
) -> Result<Vec<(String, String, u64)>, ServerFnError> {
    let base_path = std::env::current_dir()
    .map_err(|e| format!("Error getting current directory: {:?}", e)).unwrap();
    //Check if path contains "..", if so, return an error
    if path.contains("..") {
        return Err(ServerFnError::ServerError("Path contains '..'".to_string()));
    }
    let path_to_read = base_path.join("files").join(path.clone());
    logging::log!("Listing directory: {:?}", path_to_read.clone());
    let files = std::fs::read_dir(path_to_read)
        .map_err(|e| format!("Error reading directory: {:?}", e)).unwrap();
    let file_entries : Vec<(String, String, u64)> = files
        .filter_map(|entry| {
            match entry {
                Ok(entry) => {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        Some(("d".to_string(), entry.file_name().into_string().unwrap(), 0))
                    } else {
                        //get the file size in bytes
                        let metadata = entry.metadata().unwrap();
                        let size = metadata.len();
                        Some(("f".to_string(), entry.file_name().into_string().unwrap(), size))
                    }
                },
                Err(e) => {
                    Some(("f".to_string(), format!("Error reading file entry: {:?}", e).to_string().into(), 0))
                },
            }
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

#[component]
pub fn FileUploadComponent(
    path: ReadSignal<String>,
) -> impl IntoView {
    view! {
        <div class="card upload-card">
            <div class="card-header">
                <h2>"Upload Files"</h2>
            </div>
            <div class="card-body">
                <form action="/upload" method="post" enctype="multipart/form-data">
                    <input type="hidden" name="upload_path" value={path.clone()}/>
                    <input type="file" multiple name="file"/>
                    <button class="btn-primary" type="submit">"Upload"</button>
                </form>
            </div>
        </div>
    }
}

#[component]
pub fn FileListComponent(
    path: ReadSignal<String>,
    set_path: WriteSignal<String>,
) -> impl IntoView {

    let directory_listing = Resource::new(move|| path.get(), get_file_list);

    view! {
        <div>
            <div class="current-dir">
                {move || {
                    let p = path.get();
                    if p.is_empty() { "/".to_string() } else { format!("/{}", p) }
                }}
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
        .map_err(|e| format!("Error getting current directory: {:?}", e)).unwrap();
    let chat_file_path = base_path.join("chat.json");

    if !chat_file_path.exists() {
        return Ok(Vec::new()); // Just return an empty list if no file exists
    }

    let chat_file = std::fs::read_to_string(chat_file_path.clone())
        .map_err(|e| format!("Error reading chat file: {:?}", e)).unwrap();

    let chat_messages : Vec<(String, String, u64)> = if !chat_file.is_empty() {
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
    let mut chat_name_clone = chat_name.clone();

    if chat_name_clone.clone().is_empty() {
        chat_name_clone = "Anonymous".to_string();
    }

    // Read chat.json, parse it, append the new chat message, and write it back to chat.json
    let base_path = std::env::current_dir()
    .map_err(|e| format!("Error getting current directory: {:?}", e)).unwrap();
    let chat_file_path = base_path.join("chat.json");

    // Read chat file to string, create it if it doesn't exist
    if !chat_file_path.exists() {
        let chat_file = std::fs::File::create(chat_file_path.clone())
            .map_err(|e| format!("Error creating chat file: {:?}", e)).unwrap();
        chat_file.sync_all()
            .map_err(|e| format!("Error syncing chat file: {:?}", e)).unwrap();
    }

    let chat_file = std::fs::read_to_string(chat_file_path.clone())
        .map_err(|e| format!("Error reading chat file: {:?}", e)).unwrap();

    let mut chat_messages : Vec<(String, String, u64)> = Vec::new();
    // If chat file is not empty, parse it
    if !chat_file.is_empty() {
        chat_messages = serde_json::from_str(&chat_file)
        .map_err(|e| format!("Error parsing chat file: {:?}", e)).unwrap();
    }

    // Append the new chat message
    if chat_message.len() > 0 && chat_message.len() < 1000 {
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        chat_messages.push((chat_name_clone, chat_message.clone(), timestamp));
        // Write the chat messages back to chat.json
        let chat_file = std::fs::File::create(chat_file_path)
            .map_err(|e| format!("Error creating chat file: {:?}", e)).unwrap();
        serde_json::to_writer(chat_file, &chat_messages)
            .map_err(|e| format!("Error writing chat file: {:?}", e)).unwrap();

        // Send signal to trigger SSE update
        _ = CHAT_CHANNEL.send(1);
    }
    println!("Chat messages: {:?}", chat_messages);
    Ok(())
}

#[component]
pub fn ChatComponent(
    chat_version: ReadSignal<u32>,
) -> impl IntoView {
    let chat_input_ref: NodeRef<Input> = NodeRef::new();
    let name_ref: NodeRef<Input> = NodeRef::new();
    let messages_ref: NodeRef<Div> = NodeRef::new();
    let send_chat_message = ServerAction::<SendChatMessage>::new();
    let (saved_name, set_saved_name) = signal(String::new());

    // Resource refetches when action completes or SSE chat update arrives
    let chat_messages_resource = Resource::new(
        move || (chat_version.get(), send_chat_message.version().get()),
        |_| get_chat_messages()
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

    // Auto-scroll chat to bottom when messages load or update
    Effect::new(move |_: Option<()>| {
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
                            children=move |(user, message, _)| {
                                view! {
                                    <div class="chat-message">
                                        <span class="chat-author">{user}</span>
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

#[cfg(feature = "ssr")]
pub mod ssr_imports {
    pub use once_cell::sync::OnceCell;
    pub use std::sync::atomic::{AtomicI32, Ordering};

    pub static CONNECTED_USERS: AtomicI32 = AtomicI32::new(0);

    lazy_static::lazy_static! {
        pub static ref CHAT_CHANNEL: tokio::sync::broadcast::Sender<i32> = {
            let (tx, _rx) = tokio::sync::broadcast::channel(16);
            tx
        };
        pub static ref USERS_CHANNEL: tokio::sync::broadcast::Sender<i32> = {
            let (tx, _rx) = tokio::sync::broadcast::channel(16);
            tx
        };
    }

    static LOG_INIT: OnceCell<()> = OnceCell::new();

    pub fn init_logging() {
        LOG_INIT.get_or_init(|| {
            simple_logger::SimpleLogger::new().env().init().unwrap();
        });
    }
}
