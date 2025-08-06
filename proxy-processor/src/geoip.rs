use crate::error::ProcessorError;
use maxminddb::Reader;
use std::net::IpAddr;
use std::sync::Arc;

#[derive(Clone)]
pub struct GeoIpReader {
    reader: Arc<Reader<Vec<u8>>>,
}

impl GeoIpReader {
    pub fn new(db_path: &str) -> Result<Self, ProcessorError> {
        let reader = Reader::open_readfile(db_path)
            .map_err(|err| ProcessorError::GeoIpDbError(err.to_string()))?;
        Ok(GeoIpReader {
            reader: Arc::new(reader),
        })
    }

    pub fn lookup_country_iso(&self, ip: IpAddr) -> Result<Option<String>, ProcessorError> {
        self.reader
            .lookup(ip)
            .map_err(|err| ProcessorError::GeoIpDbError(err.to_string()))
    }
}
