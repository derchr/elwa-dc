use axum::{response::Html, routing::get, Router};
use strum::{EnumIter, IntoEnumIterator};

#[derive(EnumIter, Debug)]
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

#[derive(Default, Debug)]
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
    geraetetemp: u32,
    status: u32,
    dc_trenner: bool,
    dc_relais: bool,
    ac_relais: bool,

    // Misc
    betriebstag: u32,
    firmware: &'a str,
    seriennummer: &'a str,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<String> {
    log::info!("Fetch new data");

    let data = read_device();
    let data_string = std::str::from_utf8(&data).unwrap();

    let mut status = Status::default();
    for (value, tag) in data_string.split('\t').zip(StatusTag::iter()) {
        match tag {
            StatusTag::Firmware => status.firmware = value,
            StatusTag::Betriebstag => status.betriebstag = str::parse(value).unwrap(),
            StatusTag::Status => status.status = str::parse(value).unwrap(),
            StatusTag::DcTrenner => status.dc_trenner = str::parse::<u8>(value).unwrap() != 0,
            StatusTag::DcRelais => status.dc_relais = str::parse::<u8>(value).unwrap() != 0,
            StatusTag::AcRelais => status.ac_relais = str::parse::<u8>(value).unwrap() != 0,
            StatusTag::Wassertemp => status.wassertemp = str::parse::<f32>(value).unwrap() / 10.0,
            StatusTag::WassertempMin => {
                status.wassertemp_min = str::parse::<f32>(value).unwrap() / 10.0
            }
            StatusTag::WassertempMax => {
                status.wassertemp_max = str::parse::<f32>(value).unwrap() / 10.0
            }
            StatusTag::SolltempSolar => {
                status.solltemp_solar = str::parse::<f32>(value).unwrap() / 10.0
            }
            StatusTag::SolltempNetz => {
                status.solltemp_netz = str::parse::<f32>(value).unwrap() / 10.0
            }
            StatusTag::GeraeteTemp => status.geraetetemp = str::parse(value).unwrap(),
            StatusTag::IsoMessung => status.iso_messung = str::parse(value).unwrap(),
            StatusTag::Solarspannung => status.solarspannung = str::parse::<f32>(value).unwrap(),
            StatusTag::Solarstrom => status.solarstrom = str::parse::<f32>(value).unwrap(),
            StatusTag::Solarleistung => {
                status.solarleistung = str::parse::<f32>(value).unwrap() / 1000.0
            }
            StatusTag::SolarenergieHeute => {
                status.solarenergie_heute = str::parse::<f32>(value).unwrap() / 1000.0
            }
            StatusTag::SolarenergieGesamt => {
                status.solarenergie_gesamt = str::parse::<f32>(value).unwrap() / 1000.0
            }
            StatusTag::NetzenergieHeute => {
                status.netzenergie_heute = str::parse::<f32>(value).unwrap() / 1000.0
            }
            StatusTag::Seriennummer => status.seriennummer = value,
            _ => (),
        }
    }

    Html(format!(
        include_str!("index.html"),
        status.wassertemp,
        status.wassertemp_min,
        status.wassertemp_max,
        status.solltemp_solar,
        status.solltemp_netz,
        status.solarspannung,
        status.solarstrom,
        status.solarleistung,
        status.solarenergie_heute,
        status.solarenergie_gesamt,
        status.netzenergie_heute,
        status.iso_messung,
        status.geraetetemp,
        status.status,
        status.dc_trenner,
        status.dc_relais,
        status.ac_relais,
        status.betriebstag,
        status.firmware,
        status.seriennummer,
    ))
}

#[cfg(not(feature = "dummy"))]
fn read_device() -> Vec<u8> {
    use std::io::{BufRead, BufReader};
    use std::time::Duration;

    let mut port = serialport::new("/dev/ttyUSB0", 9600)
        .timeout(Duration::from_secs(5))
        .open()
        .expect("Failed to open port");

    write!(&mut port, "rs\r\n").expect("Write failed!");

    let mut reader = BufReader::new(port);

    let mut data: Vec<u8> = Vec::new();
    reader.read_until(b'\n', &mut data).expect("Found no data!");

    data
}

#[cfg(feature = "dummy")]
fn read_device() -> Vec<u8> {
    use base64::{engine::general_purpose, Engine as _};
    const SAMPLE_OUTPUT: &str = "ZHIJVjEuMzEJMzUJMTIJMQkxCTEJMjM1CTE3NQkyNDUJNzU5CTY1MAkyNQk5MAkxODkuNQkxOTAuMDMJMS4xNDM1CTIxNy4yOQk3NzgJOTE3MjUJMAktNwk3LjkJNTI1CTM2OAkzNTgJMjQwCTEJMTIwMTAwMjMwMjEwMDAyMwk3NTkJNg0K";
    general_purpose::STANDARD.decode(SAMPLE_OUTPUT).unwrap()
}
