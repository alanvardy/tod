use crate::{color, config::Config};

// Print a debug statement if in verbose mode
pub fn print(config: &Config, text: String) {
    if config.verbose.unwrap_or_default() || config.args.verbose {
        let text = format!("=== DEBUG ===\n{}\n===", text);
        let text = color::debug_string(&text);

        println!("{}", text);
    }
}
