use fmtsize::{Conventional, FmtSize};
use leptos::{html::Input, *};
use leptos::prelude::*;
use leptos::ev::SubmitEvent;
use leptos_meta::*;
use leptos_router::*;
use leptos_reactive::{
    create_resource, spawn_local, SignalGet
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
        <head>
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/shareboxx.css"/>

        // sets the document title
        <Title text="Welcome to ShareBoxx"/>
        </head>
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

    let directory_listing = Resource::new(move|| path.get(), get_file_list);
    let loading = signal(false);

    spawn_local({
        let data = directory_listing.clone();
        let loading = loading.clone();
        // run after mount
        async move {
            loading.1.set(true);
            // Only run client-side
            let val = get_file_list(path.get()).await;
            data.set(Some(val));
            loading.1.set(false);
        }
    });
    
    // Create a derived memo that only contains a value when the resource has loaded successfully.
    let files = Memo::new(move |_| directory_listing.get().and_then(|res| res.ok()));
    // Create another memo that only contains a value when the resource has an error.
    let error = Memo::new(move |_| directory_listing.get().and_then(|res| res.err()));

    view! {
        <div>
            "Current Directory: " {path}
            <p/>

            // Use <Show> to display the loading state.
            <Show when=move || loading.0.get() fallback=|| ()>
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

    let mut chat_messages : Vec<(String, String, u64)> = Vec::new();
    // If chat file is not empty, parse it
    let mut chat_messages : Vec<(String, String, u64)> = if !chat_file.is_empty() {
        serde_json::from_str(&chat_file).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    // Only return the last 5 chat messages
    let chat_messages = chat_messages.iter().rev().take(5).cloned().rev().collect();

    Ok(chat_messages)
}

#[server]
pub async fn send_chat_message(
    chat_name: String,
    chat_message: String,
) -> Result<(), ServerFnError> {
    // This gives us access to COUNT_CHANNEL for sending SSE updates
    use crate::app::ssr_imports::*;

    logging::log!("Chat message received: {:?}", chat_message);
    // Filter the chat message for XSS
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
        _ = COUNT_CHANNEL.send(&1).await;
    }
    Ok(())
}

#[component]
pub fn ChatComponent() -> impl IntoView {
    let chat_input_ref: NodeRef<Input> = NodeRef::new();

    let send_chat_message = ServerAction::<SendChatMessage>::new();

    let (version, set_version) = signal(0);

    // Create the resource for chat messages.
    // The source signal `|| ()` means it will fetch once on load.
    // We will trigger subsequent fetches manually.
    // let chat_messages_resource = create_resource(
    //     move || version.get(),
    //     |_| async move { get_chat_messages().await },
    // );

    // Create the resource only on the client:
    #[cfg(not(feature = "ssr"))]
    let chat_messages_resource = Resource::new(move || version.get(), |_| async { get_chat_messages().await });

    // On the server, supply a dummy resource so SSR can render:
    #[cfg(feature = "ssr")]
    let chat_messages_resource =
        Resource::new(move || version.get(), |_| async {
            // explicitly tell Rust what error type we’re using:
            Ok::<Vec<(String, String, u64)>, ServerFnError>(vec![(
                "".to_string(),
                "".to_string(),
                0u64,
            )])
        });

    // This Memo extracts the Vec from the resource.
    // The rest of our code will only ever have to deal with this signal,
    // which *always* contains a Vec, never an error or an option.
    let messages = Memo::new(move |_| {
        chat_messages_resource
            .get()
            .and_then(|res| res.ok())
            .unwrap_or_default()
    });

    // When the user successfully sends a message, we increment the version signal to trigger a refetch for them.
    Effect::new(move |_| {
        // .value() is a signal that returns Some(Ok(_)) on success
        if send_chat_message.value().get().is_some() {
            set_version.update(|v| *v += 1);
        }
    });

    // When an event arrives from another user, we also increment the version signal to trigger a refetch.
    #[cfg(not(feature = "ssr"))]
    {
        use futures::StreamExt;
        spawn_local(async move {
            let mut source = gloo_net::eventsource::futures::EventSource::new("/ws")
                .expect("couldn't connect to SSE stream");
            let mut stream = source.subscribe("message").unwrap();
            while let Some(_) = stream.next().await {
                set_version.update(|v| *v += 1);
            }
        });
    }

    // This on_submit now only needs to clear the input.
    // The ActionForm handles preventing the default refresh automatically.
    let on_submit = move |ev: SubmitEvent| {
        if let Some(input) = chat_input_ref.get() {
            input.set_value("");
            ev.prevent_default();
        }
    };

    view! {
        <div class="card">
            <h2 class="card-header">Chat</h2>
            <div class="card-body overflow-y-scroll" style="height: 300px;">
                <Suspense fallback=|| view! { <p>"Loading..."</p> }>
                    // 3. The View is now simple. We use <Show> to handle the empty case.
                    // This is the correct way to do conditional rendering and will not cause type errors.
                    <Show
                        when=move || !messages.get().is_empty()
                        fallback=|| view! { <p>"No messages yet."</p> }
                    >
                        <For
                            // `each` gets the data from our clean `messages` Memo.
                            // This pattern is `Fn` and is lifetime-safe.
                            each=move || messages.get()
                            key=|msg| msg.2
                            children=move |(user, message, _)| {
                                view! { <p><strong>{user}</strong>": " {message}</p> }
                            }
                        />
                    </Show>
                </Suspense>
            </div>
            <div>
                <ActionForm action=send_chat_message on:submit=on_submit>
                    <div class="input-group">
                        <input type="text" class="form-control" placeholder="Name" name="chat_name"/>
                        <input
                            type="text"
                            class="form-control"
                            placeholder="Type a chat message"
                            name="chat_message"
                            node_ref=chat_input_ref
                        />
                        <button class="btn btn-outline-secondary" type="submit">Send</button>
                    </div>
                </ActionForm>
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
