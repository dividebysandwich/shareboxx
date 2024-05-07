use fmtsize::{Conventional, FmtSize};
use leptos::{html::Input, *};
use leptos::ev::SubmitEvent;
use leptos_meta::*;
use leptos_router::*;
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
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let (path, set_path) = create_signal("".to_string());
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
                <form action="/upload" rel="external" method="post" enctype="multipart/form-data">
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
    set_path: WriteSignal<String>
) -> impl IntoView {
    // our resource
    let directory_listing = create_local_resource(
        path,
        // every time `count` changes, this will run
        |value| async move {
            logging::log!("loading data from API for path {:?}", value);
            get_file_list(value).await
        },
    );

    // the resource's loading() method gives us a
    // signal to indicate whether it's currently loading
    let loading = directory_listing.loading();
    let is_loading = move || if loading() { "Loading..." } else { "" };
    view! {
        <div>
        Current Directory: {path.clone()}
        <br/>
        <p/>
        {is_loading}
        <div class="list-group">
            {
                move || { 
                    match directory_listing.get() {
                        Some(result) => {
                            match result {
                                Ok(files) => {
                                    files.into_iter()
                                    .map(move |n| {
                                        let (file_type_clone, file_name_clone, file_size_clone) = &n.clone();
                                        let path_value_clone = path.clone().get();
                                        let mut link_target: String = "#".to_string();
                                        if file_type_clone.clone() == "f" {
                                            link_target = format!("/files/{}/{}", path_value_clone, file_name_clone.clone()).to_string();
                                        }
                                        view!{
                                        <a href={link_target} rel="external" on:click=move |ev| {
                                                let path_value = path.get();
                                                let (file_type, file_name, _file_size) = n.clone();
                                                // If path is "..", remove the last directory from the path
                                                if file_name == ".." {
                                                    ev.prevent_default();
                                                    let mut path_clone = path_value.clone();
                                                    let mut path_parts: Vec<&str> = path_clone.split("/").collect();
                                                    path_parts.pop();
                                                    path_parts.pop();
                                                    path_clone = path_parts.join("/");
                                                    if !path_clone.ends_with("/") && !path_clone.is_empty() {
                                                        path_clone.push_str("/");
                                                    }
                                                    set_path(path_clone);
                                                } else {
                                                    // if file_type is a directory, append it to the path
                                                    if file_type == "d" {
                                                        ev.prevent_default();
                                                        let mut path_clone = path_value.clone();
                                                        path_clone.push_str(file_name.clone().as_str());
                                                        path_clone.push_str("/");
                                                        set_path(path_clone);
                                                    }
                                                }
                                            } class="list-group-item list-group-item-action">
                                            <img src={if file_type_clone == "d" {"/assets/folder.png"} else {"/assets/file.png"}} style="width: 48px; height: 48px; margin-right: 10px"/>
                                            {
                                                if file_type_clone == "d" {
                                                    format!("{}/", file_name_clone)
                                                } else {
                                                    format!("{}", file_name_clone)
                                                }
                                            }
                                            <span class="float-end">
                                            {
                                                if file_type_clone == "f" {
                                                    format!("{} ", file_size_clone.fmt_size(Conventional))
                                                } else {
                                                    "".to_string()
                                                }
                                            }
                                            </span>
                                        </a>
                                        }
                                    })
                                    .collect_view()
                                }
                                Err(e) => {
                                    logging::log!("Error displaying files: {:?}", e);
                                    leptos::View::Text(view! {
                                        "ERROR: Could not display files. Please try again later."
                                    })
                                }
                            }
                        }
                        None => {
                            leptos::View::Text(view! {
                                "No files found."
                            })
                        }
                    }
                }
            }
            </div>
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
    let (chat, send_chat) = create_signal(("".to_string(), "".to_string()));
    let chat_input_ref: NodeRef<Input> = create_node_ref();
    let name_input_ref: NodeRef<Input> = create_node_ref();
    let inc = create_action(|_: &()| adjust_message_count(1, "test".into()));

    let on_submit = move |ev: SubmitEvent| {
        // Prevent the page from refreshing
        ev.prevent_default();
        // Get a reference to the chat text input box
        let new_chat_message = chat_input_ref().expect("<input> does not exist").value();
        let new_username = name_input_ref().expect("<input> does not exist").value();
        // Send the chat message to the server
        send_chat((new_username.to_string(), new_chat_message.to_string()));
        // Clear text input box
        chat_input_ref().expect("<input> does not exist").set_value("");
        inc.dispatch(());
    };

    // our resource
    let chat_messages = create_local_resource(
        chat,
        // every time `chat` changes, this will run
        |new_chat_message| async move {
            logging::log!("Chat by {}: {}", new_chat_message.0, new_chat_message.1);
            get_chat_messages(new_chat_message).await
        },
    );

    #[cfg(not(feature = "ssr"))]
    let message_count_value = {
        use futures::StreamExt;
        let mut source =
            gloo_net::eventsource::futures::EventSource::new("/ws")
                .expect("couldn't connect to SSE stream");
        let s = create_signal_from_stream(
                source
                .subscribe("message")
                .unwrap()
                .map(|value| match value {
                    Ok(value) => value
                        .1
                        .data()
                        .as_string()
                        .expect("expected string value"),
                    Err(_) => "0".to_string(),
                }),
        );
        on_cleanup(move || source.close());
        s
    };

    #[cfg(feature = "ssr")]
    let (message_count_value, _) = create_signal(None::<i32>);

    // If there's a new message count value sent from the server, initiate a GET for new chat messages by sending an empty message.
    // This could of course be done more efficiently by directly fetching the new chat message from the server.
    create_effect(move |_| {
        let count = message_count_value.get().unwrap_or_default();
        send_chat((count.to_string(), "".to_string())); // If count is not used anywhere, the effect will never be triggered.
    });

    view! {
        <div class="card">
        <h2 class="card-header">Chat</h2>
        <div class="card-body overflow-y-scroll">
          {
            move || { 
                match chat_messages.get() {
                    Some(result) => {
                        match result {
                            Ok(messages) => {
                                messages.into_iter()
                                .map(move |n| {
                                    let (user, message, _timestamp) = n.clone();
                                    view!{
                                    <div class="card">
                                        <div class="card-body">
                                            <p class="card-text">{user}: {message}</p>
                                        </div>
                                    </div>
                                    }
                                }).collect_view()
                            },
                            Err(e) => {
                                logging::log!("Error displaying chat messages: {:?}", e);
                                leptos::View::Text(view! {
                                    "ERROR: Could not display chat messages. Please try again later."
                                })
                            }
                        }
                    },
                    None => {
                        leptos::View::Text(view! {
                            "No chat messages found"
                        })
                    }
                }
            }
          }
          <div>
          <form on:submit=on_submit>
            <input type="text" class="form-control" placeholder="Name" node_ref=name_input_ref />
            <input type="text" class="form-control" placeholder="Type a chat message" node_ref=chat_input_ref />
            <button class="btn btn-outline-secondary" type="submit" id="button-send">Send</button>
            </form>
          </div>

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
