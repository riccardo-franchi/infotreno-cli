use std::io;

use chrono::NaiveTime;
use colored::Colorize;

pub async fn track(code: u32, index: Option<usize>) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "http://www.viaggiatreno.it/infomobilita/resteasy/viaggiatreno/cercaNumeroTrenoTrenoAutocomplete/{}",
        code
    );

    let res = reqwest::get(url).await?.text().await?;

    let lines: Vec<_> = res.lines().collect();

    if lines.is_empty() {
        println!("Nessun treno trovato con il codice inserito");
        return Ok(());
    }

    let index = if lines.len() > 1 && index.is_none() {
        println!("Trovato più di un treno con il codice inserito. Seleziona il treno:");

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
        return Err("Indice non valido.".into());
    }

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

    let origin_station = res["origine"].as_str().unwrap_or("--");
    let destination_station = res["destinazione"].as_str().unwrap_or("--");

    let train_label = res["compNumeroTreno"].as_str().unwrap().trim();

    let is_not_departured = res["nonPartito"].as_bool().unwrap_or_default();

    if is_not_departured {
        let departure_time = res["compOrarioPartenza"].as_str().unwrap_or("--:--");

        println!(
            "Treno {}, {} - {} \nNon ancora partito.\nPartenza prevista alle ore {}.",
            train_label.bold(),
            origin_station.cyan(),
            destination_station.cyan(),
            departure_time
        );
        return Ok(());
    }

    let delay = res["ritardo"].as_i64().map(|d| {
        if d > 0 {
            format!("+{d}")
        } else {
            d.to_string()
        }
    });
    let last_update_station = res["stazioneUltimoRilevamento"].as_str().unwrap_or("--");
    let last_update_time = parse_time(res["oraUltimoRilevamento"].as_u64())
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or("--:--".to_string());

    let stops = res["fermate"].as_array().unwrap();

    let is_arrived = stops.iter().last().unwrap()["actualFermataType"]
        .as_u64()
        .expect("AAAAA")
        == 1;

    println!(
        "Treno {}, {} - {} \nUltimo rilevamento ({}):\n\t{}, {}",
        train_label.bold(),
        origin_station.cyan(),
        destination_station.cyan(),
        last_update_time,
        last_update_station.cyan(),
        delay.unwrap_or("--".to_string()).bold()
    );
    if is_arrived {
        println!("Arrivato a destinazione.");
    } else {
        for f in stops {
            let stop_type = f["actualFermataType"].as_u64().unwrap();

            if stop_type != 0 {
                continue;
            }

            let next_stop = f["stazione"].as_str().unwrap();
            let arrival_time = parse_time(f["arrivo_teorico"].as_u64())
                .map(|t| t.format("%H:%M").to_string())
                .unwrap_or("--:--".to_string());
            println!(
                "\nProssima fermata: {} (arrivo teorico: {})",
                next_stop.cyan(),
                arrival_time
            );
            break;
        }
    }

    Ok(())
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
