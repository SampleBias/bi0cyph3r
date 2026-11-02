//! Interactive terminal workbench.
use clap::Parser;

#[derive(Parser)]
#[command(name = "bi0cyph3r", version, about = "A local terminal DNA workbench")]
struct Options {
    /// Starting palette; the home screen also has a theme picker.
    #[arg(long, value_enum, default_value = "original", env = "BIOCYPHER_THEME")]
    theme: biocypher_backend::tui::Theme,
}

fn main() -> std::process::ExitCode {
    match biocypher_backend::tui::run_with_theme(Options::parse().theme) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
