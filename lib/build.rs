fn main() {
    std::fs::create_dir_all("src/osc").unwrap();
    std::fs::write("src/osc/sine.bin", sin_lut()).unwrap();
}

fn sin_lut() -> Vec<u8> {
    // compute half of the wave to save on space
    let mut sine_lut = Vec::with_capacity(64);
    for i in 0..64 {
        let value = i as f64 / 128.0 * std::f64::consts::PI;
        let value = value.sin();
        // scale the value to the range of an i8
        let value = value * 127.0;
        let value = value.round() as i8 as u8;
        sine_lut.push(value);
    }
    sine_lut
}
