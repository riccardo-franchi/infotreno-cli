use chrono::Timelike;
use plotters::{prelude::*, style::full_palette};
use std::path::Path;

use crate::parse_trains::{Train, TrainType};
use crate::stations::STATION_DECAMETERS;

pub fn plot_trains(trains: &[Train], path: &Path) {
    const MAX_DECAMETER: u32 = STATION_DECAMETERS[STATION_DECAMETERS.len() - 1].1;

    let min_time = trains
        .iter()
        .map(|t| t.stops.first().unwrap().departure_time.unwrap())
        .min()
        .unwrap();
    let max_time = trains
        .iter()
        .map(|t| t.stops.last().unwrap().arrival_time.unwrap())
        .max()
        .unwrap();

    let root_drawing_area = BitMapBackend::new(path, (4000, 1000)).into_drawing_area();

    root_drawing_area.fill(&full_palette::GREY_200).unwrap();

    let min_time_minutes = time_to_minutes(min_time);
    let max_time_minutes = time_to_minutes(max_time);

    let mut chart = ChartBuilder::on(&root_drawing_area)
        .set_label_area_size(LabelAreaPosition::Left, 144)
        .set_label_area_size(LabelAreaPosition::Bottom, 20)
        .build_cartesian_2d(
            min_time_minutes..(max_time_minutes + 15),
            0..(MAX_DECAMETER + 100),
        )
        .unwrap();

    chart
        .configure_mesh()
        .y_labels((MAX_DECAMETER * 2) as usize)
        .x_labels((max_time_minutes - min_time_minutes) as usize)
        .y_label_formatter(
            &|y| match STATION_DECAMETERS.binary_search_by(|(_, dam)| dam.cmp(y)) {
                Ok(i) => STATION_DECAMETERS[i].0.to_string() + " ",
                Err(_) => "".to_string(),
            },
        )
        .x_label_formatter(&|x| {
            if x % 8 == 0 {
                let hour = *x / 60;
                let minute = *x % 60;
                format!("{:02}:{:02}", hour, minute)
            } else {
                "".to_string()
            }
        })
        .disable_y_mesh()
        .set_tick_mark_size(LabelAreaPosition::Left, 0)
        .draw()
        .unwrap();

    for train in trains {
        let mut points: Vec<(u32, u32)> = Vec::with_capacity(train.stops.len() * 2);

        for stop in &train.stops {
            let dam = STATION_DECAMETERS
                .iter()
                .find(|(name, _)| stop.station_name.eq(name))
                .expect("station name not found")
                .1;

            if let Some(arrival_time) = stop.arrival_time {
                points.push((time_to_minutes(arrival_time), dam));
            }
            if let Some(departure_time) = stop.departure_time {
                points.push((time_to_minutes(departure_time), dam));
            }
        }

        let line_color = match train.train_type {
            TrainType::REG => full_palette::BLUE,
            TrainType::IC => full_palette::GREEN,
            TrainType::EC => full_palette::YELLOW,
            TrainType::EXP => full_palette::PURPLE,
            TrainType::AV => full_palette::RED,
        };

        chart
            .draw_series(LineSeries::new(points.clone().into_iter(), &line_color))
            .unwrap();
    }
}

fn time_to_minutes(time: chrono::NaiveTime) -> u32 {
    time.hour() * 60 + time.minute()
}
