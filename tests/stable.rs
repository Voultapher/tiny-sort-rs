use sort_test_tools::{instantiate_sort_tests, Sort};

struct SortImpl {}

impl Sort for SortImpl {
    fn name() -> String {
        "rust_tinymergesort_stable".into()
    }

    fn sort<T>(arr: &mut [T])
    where
        T: Ord,
    {
        tiny_sort::stable::sort(arr);
    }

    fn sort_by<T, F>(arr: &mut [T], compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        tiny_sort::stable::sort_by(arr, compare);
    }
}

instantiate_sort_tests!(SortImpl);
