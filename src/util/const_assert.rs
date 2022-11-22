// copied/inspired from the crate static_assertions and const_fn_assert:
// - https://docs.rs/static_assertions/
// - https://docs.rs/const_fn_assert/

#[allow(dead_code)]
pub const CFN_ASSERT: [(); 1] = [()];

#[allow(dead_code)]
pub const fn bool_assert(x: bool) -> bool { x }

#[macro_export]
macro_rules! cfn_assert {
    ($x: expr $(,)?) => {{
        use crate::util::const_assert::*;
        let _ = CFN_ASSERT[!bool_assert($x) as usize];
    }};
}

#[macro_export]
macro_rules! cfn_assert_eq {
    ($x: expr, $y:expr $(,)?) => {
        cfn_assert!($x == $y);
    };
}


#[macro_export]
macro_rules! cfn_assert_ne {
    ($x: expr, $y:expr $(,)?) => {
        cfn_assert!($x != $y);
    };
}

#[macro_export]
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        const _: [(); 0 - !{ const ASSERT: bool = $x; ASSERT } as usize] = [];
    };
}

#[macro_export]
macro_rules! const_assert_eq {
    ($x:expr, $y:expr $(,)?) => {
        const_assert!($x == $y);
    };
}

#[macro_export]
macro_rules! const_assert_ne {
    ($x:expr, $y:expr $(,)?) => {
        const_assert!($x != $y);
    };
}
