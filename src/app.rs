use leptos::*;
use leptos_meta::*;
use leptos_router::*;


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

#[server(GetFileList)]
pub async fn get_file_list(
    path : String
) -> Result<Vec<(String, String)>, ServerFnError> {
    let base_path = std::env::current_dir()
    .map_err(|e| format!("Error getting current directory: {:?}", e)).unwrap();
    //Check if path contains "..", if so, return an error
    if path.contains("..") {
        return Err(leptos::ServerFnError::ServerError("Path contains '..'".to_string()));
    }
    let path_to_read = base_path.join("files").join(path.clone());
    logging::log!("Current directory: {:?}", path_to_read.clone());
    let files = std::fs::read_dir(path_to_read)
        .map_err(|e| format!("Error reading directory: {:?}", e)).unwrap();
    let file_entries : Vec<(String, String)> = files
        .filter_map(|entry| {
            match entry {
                Ok(entry) => {
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        Some(("d".to_string(), entry.file_name().into_string().unwrap()))
                    } else {
                        Some(("f".to_string(), entry.file_name().into_string().unwrap()))
                    }
                },
                Err(e) => {
                    Some(("f".to_string(), format!("Error reading file entry: {:?}", e).to_string().into()))
                },
            }
        })
        .collect();
    
    //If path is not empty, prepend ".." to the list of files
    if !path.is_empty() {
        let mut new_files = Vec::new();
        new_files.push(("d".to_string(), "..".to_string()));
        new_files.extend(file_entries);
        return Ok(new_files);
    }

//    logging::log!("Found {} files: {:?}", file_entries.len(), file_entries);
    Ok(file_entries)
}

#[component]
pub fn FileListComponent() -> impl IntoView {
    let (path, set_path) = create_signal("".to_string());
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
            Path: {path.clone()}<br/>
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
                                        let (file_type_clone, file_name_clone) = &n.clone();
                                        let path_value_clone = path.clone().get();
                                        let mut link_target: String = "#".to_string();
                                        if file_type_clone.clone() == "f" {
                                            link_target = format!("/files/{}/{}", path_value_clone, file_name_clone.clone()).to_string();
                                        }
                                        view! { 
                                            <a href={link_target} rel="external" on:click=move |ev| {
                                                let path_value = path.get();
                                                let (file_type, file_name) = n.clone();
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
                                            } class="list-group-item list-group-item-action">{file_name_clone}</a>
                                        }
                                    })
                                    .collect_view()        
                                }
                                Err(_e) => {
                                    leptos::View::Text(view! {
                                        "Error! {_e}"
                                    })
                                }
                            }
                        }
                        None => {
                            leptos::View::Text(view! {
                                "Error: No results found.   "
                            })
                        }
                    }
                }
            }
            </div>
        </div>
    }

}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1>"Welcome to ShareBoxx!"</h1>
        <br/>
        <FileListComponent/>
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
