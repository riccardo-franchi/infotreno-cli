use chrono::{Datelike, Utc};
use clap::{Parser, Subcommand};
use parse_trains::{Stop, Train};
use std::{fs, path::Path};

mod parse_trains;
mod plot;
mod stations;
mod track_train;

#[derive(Parser)]
#[command(version, about, long_about=None)]
#[command(next_line_help = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// track a train by its code
    /// Note: if a certain train code corresponds to multiple trains, you will be asked to choose one.
    Track {
        /// train code
        code: u32,
        /// index of the train to track, useful when the code corresponds to multiple trains
        #[clap(short, long)]
        index: Option<usize>,
        /// print all the train stops (verbose)
        #[clap(short, long)]
        #[arg(default_value_t = false)]
        stops: bool,
    },
    /// plot circulating trains between SAVONA and VENTIMIGLIA
    Plot,
}

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

    parse_trains::write_to_file(&trains, Path::new(&(filename.clone() + ".txt")))
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
        if filtered_trains[i].stops.is_empty() {
            filtered_trains.remove(i);
        }
    }

    plot::plot_trains(&filtered_trains, Path::new(&(filename + ".png")));
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Track {
            code,
            index,
            stops: _,
        } => {
            track_train::track(code, index)
                .await
                .expect("An error occurred");
        }
        Commands::Plot => {
            plot().await;
        }
    }
}
