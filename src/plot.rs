use chrono::Timelike;
use plotters::{prelude::*, style::full_palette};

use crate::parse_trains::{Train, TrainType};
use crate::stations::STATION_DECAMETERS;

pub fn plot_trains(train: &Train) {
    let min_time = train.stops[0].departure_time.unwrap();
    let max_time = train.stops[train.stops.len() - 1].arrival_time.unwrap();

    let max_decameter = STATION_DECAMETERS[STATION_DECAMETERS.len() - 1].1;

    let line_color = match train.train_type {
        TrainType::REG => full_palette::BLUE,
        TrainType::IC => full_palette::RED,
    };

    let mut points: Vec<(u32, u32)> = Vec::new();
    points.reserve(train.stops.len() * 2);

    for stop in &train.stops {
        let dam = STATION_DECAMETERS
            .iter()
            .find(|(name, _)| stop.station_name.eq(name))
            .expect("station name not found")
            .1;

        let arrival_time = stop.arrival_time.unwrap_or_default();
        let departure_time = stop.departure_time.unwrap_or(max_time);

        points.push((time_to_minutes(arrival_time), dam));
        points.push((time_to_minutes(departure_time), dam));
    }

    let root_drawing_area = BitMapBackend::new("0.1.png", (1280, 1280)).into_drawing_area();

    root_drawing_area.fill(&full_palette::GREY_200).unwrap();

    let mut chart = ChartBuilder::on(&root_drawing_area)
        .set_label_area_size(LabelAreaPosition::Left, 144)
        .set_label_area_size(LabelAreaPosition::Bottom, 20)
        .build_cartesian_2d(
            (time_to_minutes(min_time))..(time_to_minutes(max_time) + 15),
            0..(max_decameter + 100),
        )
        .unwrap();

    chart
        .configure_mesh()
        .y_labels((max_decameter * 2) as usize)
        .x_labels((max_time.hour() - min_time.hour()) as usize * 2)
        .y_label_formatter(
            &|y| match STATION_DECAMETERS.binary_search_by(|(_, dam)| dam.cmp(y)) {
                Ok(i) => STATION_DECAMETERS[i].0.to_string() + " ",
                Err(_) => "".to_string(),
            },
        )
        .disable_y_mesh()
        .set_tick_mark_size(LabelAreaPosition::Left, 0)
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(points.into_iter(), &line_color))
        .unwrap();
}

fn time_to_minutes(time: chrono::NaiveTime) -> u32 {
    time.hour() * 60 + time.minute()
}
