use std::time::{SystemTime, UNIX_EPOCH};

pub fn _print_request(
    mut _req: tonic::Request<()>,
) -> tonic::Result<tonic::Request<()>, tonic::Status> {
    // println!("intercept: {:#?}", req);
    // req.extensions_mut().insert("????");
    Ok(_req)
}

pub fn _heavy_work(level: i64) {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let mut ff: f64 = time as f64 % 10000_f64 + 12345_f64;
    for i in 1..1000000 * level {
        ff = ff * ff % i as f64 % 10000_f64 + 12345_f64;
    }
}
