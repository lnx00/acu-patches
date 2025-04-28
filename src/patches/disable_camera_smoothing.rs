use std::sync::{Arc, OnceLock, RwLock};

use crate::utils;

pub struct DisableCameraSmoothing {
    target_address: usize,
    original_bytes: Box<[u8; 2]>,
}

static INSTANCE: OnceLock<Arc<RwLock<DisableCameraSmoothing>>> = OnceLock::new();

impl DisableCameraSmoothing {
    pub fn inst() -> Arc<RwLock<Self>> {
        INSTANCE
            .get_or_init(|| {
                let game_module = libmem::find_module("ACU.exe").unwrap();
                let target_address = unsafe {
                    libmem::sig_scan("74 ? 41 8B 06 41 89 85", game_module.base, game_module.size)
                        .ok_or("signature not found")
                        .unwrap()
                };

                let original_bytes = unsafe { libmem::read_memory::<_>(target_address) };

                Arc::new(RwLock::new(Self {
                    target_address,
                    original_bytes: Box::new(original_bytes),
                }))
            })
            .clone()
    }

    pub fn enable(&mut self) -> Result<(), String> {
        let patch_bytes: [u8; 2] = [0x90, 0x90];
        utils::patch_bytes(self.target_address, &patch_bytes)?;

        Ok(())
    }

    pub fn disable(&mut self) -> Result<(), String> {
        utils::patch_bytes(self.target_address, self.original_bytes.as_slice())?;

        Ok(())
    }
}
