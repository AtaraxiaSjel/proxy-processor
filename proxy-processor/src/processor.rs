use std::net::IpAddr;

use crate::geoip::GeoIpReader;
use crate::models::{FilterOptions, GeoIpInfo, Proxy};
use dns_lookup::lookup_host;
use rayon::prelude::*;
use tracing::{error, instrument, trace};

/// Attempts to resolve a proxy's address to an IpAddr.
/// It first tries a direct parse, then falls back to a DNS lookup.
#[instrument(skip_all, level = "debug")]
fn get_proxy_ip(proxy: &Proxy) -> Option<IpAddr> {
    trace!(?proxy.address, "lookup ip address");
    proxy.address.parse().ok().or_else(|| {
        lookup_host(&proxy.address)
            .ok()
            .and_then(|mut ips| ips.next())
    })
}

/// Checks if a proxy is located in one of the specified countries.
#[instrument(level = "debug")]
fn is_proxy_in_countries(geoip: &Option<GeoIpInfo>, countries: &[String]) -> bool {
    geoip.as_ref().is_some_and(|geoip| {
        countries.contains(&geoip.iso_code)
            || geoip
                .is_in_eu
                .then(|| countries.contains(&"EU".to_string()))
                .unwrap_or_default()
    })
}

pub fn process_and_filter(
    raw_links: Vec<String>,
    filter_options: &FilterOptions,
    geoip_reader: &GeoIpReader,
) -> Vec<Proxy> {
    let mut links = raw_links
        .into_par_iter()
        .filter_map(|link| {
            crate::parser::parse_proxy(&link)
                .inspect_err(|err| error!(?err, "error parsing proxy"))
                .ok()
        })
        .map(|mut proxy| {
            // If filters by countries exist, fill geoip data for all proxies
            if (filter_options.include_countries.is_some()
                || filter_options.exclude_countries.is_some())
                && proxy.geoip.is_none()
            {
                proxy.geoip = get_proxy_ip(&proxy).and_then(|ip| {
                    geoip_reader
                        .lookup_country_info(ip)
                        .inspect_err(|err| error!(?err, "error lookup geodb"))
                        .map(|geoip| {
                            geoip.and_then(|country| {
                                country.iso_code.map(|iso| GeoIpInfo {
                                    is_in_eu: country.is_in_european_union.is_some_and(|x| x),
                                    iso_code: iso.to_string(),
                                    name: country
                                        .names
                                        .and_then(|names| names.get("en").copied())
                                        .map(String::from),
                                })
                            })
                        })
                        .ok()
                        .flatten()
                });
            }
            proxy
        })
        .filter(|proxy| {
            filter_options
                .proxy_types
                .as_ref()
                .is_none_or(|types| types.contains(&proxy.proxy_type))
        })
        .filter(|proxy| {
            filter_options
                .include_countries
                .as_ref()
                .is_none_or(|countries| is_proxy_in_countries(&proxy.geoip, countries))
        })
        .filter(|proxy| {
            filter_options
                .exclude_countries
                .as_ref()
                .is_none_or(|countries| {
                    proxy.geoip.is_some() && !is_proxy_in_countries(&proxy.geoip, countries)
                })
        })
        .collect::<Vec<Proxy>>();
    links.sort();
    links.dedup();
    links
}
