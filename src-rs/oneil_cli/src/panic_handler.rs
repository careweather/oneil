use anstream::{eprint, eprintln};
use owo_colors::OwoColorize;

pub fn register_panic_handler() {
    std::panic::set_hook(Box::new(print_oneil_panic_error));
}

fn print_oneil_panic_error(info: &std::panic::PanicHookInfo<'_>) {
    let error_message = info.payload_as_str().unwrap_or("unknown error");

    let location = info
        .location()
        .map(|location| format!("{}:{}", location.file(), location.line()));

    let version = env!("CARGO_PKG_VERSION");

    eprint!("{} ", "error:".red().bold());
    eprintln!("{}", "fatal internal bug".bold());
    eprintln!();
    eprintln!("{}", "This is a bug in Oneil itself.".italic());
    eprintln!();
    eprintln!(
        "{}",
        "Please report this crash to <https://github.com/careweather/oneil/issues>".italic()
    );
    eprintln!(
        "{}",
        "and include this error message in your report.".italic()
    );
    eprintln!();
    if let Some(location) = location {
        eprint!("{}", "Location: ".bold());
        eprintln!("{location}");
    }
    eprintln!("{} {error_message}", "Error message:".bold());
    eprintln!("{} {version}", "Version:".bold());
    eprintln!();
    eprintln!(
        "{}",
        "If possible, please also include either the code you were editing when".italic()
    );
    eprintln!(
        "{}",
        "the error occurred, or a minimal reproduction of the problem.".italic()
    );
}
