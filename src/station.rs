use chrono::Local;
use colored::Colorize;
use regex::Regex;
use tabular::{row, Table};

use crate::cli_input;

pub async fn station(
    name: &str,
    print_arrivals: bool,
    print_departures: bool,
    filter: Option<&str>,
) -> Result<(), reqwest::Error> {
    let timestamp = Local::now().format("%b %d %Y %H:%M:%S").to_string();

    // If both print_arrivals and print_departures are false, print both
    let (print_arrivals, print_departures) = if !(print_arrivals || print_departures) {
        (true, true)
    } else {
        (print_arrivals, print_departures)
    };

    let re = Regex::new(r"S[0-9]{5}").unwrap();

    if re.is_match(name) {
        return print_station_arrivals_departures(
            name,
            &timestamp,
            print_arrivals,
            print_departures,
            filter,
        )
        .await;
    }

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

        cli_input::get_index() - 1
    } else {
        0
    };

    if index >= lines.len() {
        eprintln!("Invalid index.");
        return Ok(());
    }

    print_station_arrivals_departures(
        lines[index].1,
        &timestamp,
        print_arrivals,
        print_departures,
        filter,
    )
    .await
}

async fn print_station_arrivals_departures(
    station_code: &str,
    timestamp: &str,
    print_arrivals: bool,
    print_departures: bool,
    filter: Option<&str>,
) -> Result<(), reqwest::Error> {
    let filter_train_type = |train: &serde_json::Value| {
        if filter.is_none() {
            return true;
        }

        let train_type = train["categoriaDescrizione"].as_str().unwrap_or_default();
        filter.unwrap().trim().to_lowercase() == train_type.trim().to_lowercase()
    };

    if print_arrivals {
        let url = format!(
            "http://www.viaggiatreno.it/infomobilitamobile/resteasy/viaggiatreno/arrivi/{}/{}",
            station_code, timestamp
        );

        let res = reqwest::get(url).await?.json::<serde_json::Value>().await?;

        let arrivals = res.as_array().unwrap();

        println!("\t----  {}  -----", "Arrivals".bold().green());

        let mut arrivals_table = Table::new("{:<}  {:<} {:>} {:<}  {:<}");

        for train in arrivals.iter().filter(|t| filter_train_type(t)) {
            let train_label = train["compNumeroTreno"].as_str().unwrap();
            let origin = train["origine"].as_str().unwrap();
            let arrival_time = train["compOrarioArrivo"].as_str().unwrap();
            let delay_number = train["ritardo"].as_i64().unwrap_or(0);
            dbg!(delay_number);
            let delay = if delay_number > 0 {
                format!("+{delay_number}")
            } else if delay_number < 0 {
                delay_number.to_string()
            } else {
                "".to_string()
            };

            let scheduled_platform = train["binarioProgrammatoArrivoDescrizione"]
                .as_str()
                .unwrap_or("")
                .trim();

            let actual_platform = train["binarioEffettivoArrivoDescrizione"]
                .as_str()
                .unwrap_or("")
                .trim();

            let platform = if actual_platform.is_empty() {
                scheduled_platform.to_string()
            } else {
                actual_platform.green().to_string()
            };

            arrivals_table.add_row(row!(
                train_label.bold(),
                origin,
                arrival_time,
                delay,
                platform
            ));
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

        let mut departures_table = Table::new("{:<}  {:<} {:>} {:<}  {:<}");

        for train in departures.iter().filter(|t| filter_train_type(t)) {
            let train_label = train["compNumeroTreno"].as_str().unwrap();
            let destination = train["destinazione"].as_str().unwrap();
            let departure_time = train["compOrarioPartenza"].as_str().unwrap();
            let delay_number = train["ritardo"].as_i64().unwrap_or(0);
            let delay = if delay_number > 0 {
                format!("+{delay_number}")
            } else if delay_number < 0 {
                delay_number.to_string()
            } else {
                "".to_string()
            };

            let scheduled_platform = train["binarioProgrammatoPartenzaDescrizione"]
                .as_str()
                .unwrap_or("")
                .trim();

            let actual_platform = train["binarioEffettivoPartenzaDescrizione"]
                .as_str()
                .unwrap_or("")
                .trim();

            let platform = if actual_platform.is_empty() {
                scheduled_platform.to_string()
            } else {
                actual_platform.green().to_string()
            };

            departures_table.add_row(row!(
                train_label.bold(),
                destination,
                departure_time,
                delay,
                platform
            ));
        }

        println!("{departures_table}");
    }

    Ok(())
}
