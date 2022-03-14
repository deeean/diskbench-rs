extern crate core;

use seahorse::{App, Flag, FlagType};
use crate::diskbench::{HumanReadable};

mod diskbench;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let app = App::new(env!("CARGO_PKG_NAME"))
    .description(env!("CARGO_PKG_DESCRIPTION"))
    .author(env!("CARGO_PKG_AUTHORS"))
    .version(env!("CARGO_PKG_VERSION"))
    .usage("diskbench [args]")
    .flag(
      Flag::new("iterations", FlagType::Int)
        .alias("i")
        .description(format!("default {}", diskbench::DEFAULT_ITERATIONS))
    )
    .flag(
      Flag::new("write_buffer_size", FlagType::String)
        .alias("w")
        .description(format!("default {}", diskbench::DEFAULT_WRITE_BUFFER_SIZE.to_kilobyte()))
    )
    .flag(
      Flag::new("read_buffer_size", FlagType::String)
        .alias("r")
        .description(format!("default {}", diskbench::DEFAULT_READ_BUFFER_SIZE.to_kilobyte()))
    )
    .flag(
      Flag::new("total_buffer_size", FlagType::String)
        .alias("t")
        .description(format!("default {}", diskbench::DEFAULT_TOTAL_BUFFER_SIZE.to_gigabyte()))
    )
    .action(diskbench::bench);

  app.run(args);
}
