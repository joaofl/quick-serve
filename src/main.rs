// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: MIT

// use chrono::NaiveDate;
use slint::SharedString;

// use ftp-server

slint::slint!(import { AnyServeUI } from "src/ui.slint";);

pub fn main() {
    let ui = AnyServeUI::new().unwrap();
    // ui.on_validate_date(|date: SharedString| {
    //     NaiveDate::parse_from_str(date.as_str(), "%d.%m.%Y").is_ok()
    // });
    // ui.on_compare_date(|date1: SharedString, date2: SharedString| {
    //     let date1 = match NaiveDate::parse_from_str(date1.as_str(), "%d.%m.%Y") {
    //         Err(_) => return false,
    //         Ok(x) => x,
    //     };
    //     let date2 = match NaiveDate::parse_from_str(date2.as_str(), "%d.%m.%Y") {
    //         Err(_) => return false,
    //         Ok(x) => x,
    //     };
    //     date1 <= date2
    // });

    // ftp-server.start_ftp_server();

    ui.run().unwrap();
}