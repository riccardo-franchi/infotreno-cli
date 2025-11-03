use std::time::Duration;

use chrono::{Local, NaiveTime, Offset, TimeZone};
use chrono_tz::Europe::Rome;
use colored::Colorize;
use serde_json::Value;

use crate::cli_input;

pub async fn track(
    code: u32,
    index: Option<usize>,
    print_stops: bool,
    auto_refresh: bool,
) -> Result<(), reqwest::Error> {
    let url = format!(
        "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/cercaNumeroTrenoTrenoAutocomplete/{}",
        code
    );

    let res = reqwest::get(url).await?.text().await?;

    let lines: Vec<_> = res.lines().collect();

    if lines.is_empty() {
        println!("No train found with the code provided.");
        return Ok(());
    }

    let index = if lines.len() > 1 && index.is_none() {
        println!("Found more than one train with selected code. Please select the desired one:");

        lines.clone().into_iter().enumerate().for_each(|(i, l)| {
            let l = l.split('|').next().unwrap();
            println!("{}. {}", i + 1, l);
        });

        cli_input::get_index() - 1
    } else {
        index.unwrap_or(0)
    };

    if index >= lines.len() {
        eprintln!("Invalid index.");
        return Ok(());
    }

    let mut line_content = lines[index].split('|').nth(1).unwrap().split('-').skip(1);

    let origin_id = line_content.next().unwrap();
    let timestamp = line_content.next().unwrap();

    if auto_refresh {
        loop {
            print_train_track_info(origin_id, code, timestamp, print_stops, true).await?;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    print_train_track_info(origin_id, code, timestamp, print_stops, false).await?;

    Ok(())
}

async fn print_train_track_info(
    origin_id: &str,
    code: u32,
    timestamp: &str,
    print_stops: bool,
    is_watch_mode: bool,
) -> Result<(), reqwest::Error> {
    let url = format!(
        "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/andamentoTreno/{}/{}/{}",
        origin_id, code, timestamp
    );

    let res = reqwest::get(url).await?.json::<serde_json::Value>().await?;

    // Clearing console after new request occurs
    // With this approach, old tracking data is erased once new data is fetched, avoiding clearing the console and showing blank screen while waiting for new response, with slow connections.
    if is_watch_mode {
        //Clear console
        print!("\x1B[2J\x1B[1;1H");
        println!(
            "{}",
            "Watch mode: refreshing every minute. Press Ctrl+C to exit.".dimmed()
        );
    }

    let international_origin = res["origineEstera"].as_str();
    let international_destination = res["destinazioneEstera"].as_str();

    let origin_station = res["origine"].as_str().unwrap_or("--");
    let destination_station = res["destinazione"].as_str().unwrap_or("--");

    let mut itinerary = format!("{} - {}", origin_station.cyan(), destination_station.cyan());
    if let Some(international_origin) = international_origin
        && international_origin != origin_station
    {
        itinerary = format!("{} - {}", international_origin.cyan(), itinerary);
    }
    if let Some(international_destination) = international_destination
        && international_destination != destination_station
    {
        itinerary = format!("{} - {}", itinerary, international_destination.cyan());
    }

    let train_label = res["compNumeroTreno"].as_str().unwrap().trim();

    let is_canceled = res["provvedimento"].as_u64().unwrap_or_default() == 1;

    if is_canceled {
        println!(
            "Train {}, {} \n{}\n",
            train_label.bold(),
            itinerary,
            "Canceled.".bright_red()
        );
        return Ok(());
    }

    let is_not_departured = res["nonPartito"].as_bool().unwrap_or_default();

    let stops = res["fermate"].as_array().unwrap();

    let delay_number = res["ritardo"].as_i64();
    let delay = delay_number.map(|d| {
        if d > 0 {
            format!("+{d}")
        } else {
            d.to_string()
        }
    });

    if is_not_departured {
        let departure_time = if international_origin.is_some() {
            format_time(&res["oraPartenzaEstera"])
        } else {
            res["compOrarioPartenza"]
                .as_str()
                .unwrap_or("--:--")
                .to_string()
        };

        println!(
            "Train {}, {} \nNot yet departured.\nScheduled departure time: {}.\n",
            train_label.bold(),
            itinerary,
            departure_time
        );
        if print_stops {
            print_stops_info(stops, delay_number);
        }
        return Ok(());
    }

    let last_update_station = res["stazioneUltimoRilevamento"].as_str().unwrap_or("--");
    let last_update_time = format_time(&res["oraUltimoRilevamento"]);

    let is_arrived = !stops.is_empty()
        && stops.iter().last().unwrap()["actualFermataType"]
            .as_u64()
            .unwrap()
            == 1;

    println!(
        "Train {}, {} \nLast update ({}):\n\t{}, {}",
        train_label.bold(),
        itinerary,
        last_update_time,
        last_update_station.cyan(),
        delay.unwrap_or("--".to_string()).bold()
    );

    if is_arrived {
        println!("Arrived at destination.\n");
    } else {
        for stop in stops {
            let stop_type = stop["actualFermataType"].as_u64().unwrap();

            if stop_type != 0 {
                continue;
            }

            let next_stop = stop["stazione"].as_str().unwrap();
            let scheduled_arrival_time = format_time(&stop["arrivo_teorico"]);
            let estimated_arrival_time =
                format_estimated_time(&stop["arrivo_teorico"], delay_number.unwrap_or(0));

            println!(
                "\nNext stop: {}\n\tScheduled arrival time: {}\n\tEstimated arrival time: {}\n",
                next_stop.cyan(),
                scheduled_arrival_time,
                estimated_arrival_time,
            );
            break;
        }
    }

    if print_stops {
        print_stops_info(stops, delay_number);
    }

    Ok(())
}

fn print_stops_info(stops: &[Value], delay: Option<i64>) {
    print!("Stops:");

    for (index, stop) in stops.iter().enumerate() {
        let stop_type = stop["actualFermataType"].as_u64().unwrap();

        let station = stop["stazione"].as_str().unwrap();

        let scheduled_platform = stop["binarioProgrammatoArrivoDescrizione"]
            .as_str()
            .unwrap_or_else(|| {
                stop["binarioProgrammatoPartenzaDescrizione"]
                    .as_str()
                    .unwrap_or("--")
            });

        let actual_platform = stop["binarioEffettivoArrivoDescrizione"]
            .as_str()
            .unwrap_or_else(|| {
                stop["binarioEffettivoPartenzaDescrizione"]
                    .as_str()
                    .unwrap_or("--")
            });

        let platform = if actual_platform == "--" {
            scheduled_platform.to_string()
        } else {
            actual_platform.green().to_string()
        };

        let scheduled_arrival_time = format_time(&stop["arrivo_teorico"]);
        let scheduled_departure_time = format_time(&stop["partenza_teorica"]);

        if stop_type != 0 {
            let actual_arrival_time = format_time(&stop["arrivoReale"]);
            let actual_departure_time = format_time(&stop["partenzaReale"]);

            println!("\n{} - platform {}", station.green(), platform);
            if index != 0 {
                println!(
                    "\tScheduled arrival time:   {} - actual: {}",
                    scheduled_arrival_time,
                    actual_arrival_time.bold(),
                );
            }
            if index != stops.len() - 1 {
                println!(
                    "\tScheduled departure time: {} - actual: {}",
                    scheduled_departure_time,
                    actual_departure_time.bold()
                );
            }
        } else {
            let estimated_arrival_time =
                format_estimated_time(&stop["arrivo_teorico"], delay.unwrap_or(0));
            let estimated_departure_time =
                format_estimated_time(&stop["partenza_teorica"], delay.unwrap_or(0));

            println!("\n{} - platform {}", station, platform);
            if index != 0 {
                println!(
                    "\tScheduled arrival time:   {} - estimated: {}",
                    scheduled_arrival_time,
                    estimated_arrival_time.bold(),
                );
            }
            if index != stops.len() - 1 {
                println!(
                    "\tScheduled departure time: {} - estimated: {}",
                    scheduled_departure_time,
                    estimated_departure_time.bold()
                );
            }
        }
    }

    println!();
}

fn format_time(time: &Value) -> String {
    parse_time(time.as_u64())
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or("--:--".to_string())
}

fn format_estimated_time(time: &Value, delay: i64) -> String {
    const MICROSECONDS_PER_MINUTE: i64 = 60_000;

    parse_time(
        time.as_i64()
            .map(|t| (t + MICROSECONDS_PER_MINUTE * delay) as u64),
    )
    .map(|t| t.format("%H:%M").to_string())
    .unwrap_or("--:--".to_string())
}

fn parse_time(time: Option<u64>) -> Option<NaiveTime> {
    const SECONDS_PER_DAY: u32 = 86400;

    let italian_timezone_offset = Rome
        .offset_from_utc_datetime(&Local::now().naive_utc())
        .fix()
        .local_minus_utc() as u32;

    time.map(|t| {
        NaiveTime::from_num_seconds_from_midnight_opt(
            ((t / 1000) as u32 + italian_timezone_offset) % SECONDS_PER_DAY,
            0,
        )
        .unwrap()
    })
}
