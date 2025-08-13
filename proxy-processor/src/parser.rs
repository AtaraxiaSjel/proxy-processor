use std::collections::HashMap;

use crate::error::ProcessorError;
use crate::models::{
    Proxy, ProxyDetails, ProxyType, RealitySettings, TlsSettings, TransportSettings,
};
use base64::{Engine, engine::general_purpose};
use tracing::{instrument, trace};
use url::Url;
use urlencoding::decode;

#[instrument(level = "debug")]
pub fn parse_proxy(link: &str) -> Result<Proxy, ProcessorError> {
    let link = if !link.contains("://") {
        let (link, _) = decode_base64(link)?;
        link
    } else {
        link.to_string()
    };
    let link = decode(&link).unwrap().into_owned();

    if let Some((scheme, data)) = link.split_once("://") {
        let proxy_type = match scheme {
            "vless" => ProxyType::Vless,
            "ss" => ProxyType::Shadowsocks,
            // "vmess" => ProxyType::Vmess,
            // "ssr" => ProxyType::ShadowsocksR,
            // "trojan" => ProxyType::Trojan,
            // "h2" => ProxyType::Hysteria2,
            _ => return Err(ProcessorError::UnsupportedProtocol(scheme.to_string())),
        };
        trace!(?proxy_type, "parsed proxy type");

        let (data, _) = if !data.contains('@') {
            decode_base64(data)?
        } else {
            (data.to_string(), "".to_string())
        };

        if proxy_type != ProxyType::Vmess {
            let link = format!("{scheme}://{data}");
            let url = Url::parse(&link).map_err(|err| ProcessorError::UrlParse(err.to_string()))?;
            let params: HashMap<String, String> = url
                .query_pairs()
                .map(|(k, v)| (k.into(), v.trim().into()))
                .collect();
            let address = url
                .host_str()
                .ok_or(ProcessorError::MissingComponent("address/host"))?
                .to_string();
            let port = url.port().ok_or(ProcessorError::MissingComponent("port"))?;
            let remarks = url
                .fragment()
                .map(|x| urlencoding::decode(x).unwrap_or_default().into_owned());

            match proxy_type {
                ProxyType::Vless => {
                    let uuid = if !url.username().is_empty() {
                        decode(url.username()).unwrap().into_owned()
                    } else {
                        return Err(ProcessorError::MissingComponent("uuid"));
                    };

                    let tls_settings = parse_tls_settings(&params)?;
                    let transport_settings = parse_transport_settings(&params)?;

                    let details = ProxyDetails::Vless {
                        uuid,
                        flow: params.get("flow").cloned(),
                        tls: tls_settings,
                        transport: transport_settings,
                    };

                    Ok(Proxy {
                        original_link: link.to_string(),
                        address,
                        port,
                        remarks,
                        proxy_type: ProxyType::Vless,
                        details,
                        geoip: None,
                    })
                }
                ProxyType::Shadowsocks => {
                    let base = if !url.username().is_empty() {
                        decode(url.username()).unwrap().into_owned()
                    } else {
                        return Err(ProcessorError::MissingComponent("method:password"));
                    };
                    let (ss_conf, _) = decode_base64(&base)?;
                    let (method, password) = ss_conf
                        .split_once(':')
                        .map(|(l, r)| (l.to_string(), r.to_string()))
                        .ok_or(ProcessorError::MissingComponent("method:password"))?;

                    Ok(Proxy {
                        original_link: link,
                        address,
                        port,
                        remarks,
                        proxy_type: ProxyType::Shadowsocks,
                        details: ProxyDetails::Shadowsocks { method, password },
                        geoip: None,
                    })
                }
                ProxyType::ShadowsocksR => unimplemented!(),
                ProxyType::Trojan => unimplemented!(),
                ProxyType::Hysteria2 => unimplemented!(),
                ProxyType::Vmess => unreachable!(),
            }
        } else {
            unimplemented!()
        }
    } else {
        Err(ProcessorError::InvalidLinkFormat(link))
    }
}

fn parse_tls_settings(params: &HashMap<String, String>) -> Result<TlsSettings, ProcessorError> {
    let security = params.get("security").map(|s| s.as_str());

    if !matches!(security, Some("tls") | Some("reality")) {
        return Ok(TlsSettings::default());
    }

    let reality = if security == Some("reality") {
        Some(RealitySettings {
            enabled: true,
            public_key: params
                .get("pbk")
                .map(String::from)
                .ok_or(ProcessorError::MissingComponent("pbk"))?,
            short_id: params.get("sid").map(String::from).unwrap_or_default(),
        })
    } else {
        None
    };

    Ok(TlsSettings {
        enabled: true,
        server_name: params.get("sni").cloned(),
        alpn: params
            .get("alpn")
            .map(|s| s.split(',').map(String::from).collect()),
        fingerprint: params.get("fp").map(String::from),
        reality,
    })
}

fn parse_transport_settings(
    params: &HashMap<String, String>,
) -> Result<TransportSettings, ProcessorError> {
    let parse_http = |params: &HashMap<String, String>| {
        let host = params
            .get("host")
            .map(|s| s.split(',').map(String::from).collect());
        let path = params
            .get("path")
            .map(String::from)
            .unwrap_or("/".to_string());
        let method = params.get("method").map(String::from).unwrap_or_default();
        TransportSettings::Http { host, path, method }
    };

    match params.get("type").map(|s| s.as_str()) {
        Some("quic") => Ok(TransportSettings::Quic),
        Some("grpc") => Ok(TransportSettings::Grpc {
            service_name: params
                .get("serviceName")
                .map(String::from)
                .unwrap_or_default(),
        }),
        Some("ws") => Ok(TransportSettings::WebSocket {
            path: params
                .get("path")
                .ok_or(ProcessorError::MissingComponent("path"))?
                .to_string(),
        }),
        Some("http") => Ok(parse_http(params)),
        Some("httpupgrade") => {
            let host = params
                .get("host")
                .map(|s| s.split(',').map(String::from).collect());
            let path = params
                .get("path")
                .map(String::from)
                .unwrap_or("/".to_string());
            Ok(TransportSettings::HttpUpgrade { host, path })
        }
        Some("tcp") => {
            if params.get("headerType").map(|s| s.as_str()) == Some("http") {
                Ok(parse_http(params))
            } else {
                Ok(TransportSettings::Tcp)
            }
        }
        None => Ok(TransportSettings::Tcp),
        Some(transport) => Err(ProcessorError::InvalidValue(format!(
            "transport type: {transport}"
        ))),
    }
}

/// Decodes a Base64 string from within a larger string by iteratively
/// attempting to decode progressively shorter substrings. This correctly handles
/// "garbage" suffixes that may contain valid Base64 characters.
///
/// # Arguments
///
/// * `input` - The string to search for a Base64 encoded substring.
///
/// # Returns
///
/// A `Result` containing a tuple of:
/// - The decoded byte vector.
/// - The string found after the valid Base64 data (the "suffix garbage").
///
/// Or an `Err` if no valid decodable Base64 substring is found.
fn decode_base64(input: &str) -> Result<(String, String), ProcessorError> {
    let Some(start_index) = input.find(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    else {
        return Err(ProcessorError::Base64DecodeFailed(
            "Could not find a valid Base64 characters in input".to_string(),
        ));
    };

    let mut candidate = &input[start_index..];

    while !candidate.is_empty() {
        if let Ok(decoded_data) = general_purpose::URL_SAFE_NO_PAD.decode(candidate) {
            let garbage_suffix_start = start_index + candidate.len();
            let garbage_suffix = input[garbage_suffix_start..].to_string();
            let data = String::from_utf8(decoded_data).map_err(|_| {
                ProcessorError::Base64DecodeFailed("invalid utf-8 sequence".to_string())
            })?;
            return Ok((data, garbage_suffix));
        }

        if let Some(last_char) = candidate.chars().last() {
            candidate = &candidate[..candidate.len() - last_char.len_utf8()];
        }
    }

    Err(ProcessorError::Base64DecodeFailed(
        "Could not find a valid Base64 substring to decode".to_string(),
    ))
}
