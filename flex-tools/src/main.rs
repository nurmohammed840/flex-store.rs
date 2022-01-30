use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(version, long_about = None)]
struct Cli {
	#[clap(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {}

fn main() {
	let _cli = Cli::parse();

	// match &cli.command {
	// 	_ => {}
	// }
}
