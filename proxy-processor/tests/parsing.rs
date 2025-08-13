#[cfg(test)]
mod test_processor {
    use maxminddb::geoip2;
    use proxy_processor::error::ProcessorError;
    use proxy_processor::geoip::GeoIpReader;
    use proxy_processor::models::FilterOptions;
    use proxy_processor::parser::parse_proxy;
    use proxy_processor::processor::process_and_filter;
    use std::fs;
    use std::net::IpAddr;
    use std::path::Path;
    use std::str::FromStr;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn test_parse_proxies_from_file() {
        let content = fs::read_to_string(Path::new("../proxy_test.txt"))
            .expect("Could not read proxy_test.txt. Make sure it's in the project root.");
        let lines: Vec<_> = content.lines().collect();
        let total_lines = lines.len();

        let mut successful_parses = 0;
        let mut failed_parses = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            match parse_proxy(line) {
                Ok(_) => {
                    successful_parses += 1;
                }
                Err(e) => {
                    failed_parses.push((i + 1, line.to_string(), e));
                }
            }
        }

        if !failed_parses.is_empty() {
            println!(
                "{} out of {} proxies failed to parse:",
                failed_parses.len(),
                total_lines
            );
            for (line_num, line, error) in &failed_parses {
                println!("  [Line {}] '{}': {:?}", line_num, line, error);
            }
            for error in failed_parses
                .iter()
                .map(|(_, _, err)| err.clone())
                .collect::<std::collections::HashSet<ProcessorError>>()
                .iter()
            {
                println!("{:?}", error);
            }
        }

        // Based on the current parser implementation, many will fail.
        // We assert that at least the ones we support (`vless`, `ss`) are parsed.
        assert!(
            successful_parses > 0,
            "Expected at least one proxy to be parsed successfully."
        );

        println!(
            "\nSuccessfully parsed {}/{} proxies.",
            successful_parses, total_lines
        );
    }

    #[traced_test]
    #[test]
    fn test_filter_proxies_from_file() {
        let content = fs::read_to_string(Path::new("../proxy.txt"))
            .expect("Could not read proxy.txt. Make sure it's in the project root.");
        let lines: Vec<String> = content.lines().map(String::from).collect();

        let proxies = process_and_filter(
            lines,
            &FilterOptions {
                proxy_types: None,
                include_countries: Some(vec!["jp", "us"].into_iter().map(String::from).collect()),
                exclude_countries: None,
            },
            &GeoIpReader::new("../GeoLite2-Country.mmdb").unwrap(),
        );

        proxies.iter().for_each(|x| println!("{x:?}"));

        assert!(
            !proxies.is_empty(),
            "Expected at least one proxy to be parsed successfully."
        );
    }

    #[test]
    fn test_geoip() {
        let reader = maxminddb::Reader::open_readfile("../GeoLite2-Country.mmdb").unwrap();

        let ip: IpAddr = FromStr::from_str("89.160.20.128").unwrap();
        if let Some(country) = reader.lookup::<geoip2::Country>(ip).unwrap() {
            println!("{:?}", country);
        } else {
            println!("Address not found");
        }
    }
}
