//! The command line entry point of PhairDLNA.
//!
//! What: parses the arguments, runs the command and returns the exit code.
//! Why: the binary must stay a thin shell around the testable library in
//!      `lib.rs`; everything that can be tested lives there.
//! Example: `phairdlna-cli config validate --config /etc/phairdlna/config.toml`

mod main_parse;

use std::process::ExitCode;

use main_parse::parse;
use phairdlna_cli::{run, EXIT_CONFIGURATION};

/// The usage text.
const USAGE: &str = "\
Usage: phairdlna-cli <command> [options]

Commands:
  version                  Print the version and the licence
  status                   Print server status (version, uptime, roots, transfers) (OPS-002)
  scan [root]              Scan library on demand (triggers daemon or scans in-process) (OPS-003)
  config path              Print the path the configuration would be read from
  config validate          Check a configuration file and report the first problem
  config show              Print the effective configuration
  config export            Export the effective configuration
  config import <file>     Read a configuration file and print what it would become (or --apply)
  config explain           Explain the configuration settings and tiers
  ssdp headers             Print the SSDP headers the server would send
  ssdp replay <corpus>     Replay stored M-SEARCH requests and inspect responses (INT-103)
  catalogue scan           Print the tree of the library with the client identifiers
  catalogue browse <id>    Print the document a client receives for a container
  serve                    Start the server in the foreground
  service install          Write the service file and register the service
  service uninstall        Remove the service and its file
  service status           Say what the service manager knows about the service
  service start            Start the service
  service stop             Stop the service
  diagnostics              Write the report for a support ticket
  setup                    Ask the questions of the first run and write the configuration
  describe                 Print the device description and SSDP headers (OPS-005)
  interfaces               List known network interfaces and their exposure (OPS-004)

Options:
  --config <path>          The configuration file to use
  --port <number>          The port the server listens on (serve only)
  --force                  Replace a file that is already there (service and setup)
  --dry-run                Say what would be done, and do nothing (service install only)
  --system                 Act on the service of the machine, not of the user
  --unredacted             Show the real paths of the machine (diagnostics only)
  --export <file>          Write the report to a file (diagnostics only)
  --json                   Write the report, status, interfaces or explain as JSON
  --expert                 Include expert-level settings (config explain only)
  --apply                  Write the imported configuration to disk (config import only)
  -h, --help               Print this text
";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match parse(&arguments) {
        Ok(command) => {
            let outcome = run(&command);
            print!("{}", outcome.stdout);
            ExitCode::from(outcome.exit_code.clamp(0, 255) as u8)
        }
        Err(message) => {
            eprint!("{message}\n{USAGE}");
            ExitCode::from(EXIT_CONFIGURATION as u8)
        }
    }
}
