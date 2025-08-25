use std::path::PathBuf;

use actix_files as fs;
use actix_web::{App, HttpServer, web};
use clap::Parser;

#[derive(Clone)]
struct AppData {
    idx: PathBuf,
}

#[actix_web::get("/")]
async fn index(data: web::Data<AppData>) -> actix_web::Result<fs::NamedFile> {
    Ok(fs::NamedFile::open(&data.idx)?)
}

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the index HTML document
    #[arg(long = "index")]
    idx: PathBuf,

    /// Path to the folder containing the assets
    #[arg(long)]
    assets: Option<PathBuf>,

    /// Path to the folder containing the generated WASM code
    #[arg(long)]
    pkg: PathBuf,

    /// Port to bind the server to
    #[arg(long, default_value_t = 8080)]
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let port = cli.port;
    println!("Starting server at 127.0.0.1:{port}");
    HttpServer::new(move || {
        let app = App::new() //
            .service(index)
            .service(fs::Files::new("/pkg", cli.pkg.clone()))
            .app_data(web::Data::new(AppData {
                idx: cli.idx.clone(),
            }));
        if let Some(assets) = cli.assets.clone() {
            app.service(fs::Files::new("/assets", assets))
        } else {
            app
        }
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
