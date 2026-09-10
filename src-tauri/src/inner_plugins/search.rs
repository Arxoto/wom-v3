use std::sync::Mutex;

use serde::Serialize;

use crate::inner_plugins::common::Item;

/// 缓存搜索结果
/// 纯内存计算，直接使用 [`std::sync::Mutex`]
pub struct ItemSearchStat(pub Mutex<ItemSearchResult>);

impl ItemSearchStat {
    pub fn new() -> Self {
        Self(Mutex::new(ItemSearchResult::default()))
    }
}

/// 搜索结果（后端）
#[derive(Debug, Default)]
pub struct ItemSearchResult {
    pub key_word: String,
    pub item_list: Vec<Item>,
    // 不同匹配模式的分割索引
    pub index_eq: usize,
    pub index_starts_with: usize,
    pub index_contains: usize,
    pub index_match: usize,
}

/// 分页查询搜索结果
pub const PAGE_SIZE: usize = 100;

/// 搜索结果（前端）
#[derive(Debug, Serialize)]
pub struct ItemSearchPage {
    pub total: usize,
    pub index: usize,
    pub item_list: Vec<ItemDisplay>,
    // 不同匹配模式的分割索引
    pub index_eq: usize,
    pub index_starts_with: usize,
    pub index_contains: usize,
    pub index_match: usize,
}

impl ItemSearchResult {
    pub fn is_current_result(&self, k: &str) -> bool {
        self.key_word == k
    }

    pub fn page(&self, index: usize) -> ItemSearchPage {
        let start_index = self.item_list.len().min(index);
        let final_index = self.item_list.len().min(start_index + PAGE_SIZE);

        let item_list: Vec<ItemDisplay> = self.item_list[start_index..final_index]
            .iter()
            .map(ItemDisplay::from)
            .collect();

        ItemSearchPage {
            total: self.item_list.len(),
            index: start_index,
            item_list,
            index_eq: self.index_eq,
            index_starts_with: self.index_starts_with,
            index_contains: self.index_contains,
            index_match: self.index_match,
        }
    }
}

/// [`Item`] 渲染结构
#[derive(Debug, Serialize)]
pub struct ItemDisplay {
    pub the_type: String,
    pub name: String,
    pub desc: String,
}

impl From<&Item> for ItemDisplay {
    fn from(value: &Item) -> Self {
        Self {
            the_type: value.the_type.to_string(),
            name: value.name.clone(),
            desc: value.desc.to_string(),
        }
    }
}

mod algorithm {
    use bitvec::prelude::*;

    use crate::inner_plugins::{
        base::KeyWord, common::Item, persistence::load::ItemCollection, search::ItemSearchResult,
    };

    impl KeyWord {
        #[inline]
        pub fn find_eq(&self, k: &str) -> bool {
            self.0 == k
        }

        #[inline]
        pub fn find_starts_with(&self, k: &str) -> bool {
            self.0.starts_with(k)
        }

        #[inline]
        pub fn find_contains(&self, k: &str) -> bool {
            self.0.contains(k)
        }

        pub fn find_match(&self, k: &str) -> bool {
            let key_bytes = self.0.as_bytes();
            let k_bytes = k.as_bytes();

            if k_bytes.len() > key_bytes.len() {
                return false;
            }

            // 子序列匹配 两个迭代器依次步进
            let mut key_iter = key_bytes.iter();
            k_bytes.iter().all(|s| key_iter.any(|t| t == s))
        }
    }

    impl ItemCollection {
        pub fn search(&self, k: &str) -> ItemSearchResult {
            let item_list = &self.item_list;

            let mut list_eq: Vec<Item> = vec![];
            let mut list_starts_with: Vec<Item> = vec![];
            let mut list_contains: Vec<Item> = vec![];
            let mut list_match: Vec<Item> = vec![];
            for item in item_list {
                let key_word = &item.key_word;
                if key_word.find_eq(k) {
                    list_eq.push(item.clone());
                } else if key_word.find_starts_with(k) {
                    list_starts_with.push(item.clone());
                } else if key_word.find_contains(k) {
                    list_contains.push(item.clone());
                } else if key_word.find_match(k) {
                    list_match.push(item.clone());
                }
            }

            let MergedList {
                merged_list,
                index1,
                index2,
                index3,
                index4,
            } = do_merge_and_deduplicate(list_eq, list_starts_with, list_contains, list_match);

            ItemSearchResult {
                key_word: k.to_string(),
                item_list: merged_list,
                index_eq: index1,
                index_starts_with: index2,
                index_contains: index3,
                index_match: index4,
            }
        }
    }

    impl HasId for Item {
        fn get_id(&self) -> usize {
            self.the_id as usize
        }
    }

    trait HasId {
        fn get_id(&self) -> usize;
    }

    struct MergedList<E: HasId> {
        pub merged_list: Vec<E>,
        pub index1: usize,
        pub index2: usize,
        pub index3: usize,
        pub index4: usize,
    }

    /// 消耗所有权进行合并，使用 BitVec<u64, Lsb0> 判断是否已存在
    ///
    /// 为优化性能， id 必须是纯数字
    /// - 若 id 是离散的，则使用 nohash-hasher ，把数字本身当作哈希值
    /// - 若 id 是连续的，则使用 bitvec ，内存占用低、缓存友好
    ///   - 另有 bit-set ，相比 bitvec 更便捷（无需手动处理索引越界），但是版本号是 0.9 （虽然是维护状态），考虑到维护活跃度不选择
    ///   - 另有 roaring ，更成熟、工业级的方案，更适用于海量数据的场景，低数据量没有优势
    fn append_unique<E: HasId>(
        source: Vec<E>,
        seen: &mut BitVec<u64, Lsb0>,
        target: &mut Vec<E>,
    ) -> usize {
        let origin_index = target.len();

        // 预剪枝优化 扩容 bitvec
        if let Some(max_id) = source.iter().map(|item| item.get_id()).max() {
            if max_id >= seen.len() {
                let aligned_len = (max_id + 64) & !63; // 64 取整，内存对齐
                let new_len = seen.len().saturating_mul(2).max(aligned_len); // 防止溢出
                seen.resize(new_len, false);
            }
        }

        for item in source {
            let id = item.get_id();

            // replace(index, value) 会将该位设为 true ，并返回该位原来的值
            // 如果原来是 false ，说明是第一次遇见
            if !seen.replace(id, true) {
                target.push(item);
            }
        }

        origin_index
    }

    fn do_merge_and_deduplicate<E: HasId>(
        l1: Vec<E>,
        l2: Vec<E>,
        l3: Vec<E>,
        l4: Vec<E>,
    ) -> MergedList<E> {
        // 预分配容量
        let total_potential_size = l1.len() + l2.len() + l3.len() + l4.len();
        let mut merged_list = Vec::with_capacity(total_potential_size);

        // 初始化 bitvec
        // 使用 u64 作为存储单元，大部分终端现在都是 64 位机器，性能较好
        // 初始值为 false
        // 预设 1024 位，大部分情况足够
        let mut seen_ids = bitvec![u64, Lsb0; 0; 1024];

        // 执行合并
        let index1 = append_unique(l1, &mut seen_ids, &mut merged_list);
        let index2 = append_unique(l2, &mut seen_ids, &mut merged_list);
        let index3 = append_unique(l3, &mut seen_ids, &mut merged_list);
        let index4 = append_unique(l4, &mut seen_ids, &mut merged_list);

        MergedList {
            merged_list,
            index1,
            index2,
            index3,
            index4,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        impl HasId for u32 {
            fn get_id(&self) -> usize {
                *self as usize
            }
        }

        #[test]
        fn merge_and_deduplicate() {
            let mut seen_ids = bitvec![u64, Lsb0; 0; 1024];
            let mut merged_list: Vec<u32> = Vec::new();

            let index = append_unique(vec![1, 3, 5, 7, 9], &mut seen_ids, &mut merged_list);
            assert_eq!(index, 0);
            assert_eq!(merged_list, vec![1, 3, 5, 7, 9]);

            let index = append_unique(vec![2, 3, 5, 7, 11, 13], &mut seen_ids, &mut merged_list);
            assert_eq!(index, 5);
            assert_eq!(merged_list, vec![1, 3, 5, 7, 9, 2, 11, 13]);
        }

        #[test]
        fn scale_up() {
            let mut seen_ids = bitvec![u64, Lsb0; 0; 1024];
            let mut merged_list: Vec<u32> = Vec::new();

            let index = append_unique(vec![1, 3, 5, 7, 9], &mut seen_ids, &mut merged_list);
            assert_eq!(index, 0);
            assert_eq!(merged_list, vec![1, 3, 5, 7, 9]);

            let index = append_unique(vec![2, 2048], &mut seen_ids, &mut merged_list);
            assert_eq!(index, 5);
            assert_eq!(merged_list, vec![1, 3, 5, 7, 9, 2, 2048]);

            assert_eq!(seen_ids.capacity(), 2048 + 64);
        }
    }
}
