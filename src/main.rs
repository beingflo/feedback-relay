use dotenv::dotenv;
use serde::Serialize;

#[derive(Serialize)]
struct Request {
    from: String,
    to: Vec<String>,
    subject: String,
    text: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = std::env::var("RESEND_API_KEY").expect("Resend api key must be set");

    let request = Request {
        from: "onboarding@resend.dev".to_string(),
        to: vec!["florian@marending.dev".to_string()],
        subject: "Test from rust".to_string(),
        text: "It works!".to_string(),
    };

    let client = reqwest::Client::new();

    let resp = client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
    println!("{:#?}", resp);
    Ok(())
}
