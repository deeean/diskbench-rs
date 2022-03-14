use std::fs::{File, remove_file};
use std::io::{Write, Result, Read, Seek, SeekFrom, BufReader};
use std::thread::sleep;
use std::time::{Duration, Instant};
use indicatif::ProgressBar;
use seahorse::{Context};
use parse_size::{parse_size};

const ONE_KILOBYTE: u64 = 1000;
const ONE_MEGABYTE: u64 = 1000 * ONE_KILOBYTE;

const ONE_SECOND: u64 = Duration::from_secs(1).as_nanos() as u64;

pub const DEFAULT_ITERATIONS: u64 = 5;
pub const DEFAULT_WRITE_BUFFER_SIZE: u64 = 16 * ONE_MEGABYTE;
pub const DEFAULT_READ_BUFFER_SIZE: u64 = 16 * ONE_MEGABYTE;
pub const DEFAULT_TOTAL_BUFFER_SIZE: u64 = 1000 * ONE_MEGABYTE;

pub trait HumanReadable {
  fn to_kilobyte(self) -> String;

  fn to_megabyte(self) -> String;

  fn to_gigabyte(self) -> String;
}

impl HumanReadable for u64 {
  fn to_kilobyte(self) -> String {
    format!("{:.2}KB", self as f64 / 1000.0)
  }

  fn to_megabyte(self) -> String {
    format!("{:.2}MB", self as f64 / 1000.0 / 1000.0)
  }

  fn to_gigabyte(self) -> String {
    format!("{:.2}GB", self as f64 / 1000.0 / 1000.0 / 1000.0)
  }
}

fn get_int_or_default(c: &Context, name: &str, default: u64) -> u64 {
  match c.int_flag(name) {
    Ok(value) => value as u64,
    _ => default,
  }
}

fn get_size_or_default(c: &Context, name: &str, default: u64) -> u64 {
  match c.string_flag(name) {
    Ok(value) => parse_size(value).unwrap() as u64,
    _ => default,
  }
}

fn write(name: &str, cycles: u64, buffer: &[u8]) -> Result<u64> {
  let mut file = File::create(name).expect("Cannot create test file");
  let mut accumulated = 0_u64;

  for _ in 0..cycles {
    let now = Instant::now();
    file.write_all(buffer)?;
    accumulated += now.elapsed().as_nanos() as u64;
  }

  Ok(accumulated / cycles)
}

fn read(name: &str, cycles: u64, read_buffer_size: u64) -> Result<u64> {
  let file = File::open(name).expect("Cannot open test file");
  let mut accumulated = 0_u64;
  let mut buffer = vec![0_u8; read_buffer_size as usize];
  let mut reader = BufReader::new(&file);

  for i in 0..cycles {
    let now = Instant::now();
    reader.seek(SeekFrom::Start(i * read_buffer_size / 8))?;
    reader.read(&mut buffer)?;
    accumulated += now.elapsed().as_nanos() as u64;
  }

  Ok(accumulated / cycles)
}

fn bench_inner(
  iterations: u64,
  write_buffer_size: u64,
  read_buffer_size: u64,
  total_buffer_size: u64,
) {
  let write_cycles = total_buffer_size / write_buffer_size;
  let read_cycles = total_buffer_size / read_buffer_size;
  let progress = ProgressBar::new(iterations as u64 * 2);
  let mut names = Vec::new();
  let mut boxed_write_buffer = vec![0_u8; write_buffer_size as usize].into_boxed_slice();

  println!("Iterations: {}", iterations);
  println!("Write buffer size: {}", write_buffer_size.to_kilobyte());
  println!("Read buffer size: {}", read_buffer_size.to_kilobyte());
  println!("Total buffer size: {}", total_buffer_size.to_gigabyte());

  for i in 0..write_buffer_size {
    boxed_write_buffer[i as usize] = (i % 256) as u8;
  }

  for i in 0..iterations {
    names.push(format!("diskbench{}", i))
  }

  let write_durations = names
    .iter()
    .filter_map(|name| {
      progress.inc(1);

      match write(name.as_str(), write_cycles, boxed_write_buffer.as_ref()) {
        Ok(duration) => {
          sleep(Duration::from_millis(100));
          Some(duration)
        },
        Err(err) => panic!("{}", err),
      }
    })
    .collect::<Vec<_>>();

  sleep(Duration::from_millis(100));

  let read_durations = names
    .iter()
    .filter_map(|name| {
      progress.inc(1);

      match read(name.as_str(), read_cycles, read_buffer_size) {
        Ok(duration) => {
          sleep(Duration::from_millis(100));
          Some(duration)
        },
        Err(err) => panic!("{}", err),
      }
    })
    .collect::<Vec<_>>();

  names
    .iter()
    .for_each(|name| {
      remove_file(name).expect("Cannot remove test file");
    });

  progress.finish();

  let average_write_duration = write_durations.iter().sum::<u64>() / write_durations.len() as u64;
  let average_read_duration = read_durations.iter().sum::<u64>() / read_durations.len() as u64;

  println!("Write Speed: {}/s", (ONE_SECOND / average_write_duration * write_buffer_size).to_megabyte());
  println!("Read Speed: {}/s", (ONE_SECOND / average_read_duration * read_buffer_size).to_megabyte());
}

pub fn bench(c: &Context) {
  let iterations = get_int_or_default(c, "iterations", DEFAULT_ITERATIONS);
  let write_buffer_size = get_size_or_default(c, "write_buffer_size", DEFAULT_WRITE_BUFFER_SIZE);
  let read_buffer_size = get_size_or_default(c, "read_buffer_size", DEFAULT_READ_BUFFER_SIZE);
  let total_buffer_size = get_size_or_default(c, "total_buffer_size", DEFAULT_TOTAL_BUFFER_SIZE);

  bench_inner(iterations, write_buffer_size, read_buffer_size, total_buffer_size);
}

// use std::fs::{File, remove_file};
// use std::io::{BufReader, Result, Read, Write};
// use std::time::{Duration, Instant};
//
// const ITERATION: i32 = 20;
//
// const ONE_SECOND: Duration = Duration::from_secs(1);
//
// const ONE_MEGABYTE: usize = 1000 << 10;
// const ONE_GIGABYTE: usize = 1000 << 20;
//
// const READ_BUFFER_SIZE: usize = ONE_MEGABYTE * 16;
// const WRITE_BUFFER_SIZE: usize = ONE_MEGABYTE * 16;
//
// trait HumanReadable {
//   fn as_megabyte_per_second(&self) -> String;
// }
//
// impl HumanReadable for u64 {
//   fn as_megabyte_per_second(&self) -> String {
//     format!("{:.1}MB/s", *self as f32 / 1000.0 / 1000.0)
//   }
// }
//
// fn write(filename: String, buffer: &[u8]) -> Result<u64> {
//   let mut file = File::create(filename.clone()).expect("Cannot create test file");
//   let mut elapsed: u64 = 0;
//   let iteration = ONE_GIGABYTE / WRITE_BUFFER_SIZE;
//
//   for _ in 0..iteration {
//     let now = Instant::now();
//     file.write_all(buffer)?;
//     elapsed += now.elapsed().as_nanos();
//   }
//
//   let average = elapsed / iteration as u64;
//   let byte_per_second = WRITE_BUFFER_SIZE as u64 * ONE_SECOND.as_nanos() / average;
//
//   Ok(byte_per_second)
// }
//
// fn read(filename: String) -> Result<u64> {
//   let file = File::open(filename.clone()).expect("Cannot open test file");
//   let mut reader = BufReader::new(file);
//   let mut elapsed: u64 = 0;
//   let iteration = ONE_GIGABYTE / READ_BUFFER_SIZE;
//
//   for _ in 0..iteration {
//     let mut buffer = vec![0_u8; READ_BUFFER_SIZE];
//     let now = Instant::now();
//     reader.read(&mut buffer).expect("Unable to read data");
//     elapsed += now.elapsed().as_nanos();
//   }
//
//   let average = elapsed / iteration as u64;
//   let byte_per_second = READ_BUFFER_SIZE as u64 * ONE_SECOND.as_nanos() / average;
//
//   Ok(byte_per_second)
// }
//
// fn cleanup(filenames: Vec<String>) {
//   filenames
//     .iter()
//     .for_each(|filename| {
//       remove_file(filename.clone()).expect("Cannot delete test file")
//     });
// }
//
// pub fn bench() {
//   let mut filenames = Vec::new();
//   let mut boxed_write_buffer = vec![0_u8; WRITE_BUFFER_SIZE].into_boxed_slice();
//
//   for i in 0..WRITE_BUFFER_SIZE {
//     boxed_write_buffer[i] = fastrand::u8(..);
//   }
//
//   for i in 0..ITERATION {
//     filenames.push(format!("diskbench-{}.txt", i));
//   }
//
//   let write_results = filenames
//     .iter()
//     .filter_map(|filename| {
//       match write(filename.clone(), boxed_write_buffer.as_ref()) {
//         Ok(res) => {
//           println!(".");
//           Some(res)
//         },
//         Err(err) => panic!("{}", err),
//       }
//     })
//     .collect::<Vec<_>>();
//
//
//   let read_results = filenames
//     .iter()
//     .filter_map(|filename| {
//       match read(filename.clone()) {
//         Ok(res) => {
//           println!(".");
//           Some(res)
//         },
//         Err(err) => panic!("{}", err),
//       }
//     })
//     .collect::<Vec<_>>();
//
//   println!();
//   println!("Average write speed: {}", (write_results.iter().sum::<u64>() / write_results.len() as u64).as_megabyte_per_second());
//   println!("    Min write speed: {}", write_results.iter().min().unwrap().as_megabyte_per_second());
//   println!("    Max write speed: {}", write_results.iter().max().unwrap().as_megabyte_per_second());
//   println!();
//   println!("Average read speed: {}", (read_results.iter().sum::<u64>() / read_results.len() as u64).as_megabyte_per_second());
//   println!("    Min read speed: {}", read_results.iter().min().unwrap().as_megabyte_per_second());
//   println!("    Max read speed: {}", read_results.iter().max().unwrap().as_megabyte_per_second());
//
//   cleanup(filenames);
// }