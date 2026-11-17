//! The optional native macOS application of PhairDLNA.
//!
//! What: the menu bar application that controls the headless server.
//! Why: the server is the product; the application is a front end for it. The
//!      two are separate crates so that the server can run without any
//!      graphical process (`PRD-006`).
//! Example: the application asks the administration surface for the state of
//!          the server and shows it as a line of text in the menu bar.
//!
//! **The application is a front end and nothing else.** It holds no state of
//! the server of its own, it opens no connection to anything but the
//! administration surface on the loopback interface, and it never embeds the
//! server (`MB-001`, `MB-005`).
//!
//! **The texts are in the resource files.** Every text the user reads — the
//! help of the command line, the lines of the menu, the messages of a
//! failure — comes from a resource file of the language of the user, and not
//! from the code (`UX-020`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::path::PathBuf;

use phairdlna_mac::text::Strings;

/// The name of the option that asks for the help.
const HELP: &str = "--help";
/// The name of the option that names the port of the surface.
const PORT: &str = "--port";
/// The name of the option that names the file of the token.
const TOKEN_FILE: &str = "--token-file";
/// The number the program returns when the command line was not understood.
const EXIT_USAGE: i32 = 2;
/// The number the program returns when the application could not start.
const EXIT_FAILURE: i32 = 1;

fn main() {
    let strings = Strings::for_current_locale().unwrap_or_else(|_| Strings::fallback());
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let options = match Options::parse(&arguments, &strings) {
        Ok(Some(options)) => options,
        Ok(None) => return,
        Err(reason) => {
            eprintln!("{reason}");
            std::process::exit(EXIT_USAGE);
        }
    };
    if let Err(reason) = start(&options, &strings) {
        eprintln!("{reason}");
        std::process::exit(EXIT_FAILURE);
    }
}

/// The options of the application.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Options {
    /// The port of the administration surface.
    port: u16,
    /// The file that holds the token of the installation.
    token_file: PathBuf,
}

impl Options {
    /// Reads the command line.
    ///
    /// The answer is `Ok(None)` when the user asked for the help: the help is
    /// written and the program ends (`UX-006`).
    ///
    /// **The configuration of the server is the default.** The port of the
    /// surface and the file of the token come from the configuration of the
    /// server, so that the application and the server cannot disagree; an
    /// option of the command line wins over the configuration.
    fn parse(arguments: &[String], strings: &Strings) -> Result<Option<Options>, String> {
        let mut port: Option<u16> = None;
        let mut token_file: Option<PathBuf> = None;
        let mut index = 0;
        while index < arguments.len() {
            let argument = arguments[index].as_str();
            match argument {
                HELP => {
                    print_help(strings);
                    return Ok(None);
                }
                PORT => {
                    let value = value_of(arguments, &mut index, strings)?;
                    port = Some(parse_port(&value, strings)?);
                }
                TOKEN_FILE => {
                    let value = value_of(arguments, &mut index, strings)?;
                    token_file = Some(PathBuf::from(value));
                }
                _ => {
                    let option = argument.to_owned();
                    return Err(strings.format("cli.error.unknown_option", &[("option", &option)]));
                }
            }
            index += 1;
        }
        let default_port = port
            .is_none()
            .then(of_the_configuration)
            .map(|(port, _)| port);
        let default_token_file = token_file
            .is_none()
            .then(of_the_configuration)
            .map(|(_, path)| path);
        Ok(Some(Options {
            port: port
                .or(default_port)
                .unwrap_or(phairdlna_core::config::DEFAULT_ADMIN_PORT),
            token_file: token_file.or(default_token_file).unwrap_or_else(|| {
                PathBuf::from(phairdlna_core::config::default_admin_token_file())
            }),
        }))
    }
}

/// The port of the surface and the file of the token, out of the
/// configuration of the server.
///
/// The answer is the default of the project when the configuration cannot be
/// read: the application then asks the surface where the server says it
/// listens.
fn of_the_configuration() -> (u16, PathBuf) {
    let path = phairdlna_core::config::resolve_path(None);
    let fallback = (
        phairdlna_core::config::DEFAULT_ADMIN_PORT,
        PathBuf::from(phairdlna_core::config::default_admin_token_file()),
    );
    let Ok(config) = phairdlna_core::config::Config::load(&path) else {
        return fallback;
    };
    let token_file = PathBuf::from(&config.security.admin_token_file);
    let token_file = match path.parent() {
        Some(folder) if !token_file.is_absolute() => folder.join(&token_file),
        _ => token_file,
    };
    (config.security.admin_port, token_file)
}

/// The value of an option, and the place of the next argument.
fn value_of(arguments: &[String], index: &mut usize, strings: &Strings) -> Result<String, String> {
    let option = arguments[*index].clone();
    let Some(value) = arguments.get(*index + 1) else {
        return Err(strings.format("cli.error.missing_value", &[("option", &option)]));
    };
    *index += 1;
    Ok(value.clone())
}

/// The port of the surface, out of the command line.
fn parse_port(value: &str, strings: &Strings) -> Result<u16, String> {
    match value.parse::<u16>() {
        Ok(port) if port > 0 => Ok(port),
        _ => Err(strings.format("cli.error.bad_port", &[("value", value)])),
    }
}

/// Writes the help of the command line.
fn print_help(strings: &Strings) {
    println!("{}", strings.text("cli.help.usage"));
    println!("{}", strings.text("cli.help.port"));
    println!("{}", strings.text("cli.help.token_file"));
    println!("{}", strings.text("cli.help.equivalent"));
}

/// Starts the application.
#[cfg(target_os = "macos")]
fn start(options: &Options, strings: &Strings) -> Result<(), String> {
    use phairdlna_mac::app::App;
    use phairdlna_mac::ipc::client::AdminClient;
    use phairdlna_mac::ipc::http::TcpTransport;
    use phairdlna_mac::platform::desktop;
    use phairdlna_mac::worker::Worker;

    let Some(marker) = objc2::MainThreadMarker::new() else {
        return Err(strings.text("cli.error.not_main_thread").to_owned());
    };
    // The token of the installation is read out of its file before every
    // request: a server that is started while the application runs is found
    // then, and a server that was never started is a server that is not there
    // (`MB-001`).
    let client = AdminClient::with_token_file(
        TcpTransport::loopback(options.port),
        options.token_file.clone(),
    );
    let address = client.address();
    let worker = Worker::spawn(client).map_err(|error| {
        let reason = error.reason.clone();
        strings.format("error.worker_not_started", &[("reason", &reason)])
    })?;
    let app = App::with_token_file(
        strings.clone(),
        config_path(),
        address,
        options.token_file.clone(),
    );
    let desktop = desktop::Desktop::new(marker, app, worker)
        .map_err(|reason| strings.format("cli.error.no_menu_bar", &[("reason", &reason)]))?;
    let shared = desktop::shared(desktop);
    desktop::install_handler(shared);
    let application = objc2_app_kit::NSApplication::sharedApplication(marker);
    application.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Accessory);
    application.run();
    Ok(())
}

/// Starts the application on a system that has no menu bar.
#[cfg(not(target_os = "macos"))]
fn start(_options: &Options, strings: &Strings) -> Result<(), String> {
    Err(strings.text("cli.error.not_macos").to_owned())
}

/// The file of the configuration of the server.
fn config_path() -> PathBuf {
    phairdlna_core::config::resolve_path(None)
}
