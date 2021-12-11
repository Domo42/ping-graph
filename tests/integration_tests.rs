use std::io::Error;
use std::net::IpAddr;
use std::str::FromStr;
use ping_graph::reporter::Reporter;

#[test]
fn reporter_total_value() -> Result<(), Error> {
    let mut sut = Reporter::new(&IpAddr::from_str("127.0.0.1").unwrap());
    sut.report_value(1)?;
    sut.print_total_stats();

    Ok(())
}