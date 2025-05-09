//! Error exit codes for word-tally
//!
//! Follows Unix sysexits.h convention for exit code numbers.

use anyhow::Error;
use clap::error::ErrorKind as ClapErrorKind;
use std::io;

pub mod exit_code {
    pub const SUCCESS: i32 = 0;
    pub const FAILURE: i32 = 1;
    pub const USAGE: i32 = 64;
    pub const DATAERR: i32 = 65;
    pub const NOINPUT: i32 = 66;
    pub const CANTCREAT: i32 = 73;
    pub const IOERR: i32 = 74;
    pub const NOPERM: i32 = 77;
    pub const CONFIG: i32 = 78;
}

pub fn exit_code(err: &Error) -> i32 {
    if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
        return match clap_err.kind() {
            ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => exit_code::SUCCESS,
            _ => exit_code::USAGE,
        };
    }

    if let Some(io_err) = err.downcast_ref::<io::Error>() {
        return match io_err.kind() {
            io::ErrorKind::NotFound => exit_code::NOINPUT,
            io::ErrorKind::PermissionDenied => exit_code::NOPERM,
            io::ErrorKind::AlreadyExists => exit_code::CANTCREAT,
            _ => exit_code::IOERR,
        };
    }

    if err.downcast_ref::<regex::Error>().is_some()
        || err.downcast_ref::<serde_json::Error>().is_some()
        || err.downcast_ref::<csv::Error>().is_some()
        || err.downcast_ref::<std::str::Utf8Error>().is_some()
        || err.downcast_ref::<std::string::FromUtf8Error>().is_some()
    {
        return exit_code::DATAERR;
    }

    if err.downcast_ref::<unescaper::Error>().is_some() {
        return exit_code::CONFIG;
    }

    exit_code::FAILURE
}
