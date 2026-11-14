//! Écriture d'une archive zip : fichiers compressés en deflate, répertoire central écrit à la fin.

use std::fs::{File, Metadata, OpenOptions};
use std::io::{self, ErrorKind, Seek, SeekFrom, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::jobs::job::Job;
use crate::jobs::state::Stop;
use crate::zip_header::{ZipEntry, central_header, dos_stamp, end_record, local_header, sizes_field};
use crate::zip_stream::{deflate_stream, small};

const MAX_ENTRIES: usize = 65_535;

/// Archive zip en cours d'écriture.
pub struct ZipOut {
    file: File,
    entries: Vec<ZipEntry>,
}

impl ZipOut {
    /// Crée l'archive vide à `path`, qui ne doit pas exister.
    pub fn create(path: &Path) -> io::Result<ZipOut> {
        Ok(ZipOut { file: OpenOptions::new().write(true).create_new(true).open(path)?, entries: Vec::new() })
    }

    /// Écrit l'en-tête local d'une entrée et la retient pour le répertoire central.
    fn begin(&mut self, name: &str, method: u16, meta: &Metadata, kind: u32) -> io::Result<ZipEntry> {
        if self.entries.len() >= MAX_ENTRIES {
            return Err(ErrorKind::FileTooLarge.into());
        }
        let seconds = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map_or(0.0, |d| d.as_secs_f64());
        let offset = small(self.file.stream_position()?)?;
        let mode = kind | (meta.permissions().mode() & 0o777);
        let entry = ZipEntry { name: name.to_string(), method, crc: 0, csize: 0, size: 0, offset, stamp: dos_stamp(seconds), mode };
        self.file.write_all(&local_header(&entry))?;
        Ok(entry)
    }

    /// Ajoute un dossier (nom terminé par `/`).
    pub fn add_dir(&mut self, name: &str, meta: &Metadata) -> io::Result<()> {
        let entry = self.begin(name, 0, meta, 0o040000)?;
        self.entries.push(entry);
        Ok(())
    }

    /// Ajoute un fichier compressé ; les tailles et la somme de contrôle sont reportées dans son en-tête local.
    pub fn add_file(&mut self, job: &Job, name: &str, src: &Path) -> Result<(), Stop> {
        let mut reader = File::open(src)?;
        let meta = reader.metadata()?;
        small(meta.len())?;
        let mut entry = self.begin(name, 8, &meta, 0o100000)?;
        job.update(|p| p.current = name.rsplit('/').next().unwrap_or(name).to_string());
        let start = self.file.stream_position()?;
        let (crc, size) = deflate_stream(job, &mut reader, &mut self.file)?;
        let end = self.file.stream_position()?;
        (entry.crc, entry.size, entry.csize) = (crc, small(size)?, small(end - start)?);
        self.file.seek(SeekFrom::Start(entry.offset as u64 + 14))?;
        self.file.write_all(&sizes_field(&entry))?;
        self.file.seek(SeekFrom::Start(end))?;
        self.entries.push(entry);
        Ok(())
    }

    /// Écrit le répertoire central et la fin d'archive.
    pub fn finish(mut self) -> io::Result<()> {
        let start = small(self.file.stream_position()?)?;
        let directory: Vec<u8> = self.entries.iter().flat_map(central_header).collect();
        self.file.write_all(&directory)?;
        self.file.write_all(&end_record(self.entries.len() as u16, small(directory.len() as u64)?, start))?;
        self.file.flush()
    }
}
