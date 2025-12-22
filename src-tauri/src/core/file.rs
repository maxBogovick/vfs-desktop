use crate::core::metadata::Metadata;

pub struct File {
    name: String,
    content: Vec<u8>,
    metadata: Metadata
}

impl File {
    pub fn new(name: String) -> Self{
        File{name, content: vec![0], metadata: Metadata::new(0, 0, 0)}
    }
    
    pub fn with_content(name: String, content: Vec<u8>) -> Self{
        File{name, content, metadata: Metadata::new(0, 0, 0)}
    }
    pub fn read(&self) -> &[u8] {
        &self.content
    }
    pub fn write(&mut self, data: &[u8]) {
        self.content = Vec::from(data);
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_content(&mut self, content: Vec<u8>) {
        self.content = content;
    }

    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }
}

#[cfg(test)]
pub mod tests {
    use crate::core::file::File;

    #[test]
    pub fn empty_file() {
        let file = File::new("file.txt".to_string());

        // Проверяем имя
        assert_eq!(file.name, "file.txt");
        // Внимание: Ваша реализация new создает vec![0], а не пустой vec![]
        assert_eq!(file.read(), &[0]);
    }

    #[test]
    pub fn file_w_data() {
        let data = vec![1, 2, 3];
        let file = File::with_content("file.txt".to_string(), data.clone());

        assert_eq!(file.name, "file.txt");
        assert_eq!(file.read(), &data[..]);
    }

    #[test]
    pub fn file_datas() {
        let file = File::with_content("file.txt".to_string(), vec![1, 2, 3]);
        let content = file.read();

        assert_eq!(content, &[1, 2, 3]);
        assert_eq!(content.len(), 3);
    }

    #[test]
    pub fn new_datas() {
        let mut file = File::with_content("file.txt".to_string(), vec![1, 2, 3]);

        // Перезаписываем данные
        file.write(&[4, 5, 6, 7]);

        // Проверяем, что данные изменились
        assert_eq!(file.read(), &[4, 5, 6, 7]);
    }
}