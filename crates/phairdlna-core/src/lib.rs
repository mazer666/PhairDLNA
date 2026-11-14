//! PhairDLNA domain foundation.
//! No SSDP, UPnP, indexing or streaming implementation exists yet.

/// Stable project name for status output.
pub const PROJECT_NAME: &str = "PhairDLNA";

#[cfg(test)]
mod tests {
    use super::PROJECT_NAME;

    #[test]
    fn project_name_is_stable() {
        assert_eq!(PROJECT_NAME, "PhairDLNA");
    }
}
