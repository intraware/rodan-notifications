use crate::config::Config;
use arc_swap::ArcSwap;
use std::sync::{Arc, LazyLock};

static GLOBAL_CONFIG: LazyLock<ArcSwap<Config>> =
    LazyLock::new(|| ArcSwap::from_pointee(Config::default()));

pub fn set_config(cfg: Config) {
    GLOBAL_CONFIG.store(Arc::new(cfg));
}

pub fn get_config() -> Arc<Config> {
    GLOBAL_CONFIG.load_full()
}

