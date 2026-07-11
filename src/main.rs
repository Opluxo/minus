use std::env;
use std::fs;
use std::process;

use minus_compiler::compile;
use minus_compiler::compile_to_file;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <input.minus> [output.ll]", args[0]);
        eprintln!("       {} -e <code>", args[0]);
        process::exit(1);
    }

    let result = if args[1] == "-e" {
        // Execute code from argument
        if args.len() < 3 {
            eprintln!("Usage: {} -e <code>", args[0]);
            process::exit(1);
        }
        compile(&args[2])
    } else {
        // Read from file
        let input_path = &args[1];
        let source = fs::read_to_string(input_path).unwrap_or_else(|err| {
            eprintln!("Error reading file '{}': {}", input_path, err);
            process::exit(1);
        });
        compile(&source)
    };

    match result {
        Ok(llvm_ir) => {
            if args.len() > 2 && args[1] != "-e" {
                // Write to output file
                let output_path = &args[2];
                fs::write(output_path, &llvm_ir).unwrap_or_else(|err| {
                    eprintln!("Error writing to '{}': {}", output_path, err);
                    process::exit(1);
                });
                println!("Compiled successfully to {}", output_path);
            } else {
                // Print to stdout
                println!("{}", llvm_ir);
            }
        }
        Err(err) => {
            eprintln!("Compilation error: {}", err);
            process::exit(1);
        }
    }
}
