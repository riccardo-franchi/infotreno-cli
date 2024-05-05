use scraper::{Html, Selector};

pub async fn print_news() -> Result<(), Box<dyn std::error::Error>> {
    let url =
        "http://www.viaggiatreno.it/infomobilitamobile/resteasy/viaggiatreno/infomobilitaRSS/false";

    let res = reqwest::get(url).await?.text().await?;

    let fragment = Html::parse_fragment(&res);
    let selector = Selector::parse("li").unwrap();

    for element in fragment.select(&selector) {
        let text = element.text().collect::<Vec<_>>().join(" ");
        println!("{}", text);
    }

    Ok(())
}
