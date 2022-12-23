//!
//! Cli trait for implementing a user-side command-line processor.
//! 

use std::sync::Arc;
use async_trait::async_trait;
use crate::result::Result;
use crate::terminal::Terminal;

#[async_trait]
pub trait Cli : Sync + Send {
    fn init(&self, _term : &Arc<Terminal>) -> Result<()> { Ok(()) }
    async fn digest(&self, term : Arc<Terminal>, cmd: String) -> Result<()>;
    async fn complete(&self, term : Arc<Terminal>, cmd : String) -> Result<Vec<String>>;
}
