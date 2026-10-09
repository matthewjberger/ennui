pub const fn same_name(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

#[macro_export]
macro_rules! line_up {
    ($table:expr, $field:ident) => {
        const _: () = {
            let mut index = 0;
            while index < $table.len() {
                assert!(
                    $table[index].$field as usize == index,
                    concat!(
                        "a ",
                        stringify!($table),
                        " row does not sit at the index of its ",
                        stringify!($field)
                    )
                );
                index += 1;
            }
        };
    };
}
