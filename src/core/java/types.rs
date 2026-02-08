use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdoptiumRelease {
    pub release_name: String,
    pub binaries: Vec<AdoptiumBinary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdoptiumBinary {
    pub package: AdoptiumPackage,
    pub architecture: String,
    pub os: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdoptiumPackage {
    pub link: String,
    pub name: String,
    pub checksum: String,
    pub size: u64,
}
