use std::net::IpAddr;

use crate::geoip::GeoIpReader;
use crate::models::{FilterOptions, Proxy};
use dns_lookup::lookup_host;
use rayon::prelude::*;
use tracing::error;

/// Attempts to resolve a proxy's address to an IpAddr.
/// It first tries a direct parse, then falls back to a DNS lookup.
fn get_proxy_ip(proxy: &Proxy) -> Option<IpAddr> {
    proxy.address.parse().ok().or_else(|| {
        lookup_host(&proxy.address)
            .ok()
            .and_then(|mut ips| ips.next())
    })
}

/// Checks if a proxy is located in one of the specified countries.
fn is_proxy_in_countries(proxy: &Proxy, countries: &[String], geoip_reader: &GeoIpReader) -> bool {
    get_proxy_ip(proxy)
        .and_then(|ip| {
            geoip_reader
                .lookup_country_iso(ip)
                .inspect_err(|err| error!(error = ?err, "error lookup geodb"))
                .ok()
                .unwrap_or_default()
        })
        .is_some_and(|country_code| countries.contains(&country_code))
}

pub fn process_and_filter(
    raw_links: Vec<String>,
    filter_options: &FilterOptions,
    geoip_reader: &GeoIpReader,
) -> Vec<Proxy> {
    raw_links
        .into_par_iter()
        .filter_map(|link| {
            crate::parser::parse_proxy(&link)
                .inspect_err(|err| error!(error = ?err, "error parsing proxy"))
                .ok()
        })
        .filter(|proxy| {
            filter_options
                .proxy_types
                .as_ref()
                .is_none_or(|types| types.contains(&proxy.proxy_type))
        })
        .filter(|proxy| {
            filter_options
                .country_codes
                .as_ref()
                .is_none_or(|countries| is_proxy_in_countries(proxy, countries, geoip_reader))
        })
        .collect()
}
