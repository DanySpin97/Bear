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

#include "libexec_a/Executor.h"

#include "libexec_a/Array.h"
#include "libexec_a/Resolver.h"
#include "libexec_a/Session.h"

namespace {

    struct Execution {
        const char **command;
        const char *path;
        const char *file;
        const char *search_path;
    };

    size_t length(Execution const &execution) noexcept {
        return ((execution.path != nullptr) ? 2 : 0) +
               ((execution.file != nullptr) ? 2 : 0) +
               ((execution.search_path != nullptr) ? 2 : 0) +
               ear::array::length(execution.command) +
               2;
    }

    const char **copy(Execution const &execution, const char **it, const char **it_end) noexcept {
        if (execution.path != nullptr) {
            *it++ = ear::FLAG_PATH;
            *it++ = execution.path;
        }
        if (execution.file != nullptr) {
            *it++ = ear::FLAG_FILE;
            *it++ = execution.file;
        }
        if (execution.search_path != nullptr) {
            *it++ = ear::FLAG_SEARCH_PATH;
            *it++ = execution.search_path;
        }
        *it++ = ear::FLAG_COMMAND;
        const size_t command_size = ear::array::length(execution.command);
        const char **const command_end = execution.command + (command_size + 1);
        return ear::array::copy(execution.command, command_end, it, it_end);
    }

    size_t length(ear::Session const &session) noexcept {
        return session.is_not_valid() ? 5 : 6;
    }

    const char **copy(ear::Session const &session, const char **it, const char **it_end) noexcept {
        *it++ = session.get_reporter();
        *it++ = ear::FLAG_DESTINATION;
        *it++ = session.get_destination();
        *it++ = ear::FLAG_LIBRARY;
        *it++ = session.get_library();
        if (session.is_verbose())
            *it++ = ear::FLAG_VERBOSE;
        return it;
    }
}

namespace ear {

    Executor::Executor(ear::Session const &session, ear::Resolver const &resolver) noexcept
            : session_(session)
            , resolver_(resolver)
    { }

    int Executor::execve(const char *path, char *const *argv, char *const *envp) const noexcept {
        if (session_.is_not_valid())
            return -1;

        auto fp = resolver_.execve();
        if (fp == nullptr)
            return -1;

        const Execution execution = { const_cast<const char **>(argv), path, nullptr, nullptr };

        const size_t dst_length = length(execution) + length(session_);
        const char *dst[dst_length];
        const char **const dst_end = dst + dst_length;

        const char **it = copy(session_, dst, dst_end);
        if (copy(execution, it, dst_end) == nullptr)
            return -1;

        return fp(session_.get_reporter(), const_cast<char *const *>(dst), envp);
    }

    int Executor::execvpe(const char *file, char *const *argv, char *const *envp) const noexcept {
        if (session_.is_not_valid())
            return -1;

        auto fp = resolver_.execve();
        if (fp == nullptr)
            return -1;

        const Execution execution = { const_cast<const char **>(argv), nullptr, file, nullptr };

        const size_t dst_length = length(execution) + length(session_);
        const char *dst[dst_length];
        const char **const dst_end = dst + dst_length;

        const char **it = copy(session_, dst, dst_end);
        if (copy(execution, it, dst_end) == nullptr)
            return -1;

        return fp(session_.get_reporter(), const_cast<char *const *>(dst), envp);
    }

    int Executor::execvP(const char *file, const char *search_path, char *const *argv,
                         char *const *envp) const noexcept {
        if (session_.is_not_valid())
            return -1;

        auto fp = resolver_.execve();
        if (fp == nullptr)
            return -1;

        const Execution execution = { const_cast<const char **>(argv), nullptr, file, search_path };

        const size_t dst_length = length(execution) + length(session_);
        const char *dst[dst_length];
        const char **const dst_end = dst + dst_length;

        const char **it = copy(session_, dst, dst_end);
        if (copy(execution, it, dst_end) == nullptr)
            return -1;

        return fp(session_.get_reporter(), const_cast<char *const *>(dst), envp);
    }

    int Executor::posix_spawn(pid_t *pid, const char *path, const posix_spawn_file_actions_t *file_actions,
                              const posix_spawnattr_t *attrp, char *const *argv,
                              char *const *envp) const noexcept {
        if (session_.is_not_valid())
            return -1;

        auto fp = resolver_.posix_spawn();
        if (fp == nullptr)
            return -1;

        const Execution execution = { const_cast<const char **>(argv), path, nullptr, nullptr };

        const size_t dst_length = length(execution) + length(session_);
        const char *dst[dst_length];
        const char **const dst_end = dst + dst_length;

        const char **it = copy(session_, dst, dst_end);
        if (copy(execution, it, dst_end) == nullptr)
            return -1;

        return fp(pid, session_.get_reporter(), file_actions, attrp, const_cast<char *const *>(dst), envp);
    }

    int Executor::posix_spawnp(pid_t *pid, const char *file, const posix_spawn_file_actions_t *file_actions,
                               const posix_spawnattr_t *attrp, char *const *argv,
                               char *const *envp) const noexcept {
        if (session_.is_not_valid())
            return -1;

        auto fp = resolver_.posix_spawn();
        if (fp == nullptr)
            return -1;

        const Execution execution = { const_cast<const char **>(argv), nullptr, file, nullptr };

        const size_t dst_length = length(execution) + length(session_);
        const char *dst[dst_length];
        const char **const dst_end = dst + dst_length;

        const char **it = copy(session_, dst, dst_end);
        if (copy(execution, it, dst_end) == nullptr)
            return -1;

        return fp(pid, session_.get_reporter(), file_actions, attrp, const_cast<char *const *>(dst), envp);
    }
}
