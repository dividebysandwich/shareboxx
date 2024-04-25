#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use shareboxx::app::*;

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    println!("listening on http://{}", &addr);

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", site_root))
            .service(Files::new("/files", "./files"))
            // serve the favicon from /favicon.ico
            .service(favicon)
            // uploader
            .service(web::resource("/upload").route(web::post().to(save_files)),
            )
            .leptos_routes(leptos_options.to_owned(), routes.to_owned(), App)
            .app_data(web::Data::new(leptos_options.to_owned()))
        //.wrap(middleware::Compress::default())
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon.ico")]
async fn favicon(
    leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use shareboxx::app::*;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}


#[cfg(feature = "ssr")]
#[derive(Debug, actix_multipart::form::MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    files: Vec<actix_multipart::form::tempfile::TempFile>,
    upload_path: actix_multipart::form::text::Text<String>,
}

#[cfg(feature = "ssr")]
async fn save_files(
    actix_multipart::form::MultipartForm(form): actix_multipart::form::MultipartForm<UploadForm>,
) -> Result<impl actix_web::Responder, actix_web::Error> {

    for f in form.files {
        let path = format!("./files/{}{}", form.upload_path.clone(), f.file_name.unwrap());
        f.file.persist(path).unwrap();
    }

    Ok(actix_web::web::Redirect::to("/").see_other())
}