pub struct Item {
    pub key: String,
    pub name: String,
    pub desc: String,
}

impl Item {
    #[inline]
    pub fn find_eq(&self, k: &str) -> bool {
        self.key == k
    }

    #[inline]
    pub fn find_starts_with(&self, k: &str) -> bool {
        self.key.starts_with(k)
    }

    #[inline]
    pub fn find_contains(&self, k: &str) -> bool {
        self.key.contains(k)
    }

    pub fn find_match(&self, k: &str) -> bool {
        let key_bytes = self.key.as_bytes();
        let k_bytes = k.as_bytes();

        if k_bytes.len() > key_bytes.len() {
            return false;
        }

        // 子序列匹配 两个迭代器依次步进
        let mut key_iter = key_bytes.iter();
        k_bytes.iter().all(|s| key_iter.any(|t| t == s))
    }
}
