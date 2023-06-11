use sort_test_tools::{instantiate_sort_tests, Sort};

struct SortImpl {}

impl Sort for SortImpl {
    fn name() -> String {
        "rust_tinyheapsort_unstable".into()
    }

    fn sort<T>(arr: &mut [T])
    where
        T: Ord,
    {
        tiny_sort::unstable::sort(arr);
    }

    fn sort_by<T, F>(arr: &mut [T], compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        tiny_sort::unstable::sort_by(arr, compare);
    }
}

instantiate_sort_tests!(SortImpl);
