#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use dnd_dashboard::app::{shell, App};
    use dnd_dashboard::db;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/dnd.db".to_string());

    // Make sure the directory for a file-backed database exists.
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).expect("could not create database directory");
            }
        }
    }

    db::init(&database_url)
        .await
        .expect("could not open database");
    db::ensure_default_party()
        .await
        .expect("could not create default party");

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    println!("D&D Dashboard listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
fn main() {}
