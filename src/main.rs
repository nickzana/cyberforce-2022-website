use axum::{
    body::{boxed, BoxBody, Bytes, Full},
    extract::{BodyStream, Multipart, Path, Query},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, get_service, post},
    BoxError, Form, Router,
};
use axum_extra::routing::SpaRouter;
use futures::{Stream, TryStreamExt};
use hyper::{header::CONTENT_TYPE, Body, HeaderMap, Request, StatusCode, Uri};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::read_to_string,
    io::{self, Read},
    net::SocketAddr,
    path::PathBuf,
    str::FromStr,
};
use tokio::{
    fs::{write, File, OpenOptions},
    io::{AsyncReadExt, BufWriter},
};
use tokio_util::io::StreamReader;
use tower::ServiceExt;
use tower_http::services::ServeDir;

const UPLOADS_DIRECTORY: &str = "uploads";

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
        .route("/login_fail", get(login_fail))
        .route("/thank_you", get(thank_you))
        .route("/contact_submit", post(contact_submit))
        .route("/download/:filename", get(down));

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

async fn thank_you() -> Html<String> {
    Html(static_file(["html", "thank_you.html"].into_iter().collect()).await)
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
            Redirect::temporary("/admin/")
        } else {
            Redirect::temporary(&format!("/logged_in?username={}", login.username))
        }
    } else {
        Redirect::temporary("/login_fail/")
    }
}

async fn admin() -> Html<String> {
    let mut page = static_file(["html", "admin.html"].into_iter().collect()).await;

    page = page.replace("REPLACE_FTP", get_ftp_view().await.as_str());
    page = page.replace("REPLACE_EMAILS", get_email_view().await.as_str());

    Html(page)
}

async fn get_ftp_view() -> String {
    let path = PathBuf::from_str(UPLOADS_DIRECTORY).unwrap();

    let mut view: String = String::new();

    path.read_dir()
        .unwrap()
        .map(|s| s.unwrap().file_name().to_string_lossy().to_string())
        .filter(|s| !s.ends_with(".cyberforce.json"))
        .for_each(|path| {
            view.push_str(&format!(
                "<li>{path}     <a href=\"/download/{path}\">Download</a></li>"
            ));
        });

    view
}

async fn get_email_view() -> String {
    let path = PathBuf::from_str(UPLOADS_DIRECTORY).unwrap();

    let mut view: String = String::new();

    path.read_dir()
        .unwrap()
        .map(|s| s.unwrap().file_name().to_string_lossy().to_string())
        .filter(|s| s.ends_with(".cyberforce.json"))
        .map(|s| { let mut n = String::new(); std::fs::OpenOptions::new().read(true).open(format!("{}/{}", UPLOADS_DIRECTORY, s)).unwrap().read_to_string(&mut n); n })
        .map(|s| -> ContactFormFields { serde_json::from_str(&s).unwrap() })
        .for_each(|ContactFormFields { name, email, phone }| {
            view.push_str(&format!("<div style = \"margin: 12px; padding: 16px; background-color: cyan; \"><h3>FROM: </h3>{name}<br><h3>EMAIL: </h3>{email}<br><br><h3>PHONE: </h3>{phone}</div>"))
        });

    view
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

#[derive(Serialize, Deserialize)]
struct ContactFormFields {
    name: String,
    email: String,
    phone: String,
}

async fn contact_submit(mut multipart: Multipart) -> Redirect {
    let mut name: String = String::new();
    let mut email: String = String::new();
    let mut phone: String = String::new();
    let mut filename: String = String::new();
    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name().unwrap() {
            "name" => {
                name = field.text().await.unwrap();
                continue;
            }
            "email" => {
                email = field.text().await.unwrap();
                continue;
            }
            "phone" => {
                phone = field.text().await.unwrap();
                continue;
            }
            _ => {}
        };

        filename = format!("{}", field.file_name().unwrap().to_owned());
        stream_to_file(&filename, field).await.unwrap();
    }
    let infofile = format!("{}/{}.cyberforce.json", UPLOADS_DIRECTORY, filename);

    let data = ContactFormFields { name, email, phone };

    write(infofile, serde_json::to_string(&data).unwrap())
        .await
        .unwrap();

    println!("redirecting...");

    Redirect::temporary("/thank_you/")
}

// Save a `Stream` to a file
async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let path = std::path::Path::new(UPLOADS_DIRECTORY).join(path);
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}

async fn common_download(file_name: &String) -> impl IntoResponse {
    let workdir = env::current_dir(); // may crash
    if let Ok(workdir) = workdir {
        let path = format!("{}/{}/{}", workdir.display(), UPLOADS_DIRECTORY, file_name);
        if !std::path::Path::new(&path).exists() {
            return (StatusCode::NOT_FOUND, "file not exists").into_response();
        }
        if let Ok(buff) = read_a_file(&path).await {
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, "application/octet-stream".parse().unwrap());
            let response = Response::new(Full::from(buff));
            let (mut parts, body) = response.into_parts();
            parts.status = StatusCode::OK;
            parts.headers = headers;
            let response = Response::from_parts(parts, body);
            return response.into_response();
        }
    }
    return (StatusCode::NOT_FOUND, "inner error").into_response();
}

pub async fn down(Path(file_name): Path<String>) -> impl IntoResponse {
    // pub async fn down(Path(file_name): Path<String>) -> Response<BoxBody> {
    //println!("filename: {}", &file_name);
    return common_download(&file_name).await.into_response();
}

pub async fn down2(req: Request<Body>) -> impl IntoResponse {
    let file_name = req.uri().path().to_string();
    //println!("filename: {}", &file_name);
    return common_download(&file_name).await.into_response();
}

async fn read_a_file(filename: &String) -> std::io::Result<Vec<u8>> {
    //println!("filename: {}", &filename);
    let mut data = Vec::new();
    let mut file = File::open(&filename).await.expect("no file found");
    file.read_to_end(&mut data)
        .await
        .expect("unable read_to_end");
    return Ok(data);
}
