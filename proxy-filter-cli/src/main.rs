use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

use clap::Parser;
use proxy_processor::{
    error::ExportError,
    exporter::{Exporter, LinkExporter, SingBoxExporter},
    processor::process_and_filter,
};
use proxy_processor::{
    geoip::GeoIpReader,
    models::{FilterOptions, ProxyType},
};
use reqwest::Url;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

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
    #[arg(
        long,
        short,
        default_value = "links",
        help = "Output format: links, sing-box"
    )]
    format: String,
}

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let countries = args.countries;
    // Parse list of countries, split them into include and exclude lists
    let (include, exclude) = countries.map_or((None, None), |countries| {
        let (include, exclude) =
            countries
                .into_iter()
                .fold((Vec::new(), Vec::new()), |(mut inc, mut exc), country| {
                    if let Some(stripped) = country.strip_prefix('!') {
                        exc.push(stripped.to_string());
                    } else {
                        inc.push(country);
                    }
                    (inc, exc)
                });
        (
            (!include.is_empty()).then_some(include),
            (!exclude.is_empty()).then_some(exclude),
        )
    });

    let filter_options = FilterOptions {
        exclude_countries: exclude,
        include_countries: include,
        proxy_types: args.proxy_types,
    };

    let geoip_reader = if let Ok(download_url) = Url::parse(&args.geoip_db) {
        let response = reqwest::blocking::get(download_url)?;
        GeoIpReader::from_bytes(response.bytes()?)?
    } else {
        GeoIpReader::from_path(&args.geoip_db)?
    };

    let proxy_list = if let Ok(download_url) = Url::parse(&args.input) {
        let response = reqwest::blocking::get(download_url)?;
        let lines = response.text()?.lines().map(String::from).collect();
        process_and_filter(lines, &filter_options, &geoip_reader)
    } else {
        let input_file = File::open(args.input)?;
        let reader = BufReader::new(input_file);
        let lines = reader.lines().map_while(Result::ok).collect();
        process_and_filter(lines, &filter_options, &geoip_reader)
    };

    let output = match args.format.as_str() {
        "links" => LinkExporter::new().export(&proxy_list)?,
        "sing-box" => SingBoxExporter::new().export(&proxy_list)?,
        export_format => {
            return Err(ExportError::UnsupportedProtocol(export_format.to_string()).into());
        }
    };

    let mut file = File::create(args.output)?;
    file.write_all(output.as_bytes())?;

    Ok(())
}
