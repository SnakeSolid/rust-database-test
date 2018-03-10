use std::path::Path;

pub fn is_port(val: String) -> Result<(), String> {
    match val.parse::<u16>() {
        Ok(..) => Ok(()),
        Err(..) => Err(format!(
            "Port number must be in range 0 .. 65535, but {} given",
            val
        )),
    }
}

pub fn is_file(val: String) -> Result<(), String> {
    let path = Path::new(&val);

    if path.is_file() {
        Ok(())
    } else {
        Err(format!(
            "Test suite must exists and be a file, but {} given",
            val
        ))
    }
}
