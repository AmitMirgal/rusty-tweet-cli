use clap::Parser;
use dotenv::dotenv;
use reqwest;
use std::env;
use serde_json::json;
use std::error::Error;
use serde::Deserialize;
use dialoguer::Input;
use dialoguer::Select;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    message: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletion {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from the .env file
    dotenv().ok();

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--version" {
        let version_text = env!("CARGO_PKG_VERSION").truecolor(0, 255, 136).to_string();
        println!("Version: {}", &version_text);
    } else {
        let pb = ProgressBar::new_spinner();

        let prompt_text = "Enter your message?".yellow().to_string();
        let message: String = Input::new()
                .with_prompt(&prompt_text)
                .interact_text()
                .unwrap();

        let tone = vec!["emotional", "energetic", "professional", "funny", "stadard", "friendly", "empathetic", "enthusiatic",
            "inspirational", "thoughtful", "joyful", "humble"];

        let selection_tone_prompt_text = "What tone do you choose?".blue().to_string();
        let selection_tone = Select::new()
                .with_prompt(&selection_tone_prompt_text)
                .items(&tone)
                .interact()
                .unwrap();

        let tweet_type = vec!["question", "viral", "helpful tip", "fun fact", "educational", "joke", "random"];

        let selection_tweet_type_prompt_text = "What tweet type do you choose?".purple().to_string();
        let selection_tweet_type = Select::new()
                .with_prompt(&selection_tweet_type_prompt_text)
                .items(&tweet_type)
                .interact()
                .unwrap();

        let openai_api_key =  env::var("OPENAI_API_KEY")?;

        // Specify the URL you want to make a POST request to
        let url = "https://api.openai.com/v1/chat/completions";

        // Set your Bearer token
        let bearer_token = openai_api_key;

        // Create a Headers object with custom headers
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("application/json"),
            );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", bearer_token))?,
        );

        // Create a JSON body for the POST request
        let body = json!({
            "model": "gpt-3.5-turbo-0613",
            "messages": [{"role": "user","content": format!("Create a tweet for Twitter with an {} tone and a {} tweet type based on the following text: {}",tone[selection_tone], tweet_type[selection_tweet_type], message)}],
        });

        // Start the tick
        pb.set_message("Waiting for HTTP response...");

        // You can adjust the tick duration
        pb.enable_steady_tick(Duration::from_millis(120));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg}")
                .unwrap()
                // For more spinners check out the cli-spinners project:
                // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
                .tick_strings(&[
                    "▹▹▹▹▹",
                    "▸▹▹▹▹",
                    "▹▸▹▹▹",
                    "▹▹▸▹▹",
                    "▹▹▹▸▹",
                    "▹▹▹▹▸",
                    "▪▪▪▪▪",
                ]),
            );

        // Rust openai library - https://github.com/64bit/async-openai
        // Make the POST request
        let client = reqwest::Client::new();
        let response = client.post(url).headers(headers).json(&body).send().await?;

        // Check if the request was successful (status code 2xx)
        if response.status().is_success() {
            // Get the response body as a string
            let body = response.text().await?;

            let chat_completion: ChatCompletion = serde_json::from_str(body.as_str()).expect("Failed to deserialize JSON");

            let ai_message = &chat_completion.choices[0].message.content;

            pb.finish_with_message("HTTP request successful");
            println!("Your content: {}", ai_message.green().bold());
        } else {
            pb.finish_with_message("HTTP request failed");
            println!("Request failed with status code: {:?}", response.status());
        }
    }

    Ok(())
}
