use std::env;

use miden_client::{Felt, ZERO};

pub fn prepare_felt_vec(element: u64) -> [Felt; 4] {
    [Felt::new(element), ZERO, ZERO, ZERO]
}

pub async fn delete_keystore_and_store() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let store_path = current_dir.join("store.sqlite3");
    let keystore_dir = current_dir.join("keystore");

    if tokio::fs::metadata(&store_path).await.is_ok() {
        if let Err(e) = tokio::fs::remove_file(&store_path).await {
            eprintln!("failed to remove {}: {}", store_path.display(), e);
        } else {
            println!("cleared sqlite store: {}", store_path.display());
        }
    } else {
        println!("store not found: {}", store_path.display());
    }

    match tokio::fs::read_dir(&keystore_dir).await {
        Ok(mut dir) => {
            while let Ok(Some(entry)) = dir.next_entry().await {
                let file_path = entry.path();
                if let Err(e) = tokio::fs::remove_file(&file_path).await {
                    eprintln!("failed to remove {}: {}", file_path.display(), e);
                } else {
                    println!("removed file: {}", file_path.display());
                }
            }
        }
        Err(e) => {
            println!(
                "keystore directory not found or empty: {} ({})",
                keystore_dir.display(),
                e
            );
        }
    }
}
