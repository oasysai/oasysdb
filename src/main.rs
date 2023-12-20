use dotenv::dotenv;

fn main() {
    // Load environment variables from .env file.
    // This is only needed for development.
    dotenv().ok();
}
