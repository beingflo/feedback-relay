use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::{BoxError, ServiceBuilder};

#[derive(Serialize)]
struct EmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    text: String,
}

#[derive(Serialize, Deserialize)]
struct FeedbackRequest {
    project: String,
    path: String,
    email: String,
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let app = Router::new()
        .route("/feedback", post(email_handler))
        .route("/health", get(health_handler))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(4, Duration::from_secs(60))),
        );

    axum::Server::bind(&"0.0.0.0:8008".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn email_handler(Json(body): Json<FeedbackRequest>) -> StatusCode {
    match send_email(body.project, body.path, body.email, body.content).await {
        Ok(response) => {
            println!("Email sent with status: {}", response.status());
            if response.status() == 200 {
                StatusCode::OK
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
        Err(error) => {
            println!("Email sent with error: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn health_handler() -> StatusCode {
    StatusCode::OK
}

async fn send_email(
    project: String,
    path: String,
    author_email: String,
    content: String,
) -> Result<reqwest::Response, reqwest::Error> {
    let api_key = std::env::var("RESEND_API_KEY").expect("Resend api key must be set");

    let request = EmailRequest {
        from: "info@feedback.marending.dev".to_string(),
        to: vec!["florian@marending.dev".to_string()],
        subject: format!("Feedback: {} - {}", project, path),
        text: format!(
            "{}\n\n{}",
            if author_email.is_empty() {
                "Anonymous"
            } else {
                &author_email
            },
            content
        ),
    };

    let client = reqwest::Client::new();

    client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
}
