/// A macro to print to stderr, just like println!
macro_rules! printerrln (
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);


/// Formats a string which is printed to stderr, and exits with status code 1
///
/// ```
/// error!("Failed to bring it around town: {}", arg);
/// ```
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        printerrln!($($arg)*);
        std::process::exit(1);
    })
}

/// Unwraps a Result like try!(), but calls error!("{}", msg) if the result is Err(msg)
///
/// ```
/// let value = try_error!(result);
/// ```
#[macro_export]
macro_rules! try_error {
    ($expr:expr) => ({
        match $expr {
            Ok(val) => val,
            Err(err) => error!("{}", err),
        }
    })
}

#[macro_export]
macro_rules! parse_args {
    ($argtype:ident, $argv:expr) => (
        $argtype::docopt().argv($argv.iter().map(|s| &s[..])).decode().unwrap_or_else(|e| e.exit())
    )
}

pub mod args;
pub mod env;
pub mod file;
pub mod history;
pub mod main;
pub mod remember;
pub mod request;
pub mod template;
pub mod yaml_util;

#[cfg(test)]
mod test;
