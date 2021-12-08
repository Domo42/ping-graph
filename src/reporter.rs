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
use std::cmp::{max, min};
use std::fs::{File, OpenOptions};
use std::io::{Error, Write};
use std::net::IpAddr;
use std::path::Path;
use std::time::{Duration, Instant};

use chrono::Local;

/// The reporter creates two csv files, one data file 'ping-data.cvs' with the raw information about
/// each ping and a 'ping-summary.csv' where the data is aggregated in 10min buckets.
pub struct Reporter {
    ping_counter: u32,
    output_folder: String,
    target: IpAddr,
    bucket: PingBucket,
    total: PingBucket,
    bucket_start: Instant,
}

/// Holds the aggregated information for a single entry in the summary file.
struct PingBucket {
    ping_attempts: u32,
    ping_count: u32,
    pings_lost: u32,
    latency_avg: f32,
    latency_min: u32,
    latency_max: u32,
}

/// The reporter creates two csv files, one data file 'ping-data.cvs' with the raw information about
/// each ping and a 'ping-summary.csv' where the data is aggregated in 10min buckets.
impl Reporter {
    pub fn new(ip_target: &IpAddr) -> Result<Self, Error> {
        let reporter = Reporter {
            ping_counter: 0,
            target: ip_target.clone(),
            output_folder: String::from("."),
            bucket: PingBucket::new(),
            total: PingBucket::new(),
            bucket_start: Instant::now(),
        };

        return Ok(reporter);
    }

    /// Report a returned ping within the given latency.
    pub fn report_value(&mut self, latency: u32) -> Result<(), Error> {
        self.ping_counter += 1;
        self.total.feed_latency(latency);
        self.bucket.feed_latency(latency);
        println!("{}. Reply from '{}' after {}ms", self.ping_counter, &self.target, latency);

        return Self::update_files(self, latency, false);
    }

    /// Reports that a ping has not been returned.
    pub fn report_packet_loss(&mut self, error: winping::Error ) -> Result<(), Error> {
        self.ping_counter += 1;
        self.total.feed_lost_ping();
        self.bucket.feed_lost_ping();
        println!("{}. Error: {}", self.ping_counter, error);

        return Self::update_files(self, 0, true);
    }

    /// Prints the ping statistics of all handled pings as readable output to stdout.
    pub fn print_total_stats(&self) {
        println!();
        println!("Ping statistics for {}", self.target);
        println!(
            "    Packets: Sent = {}, Received = {}, Lost = {} ({:02.2}% loss),",
            self.total.ping_count,
            self.total.ping_count - self.total.pings_lost,
            self.total.pings_lost,
            self.total.lost_percentage()
        );
        println!("Approximate round trip times in milli-seconds:");
        println!(
            "    Minimum = {}ms, Maximum = {}ms, Average = {:3.1}ms",
            self.total.latency_min,
            self.total.latency_max,
            self.total.latency_avg
        )
    }

    fn update_files(&mut self, latency: u32, is_packet_loss:bool) -> Result<(), Error> {
        let  summary_file = Self::summary_file(self)?;
        Self::check_update_summary(self, &summary_file)?;

        let data_file = Self::data_file(self)?;
        return Self::write_raw_data(self, &data_file, latency, is_packet_loss);
    }

    fn data_file(&self) -> Result<File, Error> {
        let path = Path::new(&self.output_folder).join("ping-data.csv");
        let is_new_file = !path.exists();

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path);

        if let Ok(mut f) = file.as_ref() {
            if is_new_file {
                // write header if new file did not exist before
                f.write("Time,Target,Latency (ms),IsPacketLoss\n".as_bytes())?;
            }
        }

        return file;
    }

    fn write_raw_data(&self, mut data_file: &File, latency: u32, is_packet_loss: bool) -> Result<(), Error> {
        let date_str = now_iso();
        let line = format!("{}, {}, {}, {}\n", date_str, self.target, latency, is_packet_loss);

        return data_file.write(line.as_bytes()).map(|_bytes_written| {});
    }

    fn summary_file(&self) -> Result<File, Error> {
        let path = Path::new(&self.output_folder).join("ping-summary.csv");
        let is_new_file = !path.exists();

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path);

        if let Ok(mut f) = file.as_ref() {
            if is_new_file {
                // write header if new file did not exist before
                f.write("Time, Target, Sent, Lost, Lost (%), Min Latency (ms), Max Latency (ms), Avg Latency (ms)\n".as_bytes())?;
            }
        }

        return file;
    }

    fn check_update_summary(&mut self, mut summary_file: &File) -> Result<(), Error> {
        let since_last = self.bucket_start.elapsed();
        if since_last.ge(&Duration::from_secs(600)) {
            let date_str = now_iso();
            let line = format!("{}, {}, {}, {}, {}, {}, {}, {}",
                date_str,
                self.target,
                self.bucket.ping_attempts,
                self.bucket.pings_lost,
                self.bucket.lost_percentage(),
                self.bucket.latency_min,
                self.bucket.latency_max,
                self.bucket.latency_avg,
            );

            self.bucket_start = Instant::now();
            return summary_file.write(line.as_bytes()).map(|_| {});
        } else {
            return Ok(());
        }
    }
}

impl PingBucket {
    fn new() -> PingBucket {
        return PingBucket {
            latency_avg: 0.0,
            ping_count: 0,
            pings_lost: 0,
            latency_min: 10000,
            latency_max: 0,
            ping_attempts: 0
        };
    }

    fn feed_latency(&mut self, latency: u32) {
        self.latency_min = min(self.latency_min, latency);
        self.latency_max = max(self.latency_max, latency);

        let latency_float = latency as f32;

        if self.ping_count == 0 {
            self.latency_avg = latency_float;
        } else {
            self.latency_avg = self.latency_avg + (latency_float - self.latency_avg)/(self.ping_count as f32);
        }

        self.ping_attempts += 1;
        self.ping_count += 1;
    }

    fn feed_lost_ping(&mut self) {
        self.pings_lost += 1;
        self.ping_attempts += 1;
    }

    fn lost_percentage(&self) -> f32 {
        let a = self.pings_lost as f32 / self.ping_attempts as f32;
        return 100.0 * a;
    }
}

// create an ISO 8601 string with the current time.
fn now_iso() -> String {
    let now = Local::now();
    return now.format("%FT%T%.3f%z").to_string();
}