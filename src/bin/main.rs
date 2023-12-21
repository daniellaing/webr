use axum::{routing::get, Router};
use webr::{get_page, init, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    init()?;

    let app = Router::new().route("/", get(|| async { "Hello world!" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    let page = get_page(String::from("Test page"));
    println!("{page}");

    Ok(())
}
