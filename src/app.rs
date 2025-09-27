use fmtsize::{Conventional, FmtSize};
use leptos::{html::Input, *};
use leptos::prelude::*;
use leptos::ev::SubmitEvent;
use leptos_meta::*;
use leptos_router::*;
use leptos_reactive::{
    create_local_resource, spawn_local, SignalGet
};
use leptos_router::components::{Router, Route, Routes};
#[cfg(feature = "ssr")]
use ammonia::clean;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html attr:data-bs-theme="dark"/>
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/shareboxx.css"/>

        // sets the document title
        <Title text="Welcome to ShareBoxx"/>

        // content for this welcome page
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

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let (path, set_path) = signal("".to_string());
    view! {
        <div class="container-fluid">
            <div class="row">

                <div class="col text-left">
                    <div class="container-fluid">
                        <div class="row">
                            <div class="col text-left">
                                <div class="card">
                                    <h2 class="card-header">Welcome to Shareboxx</h2>
                                    <div class="card-body">
                                        <p class="card-text">Shareboxx is a free offline fire sharing service. You can upload files and share them with others. <br/>
                                        This is an local, anonymous service with no internet connection and no accounts. Note that executables are not checked for malware, so be careful what you download.</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="row">
                            <div class="col text-left">
                                <p/>
                                <FileUploadComponent path=path/>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="col text-left">
                    <ChatComponent/>
                </div>
            </div>
        </div>
        <p/>
        <div class="card">
            <h2 class="card-header">Download Files</h2>
            <div class="card-body">
                <FileListComponent path=path set_path=set_path/>
            </div>
        </div>
        <br/>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
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
        return Err(leptos::ServerFnError::ServerError("Path contains '..'".to_string()));
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
        <div class="card">
            <h2 class="card-header">Upload Files</h2>
            <div class="card-body">
                <form action="/upload" method="post" enctype="multipart/form-data">
                    <input type="hidden" name="upload_path" value={path.clone()}/>
                    <input type="file" multiple name="file"/>
                    <button type="submit">Submit</button>
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
    let directory_listing = create_local_resource(
        move || path.get(),
        get_file_list, // every time `count` changes, this will run
    );

    // Create a derived memo that only contains a value when the resource has loaded successfully.
    let files = Memo::new(move |_| directory_listing.get().and_then(|res| res.ok()));
    // Create another memo that only contains a value when the resource has an error.
    let error = Memo::new(move |_| directory_listing.get().and_then(|res| res.err()));
    // --- FIX END ---

    view! {
        <div>
            "Current Directory: " {path}
            <p/>

            // Use <Show> to display the loading state.
            <Show when=move || directory_listing.loading().get() fallback=|| ()>
                <p>"Loading..."</p>
            </Show>

            // Use <Show> to display an error if one exists.
            <Show
                when=move || error.get().is_some()
                fallback=|| ()
            >
                <p>"ERROR: " {error.get().unwrap().to_string()}</p>
            </Show>

            // Use <Show> to display the file list when it's successfully loaded.
            <Show
                when=move || files.get().is_some()
                fallback=|| ()
            >
                <div class="list-group">
                    <For
                        each=move || files.get().unwrap_or_default()
                        key=|file| file.1.clone()
                        children=move |n| {
                            let (file_type, file_name, file_size) = n;
                            let link_target = if file_type == "f" {
                                format!("/files/{}/{}", path.get_untracked(), &file_name)
                            } else { "#".to_string() };

                            view! {
                                <a
                                    href=link_target
                                    rel="external"
                                    class="list-group-item list-group-item-action"
                                    on:click=move |ev| {
                                        // on:click logic remains the same
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
                                    <img src={if file_type == "d" { "/assets/folder.png" } else { "/assets/file.png" }} style="width: 48px; height: 48px; margin-right: 10px"/>
                                    {if file_type == "d" { format!("{}/", file_name) } else { file_name.clone() }}
                                    <span class="float-end">
                                        {if file_type == "f" { file_size.fmt_size(Conventional).to_string() } else { "".to_string() }}
                                    </span>
                                </a>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}

#[server(GetChatMessages)]
pub async fn get_chat_messages(
    new_chat_message : (String, String)
) -> Result<Vec<(String, String, u64)>, ServerFnError> {
    logging::log!("Chat message received: {:?}", new_chat_message);
    // Filter the chat message for XSS
    let mut new_username = clean(&new_chat_message.0);
    let new_chat_message = clean(&new_chat_message.1);
    
    if new_username.clone().is_empty() {
        new_username = "Anonymous".to_string();
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
    if new_chat_message.len() > 0 && new_chat_message.len() < 1000 {
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        chat_messages.push((new_username.clone(), new_chat_message.clone(), timestamp));
        // Write the chat messages back to chat.json
        let chat_file = std::fs::File::create(chat_file_path)
            .map_err(|e| format!("Error creating chat file: {:?}", e)).unwrap();
        serde_json::to_writer(chat_file, &chat_messages)
            .map_err(|e| format!("Error writing chat file: {:?}", e)).unwrap();
    }
    // Only return the last 5 chat messages
    let chat_messages = chat_messages.iter().rev().take(5).cloned().rev().collect();

    Ok(chat_messages)
}

#[component]
pub fn ChatComponent() -> impl IntoView {
    let (chat, send_chat) = signal(("".to_string(), "".to_string()));
    let chat_input_ref: NodeRef<Input> = NodeRef::new();
    let name_input_ref: NodeRef<Input> = NodeRef::new();
    let inc = Action::new(|(delta, msg): &(i32, String)| {
        // Create owned copies from the borrowed data.
        let delta_owned = *delta;
        let msg_owned = msg.clone();

        // `async move` block now captures the owned copies.
        // This makes the future 'static, satisfying the lifetime requirement.
        async move {
            adjust_message_count(delta_owned, msg_owned).await
        }
    });


    let on_submit = move |ev: SubmitEvent| {
        // Prevent the page from refreshing
        ev.prevent_default();
        // Get a reference to the chat text input box
        let new_chat_message = chat_input_ref.get().expect("<input> does not exist").value();
        let new_username = name_input_ref.get().expect("<input> does not exist").value();
        // Send the chat message to the server
        send_chat.set((new_username, new_chat_message));
        // Clear text input box
        chat_input_ref.get().expect("<input> does not exist").set_value("");
        inc.dispatch((1, "test".into()));
    };

    let chat_messages = create_local_resource(
        move || chat.get(),
        get_chat_messages, // every time `chat` changes, this will run
    );

    let messages = Memo::new(move |_| chat_messages.get().and_then(|res| res.ok()));
    let error = Memo::new(move |_| chat_messages.get().and_then(|res| res.err()));

    // This is the SSE (Server-Sent Events) handling section.
    let (message_count_value, set_message_count_value) = signal(Some("0".to_string()));

    #[cfg(not(feature = "ssr"))]
    {
        use futures::StreamExt;
        // We use spawn_local to run this non-Send future on the main thread.
        spawn_local(async move {
            let mut source = gloo_net::eventsource::futures::EventSource::new("/ws")
                .expect("couldn't connect to SSE stream");

            let mut stream = source.subscribe("message").unwrap();

            // Continuously listen for new messages on the stream
            while let Some(value) = stream.next().await {
                match value {
                    Ok(event) => {
                        let data = event.1.data().as_string().unwrap_or_default();
                        set_message_count_value.set(Some(data));
                    }
                    Err(_) => {
                        // TODO: Handle stream error, e.g., break the loop
                        break;
                    }
                }
            }
            // `source` is dropped here, which automatically closes the connection.
            // No `on_cleanup` is needed.
        });
    }

    // If there's a new message count value sent from the server, initiate a GET for new chat messages by sending an empty message.
    // This could of course be done more efficiently by directly fetching the new chat message from the server.
    Effect::new(move |_| {
        // When message_count_value changes, trigger a refetch of chat messages.
        if let Some(count) = message_count_value.get() {
            send_chat.set((count, "".to_string()));
        }
    });
    // --- FIX END ---


    view! {
        <div class="card">
            <h2 class="card-header">Chat</h2>
            <div class="card-body overflow-y-scroll">
                <Show when=move || chat_messages.loading().get() fallback=|| ()>
                    <p>"Loading chat..."</p>
                </Show>

                <Show when=move || error.get().is_some() fallback=|| ()>
                    <p>"ERROR: " {error.get().unwrap().to_string()}</p>
                </Show>

                <Show when=move || messages.get().is_some() fallback=|| ()>
                    <For
                        each=move || messages.get().unwrap_or_default()
                        key=|msg| msg.2 // timestamp
                        children=move |(user, message, _)| {
                            view! {
                                <div class="card">
                                    <div class="card-body">
                                        <p class="card-text">{user}: {message}</p>
                                    </div>
                                </div>
                            }
                        }
                    />
                </Show>
            </div>
            <div>
                <form on:submit=on_submit>
                    <input type="text" class="form-control" placeholder="Name" node_ref=name_input_ref/>
                    <input type="text" class="form-control" placeholder="Type a chat message" node_ref=chat_input_ref/>
                    <button class="btn btn-outline-secondary" type="submit">Send</button>
                </form>
            </div>
        </div>
    }
}

#[cfg(feature = "ssr")]
pub mod ssr_imports {
    pub use broadcaster::BroadcastChannel;
    pub use once_cell::sync::OnceCell;
    pub use std::sync::atomic::{AtomicI32, Ordering};

    pub static COUNT: AtomicI32 = AtomicI32::new(0);

    lazy_static::lazy_static! {
        pub static ref COUNT_CHANNEL: BroadcastChannel<i32> = BroadcastChannel::new();
    }

    static LOG_INIT: OnceCell<()> = OnceCell::new();

    pub fn init_logging() {
        LOG_INIT.get_or_init(|| {
            simple_logger::SimpleLogger::new().env().init().unwrap();
        });
    }
}

#[server]
pub async fn get_message_count() -> Result<i32, ServerFnError> {
    use ssr_imports::*;

    Ok(COUNT.load(Ordering::Relaxed))
}

#[server]
pub async fn adjust_message_count(
    delta: i32,
    msg: String,
) -> Result<i32, ServerFnError> {
    use ssr_imports::*;

    let new = COUNT.load(Ordering::Relaxed) + delta;
    COUNT.store(new, Ordering::Relaxed);
    _ = COUNT_CHANNEL.send(&new).await;
    println!("message = {:?}", msg);
    Ok(new)
}
