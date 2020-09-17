use std::io::BufRead;
use std::io::Read;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().peekable();
    // Always discard the executable name.
    args.next().unwrap();

    let program = args.next().expect("program name");

    // If started as a persistent worker.
    if let Some(arg) = args.peek() {
        if arg == "--persistent_worker" {
            // TODO: Handle remaining args and create a relevant config to launch the worker.
            eprintln!("CWD: {:?}", std::env::current_dir()?);
            for arg in args {
                eprintln!("ARG: {}", arg);
            }
            let mut buffer = [0; 60];
            eprintln!("READ {} BYTES", std::io::stdin().read(&mut buffer[..])?);
            eprintln!("STDIN: {:?}", buffer);
            std::process::exit(1);
        }
    }

    // Spawn process as normal.
    // The process wrapper does not support response files.
    let response_file_arg = args.next().unwrap();
    assert!(args.peek().is_none(), "iterator should be consumed!");
    assert!(response_file_arg.starts_with("@"));
    let response_file_path = &response_file_arg[1..];
    let file = std::io::BufReader::new(std::fs::File::open(response_file_path)?);

    let mut cmd = std::process::Command::new(program);
    for line in file.lines() {
        cmd.arg(line?);
    }
    let status = cmd.status()?;
    std::process::exit(status.code().unwrap());
}
