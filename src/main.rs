use clap::{Parser, Subcommand};

mod cli_input;
mod news;
mod station;
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
    /// track a train by its code.
    /// Note: if a certain train code corresponds to multiple trains, you will be asked to choose one
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
        /// watch mode: refresh tracking data every minute
        #[clap(short, long)]
        #[arg(default_value_t = false)]
        watch: bool,
    },
    /// find arrival and departure times of trains at a certain station.
    /// It is possible to search for a station by the beginning of its name; a prompt will ask to choose the desired station
    Station {
        /// station name or station code (e.g. "Milano Centrale" or "S01700")
        station: String,
        /// print only arrivals.
        /// Note: if neither arrivals nor departures are specified, both will be printed
        #[clap(long)]
        #[arg(default_value_t = false)]
        arrivals: bool,
        /// print only departures
        #[clap(long)]
        #[arg(default_value_t = false)]
        departures: bool,
    },
    // get information about line disruptions from Trenitalia
    News {
        /// verbose mode: print all news in expanded form.
        /// Default is to print only the titles and prompt user to select a news item to expand
        #[clap(short, long)]
        #[arg(default_value_t = false)]
        verbose: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Track {
            code,
            index,
            stops,
            watch,
        } => {
            track_train::track(code, index, stops, watch)
                .await
                .expect("An error occurred");
        }
        Commands::Station {
            station,
            arrivals,
            departures,
        } => {
            station::station(&station, arrivals, departures)
                .await
                .expect("An error occurred");
        }
        Commands::News { verbose } => {
            news::print_news(verbose).await.expect("An error occurred");
        }
    }
}
