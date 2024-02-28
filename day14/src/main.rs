use day13_api::{
    app, connect_and_migrate, database_target_for_logs, ensure_sqlite_parent_dir, init_tracing,
    load_config_from_env, shutdown_signal, AppState,
};

#[tokio::main]
async fn main() {
    // Load `.env` if present (does not override vars already set in the shell, e.g. PowerShell `$env:DATABASE_URL=...`).
    dotenvy::dotenv().ok();
    init_tracing();

    let config = match load_config_from_env() {
        Ok(c) => c,
        Err(msg) => {
            tracing::error!(%msg, "configuration error");
            eprintln!("{msg}");
            std::process::exit(1);
        }
    };

    let db_hint = database_target_for_logs(&config.database_url);

    if let Err(e) = ensure_sqlite_parent_dir(&config.database_url) {
        tracing::error!(%e, "failed to create database directory");
        std::process::exit(1);
    }

    let pool = match connect_and_migrate(&config.database_url).await {
        Ok(p) => {
            tracing::info!(%db_hint, "migrations applied; database pool ready");
            p
        }
        Err(e) => {
            tracing::error!(%e, "database setup failed");
            std::process::exit(1);
        }
    };

    let pool_shutdown = pool.clone();
    let router = app(AppState { pool });

    let bind_addr = format!("{}:{}", config.host, config.port);
    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(%e, bind_addr = %bind_addr, "failed to bind listener");
            std::process::exit(1);
        }
    };

    let local_addr = listener.local_addr().expect("listener local address");
    tracing::info!(%local_addr, "listening");
    tracing::info!(
        swagger_ui = %format!("http://{local_addr}/swagger-ui"),
        "OpenAPI UI available"
    );

    let server = axum::serve(listener, router).with_graceful_shutdown(async {
        shutdown_signal().await;
        tracing::info!("shutdown signal received");
    });

    if let Err(e) = server.await {
        tracing::error!(%e, "server error");
        std::process::exit(1);
    }

    tracing::info!("server stopped; closing database pool");
    pool_shutdown.close().await;
    tracing::info!("clean exit");
}
