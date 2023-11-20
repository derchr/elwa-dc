use std::collections::HashMap;

use anyhow::Context;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use strum::{EnumIter, IntoEnumIterator};

#[derive(EnumIter, PartialEq, Eq, Hash, Debug)]
enum StatusTag {
    Dummy0,
    Firmware,
    Betriebstag,
    Status,
    DcTrenner,
    DcRelais,
    AcRelais,
    Wassertemp,
    WassertempMin,
    WassertempMax,
    SolltempSolar,
    SolltempNetz,
    GeraeteTemp,
    IsoMessung,
    Solarspannung,
    Dummy5,
    Solarstrom,
    Solarleistung,
    SolarenergieHeute,
    SolarenergieGesamt,
    NetzenergieHeute,
    Dummy6,
    Dummy7,
    Dummy8,
    Dummy9,
    Dummy10,
    Dummy11,
    Dummy12,
    Seriennummer,
    Dummy13,
}

#[derive(Debug)]
struct Status<'a> {
    // Wasser
    wassertemp: f32,
    wassertemp_min: f32,
    wassertemp_max: f32,
    solltemp_solar: f32,
    solltemp_netz: f32,

    // Solar aktuell
    solarspannung: f32,
    solarstrom: f32,
    solarleistung: f32,

    // Historie
    solarenergie_heute: f32,
    solarenergie_gesamt: f32,
    netzenergie_heute: f32,

    // Zustand
    iso_messung: u32,
    geraetetemp: f32,
    status: u32,
    dc_trenner: bool,
    dc_relais: bool,
    ac_relais: bool,

    // Misc
    betriebstag: u32,
    firmware: &'a str,
    seriennummer: &'a str,
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal Server Error:\n{:?}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Result<Html<String>, AppError> {
    log::info!("Fetch new data");

    let data = read_device().context("Could not retrieve device data")?;
    let data_string = std::str::from_utf8(&data).unwrap();

    let status_map = StatusTag::iter()
        .zip(data_string.split('\t'))
        .collect::<HashMap<StatusTag, &str>>();

    let status = Status {
        wassertemp: status_map[&StatusTag::Wassertemp].parse::<f32>()? / 10.0,
        wassertemp_min: status_map[&StatusTag::WassertempMin].parse::<f32>()? / 10.0,
        wassertemp_max: status_map[&StatusTag::WassertempMax].parse::<f32>()? / 10.0,
        solltemp_solar: status_map[&StatusTag::SolltempSolar].parse::<f32>()? / 10.0,
        solltemp_netz: status_map[&StatusTag::SolltempNetz].parse::<f32>()? / 10.0,
        solarspannung: status_map[&StatusTag::Solarspannung].parse()?,
        solarstrom: status_map[&StatusTag::Solarstrom].parse()?,
        solarleistung: status_map[&StatusTag::Solarleistung].parse::<f32>()? / 1000.0,
        solarenergie_heute: status_map[&StatusTag::SolarenergieHeute].parse::<f32>()? / 1000.0,
        solarenergie_gesamt: status_map[&StatusTag::SolarenergieGesamt].parse::<f32>()? / 1000.0,
        netzenergie_heute: status_map[&StatusTag::NetzenergieHeute].parse::<f32>()? / 1000.0,
        iso_messung: status_map[&StatusTag::IsoMessung].parse()?,
        geraetetemp: status_map[&StatusTag::GeraeteTemp].parse()?,
        status: status_map[&StatusTag::Status].parse()?,
        dc_trenner: status_map[&StatusTag::DcTrenner].parse::<u8>()? != 0,
        dc_relais: status_map[&StatusTag::DcRelais].parse::<u8>()? != 0,
        ac_relais: status_map[&StatusTag::AcRelais].parse::<u8>()? != 0,
        betriebstag: status_map[&StatusTag::Betriebstag].parse()?,
        firmware: status_map[&StatusTag::Firmware],
        seriennummer: status_map[&StatusTag::Seriennummer],
    };

    Ok(Html(format!(
        include_str!("index.html"),
        status.wassertemp,
        status.wassertemp_min,
        status.wassertemp_max,
        status.solltemp_solar,
        status.solltemp_netz,
        status.solarspannung,
        status.solarstrom,
        status.solarleistung,
        status.solarleistung * 1000.0,
        status.solarenergie_heute,
        status.solarenergie_heute * 1000.0,
        status.solarenergie_gesamt,
        status.solarenergie_gesamt * 1000.0,
        status.netzenergie_heute,
        status.netzenergie_heute * 1000.0,
        status.iso_messung,
        status.geraetetemp,
        status.status,
        status.dc_trenner,
        status.dc_relais,
        status.ac_relais,
        status.betriebstag,
        status.firmware,
        status.seriennummer,
    )))
}

#[cfg(not(feature = "dummy"))]
fn read_device() -> anyhow::Result<Vec<u8>> {
    use std::io::{BufRead, BufReader};
    use std::time::Duration;

    let mut port = serialport::new("/dev/ttyUSB0", 9600)
        .timeout(Duration::from_millis(100))
        .open()
        .context("Could not open serial device port")?;

    write!(&mut port, "rs\r\n").context("Could not write to serial connection")?;

    let mut reader = BufReader::new(port);

    let mut data: Vec<u8> = Vec::new();
    reader
        .read_until(b'\n', &mut data)
        .context("Could not read from serial connection")?;

    Ok(data)
}

#[cfg(feature = "dummy")]
fn read_device() -> anyhow::Result<Vec<u8>> {
    use base64::{engine::general_purpose, Engine as _};
    const SAMPLE_OUTPUT: &str = "ZHIJVjEuMzEJMzUJMTIJMQkxCTEJMjM1CTE3NQkyNDUJNzU5CTY1MAkyNQk5MAkxODkuNQkxOTAuMDMJMS4xNDM1CTIxNy4yOQk3NzgJOTE3MjUJMAktNwk3LjkJNTI1CTM2OAkzNTgJMjQwCTEJMTIwMTAwMjMwMjEwMDAyMwk3NTkJNg0K";
    Ok(general_purpose::STANDARD.decode(SAMPLE_OUTPUT)?)
}
