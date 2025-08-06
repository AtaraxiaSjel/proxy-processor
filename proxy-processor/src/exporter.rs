use crate::error::ExportError;
use crate::models::Proxy;
use serde_json::Value;

pub trait Exporter {
    fn format_proxy(&self, proxy: &Proxy, index: usize) -> Result<Value, ExportError>;

    fn export(&self, proxies: &[Proxy]) -> Result<String, ExportError> {
        let outbounds: Vec<Value> = proxies
            .iter()
            .enumerate()
            .filter_map(|(i, p)| self.format_proxy(p, i).ok())
            .collect();

        serde_json::to_string_pretty(&outbounds).map_err(ExportError::from)
    }
}

pub struct SingBoxExporter;

impl Exporter for SingBoxExporter {
    fn format_proxy(&self, _proxy: &Proxy, _index: usize) -> Result<Value, ExportError> {
        unimplemented!()
    }
}
