/*  Copyright (C) 2012-2018 by László Nagy
    This file is part of Bear.

    Bear is a tool to generate compilation database for clang tooling.

    Bear is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Bear is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::env;
use std::path;
use std::process;

use chrono;

use crate::{ErrorKind, Result, ResultExt};
use crate::event::*;

pub struct Supervisor<'a> {
    sink: Box<FnMut(Event) -> Result<()> + 'a>,
}

impl<'a> Supervisor<'a> {
    pub fn new<F: 'a>(sink: F) -> Supervisor<'a>
        where F: FnMut(Event) -> Result<()> {
        Supervisor { sink: Box::new(sink) }
    }

    pub fn run(&mut self, cmd: &[String]) -> Result<()> {
        let cwd = env::current_dir()
            .chain_err(|| "unable to get current working directory")?;
//        let mut child = process::Command::new(&cmd[0]).args(&cmd[1..]).spawn()
//            .chain_err(|| format!("unable to execute process: {:?}", cmd[0]))?;
//
//        debug!("process was started: {:?}", child.id());
//        let event = Event::Created(
//            ProcessCreated {
//                pid: child.id(),
//                ppid: get_parent_pid(),
//                cwd: cwd.clone(),
//                cmd: cmd.to_vec(), },
//            chrono::Utc::now());
//        self.report(event);
//
//        match child.wait() {
//            Ok(status) => {
//                debug!("process was stopped: {:?}", child.id());
//                let event = match status.code() {
//                    Some(code) => {
//                        let message = ProcessTerminatedNormally { pid: child.id(), code };
//                        Event::TerminatedNormally(message, chrono::Utc::now())
//                    }
//                    None => {
//                        let message = ProcessTerminatedAbnormally { pid: child.id(), signal: -1 };
//                        Event::TerminatedAbnormally(message, chrono::Utc::now())
//                    }
//                };
//                self.report(event);
//            }
//            Err(_) => {
//                warn!("process was not running: {:?}", child.id());
//            }
//        }
        Ok(())
    }

    fn report(&mut self, event: Event) {
        match (self.sink)(event) {
            Ok(_) => debug!("Event sent."),
            Err(error) => debug!("Event sending failed. {:?}", error),
        }
    }
}

fn get_parent_pid() -> ProcessId {
    match env::var("INTERCEPT_PPID") {
        Ok(value) => {
            match value.parse() {
                Ok(ppid) => ppid,
                _ => 0,
            }
        },
        _ => 0,
    }
}
