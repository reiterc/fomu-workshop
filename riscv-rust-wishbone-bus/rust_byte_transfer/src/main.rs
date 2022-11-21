/* 	
@ Author: Clark Reiter

This file is used to read and write data to a FOMU chip.
The FOMU chip is an FPGA that fits in a USB port.
FOMU contains a softcore RISC-V core and a softcore USB implementation
in order to communicate with the HOST PC. The software running on the FOMU
softcore implements a simple encrpytion cipher, and the 64bit KEY is 
hardcoded in the FOMU software.

Usage: 
   TO ENCRYPT: cargo --run <file_to_encrpyt> <cipher_text_output_file>
   TO DECRYPT: cargo --run <cipher_text_output_file> <file_to_encrpyt>
 */
use std::fs::File;
use std::env;
use std::io::prelude::*;
use std::mem::transmute;
use std::process;
use wishbone_bridge::UsbBridge;

fn main() {
    
    //Create the bridge to FOMU chip
    let bridge = UsbBridge::new().pid(0x5bf0).create().unwrap();

    // Create a path to the desired file
    let args: Vec<String> = env::args().collect();
    let input_file_path = &args[1];
    let output_file_path = &args[2];
	
    // Open a file in write-only mode, returns `io::Result<File>`
    let mut new_file = match File::create(&output_file_path) {
        Err(why) => panic!("couldn't create output_file_path: {}", why),
        Ok(new_file) => new_file,
    };
		
    // Convert the input string into a vector of bytes
    let mut input_bytes = match std::fs::read(&input_file_path) {
        Ok(vec) => vec,
	Err(error) => panic!("Problem opening the file: {:?}", error),
    };
	
    // Determine the size of the input and add padding (if needed);
    // since we can only transfer in words
    while (input_bytes.len() % 4) > 0 {
        input_bytes.push(0x00);
    }

    let input_size = input_bytes.len();
    let mem_base:  u32 = 0x10000000;
    let mem_start: u32 = 0x10000004;

    for i in (0..input_size-1).step_by(4) {
        let temp = [input_bytes[i],input_bytes[i+1],input_bytes[i+2],input_bytes[i+3]];
	bridge.poke(mem_start + i as u32, u32::from_be_bytes(temp)).unwrap();
    }

    // Signal to FOMU we finished writing our data
    bridge.poke(mem_base, 0x99999999).unwrap();
	
    // wait until FOMU tells us it's finished encrpytion processing
    loop {
	if bridge.peek(mem_base).unwrap() == 0x88888888 {
	break;
	}
    }

    // Read the cipher text
    let mut output_text = vec![];
    for i in (0..input_size-1).step_by(4) {
	output_text.push(bridge.peek(mem_start + i as u32).unwrap());
    }
	
    // Write the cipher text to a file
    for i in output_text {
        let bytes: [u8; 4] = unsafe { transmute(i.to_be()) };
	new_file.write_all(&bytes).expect("Unable to write to file");
    }
	
    // Exit, for some reason the app doesnt quit on it's own.
    //println!("Exiting");
    process::exit(0);
	
}
