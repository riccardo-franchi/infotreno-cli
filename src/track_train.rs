use std::io;

use chrono::NaiveTime;
use colored::Colorize;
use serde_json::Value;

pub async fn track(
    code: u32,
    index: Option<usize>,
    print_stops: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().parse::<usize>()? - 1
    } else {
        index.unwrap_or(0)
    };

    if index >= lines.len() {
        return Err("Invalid index.".into());
    }

    let mut line_content = lines[index].split('|').nth(1).unwrap().split('-').skip(1);

    let origin_id = line_content.next().unwrap();
    let timestamp = line_content.next().unwrap();

    print_train_track_info(origin_id, code, timestamp, print_stops).await?;

    Ok(())
}

async fn print_train_track_info(
    origin_id: &str,
    code: u32,
    timestamp: &str,
    print_stops: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/andamentoTreno/{}/{}/{}",
        origin_id, code, timestamp
    );

    let res = reqwest::get(url).await?.json::<serde_json::Value>().await?;

    let international_origin = res["origineEstera"].as_str();
    let international_destination = res["destinazioneEstera"].as_str();

    let origin_station = res["origine"].as_str().unwrap_or("--");
    let destination_station = res["destinazione"].as_str().unwrap_or("--");

    let mut itinerary = format!("{} - {}", origin_station.cyan(), destination_station.cyan());
    if let Some(international_origin) = international_origin {
        itinerary = format!("{} - {}", international_origin.cyan(), itinerary);
    }
    if let Some(international_destination) = international_destination {
        itinerary = format!("{} - {}", itinerary, international_destination.cyan());
    }

    let train_label = res["compNumeroTreno"].as_str().unwrap().trim();

    let is_not_departured = res["nonPartito"].as_bool().unwrap_or_default();

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
            "Train {}, {} \nNot yet departured.\nScheduled departure time: {}.",
            train_label.bold(),
            itinerary,
            departure_time
        );
        return Ok(());
    }

    let delay_number = res["ritardo"].as_i64();
    let delay = delay_number.map(|d| {
        if d > 0 {
            format!("+{d}")
        } else {
            d.to_string()
        }
    });
    let last_update_station = res["stazioneUltimoRilevamento"].as_str().unwrap_or("--");
    let last_update_time = format_time(&res["oraUltimoRilevamento"]);

    let stops = res["fermate"].as_array().unwrap();

    let is_arrived = stops.iter().last().expect("No stop found.")["actualFermataType"]
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
        println!("Arrived at destination.");
    } else {
        for f in stops {
            let stop_type = f["actualFermataType"].as_u64().unwrap();

            if stop_type != 0 {
                continue;
            }

            let next_stop = f["stazione"].as_str().unwrap();
            let scheduled_arrival_time = format_time(&f["arrivo_teorico"]);
            let estimated_arrival_time =
                format_estimated_time(&f["arrivo_teorico"], delay_number.unwrap_or(0));

            println!(
                "\nNext stop: {}\n\tScheduled arrival time: {}\n\tEstimated arrival time: {}",
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

fn print_stops_info(_stops: &Vec<Value>, _delay: Option<i64>) {}

fn format_estimated_time(time: &Value, delay: i64) -> String {
    const MICROSECONDS_PER_MINUTE: i64 = 60_000;

    parse_time(
        time.as_i64()
            .map(|t| (t + MICROSECONDS_PER_MINUTE * delay) as u64),
    )
    .map(|t| t.format("%H:%M").to_string())
    .unwrap_or("--:--".to_string())
}

fn format_time(time: &Value) -> String {
    parse_time(time.as_u64())
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or("--:--".to_string())
}

fn parse_time(time: Option<u64>) -> Option<NaiveTime> {
    const SECONDS_PER_DAY: u32 = 86400;
    const SECONDS_PER_2_HOURS: u32 = 7200;

    time.map(|t| {
        NaiveTime::from_num_seconds_from_midnight_opt(
            ((t / 1000) as u32 + SECONDS_PER_2_HOURS) % SECONDS_PER_DAY,
            0,
        )
        .unwrap()
    })
}
