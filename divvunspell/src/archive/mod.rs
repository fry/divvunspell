use memmap::Mmap;
use std::sync::Arc;

pub mod boxf;
pub mod error;
pub mod meta;
pub mod zip;

pub use self::boxf::BoxSpellerArchive;
pub use self::zip::ZipSpellerArchive;

pub(crate) struct TempMmap {
    mmap: Arc<Mmap>,

    // Not really dead, needed to drop when TempMmap drops
    _tempdir: tempdir::TempDir,
}

pub(crate) enum MmapRef {
    Direct(Arc<Mmap>),
    Temp(TempMmap),
}

impl MmapRef {
    pub fn map(&self) -> Arc<Mmap> {
        match self {
            MmapRef::Direct(mmap) => Arc::clone(mmap),
            MmapRef::Temp(tmmap) => Arc::clone(&tmmap.mmap),
        }
    }
}
