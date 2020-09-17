use prost::Message;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;

pub mod worker_protocol {
    include!(concat!(env!("OUT_DIR"), "/blaze.worker.rs"));
}

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().peekable();
    // Always discard the executable name.
    args.next().unwrap();

    let program = args.next().expect("program name");

    // If started as a persistent worker.
    if let Some(arg) = args.peek() {
        if arg == "--persistent_worker" {
            loop {
                // TODO: Move this to a lib.rs.
                // TODO: Smarter buffer usage.
                let mut buffer = [0; 10000];
                eprintln!("READ {} BYTES", std::io::stdin().read(&mut buffer[..])?);
                let message: worker_protocol::WorkRequest =
                    prost::Message::decode_length_delimited(&buffer[..])?;
                eprintln!("Req ID: {}", message.request_id);
                eprintln!("Req args: {:?}", message.arguments);
                eprintln!("---");
                let mut cmd = std::process::Command::new(&program);
                cmd.args(message.arguments);
                let output = cmd.output()?;
                let response = worker_protocol::WorkResponse {
                    request_id: message.request_id,
                    exit_code: output.status.code().unwrap(),
                    output: String::from_utf8(output.stdout).unwrap(),
                };
                let mut response_buf = Vec::new();
                response.encode_length_delimited(&mut response_buf)?;
                std::io::stdout().write_all(&response_buf)?;
                std::io::stdout().flush()?;
            }
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
