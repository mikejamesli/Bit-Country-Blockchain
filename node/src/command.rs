// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
	chain_spec,
	cli::{Cli, Subcommand},
	service,
};
use metaverse_runtime::Block;
use sc_cli::{ChainSpec, Role, RuntimeVersion, SubstrateCli};
use sc_service::PartialComponents;

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Bit.Country Metaverse Network Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"support.anonymous.an".into()
	}

	fn copyright_start_year() -> i32 {
		2017
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"dev" => Box::new(chain_spec::development_config()?),
			"" | "local" => Box::new(chain_spec::local_testnet_config()?),
			#[cfg(feature = "with-metaverse-runtime")]
			"metaverse" => Box::new(chain_spec::metaverse_testnet_config()?),
			#[cfg(feature = "with-tewai-runtime")]
			"tewai" => Box::new(chain_spec::tewai_testnet_config()?),
			path => Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?),
		})
	}

	fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		&metaverse_runtime::VERSION
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		}
		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					import_queue,
					..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client, task_manager, ..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		}
		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client, task_manager, ..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		}
		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					import_queue,
					..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		}
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		}
		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents {
					client,
					task_manager,
					backend,
					..
				} = service::new_partial(&config)?;
				Ok((cmd.run(client, backend), task_manager))
			})
		}
		Some(Subcommand::Benchmark(cmd)) => {
			if cfg!(feature = "runtime-benchmarks") {
				let runner = cli.create_runner(cmd)?;

				runner.sync_run(|config| cmd.run::<Block, service::Executor>(config))
			} else {
				Err(
					"Benchmarking wasn't enabled when building the node. You can enable it with \
				     `--features runtime-benchmarks`."
						.into(),
				)
			}
		}
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| async move {
				match config.role {
					Role::Light => service::new_light(config),
					_ => service::new_full(config),
				}
				.map_err(sc_cli::Error::Service)
			})
		}
	}
}

//use crate::{
//    chain_spec,
//    cli::{Cli, Subcommand},
//    service,
//};
//use metaverse_runtime::opaque::Block;
//use sc_cli::{ChainSpec, Role, RuntimeVersion, SubstrateCli};
//use sc_service::PartialComponents;
//
//impl SubstrateCli for Cli {
//    fn impl_name() -> String {
//        "Metaverse Node".into()
//    }
//
//    fn impl_version() -> String {
//        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
//    }
//
//    fn description() -> String {
//        env!("CARGO_PKG_DESCRIPTION").into()
//    }
//
//    fn author() -> String {
//        env!("CARGO_PKG_AUTHORS").into()
//    }
//
//    fn support_url() -> String {
//        "https://github.com/bit-country".into()
//    }
//
//    fn copyright_start_year() -> i32 {
//        2020
//    }
//
//    fn load_spec(&self, id: &str) -> Result<Box<dyn ChainSpec>, String> {
//        todo!()
//    }
//
//    //    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>,
// String> {    //        let spec = match id {
//    //            "" => {
//    //                return Err(
//    //                    "Please specify which chain you want to run, e.g. --dev or
// --chain=tewai"    //                        .into(),
//    //                )
//    //            }
//    //            "dev" => Box::new(chain_spec::development_config()),
//    //            "local" => Box::new(chain_spec::local_testnet_config()),
//    //            "tewai" => Box::new(chain_spec::tewai_testnet_config()?),
//    //            path => Box::new(chain_spec::ChainSpec::from_json_file(
//    //                std::path::PathBuf::from(path),
//    //            )?),
//    //        };
//    //        Ok(spec)
//    //    }
//
//    fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
//        todo!()
//    }
//}
//
///// Parse and run command line arguments
//pub fn run() -> sc_cli::Result<()> {
//    let cli = Cli::from_args();
//    Ok(())
//    //    match &cli.subcommand {
//    //        None => {
//    //            let runner = cli.create_runner(&cli.run)?;
//    //            runner.run_node_until_exit(|config| async move {
//    //                match config.role {
//    //                    Role::Light => service::new_light(config),
//    //                    _ => service::new_full(config),
//    //                }
//    //                .map_err(sc_cli::Error::Service)
//    //            })
//    //        }
//    //        Some(Subcommand::Inspect(cmd)) => {
//    //            let runner = cli.create_runner(cmd)?;
//    //
//    //            runner.sync_run(|config| cmd.run::<Block, RuntimeApi, Executor>(config))
//    //        }
//    //        Some(Subcommand::Benchmark(cmd)) => {
//    //            if cfg!(feature = "runtime-benchmarks") {
//    //                let runner = cli.create_runner(cmd)?;
//    //
//    //                runner.sync_run(|config| cmd.run::<Block, Executor>(config))
//    //            } else {
//    //                Err("Benchmarking wasn't enabled when building the node. \
//    //				You can enable it with `--features runtime-benchmarks`."
//    //                    .into())
//    //            }
//    //        }
//    //        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
//    //        Some(Subcommand::Sign(cmd)) => cmd.run(),
//    //        Some(Subcommand::Verify(cmd)) => cmd.run(),
//    //        Some(Subcommand::Vanity(cmd)) => cmd.run(),
//    //        Some(Subcommand::BuildSpec(cmd)) => {
//    //            let runner = cli.create_runner(cmd)?;
//    //            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
//    //        }
//    //        Some(Subcommand::CheckBlock(cmd)) => {
//    //            let runner = cli.create_runner(cmd)?;
//    //            runner.async_run(|config| {
//    //                let PartialComponents {
//    //                    client,
//    //                    task_manager,
//    //                    import_queue,
//    //                    ..
//    //                } = new_partial(&config)?;
//    //                Ok((cmd.run(client, import_queue), task_manager))
//    //            })
//    //        }
//    //        Some(Subcommand::ExportBlocks(cmd)) => {
//    //            let runner = cli.create_runner(cmd)?;
//    //            runner.async_run(|config| {
//    //                let PartialComponents {
//    //                    client,
//    //                    task_manager,
//    //                    ..
//    //                } = new_partial(&config)?;
//    //                Ok((cmd.run(client, config.database), task_manager))
//    //            })
//    //        }
//    //        Some(Subcommand::ExportState(cmd)) => {
//    //            let runner = cli.create_runner(cmd)?;
//    //            runner.async_run(|config| {
//    //                let PartialComponents {
//    //                    client,
//    //                    task_manager,
//    //                    ..
//    //                } = new_partial(&config)?;
//    //                Ok((cmd.run(client, config.chain_spec), task_manager))
//    //            })
//    //        }
//    //        Some(Subcommand::ImportBlocks(cmd)) => {
//    //            let runner = cli.create_runner(cmd)?;
//    //            runner.async_run(|config| {
//    //                let PartialComponents {
//    //                    client,
//    //                    task_manager,
//    //                    import_queue,
//    //                    ..
//    //                } = new_partial(&config)?;
//    //                Ok((cmd.run(client, import_queue), task_manager))
//    //            })
//    //        }
//    //        Some(Subcommand::PurgeChain(cmd)) => {
//    //            let runner = cli.create_runner(cmd)?;
//    //            runner.sync_run(|config| cmd.run(config.database))
//    //        }
//    //        Some(Subcommand::Revert(cmd)) => {
//    //            let runner = cli.create_runner(cmd)?;
//    //            runner.async_run(|config| {
//    //                let PartialComponents {
//    //                    client,
//    //                    task_manager,
//    //                    backend,
//    //                    ..
//    //                } = new_partial(&config)?;
//    //                Ok((cmd.run(client, backend), task_manager))
//    //            })
//    //        }
//    //    }
//}
