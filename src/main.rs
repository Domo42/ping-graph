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

    loop {
        match pinger.send(ip_target, &mut buffer) {
            Ok(return_time) => println!("Reply from '{}' after {}ms", &ip_target, &return_time),
            Err(error) => println!("Error: {}", error),
        }

        thread::sleep(time::Duration::from_millis(1000));
    }
}

fn resolve_ip_target() -> Option<IpAddr> {
    let addr_arg = std::env::args()
        .nth(1)
        .unwrap_or(String::from(""));

    return if addr_arg.is_empty() {
        Option::None
    } else {
        let socket_addr = IpAddr::from_str(&addr_arg);
        println!("Attempted parse of IP Address: {} ", socket_addr.is_ok());

        return match socket_addr {
            Ok(addr) => Option::Some(addr),
            Err(_e) => address_lookup(&addr_arg),
        };
    }
}

fn address_lookup(target: &String) -> Option<IpAddr> {
    let resolved_addresses = lookup_host(&target).unwrap_or(vec![]);
    return resolved_addresses.first().or(Option::None).cloned();
}

fn print_usage() {
    println!("Error: Expected ping target as first argument, or can't resolve target!");
}
