
use axum::{
    routing::{get,post},
    Router,
    extract::Query,
    response::{IntoResponse},
    extract::{DefaultBodyLimit, Multipart},
    http::{StatusCode,header, HeaderMap, HeaderValue},
    body::{StreamBody}, Json,


};

use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;

use serde::Deserialize;

use tokio_util::io::ReaderStream;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(search))
        // `POST /users` goes to `create_user`
        .route("/send_file", post(send_file))

        .route("/get_file", get(recive_file))
        .layer(DefaultBodyLimit::disable());
     // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:2999".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();// run our app with hyper, listening globally on port 3000
}
#[derive(Deserialize)]
struct InputParams {
    filename: String
}

async fn search(Query(payload): Query<InputParams>) ->Result<([(axum::http::HeaderName, &'static str); 2], Json<Vec<String>>), StatusCode>{
    let mut names = Vec::new();
    for name in WalkDir::new(".\\").into_iter().filter_map(|e| {e.ok()}){
        if name.path().display().to_string().contains(&payload.filename) && name.metadata().unwrap().is_file() {
            names.push(name.clone().path().display().to_string())
        }
    }
    let headers = [
        (header::CONTENT_TYPE, "application/json; charset=utf-8"),
        (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
    ];
    Ok((headers, Json(names)))
}

async fn send_file (mut multipart: Multipart) -> (HeaderMap, &'static str) {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.file_name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let mut file = tokio::fs::File::create(name).await.unwrap();
        file.write_all(&data).await.unwrap();
    }
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    (headers,"Completed")
}

async fn recive_file (Query(payload): Query<InputParams>) -> impl IntoResponse{
    let mut combiner: String = payload.filename.clone();
    combiner.insert_str(0, ".\\");
    let file = match tokio::fs::File::open(combiner).await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let mut headers = HeaderMap::new();

    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("multipart/form-data; charset=utf-8"));
    headers.insert(header::CONTENT_DISPOSITION, HeaderValue::from_str(&format!("attachment; filename={}", payload.filename)).unwrap());
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    Ok((headers, body))

}