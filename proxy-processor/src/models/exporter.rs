use serde::{Deserialize, Serialize};

/// Sing-box outbound for supported protocols
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    #[serde(rename = "type")]
    pub type_field: String,
    pub tag: String,
    pub server: String,
    #[serde(rename = "server_port")]
    pub server_port: i64,
    pub uuid: String,
    pub flow: String,
    pub network: String,
    pub tls: Tls,
    #[serde(rename = "packet_encoding")]
    pub packet_encoding: String,
    pub multiplex: Multiplex,
    pub transport: Transport,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tls {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Multiplex {}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transport {}
