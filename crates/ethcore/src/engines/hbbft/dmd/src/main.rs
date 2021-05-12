mod create_miner;

use clap::{App, AppSettings, SubCommand};
use create_miner::create_miner;

fn main() {
    let matches = App::new("dmd v4 swiss army knife")
        .version("1.0")
        .author("David Forstenlechner <dforsten@gmail.com>")
        .about("Utilities for setting up a dmd v4 node")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("create_miner")
                .about("Creates the keys and config for a new dmd v4 miner"),
        )
        .get_matches();

    if let Some(_) = matches.subcommand_matches("create_miner") {
        create_miner();
    }
}
