/*
passmenu-rs
Copyright (C) 2020  Rupansh Sekar

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

mod config;
mod consts;
use consts::*;

extern crate dirs;
extern crate rustofi;
use dirs::home_dir;
use rustofi::{
    components::*,
    RustofiResult,
};

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};


fn main() {
    loop {
        match app_main() {
            _ => break
        }
    }
}

fn traverse_pass_dir(pass_dir: &PathBuf) -> Vec<String> {
    let mut pass_l: Vec<String> = Vec::new();
    for pass_e in fs::read_dir(pass_dir).unwrap() {
        let pass = pass_e.unwrap().path();
        if pass.is_dir() {
            pass_l.append(&mut traverse_pass_dir(&pass));
        } else {
            match pass.extension() {
                Some(s) => if s == "gpg" { pass_l.push(pass.to_str().unwrap().split(PASS_DIR).nth(1).unwrap().replace(".gpg", "").to_string()) },
                _ => {}
            }
        }
    }

    return pass_l;
}

fn app_main() -> RustofiResult {
    let pass_dir: PathBuf = match home_dir() {
        Some(p) => [p, PathBuf::from(PASS_DIR)].iter().collect(),
        _ => return RustofiResult::Exit
    };

    let args = env::args().collect::<Vec<String>>();
    let mut rofi_args = config::get_conf();
    if !args.iter().any(|s| s == "new") {
        ItemList::new(rofi_args, traverse_pass_dir(&pass_dir), Box::new(cb)).display("pass >".to_string())
    } else {
        match rofi_args.iter().position(|r| r == "-lines") {
            Some(i) => rofi_args[i+1] = "0".to_string(),
            _ => { 
                rofi_args.push("-lines".to_string());
                rofi_args.push("0".to_string())
            }
        }
        return match EntryBox::display(rofi_args, "pass --generate".to_string()) {
            RustofiResult::Selection(p) => { 
                Command::new(PASS_CMD).arg("generate").arg("--clip").arg(p).stdout(Stdio::null()).spawn().expect("FAILED TO GENERATE");
                println!("Password Generated and copied to clipboard!");
                RustofiResult::Success
            },
            _ => RustofiResult::Exit
        }
    }
}

fn cb(s: &mut String) -> Result<(), String> {
    if s != "" {
        Command::new(PASS_CMD).arg(s).arg("--clip").stdout(Stdio::null()).spawn().expect("FAILED TO DECRYPT");
        println!("Password copied to clipboard!")
    }
    Ok(())
}