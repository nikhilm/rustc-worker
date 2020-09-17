use std::io::Read;

fn main() -> std::io::Result<()> {
    eprintln!("CWD: {:?}", std::env::current_dir()?);
    for arg in std::env::args() {
        eprintln!("ARG: {}", arg);
    }
    let mut buffer = [0; 60];
    eprintln!("READ {} BYTES", std::io::stdin().read(&mut buffer[..])?);
    eprintln!("STDIN: {:?}", buffer);
    std::process::exit(1);
}
