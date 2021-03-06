//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
mod silly_rpc;

fn main() -> sc_cli::Result<()> {
	let version = sc_cli::VersionInfo {
		name: "RPC Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "node-template",
		author: "Anonymous",
		description: "RPC Node",
		support_url: "support.anonymous.an",
		copyright_start_year: 2019,
	};

	command::run(version)
}
