use std::fs::{File, remove_file};
use std::io::{BufReader, Result, Read, Write};
use std::time::{Duration, Instant};

const ITERATION: i32 = 20;

const ONE_SECOND: Duration = Duration::from_secs(1);

const ONE_MEBIBYTE: usize = 1024 << 10;
const ONE_GIBIBYTE: usize = 1024 << 20;

const READ_BUFFER_SIZE: usize = ONE_MEBIBYTE * 4;
const WRITE_BUFFER_SIZE: usize = ONE_MEBIBYTE * 16;

fn write(filename: String, buffer: &[u8]) -> Result<u128> {
  let mut file = File::create(filename.clone()).expect("Cannot create test file");
  let mut elapsed: u128 = 0;
  let iteration = ONE_GIBIBYTE / WRITE_BUFFER_SIZE;

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
  let iteration = ONE_GIBIBYTE / READ_BUFFER_SIZE;

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
    })
}

pub fn benchmark() {
  let mut filenames = Vec::new();
  let mut boxed_write_buffer = vec![0_u8; WRITE_BUFFER_SIZE].into_boxed_slice();

  for i in 0..WRITE_BUFFER_SIZE {
    boxed_write_buffer[i] = fastrand::u8(..);
  }

  for i in 0..ITERATION {
    filenames.push(format!("diskbench{}.txt", i));
  }

  let write_reports = filenames
    .iter()
    .filter_map(|filename| {
      match write(filename.clone(), boxed_write_buffer.as_ref()) {
        Ok(res) => {
          Some(res)
        },
        Err(err) => panic!("{}", err),
      }
    })
    .collect::<Vec<_>>();

  let read_reports = filenames
    .iter()
    .filter_map(|filename| {
      match read(filename.clone()) {
        Ok(res) => Some(res),
        Err(err) => panic!("{}", err),
      }
    })
    .collect::<Vec<_>>();

  let write_sum_byte_per_second: u128 = write_reports
    .iter()
    .sum();

  let read_sum_byte_per_second: u128 = read_reports
    .iter()
    .sum();

  let write_average_byte_per_second: u128 = write_sum_byte_per_second / write_reports.len() as u128;
  let read_average_byte_per_second: u128 = read_sum_byte_per_second / read_reports.len() as u128;

  println!(" Write {:.2}MB/s", write_average_byte_per_second as f32 / 1024.0 / 1024.0);
  println!("  Read {:.2}MB/s", read_average_byte_per_second as f32 / 1024.0 / 1024.0);

  cleanup(filenames);
}