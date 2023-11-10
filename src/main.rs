pub mod files;
pub mod handlers;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = &args[1];

    match command.as_str() {
        "init" => handlers::init().unwrap(),
        "commit" => {
            let description = &args[2];
            handlers::commit(description).unwrap();
        }
        _ => println!("Unknown command"),
    }
}
