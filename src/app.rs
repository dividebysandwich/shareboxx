use leptos::{logging::log, *};
use leptos_meta::*;
use leptos_router::*;


#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
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

pub fn get_file_list() -> Result<Vec<String>, String> {
    log!("Current directory: {:?}", std::env::current_dir());
    let files = std::fs::read_dir("./files")
        .map_err(|e| format!("Error reading directory: {:?}", e))?;
    let file_entries : Vec<String> = files
        .filter_map(|entry| {
            match entry {
                Ok(entry) => entry.file_name().into_string().ok(),
                Err(e) => {
                    log!("Error reading file entry: {:?}", e);
                    None
                },
            }
        })
        .collect();
    log!("Found {} files: {:?}", file_entries.len(), file_entries);
    Ok(file_entries)
}

#[component]
pub fn FileListView() -> impl IntoView {
    //let files_value = get_file_list();
    match get_file_list() {
        Ok(files) => {
            view! {
                <ul>
                    {
                        files.into_iter()
                            .map(|n| view! { <li>{n}</li>})
                            .collect_view()
                    }
                </ul>
            }
        },
        Err(_e) => {
            view! {
                <ul><li>"Error: {_e}"</li></ul>
            }
        }
    }

}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1>"Welcome to ShareBoxx!"</h1>
        Files:<br/>
        <br/>
        <FileListView/>
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
