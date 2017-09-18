use std::sync::Arc;
use std::ffi::CStr;
use std::ptr;
use std::path::Path;
use std::fs::File;
use std::io::{Read, BufReader};
use vk;
use ::{VkcResult, Device};


pub fn read_file<P: AsRef<Path>>(file: P) -> VkcResult<Vec<u8>> {
    let file_name = file.as_ref().display().to_string();
    let f = File::open(file).expect("shader file not found");
    let file_bytes = f.metadata().unwrap().len() as usize;
    let mut contents = Vec::<u8>::with_capacity(file_bytes);
    let mut reader = BufReader::new(f);
    match reader.read_to_end(&mut contents) {
        Ok(bytes) => {
            assert_eq!(bytes, file_bytes);
            println!("Read {} bytes from {}", bytes, &file_name);
        },
        Err(e) => panic!("{}", e),
    }
    Ok(contents)
}