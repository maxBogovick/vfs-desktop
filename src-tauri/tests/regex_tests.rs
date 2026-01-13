#[cfg(test)]
mod tests {
    use vfdir_lib::core::FileSystemEntry;
    use vfdir_lib::core::search::filters::name::{NameSpecification, TextMatchMode};
    use vfdir_lib::core::search::trait_file_specification::FileSpecification;
    fn create_test_file(name: &str) -> FileSystemEntry {
        FileSystemEntry {
            path: format!("/test/{}", name),
            name: name.to_string(),
            is_dir: false,
            is_file: true,
            size: Some(1024),
            modified: Some(1234567890),
            created: Some(1234567890),
            accessed: Some(1234567890),
        }
    }
    #[test]
    fn test_regex_digits() {
        let spec = NameSpecification::new(
            r"test_\d+\.txt".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("test_123.txt")));
        assert!(spec.is_satisfied_by(&create_test_file("test_1.txt")));
        assert!(!spec.is_satisfied_by(&create_test_file("test_abc.txt")));
        assert!(!spec.is_satisfied_by(&create_test_file("test_.txt")));
    }
    #[test]
    fn test_regex_start_anchor() {
        let spec = NameSpecification::new(
            r"^report".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("report_2024.pdf")));
        assert!(spec.is_satisfied_by(&create_test_file("report.txt")));
        assert!(!spec.is_satisfied_by(&create_test_file("my_report.pdf")));
    }
    #[test]
    fn test_regex_end_anchor() {
        let spec = NameSpecification::new(
            r"\.pdf$".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("document.pdf")));
        assert!(!spec.is_satisfied_by(&create_test_file("document.pdf.bak")));
        assert!(!spec.is_satisfied_by(&create_test_file("document.txt")));
    }
    #[test]
    fn test_regex_case_sensitive_default() {
        let spec = NameSpecification::new(
            r"Report".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("Report_2024.pdf")));
        assert!(!spec.is_satisfied_by(&create_test_file("report_2024.pdf")));
    }
    #[test]
    fn test_regex_case_insensitive_flag() {
        let spec = NameSpecification::new(
            r"(?i)report".into(),  // (?i) = case insensitive
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("Report_2024.pdf")));
        assert!(spec.is_satisfied_by(&create_test_file("REPORT_2024.pdf")));
        assert!(spec.is_satisfied_by(&create_test_file("report_2024.pdf")));
    }
    #[test]
    fn test_regex_date_pattern() {
        let spec = NameSpecification::new(
            r"\d{4}-\d{2}-\d{2}".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("backup_2024-01-15.zip")));
        assert!(spec.is_satisfied_by(&create_test_file("2024-12-31_log.txt")));
        assert!(!spec.is_satisfied_by(&create_test_file("backup_24-1-15.zip")));
    }
    #[test]
    fn test_regex_version() {
        let spec = NameSpecification::new(
            r"v\d+\.\d+\.\d+".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("app_v1.2.3.exe")));
        assert!(spec.is_satisfied_by(&create_test_file("v10.0.1_release.zip")));
        assert!(!spec.is_satisfied_by(&create_test_file("version_1.2.exe")));
    }
    #[test]
    fn test_regex_extension_alternatives() {
        let spec = NameSpecification::new(
            r"\.(jpg|png|gif)$".into(),
            TextMatchMode::Regex
        ).unwrap();
        assert!(spec.is_satisfied_by(&create_test_file("photo.jpg")));
        assert!(spec.is_satisfied_by(&create_test_file("icon.png")));
        assert!(spec.is_satisfied_by(&create_test_file("animation.gif")));
        assert!(!spec.is_satisfied_by(&create_test_file("document.pdf")));
    }
}