use crate::error::ProcessorError;
use maxminddb::{Reader, geoip2};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::{instrument, trace};

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

    #[instrument(skip(self))]
    pub fn lookup_country_info(
        &self,
        ip: IpAddr,
    ) -> Result<Option<geoip2::country::Country>, ProcessorError> {
        trace!(?ip, "lookup ip address in maxmind db");
        self.reader
            .lookup::<geoip2::Country>(ip)
            .map(|country| country.and_then(|c| c.country))
            .map_err(|err| ProcessorError::GeoIpDbError(err.to_string()))
            .inspect(|geo| trace!(?geo, "looked up geoip"))
    }
}
