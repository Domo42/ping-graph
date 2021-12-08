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
use std::fs::{File, OpenOptions};
use std::net::IpAddr;
use std::io::{Error, Write};
use std::path::{Path};
use chrono::{Local};

pub struct Reporter {
    ping_counter: u32,
    output_folder: String,
    target: IpAddr
}

/// The reporter creates two csv files, one data file 'ping-data.cvs' with the raw information about
/// each ping and a 'ping-summary.csv' where the data is aggregated in 10min buckets.
impl Reporter {
    pub fn new(ip_target: &IpAddr) -> Result<Self, Error> {
        let reporter = Reporter {
            ping_counter: 0,
            target: ip_target.clone(),
            output_folder: String::from("."),
        };

        return Ok(reporter);
    }

    /// Report a returned ping within the given latency.
    pub fn report_value(&mut self, latency: u32) -> Result<(), Error> {
        self.ping_counter += 1;
        println!("{}. Reply from '{}' after {}ms", self.ping_counter, &self.target, latency);

        let data_file = Self::data_file(self)?;
        return Self::write_raw_data(self, &data_file, latency, false);
    }

    /// Reports that a ping has not been returned.
    pub fn report_packet_loss(&mut self, error: winping::Error ) -> Result<(), Error> {
        self.ping_counter += 1;
        println!("{}. Error: {}", self.ping_counter, error);

        let data_file = Self::data_file(self)?;
        return Self::write_raw_data(self, &data_file, 0, true);
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

    fn write_raw_data(&self, mut data_file: &File, latency: u32, packet_loss: bool) -> Result<(), Error> {
        let now = Local::now();
        let date_str : String = now.format("%FT%T%.3f%z").to_string();
        let line = format!("{}, {}, {}, {}\n", date_str, self.target, latency, packet_loss);

        return data_file.write(line.as_bytes()).map(|_bytes_written| {});
    }
}