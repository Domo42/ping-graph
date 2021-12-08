/* Copyright 2021 Stefan Domnanovits

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License. */

use std::{thread, time};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use dns_lookup::lookup_host;
use winping::{Buffer, Pinger};

use reporter::Reporter;

mod reporter;

fn main() {
    let keep_going_shared = Arc::new(AtomicBool::new(true));
    let keep_going = keep_going_shared.clone();

    ctrlc::set_handler(move || {
        println!("Received STOP signal.");
        keep_going_shared.store(false, Ordering::Relaxed);
    }).expect("Error setting ctrl-c signal handler.");

    let ip_target = resolve_ip_target();
    match ip_target {
        Some(addr) => ping_loop(addr, &keep_going),
        None => print_usage(),
    };
}

fn ping_loop(ip_target: IpAddr, keep_going: &Arc<AtomicBool>) {
    let pinger = Pinger::new().unwrap();
    let mut reporter = Reporter::new(&ip_target).unwrap();
    let mut buffer = Buffer::new();

    while keep_going.load(Ordering::Relaxed) {
        let write_result = match pinger.send(ip_target, &mut buffer) {
            Ok(return_time) => reporter.report_value(return_time),
            Err(error) => reporter.report_packet_loss(error),
        };

        if let Err(error) = write_result {
            println!("Error reporting ping result: {}", error);
        }

        thread::sleep(time::Duration::from_millis(1000));
    }
}

// Attempt to resolve th IP address. First attempt to parse first command line
// argument directly as IP address. If it is not an IP, perform a DNS lookup for the text.
fn resolve_ip_target() -> Option<IpAddr> {
    let arg = std::env::args()
        .nth(1);

    return if let Some(addr_arg) = arg {
        match IpAddr::from_str(&addr_arg) {
            Ok(addr) => Some(addr),
            Err(_e) => address_lookup(&addr_arg),
        }
    } else {
        None
    }
}

// Resolves the given target to an IP address. Returns `None` on error.
fn address_lookup(target: &String) -> Option<IpAddr> {
    match lookup_host(&target) {
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
