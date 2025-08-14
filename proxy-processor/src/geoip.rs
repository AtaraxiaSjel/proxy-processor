use crate::error::ProcessorError;
use maxminddb::{Reader, geoip2};
use std::fs;
use std::sync::Arc;
use std::{net::IpAddr, path::Path};
use tracing::{instrument, trace};

#[derive(Clone)]
pub struct GeoIpReader {
    reader: Arc<Reader<Vec<u8>>>,
}

impl GeoIpReader {
    pub fn from_path<S: AsRef<Path>>(db_path: S) -> Result<Self, ProcessorError> {
        let buf = fs::read(db_path).map_err(|err| ProcessorError::GeoIpDbError(err.to_string()))?;
        let reader = Reader::from_source(buf)
            .map_err(|err| ProcessorError::GeoIpDbError(err.to_string()))?;
        Ok(GeoIpReader {
            reader: Arc::new(reader),
        })
    }

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self, ProcessorError> {
        let source = bytes.as_ref().to_vec();
        let reader = Reader::from_source(source)
            .map_err(|err| ProcessorError::GeoIpDbError(err.to_string()))?;
        Ok(GeoIpReader {
            reader: Arc::new(reader),
        })
    }

    #[instrument(skip(self))]
    pub fn lookup_country_info(
        &self,
        ip: IpAddr,
    ) -> Result<Option<geoip2::country::Country<'_>>, ProcessorError> {
        trace!(?ip, "lookup ip address in maxmind db");
        self.reader
            .lookup::<geoip2::Country>(ip)
            .map(|country| country.and_then(|c| c.country))
            .map_err(|err| ProcessorError::GeoIpDbError(err.to_string()))
            .inspect(|geo| trace!(?geo, "looked up geoip"))
    }
}
