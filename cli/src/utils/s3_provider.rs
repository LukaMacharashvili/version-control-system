use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use aws_sdk_s3 as s3;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use s3::operation::get_object::GetObjectOutput;
use s3::Client;

use super::traverse_directory;

pub async fn clear_bucket(client: &Client, bucket_name: &str) -> io::Result<()> {
    let objects = client
        .list_objects_v2()
        .bucket(bucket_name)
        .send()
        .await
        .unwrap();

    let mut delete_objects: Vec<ObjectIdentifier> = vec![];
    for obj in objects.contents() {
        let obj_id = ObjectIdentifier::builder()
            .set_key(Some(obj.key().unwrap().to_string()))
            .build()
            .unwrap();
        delete_objects.push(obj_id);
    }

    client
        .delete_objects()
        .bucket(bucket_name)
        .delete(
            Delete::builder()
                .set_objects(Some(delete_objects))
                .build()
                .unwrap(),
        )
        .send()
        .await
        .unwrap();

    Ok(())
}

pub async fn sync_local_history_with_s3(
    client: &Client,
    bucket_name: &str,
    local_path: &str,
) -> io::Result<()> {
    clear_bucket(client, bucket_name).await.unwrap();

    let files = traverse_directory(Some(Path::new(local_path)), None);

    for file in files {
        let body = ByteStream::from_path(&file).await.unwrap();
        let path = file.strip_prefix(".history/").unwrap();

        client
            .put_object()
            .bucket(bucket_name)
            .key(".history/".to_owned() + path.to_str().unwrap())
            .body(body)
            .send()
            .await
            .unwrap();
    }

    Ok(())
}

pub async fn create_file_from_s3object(
    client: &Client,
    destination: &str,
    bucket_name: &str,
    key: &str,
) -> io::Result<()> {
    fs::create_dir_all(Path::new(destination).parent().unwrap())?;

    let mut file = File::create(destination)?;

    let mut object = get_object(client, bucket_name, key).await?;

    while let Some(bytes) = object.body.try_next().await? {
        file.write_all(&bytes)?;
    }

    Ok(())
}

pub async fn get_object(
    client: &Client,
    bucket_name: &str,
    key: &str,
) -> Result<GetObjectOutput, io::Error> {
    client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to get object: {}", e),
            )
        })
}
