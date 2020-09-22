fn main() -> std::io::Result<()> {
    let mut args = std::env::args().peekable();
    // Always discard the executable name.
    args.next().unwrap();

    let program = std::fs::canonicalize(args.next().expect("program name"))?
        .into_os_string()
        .into_string()
        .unwrap();
    let rustc_path = std::fs::canonicalize(args.next().expect("rustc path"))?
        .into_os_string()
        .into_string()
        .unwrap();
    let compilation_mode = args.next().expect("compilation mode");
    // TODO: program and rustc_path will combine when this is merged into rules_rust.
    let worker = rustc_worker::Worker::new(program, rustc_path, compilation_mode)?;

    // If started as a persistent worker.
    if let Some(arg) = args.peek() {
        if arg == "--persistent_worker" {
            let stdin = std::io::stdin();
            let mut locked = stdin.lock();
            return worker.main_loop(&mut locked);
        }
    }

    // Spawn process as normal.
    // The process wrapper does not support response files.
    let response_file_arg = args.next().unwrap();
    // The response file has to be the last (and only) argument left.
    assert!(args.peek().is_none(), "iterator should be consumed!");
    assert!(response_file_arg.starts_with("@"));
    let response_file_path = &response_file_arg[1..];
    let status = worker.once_with_response_file(response_file_path)?;
    std::process::exit(status.code().unwrap());
}
