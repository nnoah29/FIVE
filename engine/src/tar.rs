//! Lecture des en-têtes d'une archive tar (éventuellement compressée), sans rien extraire.

use std::io::{self, Read};

use crate::tar_header::{entry_name, next_header, octal, take_data, text};
use crate::zip::Member;

/// Entrées d'un flux tar (au plus `limit`) et indicateur d'entrées en plus.
pub fn tar_entries(mut reader: impl Read, limit: usize) -> io::Result<(Vec<Member>, bool)> {
    let (mut out, mut long_name) = (Vec::new(), None::<String>);
    while let Some(header) = next_header(&mut reader, out.is_empty())? {
        let (size, kind) = (octal(&header[124..136]), header[156]);
        if matches!(kind, b'L' | b'x' | b'g') {
            let data = take_data(&mut reader, size, true)?;
            long_name = (kind == b'L').then(|| text(&data));
            continue;
        }
        if out.len() >= limit {
            return Ok((out, true));
        }
        out.push((entry_name(&header, long_name.take()), size, kind == b'5'));
        take_data(&mut reader, size, false)?;
    }
    Ok((out, false))
}
