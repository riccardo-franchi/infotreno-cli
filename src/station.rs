use chrono::Local;
use colored::Colorize;
use std::io;

pub async fn station(name: &str) -> Result<(), Box<dyn std::error::Error>> {
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

    print_station_arrivals_departures(lines[index].1, &timestamp).await
}

async fn print_station_arrivals_departures(
    station_code: &str,
    timestamp: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "http://www.viaggiatreno.it/infomobilitamobile/resteasy/viaggiatreno/arrivi/{}/{}",
        station_code, timestamp
    );

    let res = reqwest::get(url).await?.json::<serde_json::Value>().await?;

    let arrivals = res.as_array().unwrap();

    let url = format!(
        "http://www.viaggiatreno.it/infomobilitamobile/resteasy/viaggiatreno/partenze/{}/{}",
        station_code, timestamp
    );

    let res = reqwest::get(url).await?.json::<serde_json::Value>().await?;

    let departures = res.as_array().unwrap();

    println!("\t----  {}  -----", "Arrivals".bold().green());

    for train in arrivals {
        let train_label = train["compNumeroTreno"].as_str().unwrap().trim();

        println!("{}", train_label);
    }

    println!("\n\t---- {} -----", "Departures".bold().magenta());

    for train in departures {
        let train_label = train["compNumeroTreno"].as_str().unwrap().trim();

        println!("{}", train_label);
    }

    Ok(())
}
