use std::time::{SystemTime, UNIX_EPOCH};

pub fn _print_request<T>(
    mut _req: tonic::Request<T>,
) -> tonic::Result<tonic::Request<T>, tonic::Status>
where
    T: std::fmt::Debug,
{
    println!("intercept: {:#?}", _req);
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
