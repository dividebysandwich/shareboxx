#[cfg(feature = "ssr")]
use actix_files::Files;
#[cfg(feature = "ssr")]
use actix_multipart::form::{tempfile::TempFileConfig, MultipartFormConfig};
#[cfg(feature = "ssr")]
use actix_web::{web, get, App, HttpResponse, HttpServer};
#[cfg(feature = "ssr")]
use actix_web::middleware::{Next, from_fn};
#[cfg(feature = "ssr")]
use actix_web::dev::{ServiceRequest, ServiceResponse};
#[cfg(feature = "ssr")]
use actix_web::body::MessageBody;
#[cfg(feature = "ssr")]
#[cfg(feature = "ssr")]
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_actix::{generate_route_list, LeptosRoutes};
#[cfg(feature = "ssr")]
use shareboxx::app::App;

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Load Leptos options from Cargo.toml
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let routes = generate_route_list(App);
    println!("listening on http://{}", &addr);

    HttpServer::new(move || {
        let site_root = &leptos_options.site_root;

        App::new()
            .wrap(from_fn(domain_redirect))
            .app_data(web::Data::new(
                MultipartFormConfig::default()
                    .total_limit(10 * 1024 * 1024 * 1024)
                    .memory_limit(10 * 1024 * 1024)
                    .error_handler(handle_multipart_error),
            ))
            // Leptos server side API
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            // SSE events to notify clients of new chat messages
            .service(counter_events)
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", site_root.as_ref()))
            .service(Files::new("/files", "./files"))
            // serve the favicon from /favicon.ico
            .service(favicon)
            // uploader
            .service(web::resource("/upload").route(web::post().to(save_files)))

            .leptos_routes(
                routes.to_owned(),
                {
                    let options = leptos_options.clone();
                    move || shareboxx::app::shell(options.clone())
                },
            )
            .app_data(web::Data::new(leptos_options.to_owned()))
            // Store temp files on same drive, otherwise .persist() will fail due to cross-device link error
            .app_data(web::Data::new(
                TempFileConfig::default().directory("files"),
            ))
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(feature = "ssr")]
fn handle_multipart_error(err: actix_multipart::MultipartError, _req: &actix_web::HttpRequest) -> actix_web::Error {
    println!("Multipart error: {}", err);
    err.into()
}

#[cfg(feature = "ssr")]
#[get("/favicon.ico")]
async fn favicon(
    leptos_options: actix_web::web::Data<LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
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
        
        // This logic can be simplified/made more robust, but keeping it for now
        let mut new_path = path.clone();
        if let (Some(parent), Some(stem), Some(ext)) = (
            std::path::Path::new(&path).parent(),
            std::path::Path::new(&path).file_stem().and_then(|s| s.to_str()),
            std::path::Path::new(&path).extension().and_then(|s| s.to_str()),
        ) {
            let mut i = 1;
            while std::path::Path::new(&new_path).exists() {
                new_path = parent
                    .join(format!("{}-{}.{}", stem, i, ext))
                    .to_str()
                    .unwrap()
                    .to_string();
                i += 1;
            }
        }

        f.file.persist(new_path).unwrap();
    }

    Ok(actix_web::HttpResponse::Ok().body("ok"))
}

#[cfg(feature = "ssr")]
async fn domain_redirect(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    // Check if request hostname matches shareboxx.lan, otherwise redirect to it.
    if req.connection_info().host() != "shareboxx.lan" && !req.connection_info().host().starts_with("127.0.0.1") {
        return Ok(ServiceResponse::new(
            req.request().to_owned(),
            HttpResponse::TemporaryRedirect()
                .append_header(("Location", "https://shareboxx.lan"))
                .finish(),
        )
        .map_into_boxed_body());
    }

    next.call(req).await.map(ServiceResponse::map_into_boxed_body)
}

#[cfg(feature = "ssr")]
#[get("/ws")]
async fn counter_events() -> impl actix_web::Responder {
    use actix_web::web::Bytes;
    use futures::StreamExt;
    use shareboxx::app::ssr_imports::*;
    use tokio_stream::wrappers::ReceiverStream;

    let (tx, rx) = tokio::sync::mpsc::channel::<Bytes>(32);

    // Increment connected users and broadcast new count
    let new_count = CONNECTED_USERS.fetch_add(1, Ordering::Relaxed) + 1;
    _ = USERS_CHANNEL.send(new_count);

    tokio::spawn(async move {
        // Send initial user count to this client
        let _ = tx.send(Bytes::from(format!(
            "event: users\ndata: {new_count}\n\n"
        ))).await;

        let mut chat_rx = CHAT_CHANNEL.subscribe();
        let mut users_rx = USERS_CHANNEL.subscribe();

        loop {
            tokio::select! {
                result = chat_rx.recv() => {
                    match result {
                        Ok(_) => {
                            if tx.send(Bytes::from("event: chat\ndata: update\n\n")).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(_) => break,
                    }
                }
                result = users_rx.recv() => {
                    match result {
                        Ok(count) => {
                            if tx.send(Bytes::from(format!("event: users\ndata: {count}\n\n"))).await.is_err() {
                                break;
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(_) => break,
                    }
                }
                // Periodic heartbeat to detect dead connections
                _ = tokio::time::sleep(std::time::Duration::from_secs(15)) => {
                    if tx.send(Bytes::from(": heartbeat\n\n")).await.is_err() {
                        break;
                    }
                }
            }
        }

        // Client disconnected - decrement and broadcast
        let new_count = (CONNECTED_USERS.fetch_sub(1, Ordering::Relaxed) - 1).max(0);
        _ = USERS_CHANNEL.send(new_count);
    });

    let stream = ReceiverStream::new(rx).map(Ok::<_, actix_web::Error>);
    actix_web::HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .streaming(stream)
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}