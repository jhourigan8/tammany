mod block;
mod p2p;

use std::error::Error;

/// The `tokio::main` attribute sets up a tokio runtime.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    p2p::run(block::Handler::new()).await
}