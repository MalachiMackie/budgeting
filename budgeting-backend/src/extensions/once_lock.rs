use std::sync::OnceLock;

use uuid::Uuid;

#[allow(clippy::module_name_repetitions)]
pub trait OnceLockExt {
    fn init_uuid(&self) -> &Uuid;
}

impl OnceLockExt for OnceLock<Uuid> {
    fn init_uuid(&self) -> &Uuid {
        self.get_or_init(Uuid::new_v4)
    }
}
