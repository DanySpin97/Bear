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

#pragma once

#include "config.h"

namespace ear {

    constexpr char FLAG_VERBOSE[] = "--verbose";
    constexpr char FLAG_DESTINATION[] = "--report-destination";
    constexpr char FLAG_LIBRARY[] = "--session-library";
    constexpr char FLAG_PATH[] = "--exec-path";
    constexpr char FLAG_FILE[] = "--exec-file";
    constexpr char FLAG_SEARCH_PATH[] = "--exec-search_path";
    constexpr char FLAG_COMMAND[] = "--exec-command";

    class Resolver;
    class Session;

    class Executor {
    public:
        Executor(ear::Session const &session, ear::Resolver const &resolver) noexcept;

        Executor() noexcept = delete;

        Executor(const Executor &) = delete;

        Executor(Executor &&) noexcept = delete;

        ~Executor() noexcept = default;

        Executor &operator=(const Executor &) = delete;

        Executor &operator=(Executor &&) noexcept = delete;

    public:
        int execve(const char *path, char *const argv[], char *const envp[]) const noexcept;

        int execvpe(const char *file, char *const argv[], char *const envp[]) const noexcept;

        int execvP(const char *file, const char *search_path, char *const argv[], char *const envp[]) const noexcept;

        int posix_spawn(pid_t *pid, const char *path,
                        const posix_spawn_file_actions_t *file_actions,
                        const posix_spawnattr_t *attrp,
                        char *const argv[],
                        char *const envp[]) const noexcept;

        int posix_spawnp(pid_t *pid, const char *file,
                         const posix_spawn_file_actions_t *file_actions,
                         const posix_spawnattr_t *attrp,
                         char *const argv[],
                         char *const envp[]) const noexcept;

    private:
        ear::Session const &session_;
        ear::Resolver const &resolver_;
    };

}
