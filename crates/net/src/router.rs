use axum::{
    routing::get, routing::post,
    Router, handler::Handler
};

use std::fmt::Debug;



pub fn main_router<STATE>(components: Vec<(String, Router<STATE>)>, state: STATE) -> Router
where
    STATE: Clone + Send + Sync + 'static
{
    let mut app = Router::<STATE>::new();

    for (_, router) in components {
        app = app.merge(router);
    }

    app.with_state(state)
}



pub fn get_router_builder<T, S>(
    path: String,
    handler: impl Handler<T, S>  + Clone + Send + 'static
) -> (String, Router<S>)
where
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    let mut app = Router::<S>::new();
    let new_path = path.clone();
    (path, app.route(&new_path, get(handler)))
}


pub fn post_router_builder<T, S>(
    path: String,
    handler: impl Handler<T, S>  + Clone + Send + 'static
) -> (String, Router<S>)
where
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    let mut app = Router::<S>::new();
    let new_path = path.clone();
    (path, app.route(&new_path, post(handler)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::IntoResponse,
        routing::get,
        Router,
    };
    use tower::ServiceExt;
    use std::sync::Arc;

    #[axum::debug_handler]
    async fn hello_handler() -> String {
        "Hello, World!".to_string()
    }

    async fn echo_handler(body: String) -> String {
        body
    }

    #[tokio::test]
    async fn test_get_router_builder() {

        let (path, router) = get_router_builder::<_ ,_>(
            "/hello".to_string(),
            hello_handler
        );
        assert_eq!(path, "/hello");
        let app = Router::new().merge(router);

        let request = Request::builder()
            .uri("/hello")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);



        println!("{:?}", response);
    }

}
