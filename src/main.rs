use std::{thread, time};
use std::net::IpAddr;
use std::str::FromStr;

use dns_lookup::lookup_host;
use winping::{Buffer, Pinger};

fn main() {
    let ip_target = resolve_ip_target();
    match ip_target {
        Some(addr) => ping_loop(addr),
        None => print_usage(),
    }
}

fn ping_loop(ip_target: IpAddr) {
    let pinger = Pinger::new().unwrap();
    let mut buffer = Buffer::new();
    let mut counter = 1;

    loop {
        match pinger.send(ip_target, &mut buffer) {
            Ok(return_time) => println!("{}. Reply from '{}' after {}ms", counter, &ip_target, &return_time),
            Err(error) => println!("{}. Error: {}", counter, error),
        }

        counter += 1;
        thread::sleep(time::Duration::from_millis(1000));
    }
}

// Attempt to resolve th IP address. First attempt to parse first command line
// argument directly as IP address. If it is not an IP, perform a DNS lookup for the text.
fn resolve_ip_target() -> Option<IpAddr> {
    let arg = std::env::args()
        .nth(1);

    return if let Some(addr_arg) = arg {
        let socket_addr = IpAddr::from_str(&addr_arg);

        match socket_addr {
            Ok(addr) => Some(addr),
            Err(_e) => address_lookup(&addr_arg),
        }
    } else {
        None
    }
}

// Resolves the given target to an IP address. Returns `None` on error.
fn address_lookup(target: &String) -> Option<IpAddr> {
    let resolved_addresses = lookup_host(&target);
    return match resolved_addresses {
        Ok(addresses) => Some(addresses[0]),
        Err(e) => {
            println!("Error resolving '{}': {}", target, e);
            None
        },
    }
}

// Reminder to add a single command line argument, for the target host.
fn print_usage() {
    println!("Error: Expected ping target as first argument, or can't resolve target!");
}
