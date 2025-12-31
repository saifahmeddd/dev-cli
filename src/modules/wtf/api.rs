use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use crate::utils::env;

const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";

#[derive(Serialize)]
struct MessageRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    text: String,
}

pub async fn explain_error(error_msg: &str) -> Result<String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    
    if api_key.is_empty() {
        return Ok("No ANTHROPIC_API_KEY found. Please set it to use the Error Explainer.".to_string());
    }

    let client = reqwest::Client::new();
    let request = MessageRequest {
        model: "claude-3-opus-20240229".to_string(),
        max_tokens: 1024,
        messages: vec![
            Message {
                role: "user".to_string(),
                content: format!("Explain this error and provide a fix:\n\n{}", error_msg),
            }
        ],
    };

    let response = client.post(CLAUDE_API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await?;
        anyhow::bail!("API Error: {}", error_text);
    }

    let resp_body: MessageResponse = response.json().await?;
    if let Some(content) = resp_body.content.first() {
        Ok(content.text.clone())
    } else {
        Ok("No explanation returned.".to_string())
    }
}
