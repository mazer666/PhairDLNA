//! Helper functions for catalogue ordering and volume tracking.

use std::path::PathBuf;

use crate::catalogue::MediaKind;
use crate::platform::VolumeId;

use super::super::volumes::VolumeRegistry;
use super::super::ContentItem;

/// Orders the children of a container: the folders first, then the files.
///
/// A playlist is a container like a folder and is listed with them. The
/// children of a playlist are not sorted: they keep the order of the playlist
/// (`FUN-015`).
pub fn sort_children(children: &mut [ContentItem]) {
    children.sort_by(|left, right| {
        let rank = |item: &ContentItem| match item.kind {
            MediaKind::Folder | MediaKind::Playlist => 0,
            _ => 1,
        };
        rank(left)
            .cmp(&rank(right))
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
            .then_with(|| left.id.cmp(&right.id))
    });
}

/// The paths of the roots that live on a volume.
///
/// A platform that reports volumes knows which root is on which volume; the
/// portable adapter does not, and then every root shares the one identity
/// (`PLT-005`, `PLT-011`).
pub fn roots_of_volume(registry: &VolumeRegistry, volume: &VolumeId) -> Vec<PathBuf> {
    registry
        .roots()
        .iter()
        .filter(|root| &root.identity == volume)
        .map(|root| root.path.clone())
        .collect()
}
