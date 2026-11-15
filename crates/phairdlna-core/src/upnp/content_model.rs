//! The catalogue as the protocol layer sees it.
//!
//! What: the items and the containers of the catalogue, and the narrow trait
//!       the protocol layer reads them through.
//! Why: the catalogue module (F-08, F-09) owns the filesystem, the indexing
//!       and the watching of the folders. The protocol layer must not reach
//!       the filesystem, so it sees the catalogue only through this trait — a
//!       door that is narrow on purpose.
//! Example: `ContentSource::container` returns the container a browse asks
//!       for, or `None` when the server does not know it.

/// One item of the catalogue, as the client sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentItem {
    /// The identifier of the item.
    pub id: String,
    /// The identifier of the container the item lives in.
    pub parent_id: String,
    /// The name the client shows.
    pub title: String,
    /// The UPnP class of the item.
    pub class: String,
}

impl ContentItem {
    /// Whether the item matches a search expression.
    ///
    /// Only `dc:title` is searchable today; the comparison ignores the case,
    /// because a client does not know how the title was written down.
    pub fn matches(&self, property: &str, text: &str) -> bool {
        match property {
            "dc:title" => self.title.to_lowercase().contains(&text.to_lowercase()),
            _ => false,
        }
    }

    /// Builds an item.
    pub fn new(
        id: impl Into<String>,
        parent_id: impl Into<String>,
        title: impl Into<String>,
        class: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            parent_id: parent_id.into(),
            title: title.into(),
            class: class.into(),
        }
    }
}

/// A container of the catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    /// The identifier of the container.
    pub id: String,
    /// The identifier of the container this one lives in.
    pub parent_id: String,
    /// The name the client shows.
    pub title: String,
    /// The UPnP class of the container.
    pub class: String,
    /// The identifier that changes when the content of the container changes.
    pub update_id: u32,
    /// The children of the container, in the order the catalogue returns them.
    pub children: Vec<ContentItem>,
    /// The total number of children, or `None` when it cannot be computed.
    pub total: Option<u32>,
}

/// The read-only view of the catalogue the ContentDirectory service needs.
///
/// The catalogue module owns the implementation (F-08, F-09); this trait is
/// the narrow door it opens for the protocol layer, so that the protocol
/// layer cannot reach the filesystem.
pub trait ContentSource {
    /// The identifier that changes when the catalogue changes.
    fn system_update_id(&self) -> u32;

    /// The container with the given identifier, or `None` when there is none.
    fn container(&self, object_id: &str) -> Option<Container>;

    /// The items of a container whose property contains the given text.
    ///
    /// The catalogue owns the search (F-09); until it exists, the search runs
    /// over the children the container returned.
    fn search(&self, container_id: &str, property: &str, text: &str) -> Vec<ContentItem> {
        match self.container(container_id) {
            Some(container) => container
                .children
                .into_iter()
                .filter(|item| item.matches(property, text))
                .collect(),
            None => Vec::new(),
        }
    }
}
