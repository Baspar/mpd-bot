use tokio::fs::File;
use tokio::io;
use futures::stream::TryStreamExt;
use tokio_util::compat::FuturesAsyncReadCompatExt;
use crate::utils::BoxError;

pub async fn download_file(url: String, filename: String) -> Result<(), BoxError> {
    println!("Downloading {}", url);
    let mut response = reqwest::get(&url).await?
        .error_for_status()?
        .bytes_stream()
        .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
        .into_async_read()
        .compat();

    let mut file = File::create(filename).await?;
    io::copy(&mut response, &mut file).await?;
    println!("{} downloaded", url);
    Ok(())
}
