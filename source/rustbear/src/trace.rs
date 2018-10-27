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
use std::io;
use std::fs;
use std::path;
use libc;
use serde_json;

use Result;
use Error;

#[derive(Serialize, Deserialize)]
pub struct Trace {
    pid: libc::pid_t,
    cwd: path::PathBuf,
    cmd: Vec<String>
}

impl Trace {
    pub fn new(pid: libc::pid_t, cwd: path::PathBuf, cmd: Vec<String>) -> Trace {
        Trace { pid: pid, cwd: cwd, cmd: cmd }
    }

    pub fn get_pid(&self) -> libc::pid_t {
        self.pid
    }

    pub fn get_cwd(&self) -> &path::Path {
        &self.cwd
    }

    pub fn get_cmd(&self) -> &[String] {
        &self.cmd
    }

    /// Create a Trace report object from the given arguments.
    /// Capture the current process id and working directory.
    pub fn create(args: &Vec<String>) -> Result<Trace> {
        let pid: libc::pid_t = unsafe { libc::getpid() };
        let cwd = env::current_dir()?;

        Ok(Trace::new(pid, cwd, args.clone()))
    }

    /// Write a single trace entry into the given target.
    pub fn write(target: &mut io::Write, value: &Trace) -> Result<()> {
        let result = serde_json::to_writer(target, value)?;
        Ok(result)
    }

    /// Read a single trace file content from given source.
    pub fn read(source: &mut io::Read) -> Result<Trace> {
        let result = serde_json::from_reader(source)?;
        Ok(result)
    }
}


pub struct TraceDirectory {
    input: fs::ReadDir
}

impl TraceDirectory {

    /// Create a TraceDirectory object from the directory path.
    pub fn new(path: &path::Path) -> Result<TraceDirectory> {
        if path.is_dir() {
            let input = fs::read_dir(path)?;
            Ok(TraceDirectory { input: input })
        } else {
            Err(Error::RuntimeError("TraceSource should be directory".to_string()))
        }
    }

    fn is_execution_trace(path: &path::Path) -> bool {
        const EXTENSION: &'static str = ".process_start.json";

        path.to_str().map_or(false, |str| { str.ends_with(EXTENSION) })
    }
}

impl Iterator for TraceDirectory {
    type Item = path::PathBuf;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match self.input.next() {
            Some(Ok(entry)) => {
                let path = entry.path();
                if path.is_dir() {
                    self.next()
                } else if TraceDirectory::is_execution_trace(&path) {
                    Some(path.to_path_buf())
                } else {
                    self.next()
                }
            },
            Some(Err(_)) => self.next(),
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_execution_trace() {
        let valid = path::Path::new("/tmp/5432.process_start.json");
        assert!(TraceDirectory::is_execution_trace(valid));
        let invalid = path::Path::new("/tmp/something.json");
        assert!(!TraceDirectory::is_execution_trace(invalid));
    }

    #[test]
    fn read_works_on_unkown_fields() {
        let mut data = r#"{
                    "pid": 12,
                    "ppid": 11,
                    "pppid": 10,
                    "cwd": "/usr/home/user",
                    "cmd": [
                      "program",
                      "arg1",
                      "arg2"
                    ]
                  }"#.as_bytes();

        let result = Trace::read(&mut data).unwrap();

        assert_eq!(12i32, result.get_pid());
        assert_eq!(path::Path::new("/usr/home/user"), result.get_cwd());
        assert_eq!(["program", "arg1", "arg2"], result.get_cmd());
    }

    #[test]
    fn write_and_read_works() {
        let expected = Trace::new(12i32,
                                  path::PathBuf::from("/home/user"),
                                  vec!["program".to_string(), "arg".to_string()]);

        let mut data: [u8; 100] = [0; 100];
        {
            let mut buffer = &mut data[..];
            let _result = Trace::write(&mut buffer, &expected).unwrap();
        }
        {
            let end = data.iter().position(|&c| c == 0).unwrap();
            let mut buffer = &data[..end];
            let result = Trace::read(&mut buffer).unwrap();

            assert_eq!(expected.get_pid(), result.get_pid());
            assert_eq!(expected.get_cwd(), result.get_cwd());
            assert_eq!(expected.get_cmd(), result.get_cmd());
        }
    }
}