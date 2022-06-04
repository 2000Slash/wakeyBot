use std::{net::Ipv4Addr, num::ParseIntError, process::Command};

use log::{debug, warn};

mod commands;
pub mod discord;

/// decodes a &str (12c0a7ff) to a u8 vec
fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

/// fetches the current public ip from api.ipify.org
fn fetch_ip() -> Option<Ipv4Addr> {
    let output = Command::new("curl")
        .args(["https://api.ipify.org"])
        .output();
    if output.is_err() {
        warn!("Error while executing curl {:?}", output.as_ref().err());
    } else if output.as_ref().unwrap().status.success() {
        let result = String::from_utf8(output.unwrap().stdout).unwrap();
        debug!("Fetched new ip: {}", &result);
        let ip: Result<Ipv4Addr, _> = result.parse();
        if ip.is_err() {
            warn!("Could not process result from ipify, {}", result);
        } else {
            return Some(ip.unwrap());
        }
    }

    None
}
