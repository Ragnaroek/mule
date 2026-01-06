pub struct FileUpload {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub async fn open_file() -> FileUpload {
    let file = rfd::AsyncFileDialog::new().pick_file().await.unwrap();
    let bytes = file.read().await;
    FileUpload {
        name: file.file_name(),
        bytes,
    }
}
