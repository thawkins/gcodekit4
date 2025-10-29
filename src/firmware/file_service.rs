//! File service interface
//!
//! Provides a trait for controllers with file system support (like FluidNC and Smoothieware).

/// File information
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// Whether this is a directory
    pub is_directory: bool,
    /// File modification time (Unix timestamp)
    pub modified: Option<u64>,
}

/// File service progress callback
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// Trait for file system operations on controllers
pub trait FileServiceTrait: Send + Sync {
    /// List files in a directory
    fn list_files(&self, path: &str) -> anyhow::Result<Vec<FileInfo>>;

    /// Upload a file to the controller
    fn upload_file(
        &self,
        local_path: &str,
        remote_path: &str,
        callback: Option<ProgressCallback>,
    ) -> anyhow::Result<()>;

    /// Download a file from the controller
    fn download_file(
        &self,
        remote_path: &str,
        local_path: &str,
        callback: Option<ProgressCallback>,
    ) -> anyhow::Result<()>;

    /// Delete a file or directory
    fn delete_file(&self, path: &str) -> anyhow::Result<()>;

    /// Create a directory
    fn create_directory(&self, path: &str) -> anyhow::Result<()>;

    /// Rename a file or directory
    fn rename(&self, old_path: &str, new_path: &str) -> anyhow::Result<()>;

    /// Get available storage space
    fn get_storage_info(&self) -> anyhow::Result<StorageInfo>;
}

/// Storage information
#[derive(Debug, Clone)]
pub struct StorageInfo {
    /// Total storage capacity in bytes
    pub total_size: u64,
    /// Used storage in bytes
    pub used_size: u64,
    /// Available storage in bytes
    pub available_size: u64,
}

impl StorageInfo {
    /// Get percentage of storage used
    pub fn usage_percent(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.used_size as f64 / self.total_size as f64) * 100.0
        }
    }
}

/// Default no-op implementation of file service
#[derive(Debug, Clone)]
pub struct NoOpFileService;

impl FileServiceTrait for NoOpFileService {
    fn list_files(&self, _path: &str) -> anyhow::Result<Vec<FileInfo>> {
        Ok(Vec::new())
    }

    fn upload_file(
        &self,
        _local_path: &str,
        _remote_path: &str,
        _callback: Option<ProgressCallback>,
    ) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("File service not supported"))
    }

    fn download_file(
        &self,
        _remote_path: &str,
        _local_path: &str,
        _callback: Option<ProgressCallback>,
    ) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("File service not supported"))
    }

    fn delete_file(&self, _path: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("File service not supported"))
    }

    fn create_directory(&self, _path: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("File service not supported"))
    }

    fn rename(&self, _old_path: &str, _new_path: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("File service not supported"))
    }

    fn get_storage_info(&self) -> anyhow::Result<StorageInfo> {
        Err(anyhow::anyhow!("File service not supported"))
    }
}
