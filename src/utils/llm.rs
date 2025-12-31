use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::json;

#[derive(serde::Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(serde::Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(serde::Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(serde::Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(serde::Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(serde::Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(serde::Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}


pub fn call_llm_json(system_prompt: &str, user_prompt: &str) -> Result<serde_json::Value> {
    eprintln!("DEBUG: call_llm_json entered");
    // 1. Try OpenAI
    if let Ok(api_key) = get_key("OPENAI_API_KEY", "openai_api_key") {
        eprintln!("DEBUG: Found OpenAI key");
        return call_openai(api_key, system_prompt, user_prompt);
    }
    
    // 2. Try Gemini
    if let Ok(api_key) = get_key("GEMINI_API_KEY", "gemini_api_key") {
        eprintln!("DEBUG: Found Gemini key");
        return call_gemini(api_key, system_prompt, user_prompt);
    }
    
    eprintln!("DEBUG: No keys found");
    Err(anyhow::anyhow!("No API key found. Please set OPENAI_API_KEY or GEMINI_API_KEY using `dev secrets add <key>`"))
}

fn get_key(env_var: &str, secret_key: &str) -> Result<String> {
    match std::env::var(env_var) {
        Ok(k) => Ok(k),
        Err(_) => {
            let entry = keyring::Entry::new("dev-cli", secret_key)?;
            entry.get_password().map_err(|e| anyhow::anyhow!(e))
        }
    }
}

fn call_openai(api_key: String, system_prompt: &str, user_prompt: &str) -> Result<serde_json::Value> {
    let client = Client::new();
    let response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "gpt-4o",
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "response_format": { "type": "json_object" }
        }))
        .send()?
        .json::<OpenAIResponse>()?;
        
    if let Some(choice) = response.choices.first() {
        let content = &choice.message.content;
        serde_json::from_str(content).context("Failed to parse OpenAI JSON response")
    } else {
        Err(anyhow::anyhow!("No response from OpenAI"))
    }
}

fn call_gemini(api_key: String, system_prompt: &str, user_prompt: &str) -> Result<serde_json::Value> {
    let client = Client::new();
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}", api_key);
    
    // Gemini doesn't have a distinct "system" role in the basic API usually, 
    // but we can prepend it to the user prompt for simplicity, or use 'system_instruction' in newer API versions.
    // For simplicity/compatibility:
    let combined_prompt = format!("{}\n\n{}", system_prompt, user_prompt);

    let raw_res = client.post(&url)
        .json(&json!({
            "contents": [{
                "parts": [{"text": combined_prompt}]
            }],
            "generationConfig": {
                "response_mime_type": "application/json"
            }
        }))
        .send()?
        .text()?;
        
    eprintln!("DEBUG: Gemini Raw Response: {}", raw_res);
    
    let response: GeminiResponse = serde_json::from_str(&raw_res)?;

    if let Some(candidate) = response.candidates.first() {
        if let Some(part) = candidate.content.parts.first() {
             serde_json::from_str(&part.text).context("Failed to parse Gemini JSON response")
        } else {
             Err(anyhow::anyhow!("Empty content from Gemini"))
        }
    } else {
        Err(anyhow::anyhow!("No suitable response from Gemini"))
    }
}

