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

extern crate chrono;
#[macro_use]
extern crate error_chain;
extern crate libc;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate shellwords;
#[macro_use]
extern crate lazy_static;
extern crate regex;
#[macro_use]
extern crate log;

//#[cfg(test)]
//extern crate mockers;
//#[cfg(test)]
//extern crate mockers_derive;

pub mod trace;
pub mod event;
pub mod database;
pub mod compilation;

mod error {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Env(::std::env::VarError);
            String(::std::str::Utf8Error);
            Json(::serde_json::Error);
        }

        errors {
            CompilationError(msg: &'static str) {
                description("compilation error"),
                display("compilation error: '{}'", msg),
            }

            RuntimeError(msg: &'static str) {
                description("runtime error"),
                display("runtime error: '{}'", msg),
            }
        }
    }
}

pub use error::{Error, ErrorKind, Result};
