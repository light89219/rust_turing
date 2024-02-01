use day13_api::{app, connect_and_migrate, ensure_sqlite_parent_dir, init_tracing, AppState};

#[tokio::main]
async fn main() {
    // Load `.env` if present (does not override vars already set in the shell, e.g. PowerShell `$env:DATABASE_URL=...`).
    dotenvy::dotenv().ok();
    init_tracing();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/app.db".to_string());

    if let Err(e) = ensure_sqlite_parent_dir(&database_url) {
        eprintln!("failed to create database directory: {e}");
        std::process::exit(1);
    }

    let pool = connect_and_migrate(&database_url)
        .await
        .unwrap_or_else(|e| {
            eprintln!("database setup failed: {e}");
            std::process::exit(1);
        });

    let state = AppState { pool };

    let app = app(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("bind failed");
    println!("listening on http://127.0.0.1:3000");
    println!("DATABASE_URL={database_url}");
    println!("OpenAPI UI: http://127.0.0.1:3000/swagger-ui");
    axum::serve(listener, app).await.expect("server failed");
}
