use axum::{
    body::{boxed, BoxBody},
    extract::{BodyStream, Path, Query},
    response::{Html, Redirect, Response},
    routing::get,
    Form, Router,
};
use hyper::{Body, Request, StatusCode, Uri};
use serde::Deserialize;
use std::{env, fs::read_to_string, net::SocketAddr, path::PathBuf};
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
        .route("/manufacturing", get(manufacturing))
        .route("/login_submit", get(login_submit))
        .route("/admin", get(admin))
        .route("/logged_in", get(logged_in))
        .route("/login_fail", get(login_fail));

    // run it
    let addr = SocketAddr::from((
        [0, 0, 0, 0],
        env::var("PORT")
            .map(|p| -> u16 { p.as_str().parse::<u16>().unwrap() })
            .unwrap_or(80),
    ));
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

#[derive(Debug, Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

const VALID: &[(&str, &str)] = &[
    ("bob", "sjhd76eww!"),
    ("clem", "khsd54#h"),
    ("alicia", "jhsjhsd222!"),
    ("sue", "76shshs63!"),
    ("plank", "5!ys!hhsds"),
];

async fn login_submit(Form(login): Form<LoginForm>) -> Redirect {
    if VALID.contains(&(login.username.as_str(), login.password.as_str())) {
        if login.username == "plank" {
            Redirect::temporary("/admin")
        } else {
            Redirect::temporary(&format!("/logged_in?username={}", login.username))
        }
    } else {
        Redirect::temporary("/login_fail")
    }
}

async fn admin() -> Html<String> {
    Html(static_file(["html", "admin.html"].into_iter().collect()).await)
}

async fn login_fail() -> Html<String> {
    Html(static_file(["html", "login_fail.html"].into_iter().collect()).await)
}

#[derive(Deserialize)]
struct LoggedInQuery {
    username: String,
}

async fn logged_in(Query(LoggedInQuery { username }): Query<LoggedInQuery>) -> Html<String> {
    Html(
        static_file(["html", "logged_in.html"].into_iter().collect())
            .await
            .replace("REPLACE_USERNAME", &username),
    )
}
