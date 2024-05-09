use colored::Colorize;
use scraper::{Html, Selector};
use std::io;

pub async fn print_news(is_verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let url =
        "http://www.viaggiatreno.it/infomobilitamobile/resteasy/viaggiatreno/infomobilitaRSS/false";

    let res = reqwest::get(url).await?.text().await?;

    let fragment = Html::parse_fragment(&res);
    let selector = Selector::parse("li").unwrap();

    let news = fragment.select(&selector).collect::<Vec<_>>();

    for (i, element) in news.iter().enumerate() {
        let mut children_iter = element.child_elements();

        let title_element = children_iter.next().unwrap();
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
        // TODO: refactor getting index to a separate function
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let index = input.trim().parse::<usize>()? - 1;

        let info_text = news
            .get(index)
            .expect("Invalid index")
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
