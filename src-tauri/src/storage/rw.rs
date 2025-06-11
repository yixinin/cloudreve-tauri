// use std::{
//     path::PathBuf,
//     sync::{Arc, Mutex},
//     time,
// };

// use reqwest::Client;
// use serde::{Deserializer, Serialize};
// use tauri::{utils::mime_type::MimeType, AppHandle, Emitter};

// use crate::proto::{
//     self,
//     storage::{
//         BatchUrlsAck, BatchUrlsReq, CreateFileReq, DeleteFileReq, FileDetailsInfo, FileInfo,
//         FileProgress, GetCapacityAck, GetFileInfoReq, GetFilesAck, GetFilesReq, GetThumbURLAck,
//         MoveReq, RenameReq, UploadSessionAck, UploadSessionReq,
//     },
//     AuthReq,
// };

// use futures::{
//     stream::{self, StreamExt},
//     TryStreamExt,
// };

// use tokio::{fs::File, io::AsyncWriteExt, sync::oneshot::channel};
// use tokio::{io::AsyncReadExt, sync::mpsc::unbounded_channel};
// use tokio_util::codec::{BytesCodec, FramedRead};

// pub async fn upload_file(
//     app: &AppHandle,
//     token: &str,
//     file_path: PathBuf,
//     session_code: &str,
//     filename: &str,
//     chunk_size: u64,
// ) -> Result<bool, anyhow::Error> {
//     let client = Client::new();
//     let url = format!("{}/{}/0", proto::get_api_url("/file/upload"), session_code);

//     let file = File::open(file_path).await?;
//     let meta = file.metadata().await?;

//     // let stream = FramedRead::new(file, BytesCodec::new());
//     let file_size = meta.len() as usize;

//     let chunk_size = 1024 * 1024; // 1MB 块大小
//     let (tx, mut rx) = channel();

//     let stream = stream::try_unfold((file, 0), move |(mut file, mut offset)| async move {
//         if offset >= file_size {
//             return Ok(None);
//         }
//         if offset > file_size {
//             return Err(anyhow::format_err!("over sized"));
//         }
//         let mut buf = vec![0; chunk_size];
//         let n = file.read(&mut buf).await?;
//         offset += n;

//         Ok(Some((buf[..n].to_vec(), (file, offset))))
//     });

//     let progress_straem = stream.map_ok(|chunk| {
//         tx.send(chunk.len());
//         return chunk;
//     });
//     let body = reqwest::Body::wrap_stream(progress_straem);

//     tokio::spawn(async move {
//         let response = client
//             .post(&url)
//             .header("content-type", "octet-stream")
//             .header("content-length", file_size)
//             .body(body)
//             .send()
//             .await;
//         match response {
//             Ok(response) => {
//                 let status = response.status();

//                 match response.text().await {
//                     Ok(text) => {
//                         println!("upload file status code: {}, text: {}", status, &text);
//                         match serde_json::from_str::<proto::Ack<String>>(&text) {
//                             Ok(ack) => {
//                                 println!("parse upload file resp: {} {}", ack.code, ack.msg);
//                             }
//                             Err(err) => {
//                                 println!("parse upload file resp:{} err: {}", text, err);
//                             }
//                         }
//                     }
//                     Err(err) => {
//                         println!("read upload file resp err: {}", err);
//                     }
//                 }
//             }
//             Err(err) => {
//                 println!("upload file err: {}", err);
//             }
//         }
//     });

//     let app_clone = Arc::new(Mutex::new(app));
//     let filename = filename.to_string();
//     let mut uploaded = 0;
//     loop {
//         let filename = filename.clone();
//         match rx.blocking_recv() {
//             Ok(size) => {
//                 if size == 0 {
//                     return Ok(true);
//                 }
//                 uploaded += size;
//                 if let Err(err) = app_clone.lock().unwrap().emit(
//                     "upload",
//                     FileProgress {
//                         filename: filename,
//                         total: file_size,
//                         current: uploaded,
//                     },
//                 ) {
//                     println!("notify upload progress error: {}", err)
//                 }
//             }
//             Err(_) => {
//                 return Err(anyhow::format_err!("read progress end"));
//             }
//         }
//     }
// }

// pub async fn download_file(
//     app: &AppHandle,
//     file_path: &str,
//     filename: &str,
//     url: &str,
// ) -> Result<String, anyhow::Error> {
//     let client = Client::new();
//     let response = client.get(url).send().await?;
//     if response.status().is_success() {
//         let headers = response.headers();
//         let size = response.content_length().unwrap_or_default() as usize;
//         let name = match headers.get("content-disposition") {
//             Some(hv) => match extract_filename(hv.to_str()?) {
//                 Some(v) => v,
//                 None => filename.to_string(),
//             },
//             None => filename.to_string(),
//         };
//         let mut file = File::create(format!("{}{}", file_path, name)).await?;

//         let mut stream = response.bytes_stream();
//         let mut downloaded = 0;
//         while let Some(chunk) = stream.next().await {
//             let chunk = chunk?;
//             file.write_all(&chunk).await?;
//             downloaded += chunk.len();
//             if let Err(err) = app.emit(
//                 "download",
//                 FileProgress {
//                     filename: name.to_string(),
//                     total: size,
//                     current: downloaded,
//                 },
//             ) {
//                 println!("notify download progress error: {}", err)
//             }
//         }
//     }

//     Ok(format!(""))
// }

// fn extract_filename(content_disposition: &str) -> Option<String> {
//     // 匹配 filename="..." 或 filename*=utf-8''...
//     let re = regex::Regex::new(r#"filename\*?=([^;]+)"#).unwrap();
//     let captures = re.captures(content_disposition)?;

//     let filename = captures
//         .get(1)?
//         .as_str()
//         .trim_matches('"') // 去除引号
//         .trim_start_matches("utf-8''"); // 处理 filename* 格式

//     // 解码 URL 编码（如 %20 转空格）
//     urlencoding::decode(filename).ok().map(|s| s.into_owned())
// }
