use axum::{routing::get, Router};

const STATUS_KEYS: &'static [&str] = &[
    "dummy0",
    "firmware",
    "Betriebstag",
    "Status",
    "DcTrenner",
    "DcRelais",
    "AcRelais",
    "Wassertemp",
    "WassertempMin",
    "WassertempMax",
    "SolltempSolar",
    "SolltempNetz",
    "GeraeteTemp",
    "IsoMessung",
    "Solarspannung",
    "dummy5",
    "Solarstrom",
    "Solarleistung",
    "SolarenergieHeute",
    "SolarenergieGesammt",
    "Netzernergie",
    "dummy6",
    "dummy7",
    "dummy8",
    "dummy9",
    "dummy10",
    "dummy11",
    "dummy12",
    "Seriennummer",
    "dummy13",
];

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));

    axum::Server::bind(&"0.0.0.0:3001".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> String {
    log::info!("Fetch new data");

    let data = read_device();
    let data_string = std::str::from_utf8(&data).unwrap();

    let mut current_status = String::new();
    for (value, &key) in data_string.split('\t').zip(STATUS_KEYS.iter()) {
        current_status.push_str(&format!("{key}: {value}\n"));
    }

    current_status
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
