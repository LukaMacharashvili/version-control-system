pub mod handlers;
pub mod utils;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = &args[1];

    match command.as_str() {
        "init" => handlers::init().unwrap(),
        "commit" => {
            let description = &args[2];
            handlers::commit(description).unwrap();
        }
        "view" => {
            let branch_id = &args[2];
            handlers::view(branch_id).unwrap();
        }
        _ => println!("Unknown command"),
    }
}
