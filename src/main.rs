use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use tower::buffer::BufferLayer;
use tower::limit::RateLimitLayer;
use tower::load_shed::error::Overloaded;
use tower::load_shed::LoadShedLayer;
use tower::{BoxError, ServiceBuilder};
use tower_http::cors::{AllowHeaders, AllowMethods, CorsLayer};

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
    let origins = vec![
        "https://marending.dev".parse().unwrap(),
        "https://rest.quest".parse().unwrap(),
        "https://go.rest.quest".parse().unwrap(),
        "https://dd.rest.quest".parse().unwrap(),
        "https://jour.rest.quest".parse().unwrap(),
    ];

    let app = Router::new()
        .route("/feedback", post(email_handler))
        .route("/health", get(health_handler))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(BufferLayer::new(1024))
                .layer(LoadShedLayer::new())
                .layer(RateLimitLayer::new(4, Duration::from_secs(60))),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods(AllowMethods::any())
                .allow_headers(AllowHeaders::any()),
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

async fn handle_error(error: BoxError) -> Response {
    if error.is::<Overloaded>() {
        (StatusCode::TOO_MANY_REQUESTS, "Slow down").into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
    }
}
