use colored::Colorize;
use scraper::{Html, Selector};

pub async fn print_news(is_verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    let url =
        "http://www.viaggiatreno.it/infomobilitamobile/resteasy/viaggiatreno/infomobilitaRSS/false";

    let res = reqwest::get(url).await?.text().await?;

    let fragment = Html::parse_fragment(&res);
    let selector = Selector::parse("li").unwrap();

    for (i, element) in fragment.select(&selector).enumerate() {
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
        println!("{}. {}\n", i + 1, title);

        if is_verbose {
            let info_text = children_iter
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                .trim()
                .replace("\t", "");
            println!("{}", info_text);
        }
    }

    if is_verbose {
        return Ok(());
    }

    Ok(())
}
