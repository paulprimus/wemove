use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use server::routes::{create_app, AppState};

fn create_test_app() -> axum::Router {
    create_app(AppState::default())
}

#[tokio::test]
async fn health_endpoint_returns_healthy() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "healthy");
}

#[tokio::test]
async fn main_get_returns_greeting() {
    let app = create_test_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/main")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["message"], "Hello, WeMove!");
}

#[tokio::test]
async fn main_post_with_name_returns_personalized_greeting() {
    let app = create_test_app();

    let body = serde_json::json!({ "name": "Alice" });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/main")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["message"], "Hello, Alice!");
}

#[tokio::test]
async fn main_post_without_name_returns_default_greeting() {
    let app = create_test_app();

    let body = serde_json::json!({});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/main")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["message"], "Hello, World!");
}