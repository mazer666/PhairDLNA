//! The placeholders of the features that do not exist yet.
//!
//! What: the catalogue that answers a browse until the real catalogue exists.
//! Why: the catalogue is the job of F-08 and F-09, and it does not exist yet.
//!       The protocol layer must answer a browse in the meantime, and it must
//!       answer it honestly: the server knows the two root containers of
//!       ContentDirectory:1 and knows nothing else. **This is a placeholder**,
//!       and the catalogue module replaces it; nothing here is final.
//! Example: `PlaceholderCatalogue` answers a browse of the container `1`
//!       with the content directory root.

use crate::catalogue::{ContentContainer, ContentSource};

/// The catalogue that exists until the catalogue module does.
///
/// It knows the two root containers of ContentDirectory:1 — the root `0` and
/// the content directory `1` — and neither of them has a child. A browse of
/// any other identifier is the fault `701 No Such Object`, because the server
/// really does not know it (`INT-039`).
#[derive(Debug, Default, Clone, Copy)]
pub struct PlaceholderCatalogue;

impl ContentSource for PlaceholderCatalogue {
    fn system_update_id(&self) -> u32 {
        0
    }

    fn container(&self, object_id: &str) -> Option<ContentContainer> {
        let (id, parent_id, title) = match object_id {
            "0" => ("0", "-1", "Root"),
            "1" => ("1", "0", "Content Directory"),
            _ => return None,
        };
        Some(ContentContainer {
            id: id.to_owned(),
            parent_id: parent_id.to_owned(),
            title: title.to_owned(),
            update_id: 0,
            children: Vec::new(),
            total: Some(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PlaceholderCatalogue;
    use crate::catalogue::ContentSource;

    /// The two root containers are known, and nothing else is.
    #[test]
    fn the_two_root_containers_are_known() {
        let catalogue = PlaceholderCatalogue;
        assert_eq!(
            catalogue.container("0").map(|c| c.title),
            Some("Root".to_owned())
        );
        assert_eq!(
            catalogue.container("1").map(|c| c.title),
            Some("Content Directory".to_owned())
        );
        assert!(catalogue.container("999").is_none());
    }

    /// The placeholder catalogue is empty, and it says so.
    #[test]
    fn the_placeholder_catalogue_is_empty() {
        let container = PlaceholderCatalogue
            .container("1")
            .expect("the container is known");
        assert!(container.children.is_empty());
        assert_eq!(container.total, Some(0));
    }
}
