use reqwest;
use scraper::{Html, Selector};

// makes request to https://www.coingecko.com/en/coins/bitcoin/usd
// scrapes price by xpath: //*[@id="gecko-coin-page-container"]/div[2]/div/div[1]/div[2]/div[1]/span
async fn btc_price() -> f64 {
    let url = "https://www.coingecko.com/en/coins/bitcoin/usd";
    // Make HTTP request
    let client = reqwest::Client::new();
    let response = match client.get(url).send().await {
        Ok(resp) => resp,
        Err(_) => return 0.0, // Return 0.0 on error
    };
    // Get HTML content
    let html_content = match response.text().await {
        Ok(content) => content,
        Err(_) => return 0.0,
    };
    // Parse HTML
    let document = Html::parse_document(&html_content);
    // Convert XPath to CSS selector
    // The XPath //*[@id="gecko-coin-page-container"]/div[2]/div/div[1]/div[2]/div[1]/span
    // translates to: #gecko-coin-page-container > div:nth-child(2) > div > div:nth-child(1) > div:nth-child(2) > div:nth-child(1) > span
    let selector = match Selector::parse("#gecko-coin-page-container > div:nth-child(2) > div > div:nth-child(1) > div:nth-child(2) > div:nth-child(1) > span") {
        Ok(sel) => sel,
        Err(_) => return 0.0,
    };
    // Find the element and extract price
    if let Some(element) = document.select(&selector).next() {
        let price_text = element.text().collect::<String>();
        // Remove currency symbols and commas, then parse
        let cleaned_price = price_text
            .replace("$", "")
            .replace(",", "")
            .trim()
            .to_string();

        match cleaned_price.parse::<f64>() {
            Ok(price) => price,
            Err(_) => 0.0,
        }
    } else {
        0.0
    }
}