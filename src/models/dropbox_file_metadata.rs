use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DropboxFileMetadata {
    pub name: String,
    pub path_lower: String,
    pub client_modified: String,
    pub server_modified: String,
    pub size: u64,
}

impl fmt::Display for DropboxFileMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "File Name: {}\nPath: {}\nClient Modified: {}\nServer Modified: {}\nSize: {} bytes\n",
            self.name, self.path_lower, self.client_modified, self.server_modified, self.size
        )
    }
}
