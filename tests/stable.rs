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

#[test]
fn by_key() {
    // Padding values to ensure that misusing val would give other comparison results.
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Val {
        _a: u8,
        b: i32,
        _c: u32,
    }

    let input = sort_test_tools::patterns::random(50)
        .into_iter()
        .map(|val| Val {
            _a: 0,
            b: val,
            _c: val.saturating_abs() as u32,
        })
        .collect::<Vec<Val>>();

    let mut tiny_sorted = input.clone();
    let mut std_sorted = input.clone();

    tiny_sort::stable::sort_by_key(&mut tiny_sorted, |val| val.b);
    std_sorted.sort_by_key(|val| val.b);

    assert_eq!(tiny_sorted, std_sorted);
}
