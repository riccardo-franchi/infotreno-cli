use colored::Colorize;
use scraper::{Html, Selector};

use crate::cli_input;

pub async fn print_news(is_verbose: bool) -> Result<(), reqwest::Error> {
    let url =
        "http://www.viaggiatreno.it/infomobilitamobile/resteasy/viaggiatreno/infomobilitaRSS/false";

    let res = reqwest::get(url).await?.text().await?;

    let fragment = Html::parse_fragment(&res);
    let selector = Selector::parse("li").unwrap();

    let news = fragment.select(&selector).collect::<Vec<_>>();

    if news.is_empty() {
        println!("No news available.");
        return Ok(());
    }

    for (i, element) in news.iter().enumerate() {
        let mut children_iter = element.child_elements();

        let Some(title_element) = children_iter.next() else {
            break;
        };
        let title = title_element.inner_html();
        let is_highlighted = title_element
            .value()
            .attr("class")
            .unwrap()
            .contains("inEvidenza");

        let title = if is_highlighted {
            title.bright_red()
        } else {
            title.normal()
        };
        println!("{}. {}\n", i + 1, title.bold());

        if is_verbose {
            let info_text = children_iter
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                .trim()
                .replace('\t', "");
            println!("{}\n", info_text);
        }
    }

    if is_verbose {
        return Ok(());
    }

    println!("{}", "Select a news header to expand:".dimmed());

    loop {
        let index = cli_input::get_index();

        if index == 0 {
            return Ok(());
        }

        if index > news.len() {
            println!("Invalid index.");
            continue;
        }

        let info_text = news
            .get(index - 1)
            .unwrap()
            .child_elements()
            .nth(1)
            .unwrap()
            .text()
            .collect::<String>()
            .trim()
            .replace('\t', "");

        println!("{}\n", info_text);
    }
}
