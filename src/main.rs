use parse_trains::{Stop, Train};
use std::path::Path;

mod parse_trains;
mod plot;
mod stations;

#[tokio::main]
async fn main() {
    let trains = parse_trains::parse_trains()
        .await
        .expect("An error occurred");

    parse_trains::write_to_file(&trains, &Path::new("treni.txt")).expect("could not write to file");

    let station_names: Vec<&str> = stations::STATION_DECAMETERS
        .iter()
        .map(|&(name, _)| name)
        .collect();

    let filtered_trains: Vec<Train> = trains
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

    plot::plot_trains(&filtered_trains[5]);
}
