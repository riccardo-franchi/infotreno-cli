use chrono::Local;
use colored::Colorize;
use std::io;
use tabular::{Row, Table};

pub async fn station(
    name: &str,
    print_arrivals: bool,
    print_departures: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: refactor by using a single function throughout the repo to fetch data from the API providing the URL
    let url = format!(
        "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/autocompletaStazione/{}",
        name.trim()
    );

    let res = reqwest::get(url).await?.text().await?;
    let lines: Vec<(&str, &str)> = res
        .lines()
        .map(|l| {
            let mut line_section = l.split('|');
            (line_section.next().unwrap(), line_section.next().unwrap())
        })
        .collect();

    if lines.is_empty() {
        println!("No station found with the name provided.");
        return Ok(());
    }

    let index = if lines.len() > 1 {
        println!(
            "Found more than one station with the name provided. Please select the desired one:"
        );

        lines.iter().enumerate().for_each(|(i, (name, code))| {
            println!("{}. {} ({})", i + 1, name.bold(), code);
        });

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().parse::<usize>()? - 1
    } else {
        0
    };

    if index >= lines.len() {
        return Err("Invalid index.".into());
    }

    let timestamp = Local::now().format("%b %d %Y %H:%M:%S").to_string();

    // If both print_arrivals and print_departures are false, print both
    let (print_arrivals, print_departures) = if !(print_arrivals || print_departures) {
        (true, true)
    } else {
        (print_arrivals, print_departures)
    };

    print_station_arrivals_departures(lines[index].1, &timestamp, print_arrivals, print_departures)
        .await
}

async fn print_station_arrivals_departures(
    station_code: &str,
    timestamp: &str,
    print_arrivals: bool,
    print_departures: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if print_arrivals {
        let url = format!(
            "http://www.viaggiatreno.it/infomobilitamobile/resteasy/viaggiatreno/arrivi/{}/{}",
            station_code, timestamp
        );

        let res = reqwest::get(url).await?.json::<serde_json::Value>().await?;

        let arrivals = res.as_array().unwrap();

        println!("\t----  {}  -----", "Arrivals".bold().green());

        let mut arrivals_table = Table::new("{:<}\t{:<} {:>} {:<}");

        for train in arrivals {
            let train_label = train["compNumeroTreno"].as_str().unwrap();
            let origin = train["origine"].as_str().unwrap();
            let arrival_time = train["compOrarioArrivo"].as_str().unwrap();
            let delay_number = train["ritardo"].as_i64().unwrap_or(0);
            let delay = format!("+{delay_number}");

            arrivals_table.add_row(
                Row::new()
                    .with_cell(train_label.bold())
                    .with_cell(origin)
                    .with_cell(arrival_time)
                    .with_cell(if delay_number != 0 {
                        delay
                    } else {
                        "".to_string()
                    }),
            );
        }
        println!("{arrivals_table}");
    }
    if print_departures {
        let url = format!(
            "http://www.viaggiatreno.it/infomobilitamobile/resteasy/viaggiatreno/partenze/{}/{}",
            station_code, timestamp
        );

        let res = reqwest::get(url).await?.json::<serde_json::Value>().await?;

        let departures = res.as_array().unwrap();

        println!("\t---- {} -----", "Departures".bold().magenta());

        let mut departures_table = Table::new("{:<}\t{:<} {:>} {:<}");

        for train in departures {
            let train_label = train["compNumeroTreno"].as_str().unwrap();
            let destination = train["destinazione"].as_str().unwrap();
            let departure_time = train["compOrarioPartenza"].as_str().unwrap();
            let delay_number = train["ritardo"].as_i64().unwrap_or(0);
            let delay = format!("+{delay_number}");

            departures_table.add_row(
                Row::new()
                    .with_cell(train_label.bold())
                    .with_cell(destination)
                    .with_cell(departure_time)
                    .with_cell(if delay_number != 0 {
                        delay
                    } else {
                        "".to_string()
                    }),
            );
        }

        println!("{departures_table}");
    }

    Ok(())
}
