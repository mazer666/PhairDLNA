//! The catalogue commands of the command line.
//!
//! What: the two commands that show the owner what the server would publish —
//!       the tree of the library and the document a client would receive.
//! Why: the catalogue is the part of the server the owner can get wrong without
//!       noticing. A command that prints the tree with the identifiers turns
//!       that from a guess into something a person can look at (`FUN-004`,
//!       `INT-062`).
//! Example: `phairdlna catalogue scan` prints every folder and file of the
//!       library with the identifier a client would use.

use std::path::Path;

use phairdlna_core::catalogue::{Catalogue, ContentItem, ContentSource};
use phairdlna_core::config::Config;
use phairdlna_core::didl::{folder_from_catalogue, item_from_catalogue, render, DidlDocument};
use phairdlna_core::platform::LocalFileSystem;

use crate::{Outcome, BASE_URL};

/// Builds the catalogue of the configured roots.
///
/// The configuration is loaded first, so that a broken configuration is
/// reported as such and not as an empty library.
fn catalogue_of(config: &Config) -> Catalogue {
    Catalogue::build(
        &config.library.roots,
        config.server.symlink_policy,
        &LocalFileSystem,
    )
}

/// Prints the tree of the library.
pub fn scan(path: &Path) -> Outcome {
    let config = match Config::load(path) {
        Ok(config) => config,
        Err(error) => {
            return Outcome::failure(format!(
                "the configuration \"{}\" is not usable:\n{}\n",
                path.display(),
                error.message()
            ))
        }
    };
    let catalogue = catalogue_of(&config);
    let mut report = String::new();
    report.push_str(&roots_report(&catalogue));
    report.push_str("\n=== the tree ===\n\n");
    report.push_str(&tree_of(&catalogue, "1", 0));
    if !catalogue.diagnostics().is_empty() {
        report.push_str("\n=== what could not be read ===\n\n");
        for diagnostic in catalogue.diagnostics() {
            report.push_str(&format!("- {diagnostic}\n"));
        }
    }
    Outcome::ok(report)
}

/// Prints the document a client would receive for a container.
pub fn browse(path: &Path, object_id: &str) -> Outcome {
    let config = match Config::load(path) {
        Ok(config) => config,
        Err(error) => {
            return Outcome::failure(format!(
                "the configuration \"{}\" is not usable:\n{}\n",
                path.display(),
                error.message()
            ))
        }
    };
    let catalogue = catalogue_of(&config);
    let policy = phairdlna_core::didl::ProtocolPolicy::plain();
    let Some(container) = catalogue.container(object_id) else {
        return browse_item(&catalogue, object_id, &policy);
    };
    let (folders, files): (Vec<&ContentItem>, Vec<&ContentItem>) = container
        .children
        .iter()
        .partition(|item| item.kind == phairdlna_core::catalogue::MediaKind::Folder);
    let document = DidlDocument {
        containers: folders.into_iter().map(folder_from_catalogue).collect(),
        items: files
            .into_iter()
            .map(|item| item_from_catalogue(item, BASE_URL, &policy))
            .collect(),
    };
    let mut report = String::new();
    report.push_str(&format!(
        "=== the container \"{}\" ===\n\ntitle:     {}\nchildren:  {}\n\n=== the document a client receives ===\n\n",
        container.id, container.title, container.children.len()
    ));
    report.push_str(&render(&document, None));
    report.push('\n');
    Outcome::ok(report)
}

/// Prints the document of an item, when the identifier names an item.
///
/// A client that listed a container asks for the metadata of one of the items
/// it found; the catalogue answers for it (`INT-033`).
fn browse_item(
    catalogue: &Catalogue,
    object_id: &str,
    policy: &phairdlna_core::didl::ProtocolPolicy,
) -> Outcome {
    let Some(item) = catalogue.item_of(object_id) else {
        return Outcome::failure(format!(
            "the object \"{object_id}\" is not in the catalogue\n"
        ));
    };
    let document = DidlDocument::with_items(vec![item_from_catalogue(&item, BASE_URL, policy)]);
    let mut report = String::new();
    report.push_str(&format!(
        "=== the item \"{}\" ===\n\ntitle:     {}\n\n=== the document a client receives ===\n\n",
        item.id, item.title
    ));
    report.push_str(&render(&document, None));
    report.push('\n');
    Outcome::ok(report)
}

/// The report of the roots of the library.
fn roots_report(catalogue: &Catalogue) -> String {
    let mut report = String::new();
    report.push_str("=== the roots ===\n\n");
    if catalogue.roots().is_empty() {
        report.push_str("no root is configured\n");
        return report;
    }
    for root in catalogue.roots() {
        report.push_str(&format!(
            "- {}  {}\n  id:      {}\n  state:   {}\n  entries: {}\n",
            root.name,
            root.path.display(),
            root.id,
            if root.present { "present" } else { "absent" },
            root.entries
        ));
    }
    report
}

/// The tree of the catalogue, starting at a container.
fn tree_of(catalogue: &Catalogue, container_id: &str, depth: usize) -> String {
    let mut report = String::new();
    let Some(container) = catalogue.container(container_id) else {
        return report;
    };
    for (index, child) in container.children.iter().enumerate() {
        let last = index + 1 == container.children.len();
        report.push_str(&line(child, depth, last));
        if child.kind == phairdlna_core::catalogue::MediaKind::Folder {
            report.push_str(&tree_of(catalogue, &child.id, depth + 1));
        }
    }
    report
}

/// One line of the tree.
fn line(child: &ContentItem, depth: usize, last: bool) -> String {
    let branch = if last { "└─ " } else { "├─ " };
    let indent = "   ".repeat(depth);
    let kind = match child.kind {
        phairdlna_core::catalogue::MediaKind::Folder => "folder".to_owned(),
        other => format!("{other:?}").to_lowercase(),
    };
    let detail = match (&child.metadata.mime_type, child.metadata.size) {
        (Some(mime_type), Some(size)) => format!("  {mime_type}  {size} bytes"),
        (Some(mime_type), None) => format!("  {mime_type}"),
        (None, Some(size)) => format!("  {size} bytes"),
        (None, None) => String::new(),
    };
    format!(
        "{indent}{branch}{}  [{}]  {kind}{detail}\n",
        child.id, child.title
    )
}
