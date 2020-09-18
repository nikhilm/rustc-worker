use bytes::Buf;
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
            // TODO: Move this to a lib.rs.
            let stdin = std::io::stdin();
            let mut locked = stdin.lock();
            // We need some kind of circular buffer that can be written to on one end by calling
            // read, and can be read from on the other end in little bits, and then moved forward
            // by the little bit we don't care about.
            loop {
                let mut len_buffer = vec![0; 10];
                let mut offset = 0;
                let msg_len = loop {
                    let n = locked.read(&mut len_buffer[offset..])?;
                    offset += n;
                    eprintln!(
                        "Read {} offset {} has {} {:?}",
                        n,
                        offset,
                        len_buffer.len(),
                        &len_buffer[..len_buffer.len()]
                    );
                    match prost::decode_length_delimiter(&len_buffer[..]) {
                        Ok(n) => {
                            eprintln!("Know how much to read {}", n);
                            break n;
                        }
                        _ => {
                            if offset >= len_buffer.len() {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Invalid length",
                                ));
                            }
                        }
                    }
                };
                // Infer how much of the len buffer actually has data.
                let encoded_len = prost::length_delimiter_len(msg_len);
                eprintln!(
                    "Encoded len {}, so need to grab rest of vec {:?}",
                    encoded_len,
                    &len_buffer[encoded_len..]
                );
                let mut data_buffer = vec![0; msg_len];
                eprintln!("Created data bufer with len {}", data_buffer.len());
                let already_read = len_buffer.len() - encoded_len;
                // Copy over left over data after reading len.
                (&mut data_buffer[0..already_read]).copy_from_slice(&len_buffer[encoded_len..]);
                // Read the rest.
                locked.read_exact(&mut data_buffer[already_read..])?;
                eprintln!("Beginning of data buffer is {:?}", &data_buffer[..20]);
                let message: worker_protocol::WorkRequest =
                    prost::Message::decode(&data_buffer[..])?;
                eprintln!("Req ID: {}", message.request_id);
                eprintln!("Req args: {:?}", message.arguments);
                eprintln!("---");
                let mut cmd = std::process::Command::new(&program);
                // TODO: Use workspace name etc.
                std::fs::create_dir_all("/tmp/rustc-worker-ninjars/incremental")?;
                cmd.args(message.arguments);
                cmd.arg("--codegen=incremental=/tmp/rustc-worker-ninjars/incremental");
                let output = cmd.output()?;
                let response = worker_protocol::WorkResponse {
                    request_id: message.request_id,
                    exit_code: output.status.code().unwrap(),
                    output: String::from_utf8(output.stderr).unwrap(),
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
