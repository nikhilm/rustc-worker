/// An input file.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Input {
    /// The path in the file system where to read this input artifact from. This is
    /// either a path relative to the execution root (the worker process is
    /// launched with the working directory set to the execution root), or an
    /// absolute path.
    #[prost(string, tag = "1")]
    pub path: std::string::String,
    /// A hash-value of the contents. The format of the contents is unspecified and
    /// the digest should be treated as an opaque token.
    #[prost(bytes, tag = "2")]
    pub digest: std::vec::Vec<u8>,
}
/// This represents a single work unit that Blaze sends to the worker.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkRequest {
    #[prost(string, repeated, tag = "1")]
    pub arguments: ::std::vec::Vec<std::string::String>,
    /// The inputs that the worker is allowed to read during execution of this
    /// request.
    #[prost(message, repeated, tag = "2")]
    pub inputs: ::std::vec::Vec<Input>,
    /// To support multiplex worker, each WorkRequest must have an unique ID. This
    /// ID should be attached unchanged to the WorkResponse.
    #[prost(int32, tag = "3")]
    pub request_id: i32,
}
/// The worker sends this message to Blaze when it finished its work on the
/// WorkRequest message.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WorkResponse {
    #[prost(int32, tag = "1")]
    pub exit_code: i32,
    /// This is printed to the user after the WorkResponse has been received and is
    /// supposed to contain compiler warnings / errors etc. - thus we'll use a
    /// string type here, which gives us UTF-8 encoding.
    #[prost(string, tag = "2")]
    pub output: std::string::String,
    /// To support multiplex worker, each WorkResponse must have an unique ID.
    /// Since worker processes which support multiplex worker will handle multiple
    /// WorkRequests in parallel, this ID will be used to determined which
    /// WorkerProxy does this WorkResponse belong to.
    #[prost(int32, tag = "3")]
    pub request_id: i32,
}
