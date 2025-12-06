use tracing::debug;

pub fn greet(name: &str) -> String {
    debug!(name = name, "Greeting user");
    format!("Hello, {}! from core", name)
}