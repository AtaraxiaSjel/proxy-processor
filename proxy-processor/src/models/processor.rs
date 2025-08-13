use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Vmess,
    Vless,
    Trojan,
    Shadowsocks,
    ShadowsocksR,
    Hysteria2,
}

impl fmt::Display for ProxyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProxyType::Vmess => write!(f, "vmess"),
            ProxyType::Vless => write!(f, "vless"),
            ProxyType::Trojan => write!(f, "trojan"),
            ProxyType::Shadowsocks => write!(f, "ss"),
            ProxyType::ShadowsocksR => write!(f, "ssr"),
            ProxyType::Hysteria2 => write!(f, "h2"),
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
#[error("invalid proxy type")]
pub struct ParseProxyTypeError;

impl FromStr for ProxyType {
    type Err = ParseProxyTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vmess" => Ok(ProxyType::Vmess),
            "vless" => Ok(ProxyType::Vless),
            "trojan" => Ok(ProxyType::Trojan),
            "shadowsocks" | "ss" => Ok(ProxyType::Shadowsocks),
            "shadowsocksr" | "ssr" => Ok(ProxyType::ShadowsocksR),
            "hysteria2" | "h2" => Ok(ProxyType::Hysteria2),
            _ => Err(ParseProxyTypeError),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TlsSettings {
    pub enabled: bool,
    pub server_name: Option<String>,
    pub alpn: Option<Vec<String>>,
    pub fingerprint: Option<String>,
    pub reality: Option<RealitySettings>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RealitySettings {
    pub enabled: bool,
    pub public_key: String,
    pub short_id: String,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum TransportSettings {
    Grpc {
        service_name: String,
    },
    Http {
        host: Option<Vec<String>>,
        path: String,
        method: String,
    },
    HttpUpgrade {
        host: Option<Vec<String>>,
        path: String,
    },
    Quic,
    #[default]
    Tcp,
    WebSocket {
        path: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum ProxyDetails {
    #[default]
    Generic,
    Vmess {
        uuid: String,
        tls: TlsSettings,
        transport: TransportSettings,
    },
    Vless {
        uuid: String,
        flow: Option<String>,
        tls: TlsSettings,
        transport: TransportSettings,
    },
    Trojan {
        password: String,
        tls: TlsSettings,
        transport: TransportSettings,
    },
    Shadowsocks {
        method: String,
        password: String,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Proxy {
    pub original_link: String,
    pub address: String,
    pub port: u16,
    pub remarks: Option<String>,
    pub proxy_type: ProxyType,
    pub details: ProxyDetails,
    pub geoip: Option<GeoIpInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct GeoIpInfo {
    pub is_in_eu: bool,
    pub iso_code: String,
    pub name: Option<String>,
}

#[derive(Debug, Default)]
pub struct FilterOptions {
    pub proxy_types: Option<Vec<ProxyType>>,
    pub include_countries: Option<Vec<String>>,
    pub exclude_countries: Option<Vec<String>>,
}

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct VmessBase64Config {
//     #[serde(rename = "ps")]
//     pub remarks: String,
//     #[serde(rename = "add")]
//     pub address: String,
//     pub port: u16,
//     pub id: String,
//     pub net: String,
//     #[serde(rename = "type")]
//     pub header_type: String,
//     pub tls: String,
// }
