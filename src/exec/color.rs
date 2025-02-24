//! Detects whether a stream supports color, and gives details about that
//! support. It takes into account the `NO_COLOR` environment variable.
//!
//! This module is liberally copied from [supports-color], replacing `Stream`
//! with `&impl IsTerminal` and eliminating the cache. [supports-color] is
//! distributed under the Apache 2.0 License.
//!
//! [supports-color]: https://github.com/zkat/supports-color/blob/main/src/lib.rs,

#![allow(clippy::bool_to_int_with_if)]

use std::env;
use std::io::IsTerminal;

fn env_force_color() -> usize {
    if let Ok(force) = env::var("FORCE_COLOR") {
        match force.as_ref() {
            "true" | "" => 1,
            "false" => 0,
            f => std::cmp::min(f.parse().unwrap_or(1), 3),
        }
    } else if let Ok(cli_clr_force) = env::var("CLICOLOR_FORCE") {
        if cli_clr_force != "0" {
            1
        } else {
            0
        }
    } else {
        0
    }
}

fn env_no_color() -> bool {
    match as_str(&env::var("NO_COLOR")) {
        Ok("0") | Err(_) => false,
        Ok(_) => true,
    }
}

// same as Option::as_deref
fn as_str<E>(option: &Result<String, E>) -> Result<&str, &E> {
    match option {
        Ok(inner) => Ok(inner),
        Err(e) => Err(e),
    }
}

fn translate_level(level: usize) -> Option<ColorLevel> {
    if level == 0 {
        None
    } else {
        Some(ColorLevel {
            level,
            has_basic: true,
            has_256: level >= 2,
            has_16m: level >= 3,
        })
    }
}

fn supports_color(stream: &impl IsTerminal) -> usize {
    let force_color = env_force_color();
    if force_color > 0 {
        force_color
    } else if env_no_color()
        || as_str(&env::var("TERM")) == Ok("dumb")
        || !(stream.is_terminal() || env::var("IGNORE_IS_TERMINAL").map_or(false, |v| v != "0"))
    {
        0
    } else if env::var("COLORTERM").map(|colorterm| check_colorterm_16m(&colorterm)) == Ok(true)
        || env::var("TERM").map(|term| check_term_16m(&term)) == Ok(true)
        || as_str(&env::var("TERM_PROGRAM")) == Ok("iTerm.app")
    {
        3
    } else if as_str(&env::var("TERM_PROGRAM")) == Ok("Apple_Terminal")
        || env::var("TERM").map(|term| check_256_color(&term)) == Ok(true)
    {
        2
    } else if env::var("COLORTERM").is_ok()
        || check_ansi_color(env::var("TERM").ok().as_deref())
        || env::var("CLICOLOR").map_or(false, |v| v != "0")
        || is_ci::uncached()
    {
        1
    } else {
        0
    }
}

#[cfg(windows)]
fn check_ansi_color(term: Option<&str>) -> bool {
    if let Some(term) = term {
        // cygwin doesn't seem to support ANSI escape sequences and instead has its own variety.
        term != "dumb" && term != "cygwin"
    } else {
        // TERM is generally not set on Windows. It's reasonable to assume that all Windows
        // terminals support ANSI escape sequences (since Windows 10 version 1511).
        true
    }
}

#[cfg(not(windows))]
fn check_ansi_color(term: Option<&str>) -> bool {
    if let Some(term) = term {
        // dumb terminals don't support ANSI escape sequences.
        term != "dumb"
    } else {
        // TERM is not set, which is really weird on Unix systems.
        false
    }
}

fn check_colorterm_16m(colorterm: &str) -> bool {
    colorterm == "truecolor" || colorterm == "24bit"
}

fn check_term_16m(term: &str) -> bool {
    term.ends_with("direct") || term.ends_with("truecolor")
}

fn check_256_color(term: &str) -> bool {
    term.ends_with("256") || term.ends_with("256color")
}

/**
Returns a [ColorLevel] if a [Stream] supports terminal colors.
*/
pub(crate) fn on(stream: &impl IsTerminal) -> Option<ColorLevel> {
    translate_level(supports_color(stream))
}

/**
Color level support details.

This type is returned from [on]. See documentation for its fields for more details.
*/
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct ColorLevel {
    level: usize,
    /// Basic ANSI colors are supported.
    pub has_basic: bool,
    /// 256-bit colors are supported.
    pub has_256: bool,
    /// 16 million (RGB) colors are supported.
    pub has_16m: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io, sync::Mutex};

    // needed to prevent race conditions when mutating the environment
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn set_up() {
        // clears process env variable
        env::vars().for_each(|(k, _v)| env::remove_var(k));
    }

    #[test]
    fn test_empty_env() {
        let _test_guard = TEST_LOCK.lock().unwrap();
        set_up();

        assert_eq!(on(&io::stdout()), None);
    }

    #[test]
    fn test_clicolor_ansi() {
        let _test_guard = TEST_LOCK.lock().unwrap();
        set_up();

        env::set_var("IGNORE_IS_TERMINAL", "1");
        env::set_var("CLICOLOR", "1");
        let expected = Some(ColorLevel {
            level: 1,
            has_basic: true,
            has_256: false,
            has_16m: false,
        });
        assert_eq!(on(&io::stdout()), expected);

        env::set_var("CLICOLOR", "0");
        assert_eq!(on(&io::stdout()), None);
    }

    #[test]
    fn test_clicolor_force_ansi() {
        let _test_guard = TEST_LOCK.lock().unwrap();
        set_up();

        env::set_var("CLICOLOR", "0");
        env::set_var("CLICOLOR_FORCE", "1");
        let expected = Some(ColorLevel {
            level: 1,
            has_basic: true,
            has_256: false,
            has_16m: false,
        });
        assert_eq!(on(&io::stdout()), expected);
    }
}
