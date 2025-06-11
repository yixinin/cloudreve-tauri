// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crcli_lib::proto::{storage::GetFilesAck, Ack};

fn main() {
    crcli_lib::run()
}
