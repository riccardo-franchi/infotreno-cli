use chrono::{Datelike, NaiveTime, Utc};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub enum TrainType {
    REG,
    IC,
    EC,
    AV,
}

#[derive(Debug)]
pub struct Train {
    pub code: u32,
    pub origin_id: String,
    pub train_type: TrainType,
    pub stops: Vec<Stop>,
}

#[derive(Debug)]
pub struct Stop {
    pub station_name: String,
    pub arrival_time: Option<NaiveTime>,
    pub departure_time: Option<NaiveTime>,
    pub actual_arrival_time: Option<NaiveTime>,
    pub actual_departure_time: Option<NaiveTime>,
}

pub async fn parse_trains() -> Result<Vec<Train>, Box<dyn std::error::Error>> {
    let train_codes = parse_numbers().await?;

    let mut trains: Vec<Train> = Vec::new();
    trains.reserve_exact(train_codes.len());

    for code in train_codes {
        let url = format!(
            "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/cercaNumeroTrenoTrenoAutocomplete/{}",
            code
        );

        let res = reqwest::get(url).await?.text().await?;

        const IC_START_OR_END_STATIONS: [&str; 3] =
            ["MILANO CENTRALE", "VENTIMIGLIA", "ROMA TERMINI"];

        let line_content = res
            .lines()
            .filter(|l| {
                // Filter out lines that are not InterCity trains
                if code < 500 || code >= 800 {
                    true
                } else {
                    IC_START_OR_END_STATIONS.iter().any(|&s| l.contains(s))
                }
            })
            .next()
            .unwrap()
            .split('-')
            .collect::<Vec<&str>>();

        let origin_id = line_content.iter().nth(2).unwrap().trim().to_string();

        let timestamp = line_content.iter().rev().next().unwrap().trim().to_string();

        let url = format!(
            "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/andamentoTreno/{}/{}/{}",
            origin_id, code, timestamp
        );

        let res = reqwest::get(url).await?.json::<serde_json::Value>().await?;

        let train_type = match res["categoria"].as_str().unwrap() {
            "REG" => TrainType::REG,
            "IC" => TrainType::IC,
            "EC" => TrainType::EC,
            "" => TrainType::AV,
            _ => panic!("Unknown train type"),
        };

        let mut train = Train {
            code,
            origin_id,
            train_type,
            stops: Vec::new(),
        };

        for f in res["fermate"].as_array().unwrap() {
            let station_name = f["stazione"].as_str().unwrap().to_string();

            let arrival_time = parse_time(f["arrivo_teorico"].as_u64());
            let departure_time = parse_time(f["partenza_teorica"].as_u64());
            let actual_arrival_time = parse_time(f["arrivoReale"].as_u64());
            let actual_departure_time = parse_time(f["partenzaReale"].as_u64());

            train.stops.push(Stop {
                station_name,
                arrival_time,
                departure_time,
                actual_arrival_time,
                actual_departure_time,
            });
        }

        trains.push(train);
    }

    Ok(trains)
}

async fn parse_numbers() -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    const START_END_STATIONS: [&str; 7] = [
        "FINALE LIGURE MARINA",
        "ALBENGA",
        "ALASSIO",
        "IMPERIA",
        "TAGGIA ARMA",
        "SANREMO",
        "VENTIMIGLIA",
    ];

    let date = Utc::now().date_naive();
    let day = date.day();
    let month = date.month();
    let year = date.year();

    let url = format!(
        "https://trainstats.altervista.org/exportcsv.php?data={}_{}_{}",
        day, month, year
    );

    let train_codes = reqwest::get(url)
        .await?
        .text()
        .await?
        .lines()
        .filter(|l| START_END_STATIONS.iter().any(|&s| l.contains(s)))
        .map(|l| l.split(',').nth(1).unwrap().parse().unwrap())
        .collect::<Vec<u32>>();

    Ok(train_codes)
}

fn parse_time(time: Option<u64>) -> Option<NaiveTime> {
    const SECONDS_PER_DAY: u32 = 86400;
    const SECONDS_PER_2_HOURS: u32 = 7200;
    time.map(|t| {
        NaiveTime::from_num_seconds_from_midnight_opt(
            (t / 1000) as u32 % SECONDS_PER_DAY + SECONDS_PER_2_HOURS,
            0,
        )
        .unwrap()
    })
}

pub fn write_to_file(trains: &Vec<Train>, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;
    writeln!(file, "{:#?}", trains)?;

    Ok(())
}
