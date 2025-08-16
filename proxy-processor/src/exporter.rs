use crate::error::ExportError;
use crate::models::{Proxy, ProxyDetails, ProxyType, TlsSettings, TransportSettings};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::trace;

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

pub trait Exporter {
    fn format_proxy(&self, proxy: &Proxy, index: usize) -> Result<Value, ExportError>;

    // TODO: proper errors
    fn export(&self, proxies: &[Proxy]) -> Result<String, ExportError> {
        Ok(proxies
            .iter()
            .enumerate()
            .filter_map(|(i, p)| self.format_proxy(p, i).ok())
            .map(|v| {
                v.as_str()
                    .expect("Value in this scope should always be a String")
                    .to_string()
            })
            .join(LINE_ENDING))
    }
}

#[derive(Default)]
pub struct LinkExporter;

impl LinkExporter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Exporter for LinkExporter {
    fn format_proxy(&self, proxy: &Proxy, _index: usize) -> Result<Value, ExportError> {
        Ok(Value::String(proxy.original_link.clone()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UrltestOutbound {
    #[serde(rename = "type")]
    proxy_type: String,
    tag: String,
    outbounds: Vec<String>,
    url: String,
    interval: String,
    tolerance: i64,
    idle_timeout: String,
    interrupt_exist_connections: bool,
}

impl Default for UrltestOutbound {
    fn default() -> Self {
        Self {
            proxy_type: "urltest".to_string(),
            tag: "urltest-out".to_string(),
            outbounds: vec![],
            url: "https://www.gstatic.com/generate_204".to_string(),
            interval: "1m".to_string(),
            tolerance: 50,
            idle_timeout: "10m".to_string(),
            interrupt_exist_connections: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OutboundsJson {
    outbounds: Vec<OutboundsEnum>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum OutboundsEnum {
    UrltestOutbound(UrltestOutbound),
    Value(Value),
}

impl FromIterator<OutboundsEnum> for OutboundsJson {
    fn from_iter<T: IntoIterator<Item = OutboundsEnum>>(iter: T) -> Self {
        let mut outbounds = vec![];
        for i in iter {
            outbounds.push(i);
        }
        Self { outbounds }
    }
}

#[derive(Default)]
pub struct SingBoxExporter;

impl SingBoxExporter {
    pub fn new() -> Self {
        Self {}
    }

    fn format_tls(tls_settings: &TlsSettings) -> Result<Value, ExportError> {
        if !tls_settings.enabled {
            return Ok(json!({ "enabled": false }));
        }

        let mut tls_json = json!({ "enabled": true });
        let obj = tls_json.as_object_mut().unwrap();

        if let Some(server_name) = &tls_settings.server_name {
            obj.insert("server_name".to_string(), server_name.clone().into());
        }
        if let Some(alpn) = &tls_settings.alpn {
            obj.insert("alpn".to_string(), json!(alpn));
        }
        if let Some(fingerprint) = &tls_settings.fingerprint {
            if !fingerprint.is_empty() {
                obj.insert(
                    "utls".to_string(),
                    json!({
                        "enabled": true,
                        "fingerprint": fingerprint
                    }),
                );
            }
        }
        if let Some(reality) = &tls_settings.reality {
            if reality.enabled {
                obj.insert(
                    "reality".to_string(),
                    json!({
                        "enabled": true,
                        "public_key": reality.public_key,
                        "short_id": reality.short_id,
                    }),
                );
            }
        }

        Ok(tls_json)
    }

    fn format_transport(
        transport_settings: &TransportSettings,
    ) -> Result<Option<Value>, ExportError> {
        let transport_json = match transport_settings {
            TransportSettings::Quic => json!({ "type": "quic" }),
            TransportSettings::Grpc { service_name } => json!({
                "type": "grpc",
                "service_name": service_name,
            }),
            TransportSettings::Http { host, path, method } => {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), "http".into());
                if let Some(h) = host {
                    obj.insert("host".to_string(), json!(h));
                }
                obj.insert("path".to_string(), path.clone().into());
                obj.insert("method".to_string(), method.clone().into());
                Value::Object(obj)
            }
            TransportSettings::HttpUpgrade { host, path } => {
                let mut obj = serde_json::Map::new();
                obj.insert("type".to_string(), "http".into());
                if let Some(h) = host {
                    obj.insert("host".to_string(), json!(h));
                }
                obj.insert("path".to_string(), path.clone().into());
                Value::Object(obj)
            }

            TransportSettings::WebSocket { path } => json!({
                "type": "ws",
                "path": path,
            }),
            TransportSettings::Tcp => {
                return Ok(None);
            }
        };
        Ok(Some(transport_json))
    }
}

impl Exporter for SingBoxExporter {
    fn format_proxy(&self, proxy: &Proxy, index: usize) -> Result<Value, ExportError> {
        let tag = proxy
            .geoip
            .as_ref()
            .map(|geo| {
                format!(
                    "{}-{}",
                    geo.name.clone().unwrap_or(geo.iso_code.clone()),
                    index + 1
                )
            })
            .unwrap_or_else(|| format!("{}-{}", proxy.proxy_type, index + 1));

        trace!("got tag for proxy: {tag}");

        let mut outbound = json!({
            "tag": Value::String(tag),
            "server": proxy.address,
            "server_port": proxy.port,
        });
        let obj = outbound.as_object_mut().unwrap();

        match proxy.proxy_type {
            ProxyType::Vmess => {
                // if let ProxyDetails::Vmess { uuid, tls, transport, .. } = &proxy.details {
                //     obj.insert("type".to_string(), "vmess".into());
                //     obj.insert("uuid".to_string(), uuid.clone().into());
                //     obj.insert("security".to_string(), "auto".into());
                //     obj.insert("alter_id".to_string(), 0.into());

                //     if tls.enabled {
                //         obj.insert("tls".to_string(), Self::format_tls(tls)?);
                //     }
                //     if let Some(transport_value) = Self::format_transport(transport)? {
                //         obj.insert("transport".to_string(), transport_value);
                //     }
                // } else {
                //     return Err(ExportError::UnsupportedProtocol(
                //         "Mismatched proxy type and details for Vmess".to_string(),
                //     ));
                // }
                unimplemented!()
            }
            ProxyType::Vless => {
                if let ProxyDetails::Vless {
                    uuid,
                    flow: _,
                    tls,
                    transport,
                } = &proxy.details
                {
                    obj.insert("type".to_string(), "vless".into());
                    obj.insert("uuid".to_string(), uuid.clone().into());
                    obj.insert("packet_encoding".to_string(), "xudp".into());
                    // if let Some(f) = flow {
                    //     obj.insert("flow".to_string(), f.clone().into());
                    // }
                    if tls.enabled {
                        obj.insert("tls".to_string(), Self::format_tls(tls)?);
                    }
                    if let Some(transport_value) = Self::format_transport(transport)? {
                        obj.insert("transport".to_string(), transport_value);
                    }
                } else {
                    return Err(ExportError::UnsupportedProtocol(
                        "Mismatched proxy type and details for Vless".to_string(),
                    ));
                }
            }
            ProxyType::Trojan => {
                // if let ProxyDetails::Trojan {
                //     password,
                //     tls,
                //     transport,
                // } = &proxy.details
                // {
                //     obj.insert("type".to_string(), "trojan".into());
                //     obj.insert("password".to_string(), password.clone().into());
                //     if tls.enabled {
                //         obj.insert("tls".to_string(), Self::format_tls(tls)?);
                //     }
                //     if let Some(transport_value) = Self::format_transport(transport)? {
                //         obj.insert("transport".to_string(), transport_value);
                //     }
                // } else {
                //     return Err(ExportError::UnsupportedProtocol(
                //         "Mismatched proxy type and details for Trojan".to_string(),
                //     ));
                // }
                unimplemented!()
            }
            ProxyType::Shadowsocks => {
                if let ProxyDetails::Shadowsocks { method, password } = &proxy.details {
                    obj.insert("type".to_string(), "shadowsocks".into());
                    obj.insert("method".to_string(), method.clone().into());
                    obj.insert("password".to_string(), password.clone().into());
                } else {
                    return Err(ExportError::UnsupportedProtocol(
                        "Mismatched proxy type and details for Shadowsocks".to_string(),
                    ));
                }
            }
            pt => return Err(ExportError::UnsupportedProtocol(pt.to_string())),
        }
        Ok(outbound)
    }

    fn export(&self, proxies: &[Proxy]) -> Result<String, ExportError> {
        let (outbounds, tags): (Vec<Value>, Vec<String>) = proxies
            .iter()
            .enumerate()
            .filter_map(|(i, p)| self.format_proxy(p, i).ok())
            .filter_map(|outbound| {
                let tag = outbound
                    .get("tag")
                    .and_then(|t| t.as_str())
                    .map(String::from);
                tag.map(|t| (outbound, t))
            })
            .unzip();

        trace!(?tags, "Got tags for sing-box exporter");

        let urltest = OutboundsEnum::UrltestOutbound(UrltestOutbound {
            outbounds: tags,
            ..Default::default()
        });

        let all_outbounds_iter =
            std::iter::once(urltest).chain(outbounds.into_iter().map(OutboundsEnum::Value));

        let out = all_outbounds_iter.collect::<OutboundsJson>();

        serde_json::to_string_pretty(&out).map_err(From::from)
    }
}
