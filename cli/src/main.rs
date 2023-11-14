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
        .subcommand(
            Command::new("clone")
                .about("Clones the remote repository")
                .arg(
                    Arg::new("bucket")
                        .required(true)
                        .short('n')
                        .help("Repository S3 bucket bucket"),
                ),
        )
        .subcommand(
            Command::new("set-remote")
                .about("Sets the remote repository bucket")
                .arg(
                    Arg::new("bucket")
                        .required(true)
                        .short('n')
                        .help("Repository S3 bucket bucket"),
                ),
        )
        .subcommand(Command::new("pull").about("Pulls the changes"))
        .subcommand(Command::new("push").about("Syncs the changes"))
}

#[::tokio::main]
async fn main() {
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_s3::Client::new(&config);

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
        Some(("clone", sub_matches)) => {
            let bucket_name = sub_matches.get_one::<String>("bucket");
            handlers::clone(&client, bucket_name.unwrap_or(&"".to_owned()))
                .await
                .unwrap();
        }
        Some(("pull", _)) => {
            // TODO: Currently it will overwrite the local unpushed commits
            // TODO: Implement push
        }
        Some(("push", _)) => {
            // TODO: Currently it will only push if there are no pushed commits in the remote repository that is not in the local repository
            // TODO: Implement push
        }
        Some(("set-remote", sub_matches)) => {
            let bucket_name = sub_matches.get_one::<String>("bucket");
            handlers::set_remote(bucket_name.unwrap_or(&"".to_owned())).unwrap();
        }
        _ => unreachable!(),
    }
}
