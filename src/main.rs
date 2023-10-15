use chrono::{Datelike, Utc};
use parse_trains::{Stop, Train};
use std::{fs, path::Path};

mod parse_trains;
mod plot;
mod stations;

#[tokio::main]
async fn main() {
    println!("Parsing trains...");

    let trains = parse_trains::parse_trains()
        .await
        .expect("An error occurred");

    println!("Writing to file...");

    let date = Utc::now().date_naive();
    let day = date.day();
    let month = date.month();
    let year = date.year();

    fs::create_dir_all(Path::new("data")).expect("could not create data directory");
    let filename = format!("data/treni_{}_{}_{}", day, month, year);

    parse_trains::write_to_file(&trains, &Path::new(&(filename.clone() + ".txt")))
        .expect("could not write to file");

    println!("Plotting...");

    let station_names: Vec<&str> = stations::STATION_DECAMETERS
        .iter()
        .map(|&(name, _)| name)
        .collect();

    let mut filtered_trains: Vec<Train> = trains
        .into_iter()
        .map(|t| {
            let filtered_stops: Vec<Stop> = t
                .stops
                .into_iter()
                .filter(|s| station_names.contains(&s.station_name.as_str()))
                .collect();

            Train {
                stops: filtered_stops,
                ..t
            }
        })
        .collect();

    for i in (0..filtered_trains.len()).rev() {
        if filtered_trains[i].stops.len() == 0 {
            filtered_trains.remove(i);
        }
    }

    plot::plot_trains(&filtered_trains, &Path::new(&(filename + ".png")));
}
