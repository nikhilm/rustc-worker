use prost::Message;
use std::io;
use std::io::BufRead;
use std::io::Write;

pub mod worker_protocol {
    include!(concat!(env!("OUT_DIR"), "/blaze.worker.rs"));
}

pub struct Worker {
    rustc: String,
    incremental_dir: String,
}

impl Worker {
    pub fn new<P: Into<String>>(rustc: P) -> io::Result<Self> {
        // TODO: Use workspace name etc.
        std::fs::create_dir_all("/tmp/rustc-worker-ninjars/incremental")?;
        Ok(Worker {
            rustc: rustc.into(),
            incremental_dir: "/tmp/rustc-worker-ninjars/incremental".into(),
        })
    }

    fn handle_request(
        &self,
        request: worker_protocol::WorkRequest,
    ) -> io::Result<worker_protocol::WorkResponse> {
        let mut cmd = std::process::Command::new(&self.rustc);
        cmd.args(request.arguments);
        // TODO: Do this format only once.
        cmd.arg(format!("--codegen=incremental={}", self.incremental_dir));
        let output = cmd.output()?;
        Ok(worker_protocol::WorkResponse {
            request_id: request.request_id,
            exit_code: output.status.code().unwrap(),
            output: String::from_utf8(output.stderr).expect("TODO: use the Result"),
        })
    }

    pub fn main_loop<R: io::Read>(&self, reader: &mut R) -> io::Result<()> {
        // We need some kind of circular buffer that can be written to on one end by calling
        // read, and can be read from on the other end in little bits, and then moved forward
        // by the little bit we don't care about.
        loop {
            let mut len_buffer = vec![0; 10];
            let mut offset = 0;
            let msg_len = loop {
                let n = reader.read(&mut len_buffer[offset..])?;
                if n == 0 {
                    return Ok(());
                }
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
            // Read the rest. We don't handle EOF here because when a request length was sent, we
            // expect a valid message to follow. Not getting that is an error.
            reader.read_exact(&mut data_buffer[already_read..])?;
            eprintln!("Beginning of data buffer is {:?}", &data_buffer[..20]);
            let message: worker_protocol::WorkRequest = prost::Message::decode(&data_buffer[..])?;
            eprintln!("Req ID: {}", message.request_id);
            eprintln!("Req args: {:?}", message.arguments);
            eprintln!("---");

            let response = self.handle_request(message)?;
            let mut response_buf = Vec::new();
            response.encode_length_delimited(&mut response_buf)?;
            std::io::stdout().write_all(&response_buf)?;
            std::io::stdout().flush()?;
        }
    }

    pub fn once_with_response_file<P: AsRef<std::path::Path>>(
        &self,
        response_file_path: P,
    ) -> io::Result<std::process::ExitStatus> {
        let file = std::io::BufReader::new(std::fs::File::open(response_file_path)?);

        let mut cmd = std::process::Command::new(&self.rustc);
        for line in file.lines() {
            cmd.arg(line?);
        }
        cmd.status()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_eof() {}
}
