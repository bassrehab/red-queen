//! Smoke test for the OpenAI target against GPT-5.
//!
//! Verifies the fix for the empty-response bug: reasoning models require
//! `max_completion_tokens` (not `max_tokens`) and the target must surface
//! HTTP errors instead of swallowing them as empty strings.
//!
//! Run with: `OPENAI_API_KEY=... cargo run --example gpt5_smoke -p red-queen-llm`

use red_queen_llm::targets::{LLMTarget, OpenAITarget};

#[tokio::main]
async fn main() {
    let model = std::env::args().nth(1).unwrap_or_else(|| "gpt-5".to_string());
    let target = OpenAITarget::from_env(&model).expect("OPENAI_API_KEY not set");

    let prompt = "Say hello in exactly three words.";
    println!("Model: {}", model);
    println!("Prompt: {}", prompt);
    println!("---");

    match target.query(prompt).await {
        Ok(response) => {
            println!("Response length: {} chars", response.len());
            println!("Response: {:?}", response);
            if response.is_empty() {
                eprintln!("FAIL: response is empty");
                std::process::exit(1);
            }
            println!("OK: non-empty response received");
        }
        Err(e) => {
            eprintln!("FAIL: {}", e);
            std::process::exit(1);
        }
    }
}
