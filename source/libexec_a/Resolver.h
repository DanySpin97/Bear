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

    struct Resolver {

        using execve_t =
                int (*)(const char *path,
                        char *const argv[],
                        char *const envp[]);

        using posix_spawn_t =
                int (*)(pid_t *pid,
                        const char *path,
                        const posix_spawn_file_actions_t *file_actions,
                        const posix_spawnattr_t *attrp,
                        char *const argv[],
                        char *const envp[]);

        virtual ~Resolver() noexcept = default;

        virtual execve_t execve() const noexcept = 0;
        virtual posix_spawn_t posix_spawn() const noexcept = 0;
    };

}
