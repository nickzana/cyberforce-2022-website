use axum::{
    body::{boxed, BoxBody},
    extract::{BodyStream, Path, Query},
    response::{Html, Redirect, Response},
    routing::{get, post},
    Router,
};
use hyper::{Body, Request, StatusCode, Uri};
use serde::Deserialize;
use std::{fs::read_to_string, net::SocketAddr, path::PathBuf};
use tower::ServiceExt;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(home))
        .route("/contact", get(contact))
        .route("/solar", get(solar))
        .route("/login", get(login))
        .route("/manufacturing", get(manufacturing));

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn home() -> Html<String> {
    Html(static_file(["html", "home.html"].into_iter().collect()).await)
}

async fn contact() -> Html<String> {
    Html(static_file(["html", "contact.html"].into_iter().collect()).await)
}

async fn manufacturing() -> Html<String> {
    Html(static_file(["html", "manufacturing.html"].into_iter().collect()).await)
}

async fn solar() -> Html<String> {
    Html(static_file(["html", "solar.html"].into_iter().collect()).await)
}

async fn login() -> Html<String> {
    Html(static_file(["html", "login.html"].into_iter().collect()).await)
}

async fn static_file(path: PathBuf) -> String {
    println!("{:#?}", path.canonicalize().unwrap());
    read_to_string(path).unwrap()
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

// Handler that streams the request body to a file.
//
// POST'ing to `/file_upload/foo.txt` will create a file called `foo.txt`.
async fn save_request_body(
    Path(file_name): Path<String>,
    body: BodyStream,
) -> Result<(), (StatusCode, String)> {
    todo!()
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    match ServeDir::new(".").oneshot(req).await {
        Ok(response) => Ok(response.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}
