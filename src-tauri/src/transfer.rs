// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use futures_util::TryStreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
};
use tokio_util::codec::{BytesCodec, FramedRead};

use read_progress_stream::ReadProgressStream;

use crate::proto::Ack;
use crate::proto::Result;
use std::{collections::HashMap, sync::Mutex};

#[derive(Clone, Serialize)]
struct ProgressPayload {
    id: u32,
    chunk: u64,
    progress: u64,
    total: u64,
}

pub async fn download(
    app: &AppHandle,
    id: u32,
    url: &str,
    file_path: &str,
    headers: Option<HashMap<String, String>>,
) -> Result<u32> {
    let client = reqwest::Client::new();

    let mut request = client.get(url);
    // Loop trought the headers keys and values
    // and add them to the request object.
    if let Some(headers) = headers {
        for (key, value) in headers {
            request = request.header(&key, value);
        }
    }

    let response = request.send().await?;
    let total = response.content_length().unwrap_or(0);

    let mut file = BufWriter::new(File::create(file_path).await?);
    let mut stream = response.bytes_stream();

    let mut progress = 0;
    while let Some(chunk) = stream.try_next().await? {
        file.write_all(&chunk).await?;
        let chunk_size = chunk.len() as u64;
        progress += chunk_size;
        let _ = app.emit(
            "download://progress",
            ProgressPayload {
                id,
                chunk: chunk_size,
                progress: progress,
                total,
            },
        );
    }
    file.flush().await?;

    Ok(id)
}

pub async fn upload(
    app: &AppHandle,
    id: u32,
    url: &str,
    file_path: &str,
    headers: Option<HashMap<String, String>>,
) -> Result<u32> {
    // Read the file
    let file = File::open(file_path).await?;
    let file_len = file.metadata().await.unwrap().len();

    // Create the request and attach the file to the body
    let client = reqwest::Client::new();
    let mut request = client
        .post(url)
        .header(reqwest::header::CONTENT_LENGTH, file_len)
        .body(file_to_body(id, app, file, file_len));

    // Loop trought the headers keys and values
    // and add them to the request object.
    if let Some(headers) = headers {
        for (key, value) in headers {
            request = request.header(&key, value);
        }
    }

    let ack = request.send().await?.json::<Ack<String>>().await?;
    if ack.code == 0 {
        Ok(id)
    } else {
        return Err(crate::proto::AppError::Message(ack.code, ack.msg));
    }
}

fn file_to_body(id: u32, app: &AppHandle, file: File, total: u64) -> reqwest::Body {
    let stream = FramedRead::new(file, BytesCodec::new()).map_ok(|r| r.freeze());
    let window = Mutex::new(app.clone());
    reqwest::Body::wrap_stream(ReadProgressStream::new(
        stream,
        Box::new(move |chunk_size, progress| {
            let _ = window.lock().unwrap().emit(
                "upload://progress",
                ProgressPayload {
                    id,
                    chunk: chunk_size,
                    progress,
                    total,
                },
            );
        }),
    ))
}
