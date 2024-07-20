use anyhow::anyhow;
// use cardano_serialization_lib::crypto::PrivateKey;
use std::fs;
use std::future::Future;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

lazy_static::lazy_static! {
  pub static ref WORK_DIR: Result<PathBuf, String> = work_dir();
  pub static ref CACHE_DIR: Result<PathBuf, String> = cache_dir();
}

pub trait FileBytes: Sized {
    fn as_file_bytes(&self) -> anyhow::Result<Vec<u8>>;
    fn from_file_bytes(bytes: &[u8]) -> anyhow::Result<Self>;

    fn from_file(path: &Path) -> anyhow::Result<Self> {
        let mut file = fs::File::open(path)?;
        let mut read_data = Vec::new();
        file.read_to_end(&mut read_data)?;
        Self::from_file_bytes(&read_data)
    }
    fn to_file(&self, path: &Path) -> anyhow::Result<()> {
        fs::write(path, self.as_file_bytes()?)?;
        Ok(())
    }
}

pub trait FromFileOrNew: FileBytes {
    fn from_file_or_save_new<Fut>(
        file_id: &str,
        make_new: Fut,
    ) -> impl Future<Output = anyhow::Result<Self>>
    where
        Fut: std::future::Future<Output = anyhow::Result<Self>> + Send;

    // this is separated into a function to avoid unclonable reference to lazy_static inside an async fn
    fn file_path(file_id: &str) -> anyhow::Result<PathBuf> {
        let cache_dir = CACHE_DIR.as_ref().map_err(|e| anyhow!(e))?;
        Ok(cache_dir.join(file_id))
    }
}
impl<T: FileBytes> FromFileOrNew for T {
    async fn from_file_or_save_new<Fut>(file_id: &str, make_new: Fut) -> anyhow::Result<Self>
    where
        Fut: std::future::Future<Output = anyhow::Result<Self>> + Send,
    {
        let file_path = Self::file_path(file_id)?;

        // if file, load from file. else generate new and save to file
        if Path::new(&file_path).exists() {
            Self::from_file(&file_path)
        } else {
            let new = make_new.await?;
            fs::write(file_path, new.as_file_bytes()?).expect("Unable to write file");
            Ok(new)
        }
    }
}

pub trait CachedOrDefault: FromFileOrNew + Default {
    fn cached_or_default(file_id: &str) -> impl Future<Output = anyhow::Result<Self>> {
        Self::from_file_or_save_new(file_id, async { Ok(Self::default()) })
    }
}
impl<T: FromFileOrNew + Default> CachedOrDefault for T {} // auto-implement for all possible types

pub fn work_dir() -> Result<PathBuf, String> {
    let wd_bytes_utf8 = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| e.to_string())?
        .stdout;
    let path_string = String::from_utf8_lossy(&wd_bytes_utf8).to_string();
    Ok(PathBuf::from(&path_string.trim()))
}
pub fn cache_dir() -> Result<PathBuf, String> {
    let cache_dir: PathBuf = WORK_DIR.as_ref()?.join(".cache/test_data");
    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
    }
    Ok(cache_dir)
}

pub mod implementations {
    use super::*;

    // impl FileBytes for PrivateKey {
    //     fn as_file_bytes(&self) -> anyhow::Result<Vec<u8>> {
    //         Ok(self.to_bech32().as_bytes().to_vec())
    //     }
    //     fn from_file_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
    //         Ok(PrivateKey::from_bech32(&String::from_utf8(
    //             bytes.to_vec(),
    //         )?)?)
    //     }
    // }

    impl FileBytes for () {
        // this impl is for when we want to avoid redoing steps that don't have an output, like mint_and_distribute
        // the file is simply a marker that the step has been done
        fn as_file_bytes(&self) -> anyhow::Result<Vec<u8>> {
            Ok(b"ok".to_vec())
        }
        fn from_file_bytes(_: &[u8]) -> anyhow::Result<Self> {
            Ok(())
        }
    }
}

pub mod cache_counter {
    use super::*;

    #[derive(Default, Debug)]
    pub struct CacheCounter(pub usize);
    impl FileBytes for CacheCounter {
        fn as_file_bytes(&self) -> anyhow::Result<Vec<u8>> {
            Ok(self.0.to_string().as_bytes().to_vec())
        }
        fn from_file_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
            Ok(CacheCounter(String::from_utf8(bytes.to_vec())?.parse()?))
        }
    }
    impl CacheCounter {
        pub async fn next(file_id: &str) -> anyhow::Result<Self> {
            let mut counter = CacheCounter::cached_or_default(file_id).await?;
            counter.0 += 1;
            counter.to_file(&Self::file_path(file_id)?)?;
            Ok(counter)
        }
    }
    impl std::fmt::Display for CacheCounter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::cache_counter::CacheCounter;
    use test_utils::TestResult;

    #[tokio::test]
    async fn test_counter() -> TestResult {
        let counter = CacheCounter::next("test_counter").await?;
        dbg!(&counter);
        Ok(())
    }
}
