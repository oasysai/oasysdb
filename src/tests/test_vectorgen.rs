use crate::vectorgen::*;
use dotenv::dotenv;
use std::env;

/// Setup the test environment.
fn setup_environment() {
    dotenv().ok();
}

/// Get the environment variable by the key.
fn getenv(key: &str) -> String {
    let message = format!("Environment variable not found: {key}");
    env::var(key).expect(&message)
}

fn model_openai() -> OpenAI {
    let api_key = getenv("OPENAI_API_KEY");
    let model = "text-embedding-3-small";
    OpenAI::new(&api_key, model)
}

#[test]
fn openai_create_vector() {
    setup_environment();
    let model = model_openai();

    let content = "OasysDB is awesome!";
    let vector = model.create_vector(content).unwrap();
    assert_eq!(vector.len(), 1536);
}
