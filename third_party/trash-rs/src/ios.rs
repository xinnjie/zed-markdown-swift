use std::path::PathBuf;

use crate::{Error, TrashContext, TrashItem};

#[derive(Clone, Default, Debug)]
pub struct PlatformTrashContext;

impl PlatformTrashContext {
    pub const fn new() -> Self {
        Self
    }
}

impl TrashContext {
    pub(crate) fn delete_all_canonicalized(
        &self,
        _full_paths: Vec<PathBuf>,
        _with_info: bool,
    ) -> Result<Option<Vec<TrashItem>>, Error> {
        Err(Error::Unknown {
            description: "iOS has no trash".into(),
        })
    }
}

pub fn restore_all<I>(_items: I) -> Result<(), Error>
where
    I: IntoIterator<Item = TrashItem>,
{
    Err(Error::Unknown {
        description: "iOS has no trash".into(),
    })
}
