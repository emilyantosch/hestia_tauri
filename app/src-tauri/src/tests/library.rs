use crate::config::library::Library;

#[tokio::test]
async fn check_library_creation_successful() {
    let mut path = dirs::data_dir().unwrap().to_path_buf();
    path.push("hestia/test_lib/");
    let lib = Library::new(path)
        .await
        .expect("Failed to allocate new Library");
    let lib_config = lib.library_config;
    let lib_path = lib_config.lock().await;
    let lib_paths = &lib_path.as_ref().unwrap().library_paths;
    println!("{lib_paths:#?}");

    dbg!(&lib_paths);

    assert!(lib_paths.is_none());
}
