use trill::args::CliArgs;
use trill::cli::Cli;

fn main() {
    let args = CliArgs::parse();
    let cli = Cli::new(args);
}
