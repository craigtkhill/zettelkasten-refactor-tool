// src/models/frontmatter.rs
use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct Frontmatter {
    pub tags: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmatter_deserialize() {
        let yaml = "
            tags:
              - tag1
              - tag2
        ";
        let frontmatter: Frontmatter = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(frontmatter.tags.unwrap(), vec!["tag1", "tag2"]);
    }

    #[test]
    fn test_frontmatter_no_tags() {
        let yaml = "{}";
        let frontmatter: Frontmatter = serde_yaml_ng::from_str(yaml).unwrap();
        assert!(frontmatter.tags.is_none());
    }
}
