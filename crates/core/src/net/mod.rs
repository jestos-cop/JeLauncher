mod cache;
mod client;
mod download;

pub use client::HttpClient;
pub use download::{Download, DownloadProgress, sha1_of};
