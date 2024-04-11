use chrono::{Datelike, Utc};
use colored::Colorize;
use parse_trains::{Stop, Train};
use std::{env, fs, path::Path};
use track_train::track;

mod parse_trains;
mod plot;
mod stations;
mod track_train;

async fn plot() {
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

#[tokio::main]
async fn main() {
    let mut args_iter = env::args().skip(1);

    let help_message = format!(
        "Usage. Available commands:
        {0} {1} 
            print information about train punctuality.
            Note: if a given {1} corresponds to multiple trains, a prompt will ask to choose one.

        {2}
            plot currently circulating trains between SAVONA and VENTIMIGLIA.

        {3}
            view this help message.",
        "t, track".bold(),
        "<train_code>".underline(),
        "p, plot".bold(),
        "h, help".bold()
    );

    if let Some(command) = args_iter.next() {
        match command.as_str() {
            "track" | "t" => {
                if let Some(code) = args_iter.next() {
                    let code = code.parse::<u32>().expect("Invalid train code.");
                    track(code).await.expect("An error occurred.");
                } else {
                    println!("Please provide a train code.");
                }
            }
            "plot" | "p" => {
                plot().await;
            }
            "help" | "h" => {
                println!("{help_message}");
            }
            _ => {
                println!("Unknown command, run with 'help' to see available commands.");
            }
        }
    } else {
        println!("{help_message}");
    }
}
