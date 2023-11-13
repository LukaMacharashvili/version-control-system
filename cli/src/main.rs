pub mod handlers;
pub mod utils;
use clap::{Arg, Command};

fn cli() -> Command {
    Command::new("rgit")
        .about("A simple version control system written in Rust")
        .subcommand_required(true)
        .subcommand(Command::new("init").about("Initializes a new repository"))
        .subcommand(
            Command::new("commit").about("Commits the changes").arg(
                Arg::new("description")
                    .required(true)
                    .short('d')
                    .help("commit description"),
            ),
        )
        .subcommand(
            Command::new("view")
                .about("Views the commit")
                .arg(Arg::new("id").help("commit id").required(true).short('i')),
        )
        .subcommand(Command::new("commits").about("Views all the commits"))
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            println!("Initializing a new repository");
            handlers::init().unwrap();
        }
        Some(("commit", sub_matches)) => {
            let description = sub_matches.get_one::<String>("description");
            println!("Committing the changes");
            handlers::commit(description.unwrap_or(&"".to_owned())).unwrap();
        }
        Some(("view", sub_matches)) => {
            let id = sub_matches.get_one::<String>("id");
            handlers::view(&id.unwrap_or(&"".to_owned())).unwrap();
        }
        Some(("commits", _)) => {
            handlers::commits().unwrap();
        }
        _ => unreachable!(),
    }
}
