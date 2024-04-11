use std::io;

use chrono::NaiveTime;
use colored::Colorize;

pub async fn track(code: u32) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/cercaNumeroTrenoTrenoAutocomplete/{}",
        code
    );

    let res = reqwest::get(url).await?.text().await?;

    let lines: Vec<_> = res.lines().collect();

    //TODO: pass it as optional command line argument

    let index = if lines.len() > 1 {
        println!("Found more than one train with given code. Please select one:");

        lines.clone().into_iter().enumerate().for_each(|(i, l)| {
            let l = l.split('|').next().unwrap();
            println!("{}. {}", i + 1, l);
        });

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().parse::<usize>()? - 1
    } else {
        0
    };

    let mut line_content = lines[index].split('|').nth(1).unwrap().split('-').skip(1);

    let origin_id = line_content.next().unwrap();
    let timestamp = line_content.next().unwrap();

    print_train_track_info(origin_id, code, timestamp).await?;

    Ok(())
}

async fn print_train_track_info(
    origin_id: &str,
    code: u32,
    timestamp: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/andamentoTreno/{}/{}/{}",
        origin_id, code, timestamp
    );

    let res = reqwest::get(url).await?.json::<serde_json::Value>().await?;

    let last_update_station = res["stazioneUltimoRilevamento"].as_str().unwrap_or("--");
    let delay = res["ritardo"].as_i64().map(|d| {
        if d > 0 {
            format!("+{d}")
        } else {
            d.to_string()
        }
    });
    let last_update_time = parse_time(res["oraUltimoRilevamento"].as_u64())
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or("--:--".to_string());

    let origin_station = res["origine"].as_str().unwrap();
    let destination_station = res["destinazione"].as_str().unwrap();

    let train_label = res["compNumeroTreno"].as_str().unwrap();

    //TODO: Print next stop info
    // Print if train is not yet departed or arrived

    println!(
        "Treno {}, {} - {} \nUltimo rilevamento ({}):\n\t{}, {}",
        train_label.bold(),
        origin_station.blue(),
        destination_station.blue(),
        last_update_time,
        last_update_station.blue(),
        delay.unwrap_or("--".to_string()).bold()
    );

    Ok(())
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
