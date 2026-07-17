/// Common error type for x-chaos crates.
#[derive(Debug)]
pub enum Error {
    Init,
    NoDevice,
    Stream,
    Unsupported,
}
