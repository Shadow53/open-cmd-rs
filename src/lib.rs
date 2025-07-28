//! Generate commands for opening paths and URIs in the default system handler.
//!
//! These methods return [`std::process::Command`] instances that can immediately be run to open
//! the given target, or modified to provide different stdin/stdout/stderr streams.
//!
//! This crate used <https://dwheeler.com/essays/open-files-urls.html> as a reference.

#![deny(clippy::all)]
#![deny(clippy::correctness)]
#![deny(clippy::style)]
#![deny(clippy::complexity)]
#![deny(clippy::perf)]
#![deny(clippy::pedantic)]
#![deny(rustdoc::missing_doc_code_examples)]
#![deny(
    absolute_paths_not_starting_with_crate,
    anonymous_parameters,
    bad_style,
    dead_code,
    keyword_idents,
    improper_ctypes,
    macro_use_extern_crate,
    meta_variable_misuse, // May have false positives
    missing_abi,
    missing_debug_implementations, // can affect compile time/code size
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    noop_method_call,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    semicolon_in_expressions_from_macros,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unreachable_pub,
    unsafe_code,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_parens,
    unused_qualifications,
    variant_size_differences,
    while_true
)]

use std::{path::PathBuf, process::Command};
use thiserror::Error;

mod path_or_uri;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
use linux::open as sys_open;
#[cfg(target_os = "macos")]
use macos::open as sys_open;
#[cfg(target_os = "windows")]
use windows::open as sys_open;

pub use path_or_uri::PathOrURI;

/// Type alias for the most common results in this crate.
pub type Result<T = Command, E = Error> = std::result::Result<T, E>;

/// The environment variable checked when opening in a web browser.
pub const BROWSER_ENV: &str = "BROWSER";
/// The environment variable checked when opening in a text editor.
pub const EDITOR_ENV: &str = "EDITOR";

#[derive(Debug, Error)]
/// Errors that may occur when generating a [`Command`].
pub enum Error {
    /// Failed to convert a file path to a URI. This may be returned on Windows, where this is used
    /// to ensure no odd behavior with confusing paths and CLI options (which start with a `/` on
    /// Windows).
    ///
    /// See [`PathOrURI::url()`] for possible error cases.
    #[error("could not convert file path to URI: {0:?}")]
    FileToURI(PathBuf),
    /// An I/O error occurred while setting up the command.
    #[error("I/O error occurred: {0}")]
    IO(#[from] std::io::Error),
    /// A required executable was not found. This will most likely always be `xdg-open` on Unix-y
    /// systems, as Windows as macOS use built-in commands.
    ///
    /// If this error occurs with `xdg-open`, the user should install the `xdg-utils` for their
    /// system.
    #[error("executable {exe} not found: {error}")]
    NotFound {
        /// The program that couldn't be found.
        exe: String,
        #[source]
        /// The error returned by `which`.
        error: which::Error,
    },
}

#[inline]
fn ensure_command(cmd: &str) -> Result<()> {
    #[cfg(feature = "tracing")]
    tracing::trace!("checking if executable \"{}\" exists", cmd);

    which::which(cmd)
        .map(|_| ())
        .map_err(|error| Error::NotFound {
            exe: cmd.to_string(),
            error,
        })
}

#[inline]
fn open_with_command(cmd: &str, target: &PathOrURI) -> Result {
    ensure_command(cmd)?;

    #[cfg(feature = "tracing")]
    tracing::debug!("opening {} with {}", target, cmd);

    let mut cmd = Command::new(cmd);
    cmd.arg(target.to_string());
    Ok(cmd)
}

#[inline]
fn open_env(env: &str, target: &PathOrURI) -> Result {
    #[cfg(feature = "tracing")]
    tracing::trace!("checking if {} exists in environment", env);

    if let Ok(cmd) = std::env::var(env) {
        #[cfg(feature = "tracing")]
        tracing::trace!("found {} = {}", env, cmd);
        open_with_command(&cmd, target)
    } else {
        #[cfg(feature = "tracing")]
        tracing::trace!("{} not found, using system default handler", env);
        sys_open(target)
    }
}

/// Open the target in the default system handler.
///
/// This function ignores special environment variables that can tell CLI apps what to use. If you
/// want to consider those variables, use [`open_browser`] or [`open_editor`].
///
/// # Errors
///
/// See [`Error`].
pub fn open<T>(target: T) -> Result
where
    PathOrURI: From<T>,
{
    sys_open(&PathOrURI::from(target))
}

/// Open the target in the web browser specified by [`BROWSER_ENV`], or the system handler if not
/// set.
///
/// # Errors
///
/// See [`Error`].
pub fn open_browser<T>(target: T) -> Result
where
    PathOrURI: From<T>,
{
    open_env(BROWSER_ENV, &PathOrURI::from(target))
}

/// Open the target in the text editor specified by [`EDITOR_ENV`], or the system handler if not
/// set.
///
/// # Errors
///
/// See [`Error`].
pub fn open_editor<T>(target: T) -> Result
where
    PathOrURI: From<T>,
{
    open_env(EDITOR_ENV, &PathOrURI::from(target))
}
