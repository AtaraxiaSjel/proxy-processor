use clap::Parser;
use proxy_processor::models::ProxyType;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,
    #[arg(short, long)]
    output: String,
    #[arg(long = "geoip", default_value = "GeoLite2-Country.mmdb")]
    geoip_db: String,
    #[arg(
        long = "types",
        short = 't',
        value_delimiter = ',',
        help = "Filter by type: vless,ss"
    )]
    proxy_types: Option<Vec<ProxyType>>,
    #[arg(
        long,
        short,
        value_delimiter = ',',
        help = "Filter by country codes: US,DE,JP"
    )]
    countries: Option<Vec<String>>,
    #[arg(long, default_value = "links", help = "Output format: links, sing-box")]
    format: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("App logic to be implemented!");

    Ok(())
}
