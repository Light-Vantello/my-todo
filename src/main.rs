mod handlers;
mod repositories;

use crate::repositories::{
    TodoRepository,
    TodoRepositoryForMemory,
};
use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use handlers::{all_todo, create_todo, delete_todo, find_todo, update_todo};
use std::net::SocketAddr;
use std::{
    env,
    sync::Arc,
};

#[tokio::main]
async fn main() {
    // loggingの初期化
    let log_level = env::var("RUST_LOG").unwrap_or("into".to_string());

    env::set_var("RUST_LOG", log_level);

    tracing_subscriber::fmt::init();

    let repository = TodoRepositoryForMemory::new();
    let app = create_app(repository);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<T: TodoRepository>(repository: T) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/todos", post(create_todo::<T>)
            .get(all_todo::<T>))
        .route("/todos/:id",
                get(find_todo::<T>)
                .delete(delete_todo::<T>)
                .patch(update_todo::<T>),)
        .layer(Extension(Arc::new(repository)))
}


async fn root() -> &'static str {
    "Hello, World!"
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::repositories::{CreateTodo, Todo};
    use axum::response::Response;
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
    };
    use tower::ServiceExt;

    fn build_todo_req_with_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }

    fn build_todo_req_with_empty(method: Method, path: &str) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .body(Body::empty())
            .unwrap()
    }

    async fn res_to_todo(res: Response) -> Todo {
        let bytes = hyper::body::to_bytes(res.into_body())
            .await.unwrap();
        
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();

        let todo: Todo = serde_json::from_str(&body).expect(&format!("cannot convert Todo instance. body: {}", body));
        
        todo
    }

    #[tokio::test]
    async fn should_cerated_todo() {
        let expected = Todo::new(1, "should_return_created_todo".to_string());

        let repository = TodoRepositoryForMemory::new();

        let req = build_todo_req_with_json(
            "/todos",
            Method::POST,
            r#"{ "text": "shold_return_created_todo" }"#.to_string(),
        );

        let res = create_app(repository).oneshot(req).await.unwrap();

        let todo = res_to_todo(res).await;

        assert_eq!(expected, todo);
    }
}
