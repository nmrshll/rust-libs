use std::convert::Infallible;
use std::fs;
use std::future::Future;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

lazy_static::lazy_static! {
  pub static ref GIT_WORK_DIR: Result<PathBuf, String> = CacheInRepo::work_dir();
}
pub mod prelude {
    pub use crate::{FileBytes, FromFileOrNew};
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
        // ensure parent directory exists
        let parent_dir = path.parent().ok_or(anyhow::Error::msg("No parent dir"))?;
        fs::create_dir_all(parent_dir)?;

        fs::write(path, self.as_file_bytes()?)?;
        Ok(())
    }
}

/// Trait for auto-implementing FileBytes using JSON serialization
/// usage: impl JsonFileBytes for MyType {}
pub trait JsonFileBytes: Sized + serde::ser::Serialize + serde::de::DeserializeOwned {
    fn as_file_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec_pretty(self)?)
    }
    fn from_file_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
impl<T> FileBytes for T
where
    T: JsonFileBytes,
{
    fn as_file_bytes(&self) -> anyhow::Result<Vec<u8>> {
        <Self as JsonFileBytes>::as_file_bytes(self)
    }
    fn from_file_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        <Self as JsonFileBytes>::from_file_bytes(bytes)
    }
}

pub trait FromFileOrNew<CacheDir>: FileBytes
where
    CacheDir: CacheLocation,
{
    fn from_file_or_save_new<Fut, E>(
        file_id: &str,
        make_new: Fut,
    ) -> impl Future<Output = anyhow::Result<Self>>
    where
        Fut: std::future::Future<Output = Result<Self, E>> + Send,
        anyhow::Error: From<E>,
    {
        async {
            let file_path = CacheDir::file_path(file_id)?;

            // if file, load from file. else generate new and save to file
            if Path::new(&file_path).exists() {
                Self::from_file(&file_path)
            } else {
                let new = make_new.await.map_err(anyhow::Error::from)?;
                fs::write(file_path, new.as_file_bytes()?).expect("Unable to write file");
                Ok(new)
            }
        }
    }

    // // this is separated into a function to avoid unclonable reference to lazy_static inside an async fn
    // fn file_path(file_id: &str) -> anyhow::Result<PathBuf> {
    //     let cache_dir = CACHE_DIR.as_ref().map_err(|e| anyhow!(e))?;
    //     Ok(cache_dir.join(file_id))
    // }
}
// auto-implement FromFileOrNew for all FileBytes types
impl<T: FileBytes> FromFileOrNew<CacheInRepo> for T {}

pub trait CachedOrDefault: FromFileOrNew<CacheInRepo> + Default {
    fn cached_or_default(file_id: &str) -> impl Future<Output = anyhow::Result<Self>> {
        Self::from_file_or_save_new::<_, Infallible>(file_id, async { Ok(Self::default()) })
    }
}
impl<T: FromFileOrNew<CacheInRepo> + Default> CachedOrDefault for T {} // auto-implement for all possible types

pub trait CacheLocation {
    fn cache_dir() -> anyhow::Result<PathBuf>;
    fn file_path(path_relative: &str) -> anyhow::Result<PathBuf> {
        let cache_dir = Self::cache_dir()?;
        Ok(cache_dir.join(path_relative))
    }
}
pub struct CacheInRepo {}
impl CacheInRepo {
    pub fn work_dir() -> Result<PathBuf, String> {
        let wd_bytes_utf8 = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| e.to_string())?
            .stdout;
        let path_string = String::from_utf8_lossy(&wd_bytes_utf8).to_string();
        Ok(PathBuf::from(&path_string.trim()))
    }
}
impl CacheLocation for CacheInRepo {
    fn cache_dir() -> anyhow::Result<PathBuf> {
        let work_dir = GIT_WORK_DIR.as_ref().map_err(anyhow::Error::msg)?;
        let cache_dir = work_dir.join(".cache");
        Ok(cache_dir)
    }
}

pub struct RepoOrXdg {}
impl CacheLocation for RepoOrXdg {
    fn cache_dir() -> anyhow::Result<PathBuf> {
        use anyhow::anyhow as err;
        if let Ok(git_repo_dir) = GIT_WORK_DIR.as_ref() {
            let cache_dir = git_repo_dir.join(".cache");
            return Ok(cache_dir);
        }
        if let Ok(xdg_cache_home) = std::env::var("XDG_CACHE_HOME") {
            let cache_dir = PathBuf::from(xdg_cache_home).join("accounting");
            return Ok(cache_dir);
        }
        if let Ok(home_dir) = std::env::var("HOME") {
            let cache_dir = PathBuf::from(home_dir).join(".cache").join("accounting");
            return Ok(cache_dir);
        }
        Err(err!("No suitable cache dir found"))
    }
}
impl RepoOrXdg {
    pub fn work_dir() -> Result<PathBuf, String> {
        let wd_bytes_utf8 = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .map_err(|e| e.to_string())?
            .stdout;
        let path_string = String::from_utf8_lossy(&wd_bytes_utf8).to_string();
        Ok(PathBuf::from(&path_string.trim()))
    }
}

pub trait Cacheable: FileBytes {
    // if Path doesn't depend on &self, only override this one
    fn static_relative_path_str() -> &'static str {
        std::any::type_name::<Self>()
    }
    fn static_relative_path() -> &'static Path {
        &Path::new(Self::static_relative_path_str())
    }
    // if Path depends on &self, override this one (only)
    fn relative_path_str(&self) -> String {
        Self::static_relative_path().to_string_lossy().to_string()
    }
    fn relative_path(&self) -> PathBuf {
        PathBuf::from(self.relative_path_str())
    }
    fn is_expired(&self) -> bool {
        false
    }

    fn to_cache(&self) -> anyhow::Result<PathBuf> {
        let file_path = RepoOrXdg::file_path(&self.relative_path_str())?;
        self.to_file(&file_path)?;
        Ok(file_path)
    }
    fn from_cache(file_path: &Path) -> anyhow::Result<Self> {
        let loaded = Self::from_file(&file_path)?;
        if loaded.is_expired() {
            fs::remove_file(file_path).map_err(|e| anyhow::Error::new(e))?;
            return Err(anyhow::Error::msg("Cache expired"));
        }
        return Ok(loaded);
    }
}

pub mod implementations {
    use super::*;

    // impl FileBytes for cardano_serialization_lib::crypto::PrivateKey {
    //     fn as_file_bytes(&self) -> anyhow::Result<Vec<u8>> {
    //         Ok(self.to_bech32().as_bytes().to_vec())
    //     }
    //     fn from_file_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
    //         Ok(Self::from_bech32(&String::from_utf8(
    //             bytes.to_vec(),
    //         )?)?)
    //     }
    // }

    impl FileBytes for () {
        // if we need to avoid repeating a step, but that step doesn't have an output,
        // this file is simply a marker that the step has been done
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
            counter.to_file(&CacheInRepo::file_path(file_id)?)?;
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
