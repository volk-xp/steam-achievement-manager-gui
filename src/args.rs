use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "sam",
    about = "Steam Achievement Manager",
    long_about = "Unlock or reset Steam achievements for games in your library.\n\nRun with no arguments to pick a game from the window."
)]
pub struct Args {
    /// Open this App ID straight away, e.g. --id 367520
    #[arg(short, long)]
    pub id: Option<u32>,
}

pub fn parse() -> Args {
    Args::parse()
}
