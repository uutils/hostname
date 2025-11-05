use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use uucore::error::UResult;

pub(crate) fn parse_domain_name_file(path: &Path) -> UResult<Vec<u8>> {
    let mut file = std::fs::File::open(path).map(BufReader::new)?;

    let first_byte = loop {
        let mut first_byte = [0_u8; 1];
        if let Err(err) = file.read_exact(&mut first_byte)
            && err.kind() == std::io::ErrorKind::UnexpectedEof
        {
            return Ok(Vec::default()); // Empty name.
        }

        match first_byte[0] {
            b'\r' | b'\n' => {} // Empty line. Skip.

            b'#' => {
                file.skip_until(b'\n')?; // Comment line. Skip.
            }

            first_byte => break first_byte,
        }
    };

    let mut buffer = Vec::with_capacity(256);
    buffer.push(first_byte);
    file.read_until(b'\n', &mut buffer)?;
    while matches!(buffer.last().copied(), Some(b'\r' | b'\n')) {
        buffer.pop();
    }
    Ok(buffer)
}
