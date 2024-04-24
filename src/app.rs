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
) -> Result<Vec<String>, ServerFnError> {
    let base_path = std::env::current_dir()
    .map_err(|e| format!("Error getting current directory: {:?}", e)).unwrap();
    let path_to_read = base_path.join("files").join(path);
    logging::log!("Current directory: {:?}", path_to_read.clone());
    let files = std::fs::read_dir(path_to_read)
        .map_err(|e| format!("Error reading directory: {:?}", e)).unwrap();
    let file_entries : Vec<String> = files
        .filter_map(|entry| {
            match entry {
                Ok(entry) => entry.file_name().into_string().ok(),
                Err(e) => {
                    Some(format!("Error reading file entry: {:?}", e).to_string())
                },
            }
        })
        .collect();
    logging::log!("Found {} files: {:?}", file_entries.len(), file_entries);
    Ok(file_entries)
}

#[component]
pub fn FileListComponent() -> impl IntoView {
    let (path, set_path) = create_signal("".to_string());
    // our resource
    let directory_listing = create_resource(
        path,
        // every time `count` changes, this will run
        |value| async move {
            logging::log!("loading data from API for path {:?}", value);
            get_file_list(value).await
        },
    );

    let directory_listing_result = move || {
//        let loading_vec = vec!["Loading...".to_string()];
        directory_listing
            .read()
    };

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
                match directory_listing_result() {
                    Some(result) => {
                        match result {
                            Ok(files) => {
                                files.into_iter()
                                .map(move |n| {
                                    let n_clone = &n.clone();
                                    view! { 
                                        <a href="#" rel="external" on:click=move |ev| {
                                            ev.prevent_default();
                                            set_path(n.clone());
                                        } class="list-group-item list-group-item-action">{n_clone}</a>
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
                
                // directory_listing_result.into_iter()
                //     .map(move |n| {
                //         let n_clone = &n.clone();
                //         view! { <a href="#" rel="external" on:click=move |ev| {
                //             ev.prevent_default();
                //             set_path(n.clone());
                //         } class="list-group-item list-group-item-action">{n_clone}</a>}
                //     })
                //     .collect_view()
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
