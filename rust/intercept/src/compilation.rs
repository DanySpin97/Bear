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

mod execution {
    use std::path;

    use database;
    use ErrorKind;
    use Result;

    use compilation::compiler;
    use compilation::flags;
    use compilation::pass;

    #[derive(Default)]
    struct CompilerExecution {
        directory: path::PathBuf,
        compiler: path::PathBuf,
        pass: pass::CompilerPass,
        flags: Vec<String>,
        inputs: Vec<path::PathBuf>,
        output: Option<path::PathBuf>,
    }

    impl CompilerExecution {
        fn new(working_dir: &str, compiler: &str) -> Self {
            let mut result: Self = Default::default();
            result.directory = path::PathBuf::from(working_dir);
            result.compiler = path::PathBuf::from(compiler);
            result
        }

        fn pass(&mut self, flag: &str) -> bool {
            self.pass.take(flag)
        }

        fn output(&mut self, flag: &str, it: &mut flags::FlagIterator) -> bool {
            if "-o" == flag {
                if let Some(output) = it.next() {
                    self.output = Some(path::PathBuf::from(output));
                }
                return true;
            }
            false
        }

        fn flags(&mut self, flag: &str, it: &mut flags::FlagIterator) -> bool {
            //            # some parameters look like a filename, take those explicitly
            //            elif arg in {'-D', '-I'}:
            //                result.flags.extend([arg, next(args)])
            //            # and consider everything else as compile option.
            //            else:
            //                result.flags.append(arg)
            unimplemented!()
        }

        fn source(&mut self, file: &str) -> bool {
            //            # parameter which looks source file is taken...
            //            elif re.match(r'^[^-].+', arg) and classify_source(arg):
            //                result.files.append(arg)
            unimplemented!()
        }

        fn build(&self) -> Result<Vec<database::Entry>> {
            if !self.pass.is_compiling() {
                Err(ErrorKind::CompilationError("Compiler is not doing compilation.").into())
            } else if self.inputs.is_empty() {
                Err(ErrorKind::CompilationError("Have not found source files.").into())
            } else {
                let entries: Vec<_> = self
                    .inputs
                    .iter()
                    .map(|input| {
                        let mut command: Vec<String> = Vec::new();
                        command.push(
                            self.compiler
                                .clone()
                                .into_os_string()
                                .into_string()
                                .unwrap(),
                        );
                        command.push("-c".to_string());
                        command.extend_from_slice(self.flags.as_ref());
                        command.push(input.clone().into_os_string().into_string().unwrap());
                        database::Entry {
                            directory: self.directory.clone(),
                            file: input.clone(),
                            command: command,
                            output: self.output.clone(),
                        }
                    }).collect();
                Ok(entries)
            }
        }
    }

    /// Returns a value when the command is a compilation, None otherwise.
    ///
    /// # Arguments
    /// `classifier` - helper object to detect compiler
    /// `command` - the command to classify
    fn parse_command(
        classifier: &compiler::Classifier,
        working_dir: &str,
        command: &[String],
    ) -> Result<Vec<database::Entry>> {
        debug!("input was: {:?}", command);
        match classifier.split(command) {
            Some(compiler_and_parameters) => {
                let mut result =
                    CompilerExecution::new(working_dir, compiler_and_parameters.0.as_str());
                let mut it = flags::FlagIterator::from(compiler_and_parameters.1);
                while let Some(arg) = it.next() {
                    // if it's a pass modifier flag, update it and move on.
                    if result.pass(arg.as_str()) {
                        continue;
                    }
                    // take the output flag
                    if result.output(arg.as_str(), &mut it) {
                        continue;
                    }
                    // take flags
                    if result.flags(arg.as_str(), &mut it) {
                        continue;
                    }
                    // take the rest as source file
                    if result.source(arg.as_str()) {
                        continue;
                    }
                }
                result.build()
            }
            _ => Err(ErrorKind::CompilationError("Compiler not recognized.").into()),
        }
    }
}

mod pass {
    use std::collections;
    use std::mem;

    #[derive(Debug, PartialEq)]
    pub enum CompilerPass {
        Preprocessor,
        Compilation,
        Linking,
        Internal,
    }

    lazy_static! {
        static ref PHASE_FLAGS: collections::BTreeMap<&'static str, CompilerPass> = {
            let mut m = collections::BTreeMap::new();
            m.insert("-v", CompilerPass::Internal);
            m.insert("-###", CompilerPass::Internal);
            m.insert("-cc1", CompilerPass::Internal);
            m.insert("-cc1as", CompilerPass::Internal);
            m.insert("-E", CompilerPass::Preprocessor);
            m.insert("-M", CompilerPass::Preprocessor);
            m.insert("-MM", CompilerPass::Preprocessor);
            m.insert("-c", CompilerPass::Compilation);
            m.insert("-S", CompilerPass::Compilation);
            m
        };
    }

    impl Default for CompilerPass {
        fn default() -> CompilerPass {
            CompilerPass::Linking
        }
    }

    impl CompilerPass {
        pub fn take(&mut self, string: &str) -> bool {
            if let Some(pass) = PHASE_FLAGS.get(string) {
                self.update(pass);
                true
            } else {
                false
            }
        }

        pub fn is_compiling(&self) -> bool {
            self == &CompilerPass::Compilation || self == &CompilerPass::Linking
        }

        fn update(&mut self, new_state: &CompilerPass) {
            match (&self, new_state) {
                (CompilerPass::Linking, CompilerPass::Internal) => {
                    mem::replace(self, CompilerPass::Internal);
                }
                (CompilerPass::Linking, CompilerPass::Compilation) => {
                    mem::replace(self, CompilerPass::Compilation);
                }
                (CompilerPass::Linking, CompilerPass::Preprocessor) => {
                    mem::replace(self, CompilerPass::Preprocessor);
                }
                (CompilerPass::Compilation, CompilerPass::Internal) => {
                    mem::replace(self, CompilerPass::Internal);
                }
                (CompilerPass::Compilation, CompilerPass::Preprocessor) => {
                    mem::replace(self, CompilerPass::Preprocessor);
                }
                (CompilerPass::Preprocessor, CompilerPass::Internal) => {
                    mem::replace(self, CompilerPass::Internal);
                }
                _ => (),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_default_stays_linker() {
            let mut sut: CompilerPass = Default::default();
            assert_eq!(CompilerPass::Linking, sut);

            assert_eq!(false, sut.take("--not_this"));
            assert_eq!(CompilerPass::Linking, sut);
        }

        #[test]
        fn test_compilation_updates_linker() {
            let mut sut: CompilerPass = Default::default();
            assert_eq!(CompilerPass::Linking, sut);

            assert_eq!(true, sut.take("-c"));
            assert_eq!(false, sut.take("--not_this"));
            assert_eq!(CompilerPass::Compilation, sut);
        }

        #[test]
        fn test_prepocessor_updates_linker() {
            let mut sut: CompilerPass = Default::default();
            assert_eq!(CompilerPass::Linking, sut);

            assert_eq!(true, sut.take("-E"));
            assert_eq!(false, sut.take("--not_this"));
            assert_eq!(CompilerPass::Preprocessor, sut);
        }

        #[test]
        fn test_internal_updates_linker() {
            let mut sut: CompilerPass = Default::default();
            assert_eq!(CompilerPass::Linking, sut);

            assert_eq!(true, sut.take("-###"));
            assert_eq!(false, sut.take("--not_this"));
            assert_eq!(CompilerPass::Internal, sut);
        }

        #[test]
        fn test_is_compiling() {
            assert_eq!(true, CompilerPass::Compilation.is_compiling());
            assert_eq!(true, CompilerPass::Linking.is_compiling());

            assert_eq!(false, CompilerPass::Preprocessor.is_compiling());
            assert_eq!(false, CompilerPass::Internal.is_compiling());
        }
    }
}

mod flags {
    use std::collections;

    lazy_static! {
        /// Map of ignored compiler option for the creation of a compilation database.
        /// This map is used in split_command method, which classifies the parameters
        /// and ignores the selected ones. Please note that other parameters might be
        /// ignored as well.
        ///
        /// Option names are mapped to the number of following arguments which should
        /// be skipped.
        static ref IGNORED_FLAGS: collections::BTreeMap<&'static str, u8> = {
            let mut m = collections::BTreeMap::new();
            // preprocessor macros, ignored because would cause duplicate entries in
            // the output (the only difference would be these flags). this is actual
            // finding from users, who suffered longer execution time caused by the
            // duplicates.
            m.insert("-MD",         0u8);
            m.insert("-MMD",        0u8);
            m.insert("-MG",         0u8);
            m.insert("-MP",         0u8);
            m.insert("-MF",         1u8);
            m.insert("-MT",         1u8);
            m.insert("-MQ",         1u8);
            // linker options, ignored because for compilation database will contain
            // compilation commands only. so, the compiler would ignore these flags
            // anyway. the benefit to get rid of them is to make the output more
            // readable.
            m.insert("-static",     0u8);
            m.insert("-shared",     0u8);
            m.insert("-s",          0u8);
            m.insert("-rdynamic",   0u8);
            m.insert("-l",          1u8);
            m.insert("-L",          1u8);
            m.insert("-u",          1u8);
            m.insert("-z",          1u8);
            m.insert("-T",          1u8);
            m.insert("-Xlinker",    1u8);
            // clang-cl / msvc cl specific flags
            // consider moving visual studio specific warning flags also
            m.insert("-nologo",     0u8);
            m.insert("-EHsc",       0u8);
            m.insert("-EHa",        0u8);
            m
        };

        /// Typical linker flags also not really needed for a compilation.
        static ref LINKER_FLAG: regex::Regex =
            regex::Regex::new(r"^-(l|L|Wl,).+").unwrap();
    }

    pub struct FlagIterator {
        inner: Box<Iterator<Item = String>>,
    }

    impl FlagIterator {
        pub fn from(collection: Vec<String>) -> Self {
            Self {
                inner: Box::new(collection.into_iter()),
            }
        }
    }

    impl Iterator for FlagIterator {
        type Item = String;

        fn next(&mut self) -> Option<<Self as Iterator>::Item> {
            while let Some(flag) = self.inner.next() {
                // Skip flags which matches from the given map.
                if let Some(skip) = IGNORED_FLAGS.get(flag.as_str()) {
                    for _ in 0..*skip {
                        self.inner.next();
                    }
                    return self.next();
                // Skip linker flags too.
                } else if LINKER_FLAG.is_match(flag.as_str()) {
                    return self.next();
                } else {
                    return Some(flag);
                }
            }
            None
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn assert_ignored_eq(expected: &[&str], input: &[&str]) {
            let input_vec: Vec<String> = input.iter().map(|str| str.to_string()).collect();
            let expected_vec: Vec<String> = expected.iter().map(|str| str.to_string()).collect();

            let mut sut = FlagIterator::from(input_vec);
            let result: Vec<_> = sut.collect();
            assert_eq!(expected_vec, result);
        }

        #[test]
        fn test_empty() {
            assert_ignored_eq(&[], &[]);
        }

        #[test]
        fn test_not_skip() {
            assert_ignored_eq(&["a", "b", "c"], &["a", "b", "c"]);
            assert_ignored_eq(&["-a", "-b", "-c"], &["-a", "-b", "-c"]);
            assert_ignored_eq(&["/a", "/b", "/c"], &["/a", "/b", "/c"]);
        }

        #[test]
        fn test_skip_given_flags() {
            assert_ignored_eq(&["a", "b"], &["a", "-MD", "b"]);
            assert_ignored_eq(&["a", "b"], &["a", "-MMD", "b"]);
            assert_ignored_eq(&["a", "b"], &["a", "-MF", "file", "b"]);

            assert_ignored_eq(&["a", "b"], &["a", "-MG", "-MT", "skip", "b"]);
            assert_ignored_eq(&["a", "b", "c"], &["a", "-MG", "b", "-MT", "skip", "c"]);
        }

        #[test]
        fn test_skip_linker_flags() {
            assert_ignored_eq(&["a", "b"], &["a", "-live", "b"]);
            assert_ignored_eq(&["a", "b"], &["a", "-L/path", "b"]);
            assert_ignored_eq(&["a", "b"], &["a", "-Wl,option", "b"]);

            assert_ignored_eq(&["a", "b"], &["a", "-live", "-L/path", "b"]);
        }
    }
}

mod compiler {
    use regex;
    use shellwords;
    use std::path;
    use std::process;
    use std::str;

    use ErrorKind;
    use Result;

    lazy_static! {
        /// Known C/C++ compiler wrapper name patterns.
        static ref COMPILER_PATTERN_WRAPPER: regex::Regex =
            regex::Regex::new(r"^(distcc|ccache)$").unwrap();

        /// Known MPI compiler wrapper name patterns.
        static ref COMPILER_PATTERNS_MPI_WRAPPER: regex::Regex =
            regex::Regex::new(r"^mpi(cc|cxx|CC|c\+\+)$").unwrap();

        /// Known C compiler executable name patterns.
        static ref COMPILER_PATTERNS_CC: Vec<regex::Regex> = vec![
            regex::Regex::new(r"^([^-]*-)*[mg]cc(-?\d+(\.\d+){0,2})?$").unwrap(),
            regex::Regex::new(r"^([^-]*-)*clang(-\d+(\.\d+){0,2})?$").unwrap(),
            regex::Regex::new(r"^(|i)cc$").unwrap(),
            regex::Regex::new(r"^(g|)xlc$").unwrap(),
        ];

        /// Known C++ compiler executable name patterns.
        static ref COMPILER_PATTERNS_CXX: Vec<regex::Regex> = vec![
            regex::Regex::new(r"^(c\+\+|cxx|CC)$").unwrap(),
            regex::Regex::new(r"^([^-]*-)*[mg]\+\+(-?\d+(\.\d+){0,2})?$").unwrap(),
            regex::Regex::new(r"^([^-]*-)*clang\+\+(-\d+(\.\d+){0,2})?$").unwrap(),
            regex::Regex::new(r"^icpc$").unwrap(),
            regex::Regex::new(r"^(g|)xl(C|c\+\+)$").unwrap(),
        ];
    }

    pub struct Classifier {
        ignore: bool,
        c_compilers: Vec<String>,
        cxx_compilers: Vec<String>,
    }

    impl Classifier {
        /// Create a new Category object.
        ///
        /// # Arguments
        /// `only_use` - use only the given compiler names for classification,
        /// `c_compilers` - list of C compiler names,
        /// `cxx_compilers` - list of C++ compiler names.
        pub fn new(only_use: bool, c_compilers: &[String], cxx_compilers: &[String]) -> Self {
            let c_compiler_names: Vec<_> = c_compilers
                .into_iter()
                .map(|path| basename(&path))
                .collect();
            let cxx_compiler_names: Vec<_> = cxx_compilers
                .into_iter()
                .map(|path| basename(&path))
                .collect();

            Self {
                ignore: only_use,
                c_compilers: c_compiler_names,
                cxx_compilers: cxx_compiler_names,
            }
        }

        /// A predicate to decide whether the command is a compiler call.
        ///
        /// # Arguments
        /// `command` - the command to classify
        ///
        /// # Returns
        /// None if the command is not a compilation, or a tuple (compiler, arguments) otherwise.
        pub fn split(&self, command: &[String]) -> Option<(String, Vec<String>)> {
            match command.split_first() {
                Some((executable, parameters)) => {
                    // 'wrapper' 'parameters' and
                    // 'wrapper' 'compiler' 'parameters' are valid.
                    // Additionally, a wrapper can wrap another wrapper.
                    if self.is_wrapper(&executable) {
                        let result = self.split(parameters);
                        // Compiler wrapper without compiler is a 'C' compiler.
                        if result.is_some() {
                            result
                        } else {
                            Some((executable.clone(), parameters.to_vec()))
                        }
                    // MPI compiler wrappers add extra parameters
                    } else if self.is_mpi_wrapper(executable) {
                        match get_mpi_call(executable) {
                            Ok(mut mpi_call) => {
                                mpi_call.extend_from_slice(parameters);
                                self.split(mpi_call.as_ref())
                            }
                            _ => None,
                        }
                    // and 'compiler' 'parameters' is valid.
                    } else if self.is_c_compiler(&executable) {
                        Some((executable.clone(), parameters.to_vec()))
                    } else if self.is_cxx_compiler(&executable) {
                        Some((executable.clone(), parameters.to_vec()))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }

        /// Match against known compiler wrappers.
        fn is_wrapper(&self, executable: &str) -> bool {
            let program = basename(executable);
            COMPILER_PATTERN_WRAPPER.is_match(&program)
        }

        /// Match against known MPI compiler wrappers.
        fn is_mpi_wrapper(&self, executable: &str) -> bool {
            let program = basename(executable);
            COMPILER_PATTERNS_MPI_WRAPPER.is_match(&program)
        }

        /// Match against known C compiler names.
        fn is_c_compiler(&self, executable: &str) -> bool {
            let program = basename(executable);
            let use_match = self.c_compilers.contains(&program);
            if self.ignore {
                use_match
            } else {
                use_match || is_pattern_match(&program, &COMPILER_PATTERNS_CC)
            }
        }

        /// Match against known C++ compiler names.
        fn is_cxx_compiler(&self, executable: &str) -> bool {
            let program = basename(executable);
            let use_match = self.cxx_compilers.contains(&program);
            if self.ignore {
                use_match
            } else {
                use_match || is_pattern_match(&program, &COMPILER_PATTERNS_CXX)
            }
        }
    }

    /// Takes a command string and returns as a list.
    fn shell_split(string: &str) -> Result<Vec<String>> {
        match shellwords::split(string) {
            Ok(value) => Ok(value),
            _ => bail!(ErrorKind::RuntimeError("Can't parse shell command")),
        }
    }

    /// Provide information on how the underlying compiler would have been
    /// invoked without the MPI compiler wrapper.
    fn get_mpi_call(wrapper: &str) -> Result<Vec<String>> {
        fn run_mpi_wrapper(wrapper: &str, flag: &str) -> Result<Vec<String>> {
            let child = process::Command::new(wrapper)
                .arg(flag)
                .stdout(process::Stdio::piped())
                .spawn()?;
            let output = child.wait_with_output()?;
            // Take the stdout if the process was successful.
            if output.status.success() {
                // Take only the first line and treat as it would be a shell command.
                let output_string = str::from_utf8(output.stdout.as_slice())?;
                match output_string.lines().next() {
                    Some(first_line) => shell_split(first_line),
                    _ => bail!(ErrorKind::RuntimeError("Empty output of wrapper")),
                }
            } else {
                bail!(ErrorKind::RuntimeError("Process failed."))
            }
        }

        // Try both flags with the wrapper and return the first successful result.
        ["--show", "--showme"]
            .iter()
            .map(|&query_flatg| run_mpi_wrapper(wrapper, &query_flatg))
            .find(Result::is_ok)
            .unwrap_or(bail!(ErrorKind::CompilationError(
                "Could not determinate MPI flags.",
            )))
    }

    /// Match against a list of regex and return true if any of those were match.
    fn is_pattern_match(candidate: &str, patterns: &[regex::Regex]) -> bool {
        patterns.iter().any(|pattern| pattern.is_match(candidate))
    }

    /// Returns the filename of the given path (rendered as String).
    fn basename(file: &str) -> String {
        let path = path::PathBuf::from(file);
        match path.file_name().map(|path| path.to_str()) {
            Some(Some(str)) => str.to_string(),
            _ => file.to_string(),
        }
    }
}
