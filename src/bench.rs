use std::fs::{File, remove_file};
use std::io::{BufReader, Result, Read, Write};
use std::time::{Duration, Instant};

const ITERATION: i32 = 20;

const ONE_SECOND: Duration = Duration::from_secs(1);

const ONE_MEGABYTE: usize = 1000 << 10;
const ONE_GIGABYTE: usize = 1000 << 20;

const READ_BUFFER_SIZE: usize = ONE_MEGABYTE * 4;
const WRITE_BUFFER_SIZE: usize = ONE_MEGABYTE * 16;

trait HumanReadable {
  fn as_megabyte_per_second(&self) -> f32;
}

impl HumanReadable for u128 {
  fn as_megabyte_per_second(&self) -> f32 {
    *self as f32 / 1000.0 / 1000.0
  }
}

fn warmup() -> Result<()> {
  let mut file = File::create("diskbench-warmup.txt").expect("Cannot create test file");
  let mut buffer = vec![0_u8; ONE_MEGABYTE];

  for i in 0..ONE_MEGABYTE {
    buffer[i] = fastrand::u8(..);
  }

  file.write_all(&buffer)?;
  file.sync_data()?;

  remove_file("diskbench-warmup.txt")
}

fn write(filename: String, buffer: &[u8]) -> Result<u128> {
  let mut file = File::create(filename.clone()).expect("Cannot create test file");
  let mut elapsed: u128 = 0;
  let iteration = ONE_GIGABYTE / WRITE_BUFFER_SIZE;

  for _ in 0..iteration {
    let now = Instant::now();
    file.write_all(buffer)?;
    elapsed += now.elapsed().as_nanos();
  }

  file.sync_all()?;

  let average = elapsed / iteration as u128;
  let byte_per_second = WRITE_BUFFER_SIZE as u128 * ONE_SECOND.as_nanos() / average;

  Ok(byte_per_second)
}

fn read(filename: String) -> Result<u128> {
  let file = File::open(filename.clone()).expect("Cannot open test file");
  let mut reader = BufReader::new(file);
  let mut buffer = vec![0_u8; READ_BUFFER_SIZE];
  let mut elapsed: u128 = 0;
  let iteration = ONE_GIGABYTE / READ_BUFFER_SIZE;

  for _ in 0..iteration {
    let now = Instant::now();
    reader.read(&mut buffer).expect("Unable to read data");
    elapsed += now.elapsed().as_nanos();
  }

  let average = elapsed / iteration as u128;
  let byte_per_second = READ_BUFFER_SIZE as u128 * ONE_SECOND.as_nanos() / average;

  Ok(byte_per_second)
}

fn cleanup(filenames: Vec<String>) {
  filenames
    .iter()
    .for_each(|filename| {
      remove_file(filename.clone()).expect("Cannot delete test file")
    });
}

pub fn benchmark() {
  match warmup() {
    Err(err) => panic!("{}", err),
    _ => {}
  }

  let mut filenames = Vec::new();
  let mut boxed_write_buffer = vec![0_u8; WRITE_BUFFER_SIZE].into_boxed_slice();

  for i in 0..WRITE_BUFFER_SIZE {
    boxed_write_buffer[i] = fastrand::u8(..);
  }

  for i in 0..ITERATION {
    filenames.push(format!("diskbench-{}.txt", i));
  }

  let write_results = filenames
    .iter()
    .filter_map(|filename| {
      match write(filename.clone(), boxed_write_buffer.as_ref()) {
        Ok(res) => {
          println!(".");
          Some(res)
        },
        Err(err) => panic!("{}", err),
      }
    })
    .collect::<Vec<_>>();

  let read_results = filenames
    .iter()
    .filter_map(|filename| {
      match read(filename.clone()) {
        Ok(res) => {
          println!(".");
          Some(res)
        },
        Err(err) => panic!("{}", err),
      }
    })
    .collect::<Vec<_>>();

  println!();
  println!("Average write speed: {:.2}MB/s", (write_results.iter().sum::<u128>() / write_results.len() as u128).as_megabyte_per_second());
  println!("    Min write speed: {:.2}MB/s", write_results.iter().min().unwrap().as_megabyte_per_second());
  println!("    Max write speed: {:.2}MB/s", write_results.iter().max().unwrap().as_megabyte_per_second());
  println!();
  println!(" Average read speed: {:.2}MB/s", (read_results.iter().sum::<u128>() / read_results.len() as u128).as_megabyte_per_second());
  println!("     Min read speed: {:.2}MB/s", read_results.iter().min().unwrap().as_megabyte_per_second());
  println!("     Max read speed: {:.2}MB/s", read_results.iter().max().unwrap().as_megabyte_per_second());

  cleanup(filenames);
}