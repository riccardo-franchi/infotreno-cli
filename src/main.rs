use std::path::Path;

mod parse_trains;

#[tokio::main]
async fn main() {
    let trains = parse_trains::parse_trains()
        .await
        .expect("An error occurred");

    parse_trains::write_to_file(trains, &Path::new("treni.txt")).expect("could not write to file");
}
