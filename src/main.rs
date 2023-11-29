// use axum::{
//     extract::Path,
//     routing::{get, post},
//     http::StatusCode,
//     response::IntoResponse,
//     Json, Router,
// };

// async fn api_request(Path(method): Path<String>) -> impl IntoResponse  {
//     format!("call from {method}")
// }

// async fn root() -> impl IntoResponse {
//     "root"
// }

// #[tokio::main]
// async fn main() {
//     let app = Router::new()
//         .route("/api/:method", get(api_request).post(api_request))
//         .route("/", get(root));
    
//     axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
//         .serve(app.into_make_service())
//         .await
//         .unwrap();
// }

use axum::{
    extract::{Form, Extension},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use std::{
    env,
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    // Создаем хранилище для команды выполнения PHP-скрипта
    let php_command: Mutex<Command> = Mutex::new(Command::new("php-cgi"));

    // Создаем обработчик для выполнения PHP-скрипта
    async fn php_handler((form, path): (Form<MyFormData>, Extension<PathBuf>)) -> Html<String> {
        let mut command = Command::new("php-cgi");
        command.arg("-f").arg(&path);
        if let Some(data) = form.into_inner().data {
            let stdin = command.stdin.as_mut().expect("failed to get stdin");
            write!(stdin, "{}", data).expect("failed to write to stdin");
        }
        let output = command.output().expect("failed to execute command");
        let php_output = String::from_utf8_lossy(&output.stdout).into_owned();
        Html(php_output)
    }

    // Задаем путь к каталогу, где находятся PHP-скрипты
    let php_scripts_dir = env::current_dir().unwrap().join("php_scripts");

    // Создаем маршрутизатор сервера
    let router = Router::new()
        .route("/php/{path:.*}", get(php_handler).boxed())
        .route("/static/{filename:.*}", get(static_handler))
        .layer(
            AddExtensionLayer::new(php_scripts_dir)
        );

    // Запускаем сервер
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn static_handler((Extension(dir), path): (Extension<PathBuf>, PathBuf)) -> Result<Vec<u8>, StatusCode> {
    let file_path = dir.join(&path);

    // Проверяем, что файл существует и является файлом (а не директорией)
    if file_path.exists() && file_path.is_file() {
        Ok(tokio::fs::read(file_path).await.unwrap())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
