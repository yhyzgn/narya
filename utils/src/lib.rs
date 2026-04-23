use once_cell::sync::Lazy;
use tokio::runtime::{Runtime, Builder};

pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create global Tokio runtime")
});
