use colored::Colorize;
use scraper::{Html, Selector};

pub async fn print_news() -> Result<(), Box<dyn std::error::Error>> {
    let url =
        "http://www.viaggiatreno.it/infomobilitamobile/resteasy/viaggiatreno/infomobilitaRSS/false";

    let res = reqwest::get(url).await?.text().await?;

    let fragment = Html::parse_fragment(&res);
    let selector = Selector::parse("li").unwrap();

    for (i, element) in fragment.select(&selector).enumerate() {
        let title_element = element.child_elements().next().unwrap();
        let title = title_element.inner_html();
        let highlight = title_element
            .value()
            .attr("class")
            .unwrap()
            .contains("inEvidenza");

        let title = if highlight {
            title.bright_red()
        } else {
            title.normal()
        };
        println!("{}. {}\n", i + 1, title);
    }

    Ok(())
}
