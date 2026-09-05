use gpui::*;
use std::borrow::Cow;

pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> anyhow::Result<Option<Cow<'static, [u8]>>> {
        // Liora icons use virtual `liora-icon://` paths. Resolve those first;
        // ordinary application assets continue to come from the filesystem.
        if let Some(bytes) = liora::icons::IconAssetSource.load(path)? {
            return Ok(Some(bytes));
        }

        match std::fs::read(path) {
            Ok(file) => Ok(Some(Cow::Owned(file))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn list(&self, _path: &str) -> anyhow::Result<Vec<SharedString>> {
        Ok(vec![])
    }
}
