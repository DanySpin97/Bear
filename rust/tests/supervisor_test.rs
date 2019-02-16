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

extern crate intercept;

use intercept::event::Event;
use intercept::supervisor::Supervisor;

macro_rules! vec_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}

macro_rules! slice_of_strings {
    ($($x:expr),*) => (vec![$($x.to_string()),*].as_ref());
}

#[cfg(unix)]
mod unix {
    use super::*;

    mod test_exit_code {
        use super::*;

        #[test]
        fn success() {
            let mut sut = Supervisor::new(|_: Event| ());

            let result = sut.run(slice_of_strings!("/usr/bin/true"));
            assert_eq!(true, result.is_ok());
            assert_eq!(0i32, result.unwrap());
        }

        #[test]
        fn fail() {
            let mut sut = Supervisor::new(|_: Event| ());

            let result = sut.run(slice_of_strings!("/usr/bin/false"));
            assert_eq!(true, result.is_ok());
            assert_eq!(1i32, result.unwrap());
        }

        #[test]
        fn exec_failure() {
            let mut sut = Supervisor::new(|_: Event| ());

            let result = sut.run(slice_of_strings!("./path/to/not/exists"));
            assert_eq!(false, result.is_ok());
        }
    }

    mod test_events {
        use super::*;
        use std::env;
        use std::process;

        fn run_supervisor(args: &[String]) -> Vec<Event> {
            let mut events: Vec<Event> = vec![];
            {
                let mut sut = Supervisor::new(|event: Event| {
                    (&mut events).push(event);
                });
                let _ = sut.run(args);
            }
            events
        }

        #[test]
        fn success() {
            let command = vec_of_strings!("/usr/bin/true");
            let events = run_supervisor(command.as_ref());

            assert_eq!(2usize, (&events).len());
            match events[0] {
                Event::Created { pid, ppid, ref cwd, ref cmd, .. } => {
                    assert_ne!(0, pid);
                    assert_ne!(process::id(), pid);
                    assert_ne!(std::os::unix::process::parent_id(), pid);
                    assert_eq!(std::os::unix::process::parent_id(), ppid);
                    assert_eq!(env::current_dir().unwrap().as_os_str(), cwd.as_os_str());
                    assert_eq!(&command, cmd);
                },
                _ => assert_eq!(true, false),
            }
            match events[1] {
                Event::TerminatedNormally { pid, code, .. } => {
                    // TODO: check if it is the same as the create one
                    assert_ne!(0, pid);
                    assert_ne!(process::id(), pid);
                    assert_eq!(0i32, code);
                },
                _ => assert_eq!(true, false),
            }
        }

        #[test]
        fn exec_failure() {
            let events = run_supervisor(slice_of_strings!("./path/to/not/exists"));
            assert_eq!(0usize, (&events).len());
        }

    }
}
