use std::path::PathBuf;

use clap::{crate_authors, crate_version, value_parser, Arg, ArgMatches, Command};

pub fn parse_args() -> ArgMatches {
    Command::new("Rust8")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Rust8 is a CHIP-8 interpreter")
        .arg(
            Arg::new("rom")
                .help("Path to the ROM to load")
                .required(false)
                .value_name("ROM")
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("clock_speed")
                .short('c')
                .long("clock-speed")
                .value_name("SPEED")
                .help("Set cpu clock speed (in Hz)")
                .default_value("700")
                .value_parser(value_parser!(u32)),
        )
        .arg(
            Arg::new("modified_shift")
                .short('s')
                .long("-use-modified-shift")
                .help("Use modified 8XY6/8XYE operation (shift VX, not VY)")
                .default_value("true")
                .min_values(0)
                .value_name("bool")
                .require_equals(true)
                .default_missing_value("true")
                .value_parser(value_parser!(bool)),
        )
        .arg(
            Arg::new("modified_load")
                .short('l')
                .long("-use-modified-load")
                .help("Use modified FX55/FX65 operation (don't increase I by X + 1)")
                .default_value("false")
                .min_values(0)
                .value_name("bool")
                .require_equals(true)
                .default_missing_value("true")
                .value_parser(value_parser!(bool)),
        )
        .get_matches()
}
