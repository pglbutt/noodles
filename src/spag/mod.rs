/// Formats a string which is printed to stderr, and exits with status code 1
///
/// ```
/// error!("Failed to bring it around town: {}", arg);
/// ```
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        match writeln!(&mut std::io::stderr(), $($arg)*) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
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

pub mod main;
pub mod request;
pub mod file;
pub mod env;
pub mod template;
pub mod history;
pub mod args;

#[cfg(test)]
mod test;
