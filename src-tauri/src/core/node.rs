use crate::core::directory::Directory;
use crate::core::file::File;
use crate::core::metadata::Metadata;

pub enum  Node{
    File(File),
    Directory(Directory)
}

impl Node {
    pub fn name(&self) -> &str{
        match self { Node::Directory(d) => d.name(), Node::File(f) => f.name() }
    }

    pub fn is_dir(&self) -> bool {
        match self {
            Node::Directory(d) => true,
            _ => false
        }
    }
    pub fn is_file(&self) -> bool{
        match self {
            Node::File(f) => true,
            _ => false
        }
    }
    pub fn metadata(&self) -> &Metadata{
        match self {
            Node::File(f) => f.metadata(),
            Node::Directory(d) => d.metadata()
        }
    }
    pub fn as_file(&self) -> Option<&File> {
        match self {
            Node::File(f) => Some(f),
            _ => None
        }

    }
    pub fn as_dir(&self) -> Option<&Directory> {
        match self {
            Node::Directory(d) => Some(d),
            _ => None
        }
    }
    pub fn as_file_mut(&mut self) -> Option<&mut File>{
        match self {
            Node::File(f) => Some(f),
            _ => None
        }
    }
    pub fn as_dir_mut(&mut self) -> Option<&mut Directory>{
        match self {
            Node::Directory(d) => Some(d),
            _ => None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::core::directory::Directory;
    use crate::core::file::File;
    use crate::core::node::Node;

    #[test]
    pub fn create_directory() {
        let d = Directory::new("".to_string());
        let new = Node::Directory(d);
        assert!(new.is_dir());
    }
    #[test]
    pub fn create_file() {
        let f = File::new("".to_string());
        let new = Node::File(f);
        assert!(new.is_file());
    }
    #[test]
    pub fn mut_file() {
        let mut f = File::new("".to_string());
        f.set_name("anything".to_string());
        assert_eq!(f.name(), "anything")
    }
    #[test]
    pub fn mut_dir() {
        let mut d = Directory::new("".to_string());
        d.set_name("any".to_string());
        assert_eq!(d.name(), "any")

    }
}