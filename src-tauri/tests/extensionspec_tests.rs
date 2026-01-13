#[cfg(test)]
mod test{
    use vfdir_lib::core::FileSystemEntry;
    use vfdir_lib::core::search::filters::extension::ExtensionSpecification;
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
    fn test_extension_normalization_without_dot() {
        let spec = ExtensionSpecification::new("pdf".into());
        assert_eq!(spec.extension, ".pdf");
    }
    #[test]
    fn test_extension_normalization_with_dot() {
        let spec = ExtensionSpecification::new(".pdf".into());
        assert_eq!(spec.extension, ".pdf");
    }
    #[test]
    fn test_extension_normalization_uppercase() {
        let spec = ExtensionSpecification::new("PDF".into());
        assert_eq!(spec.extension, ".pdf");
        let spec = ExtensionSpecification::new(".JPG".into());
        assert_eq!(spec.extension, ".jpg");
    }
    #[test]
    fn test_extension_match_exact() {
        let spec = ExtensionSpecification::new("txt".into());
        assert!(spec.is_satisfied_by(&create_test_file("document.txt")));
        assert!(spec.is_satisfied_by(&create_test_file("README.txt")));
    }
    #[test]
    fn test_extension_match_case_insensitive() {
        let spec = ExtensionSpecification::new("pdf".into());
        assert!(spec.is_satisfied_by(&create_test_file("document.pdf")));
        assert!(spec.is_satisfied_by(&create_test_file("report.PDF")));
        assert!(spec.is_satisfied_by(&create_test_file("file.Pdf")));
    }
    #[test]
    fn test_extension_no_match() {
        let spec = ExtensionSpecification::new("pdf".into());
        assert!(!spec.is_satisfied_by(&create_test_file("document.txt")));
        assert!(!spec.is_satisfied_by(&create_test_file("image.jpg")));
        assert!(!spec.is_satisfied_by(&create_test_file("README")));
    }
    #[test]
    fn test_extension_compound() {
        // Файлы с составным расширением
        let spec = ExtensionSpecification::new("gz".into());
        assert!(spec.is_satisfied_by(&create_test_file("archive.tar.gz")));
        assert!(spec.is_satisfied_by(&create_test_file("file.gz")));
        assert!(!spec.is_satisfied_by(&create_test_file("archive.tar")));
    }
    #[test]
    fn test_extension_hidden_file() {
        let spec = ExtensionSpecification::new("gitignore".into());
        assert!(spec.is_satisfied_by(&create_test_file(".gitignore")));
    }
    #[test]
    fn test_extension_no_extension() {
        let spec = ExtensionSpecification::new("txt".into());
        assert!(!spec.is_satisfied_by(&create_test_file("README")));
        assert!(!spec.is_satisfied_by(&create_test_file("Makefile")));
    }
    #[test]
    fn test_extension_realistic_images() {
        let jpg_spec = ExtensionSpecification::new("jpg".into());
        let png_spec = ExtensionSpecification::new("png".into());
        // JPG файлы
        assert!(jpg_spec.is_satisfied_by(&create_test_file("photo.jpg")));
        assert!(jpg_spec.is_satisfied_by(&create_test_file("IMG_001.JPG")));
        assert!(!jpg_spec.is_satisfied_by(&create_test_file("photo.png")));
        // PNG файлы
        assert!(png_spec.is_satisfied_by(&create_test_file("icon.png")));
        assert!(png_spec.is_satisfied_by(&create_test_file("screenshot.PNG")));
        assert!(!png_spec.is_satisfied_by(&create_test_file("icon.jpg")));
    }
    #[test]
    fn test_extension_documents() {
        let pdf_spec = ExtensionSpecification::new("pdf".into());
        let docx_spec = ExtensionSpecification::new("docx".into());
        assert!(pdf_spec.is_satisfied_by(&create_test_file("report.pdf")));
        assert!(docx_spec.is_satisfied_by(&create_test_file("letter.docx")));
        assert!(!pdf_spec.is_satisfied_by(&create_test_file("letter.docx")));
        assert!(!docx_spec.is_satisfied_by(&create_test_file("report.pdf")));
    }
}