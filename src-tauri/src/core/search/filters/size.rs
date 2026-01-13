use crate::core::FileSystemEntry;
use crate::core::search::trait_file_specification::FileSpecification;

pub struct SizeSpec {
    pub min_bytes: Option<u64>,
    pub max_bytes: Option<u64>,
}
impl SizeSpec {
    pub fn new(min_bytes: Option<u64>, max_bytes: Option<u64>) -> Self {
        Self { min_bytes, max_bytes }
    }
}
impl FileSpecification for SizeSpec {
    fn is_satisfied_by(&self, item: &FileSystemEntry) -> bool {
        // 🎯 ВАША ЗАДАЧА:
        // 1. Получите размер из item.size (это Option<u64>)
        //    Если size = None, что вернуть? (подсказка: false, т.к. размер неизвестен)
        // 2. Проверьте минимальную границу:
        //    if let Some(min) = self.min_bytes {
        //        if size < min { return false; }
        //    }
        // 3. Проверьте максимальную границу:
        //    if let Some(max) = self.max_bytes {
        //        if size > max { return false; }
        //    }
        // 4. Если оба условия прошли, верните true
        // Альтернативный подход (короче):
        // let size = item.size?;  // вернет false если None
        // self.min_bytes.map_or(true, |min| size >= min) &&
        // self.max_bytes.map_or(true, |max| size <= max)
        match item.size {
            None => false,
            Some(m) => {
                if self.min_bytes.is_none() && self.max_bytes.is_none() {
                    true
                } else if let Some(min) = self.min_bytes {
                    if let Some(max) = self.max_bytes {
                        m >= min && m <= max
                    } else {
                        m >= min
                    }
                } else if let Some(max) = self.max_bytes {
                    m <= max
                } else {
                    false
                }
            }
        }
        // Option<u64> может быть:
        /*match item.size {
            Some(size) => println!("Размер: {} байт", size),
            None => println!("Размер неизвестен (директория или ошибка)"),
        }*/
    }
}