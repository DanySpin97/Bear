/*  Copyright (C) 2012-2017 by László Nagy
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

#include "intercept_a/SystemCalls.h"

#include <wait.h>
#include <spawn.h>

#include <cstring>
#include <memory>
#include <fstream>
#include <filesystem>

namespace {

    template <typename T>
    pear::Result<T> failure(const char *message) noexcept {
        std::string result = message != nullptr ? std::string(message) : std::string();

        const size_t buffer_length = 1024 + strlen(message);
        char buffer[buffer_length];
        if (0 == strerror_r(errno, buffer, buffer_length)) {
            result += std::string(": ");
            result += std::string(buffer);
        } else {
            result += std::string(": unkown error.");
        }
        errno = ENOENT;
        return ::pear::Err(std::runtime_error(result));
    };

}


namespace pear {

    Result<pid_t>
    fork_with_execvp(const char *file, const char *search_path, const char **argv, const char **envp) noexcept {
#ifdef HAVE_EXECVP2
        // TODO: implement it
#else
        return spawnp(file, argv, envp);
#endif
    }

    Result<int> spawn(const char **argv, const char **envp) noexcept {
        pid_t child;
        if (0 != posix_spawn(&child, argv[0], nullptr, nullptr,
                              const_cast<char **>(argv),
                              const_cast<char **>(envp))) {
            return failure<pid_t>("posix_spawn");
        } else {
            return Ok(child);
        }
    }

    Result<int> spawnp(const char *file, const char **argv, const char **envp) noexcept {
        pid_t child;
        if (0 != posix_spawnp(&child, file, nullptr, nullptr,
                              const_cast<char **>(argv),
                              const_cast<char **>(envp))) {
            return failure<pid_t>("posix_spawn");
        } else {
            return Ok(child);
        }
    }

    Result<int> wait_pid(pid_t pid) noexcept {
        int status;
        if (-1 == waitpid(pid, &status, 0)) {
            return failure<int>("waitpid");
        } else {
            const int result = WIFEXITED(status) ? WEXITSTATUS(status) : EXIT_FAILURE;
            return Ok(result);
        }
    }

    Result<pid_t> get_pid() noexcept {
        return Ok(getpid());
    }

    Result<pid_t> get_ppid() noexcept {
        return Ok(getppid());
    }

    Result<std::string> get_cwd() noexcept {
        constexpr static const size_t buffer_size = 8192;

        char buffer[buffer_size];
        if (nullptr == getcwd(buffer, buffer_size)) {
            return failure<std::string>("getcwd");
        } else {
            return Ok(std::string(buffer));
        }
    }

    Result<std::shared_ptr<std::ostream>> temp_file(const char *dir, const char *suffix) noexcept {
        auto path = std::filesystem::path(dir) / "XXXXXX";
        if (-1 == mkstemp(const_cast<char *>(path.c_str()))) {
            return failure<std::shared_ptr<std::ostream>>("mkstemp");
        } else {
            const auto &new_file = path.filename().string() + suffix;
            auto result = std::make_shared<std::ofstream>(path.replace_filename(new_file));
            return Ok(std::dynamic_pointer_cast<std::ostream>(result));
        }
    }

}
