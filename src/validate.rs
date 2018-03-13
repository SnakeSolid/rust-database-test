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

pub fn is_n_workers(val: String) -> Result<(), String> {
    match val.parse::<u8>() {
        Ok(n) if n > 0 && n <= 100 => Ok(()),
        Ok(..) | Err(..) => Err(format!(
            "Number of workers must be in range 1 .. 100, but {} given",
            val
        )),
    }
}

pub fn is_exists(val: String) -> Result<(), String> {
    let path = Path::new(&val);

    if path.exists() {
        Ok(())
    } else {
        Err(format!("Test suite {} must exists", val))
    }
}
