#![cfg(feature = "nightly")]

use dotenv::dotenv;
use std::env;

pub fn print_nightly_warning() {
    dotenv().ok();
    if env::var("SUPABASE_RS_NO_NIGHTLY_MSG").unwrap_or_else(|_| "false".to_owned()) == "true" {
        return;
    }
    println!("\x1b[34;1mYou are currently in NIGHTLY\x1b[0m");
    println!("\x1b[34;1mWarning: This is a nightly build and may contain bugs.\x1b[0m");
    println!("\x1b[34;1mFeatures 'Force rustls' and 'Graphql support' are currently under nightly mode.\x1b[0m");
    println!("\x1b[34;1mTo disable this message, set the environment variable SUPABASE_RS_NO_NIGHTLY_MSG to 'true'.\x1b[0m");
}

pub fn print_if_dev(message: &str) {
    dotenv().ok();
    if env::var("SUPABASE_RS_DEV").unwrap_or_else(|_| "false".to_owned()) == "true" {
        return;
    }
    println!("\x1b[34m{}\x1b[0m", message);
}
