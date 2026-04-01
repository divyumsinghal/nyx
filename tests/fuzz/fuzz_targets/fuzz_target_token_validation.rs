#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(token) = std::str::from_utf8(data) {
        let validation = jsonwebtoken::Validation::default();
        let _ = jsonwebtoken::decode_header(token);
        let _ = validation;
    }
});
