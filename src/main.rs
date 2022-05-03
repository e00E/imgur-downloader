// TODO:
// * better argument parsing and more arguments (target directory, concurrent number)
// * better error messages
// * maybe try using HEAD request to determine size of media instead of the json size field because
//   the latter is sometimes incorrect.

use anyhow::{anyhow, Result};
use futures::stream::{StreamExt, TryStreamExt};
use reqwest::Client;
use serde::Deserialize;
use std::{
    env,
    path::{Path, PathBuf},
};
use tokio::{fs, io};

fn is_ascii_alphanumeric(s: &str) -> bool {
    s.chars().all(|char| char.is_ascii_alphanumeric())
}

fn extract_album_id_from_argument(s: &str) -> Option<&str> {
    if is_ascii_alphanumeric(s) && !s.is_empty() {
        return Some(s);
    }
    let s = s.get(s.rfind('/')? + 1..)?;
    if is_ascii_alphanumeric(s) && !s.is_empty() {
        return Some(s);
    }
    None
}

#[derive(Debug, Deserialize)]
struct AlbumResponse {
    media: Vec<MediaResponse>,
}

#[derive(Debug, Deserialize)]
struct MediaResponse {
    url: String,
    ext: String,
    size: u64,
}

async fn get_album(id: &str, client: &Client) -> Result<AlbumResponse> {
    let url = format!(
        "https://api.imgur.com/post/v1/albums/{}?client_id=546c25a59c58ad7&include=media",
        id
    );
    let response = client.get(url.as_str()).send().await?.error_for_status()?;
    response.json().await.map_err(Into::into)
}

async fn get_media(media: &MediaResponse, client: &Client) -> Result<impl io::AsyncRead> {
    let response = client.get(media.url.as_str()).send().await?;
    let stream = response.bytes_stream();
    let reader = tokio_util::io::StreamReader::new(
        stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err)),
    );
    Ok(reader)
}

fn digits_in_decmial_representation(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    ((n as f32).log10() + 1.0).floor() as usize
}

fn file_name(media: &MediaResponse, index: usize, media_count: usize) -> PathBuf {
    assert!(index < media_count);
    let max_digits = digits_in_decmial_representation(media_count - 1);
    let index_digits = digits_in_decmial_representation(index);
    let leading_zeroes = max_digits - index_digits;
    let name = format!("{}{}.{}", "0".repeat(leading_zeroes), index, media.ext);
    PathBuf::from(name)
}

async fn download_media(media: &MediaResponse, destination: &Path, client: &Client) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(destination)
        .await?;
    if media.size == file.metadata().await?.len() {
        println!(
            "Skipping {} because it already exists and the file size is correct.",
            media.url
        );
        return Ok(());
    }
    file.set_len(0).await?;
    println!(
        "Downloading {} to {}.",
        media.url,
        destination.to_string_lossy()
    );
    let mut reader = get_media(media, client).await?;
    io::copy(&mut reader, &mut file).await?;
    Ok(())
}

fn print_help_and_exit() -> ! {
    let help = "Provide a single command line argument that is either an imgur album id or the full url to the album.";
    println!("{}", help);
    std::process::exit(1);
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 || args[1] == "--help" {
        print_help_and_exit();
    }
    let album_id = extract_album_id_from_argument(args[1].as_str())
        .ok_or_else(|| anyhow!("failed to extract album id from first argument"))?;

    let client = Client::builder().build()?;
    println!("Retrieving album information for id {}.", album_id);
    let album = get_album(album_id, &client).await?;
    let destination = Path::new(album_id);
    fs::create_dir_all(destination).await?;
    let media_count = album.media.len();
    println!(
        "Downloading {} media from album id {} to directory {}.",
        media_count,
        album_id,
        destination.to_string_lossy(),
    );

    let media = futures::stream::iter(album.media.into_iter().enumerate());
    media
        .for_each_concurrent(2, |(index, media)| {
            let client = &client;
            async move {
                let mut path = destination.to_path_buf();
                path.push(file_name(&media, index, media_count));
                if let Err(err) = download_media(&media, path.as_path(), client).await {
                    println!(
                        "Failed to download {} to {}: {:?}.",
                        media.url,
                        path.to_string_lossy(),
                        err
                    );
                }
            }
        })
        .await;

    println!("Done");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_album_id_() {
        assert_eq!(
            extract_album_id_from_argument("https://imgur.com/gallery/aA1b"),
            Some("aA1b")
        );
        assert_eq!(
            extract_album_id_from_argument("https://imgur.com/a/aA1b"),
            Some("aA1b")
        );
        assert_eq!(extract_album_id_from_argument("aA1b"), Some("aA1b"));
    }

    #[test]
    fn decimal_digits_() {
        assert_eq!(digits_in_decmial_representation(0), 1);
        assert_eq!(digits_in_decmial_representation(1), 1);
        assert_eq!(digits_in_decmial_representation(9), 1);
        assert_eq!(digits_in_decmial_representation(10), 2);
        assert_eq!(digits_in_decmial_representation(11), 2);
        assert_eq!(digits_in_decmial_representation(99), 2);
        assert_eq!(digits_in_decmial_representation(100), 3);
    }
}
