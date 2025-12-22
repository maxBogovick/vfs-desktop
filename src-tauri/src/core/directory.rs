use crate::core::metadata::Metadata;
use crate::core::node::Node;

pub struct  Directory{
    name: String,
    child: Vec<Node>,
    metadata: Metadata
}

impl Directory {
    pub fn new(name: String) -> Self {
        Self { name, child: Vec::new(), metadata: Metadata::new(0, 0, 0) }
    }
    
    

    pub fn find_child(&self, n: &str) -> Option<&String>{
        if n == self.name { Some(&self.name) }
        else { None }
    }
    pub fn find_name_mut(&mut self, n: &str) -> Option<&mut String>{
        if n == self.name { Some(&mut self.name) }
        else { None }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn child(&self) -> &Vec<Node> {
        &self.child
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_child(&mut self, child: Vec<Node>) {
        self.child = child;
    }

    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }
}

#[cfg(test)]
pub mod tests {
    use crate::core::directory::Directory;

    #[test]
    pub fn add_dir() {
        let dir = Directory::new("new dir".to_string());

        // Так как модуль тестов находится внутри файла, мы имеем доступ к приватным полям
        assert_eq!(dir.name, "new dir");
        assert!(dir.child.is_empty()); // Проверяем, что список детей пуст при создании
    }

    #[test]
    pub fn research_folders() {
        let directory = Directory::new("one".to_string());

        // Текущая логика: возвращает Some, если имя совпадает с именем самой директории
        let found = directory.find_child("one");
        assert_eq!(found, Some(&"one".to_string()));

        // Проверяем случай, когда имя не совпадает
        let not_found = directory.find_child("two");
        assert_eq!(not_found, None);
    }

    #[test]
    pub fn search_n_edit() {
        let mut directory = Directory::new("one".to_string());

        // Получаем изменяемую ссылку
        let found_mut = directory.find_name_mut("one");
        assert!(found_mut.is_some());

        // Изменяем имя директории через полученную ссылку
        if let Some(name_ref) = found_mut {
            *name_ref = "changed".to_string();
        }

        // Проверяем, что имя директории действительно изменилось
        assert_eq!(directory.name, "changed");

        // Проверяем, что по старому имени больше не находится
        assert_eq!(directory.find_child("one"), None);
    }
}