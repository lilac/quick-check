// vim: sts=4 sw=4 et

use lazy::Lazy;
use super::std;

use std::hashmap::HashMap;

/**
 The Shrink trait is used when trying to reduce a testcase to a minimal testcase.
 Shrink should generate "simpler" values, the simplest first.
 */
pub trait Shrink {
    fn shrink(&self) -> Lazy<Self> {
        Lazy::new()
    }
}

impl Shrink for () {}
impl Shrink for bool {}
impl Shrink for char {}
impl Shrink for f32 {}
impl Shrink for f64 {}
impl Shrink for i8 {}
impl Shrink for int {}

fn mpowers_of_two<T: Num + Ord>(n: T) -> ~[T] {
    /* generate ~[0, n/2, n - n/4, n - n/8, n - n/16, .., n - 1] */
    use std::num::One;
    let mut ret = ~[std::num::Zero::zero()];
    let one:T = One::one();
    let two = one + one;
    let mut div = one + one;
    /* check for end or overflow */
    while div < n && div >= two{
        let next = n/div;
        ret.push(n - next);
        div = div * two;
    }
    ret
}

macro_rules! shrink_uint(
    ($x:expr) => (match $x {
            0 => ~[],
            1 => ~[0],
            2 => ~[0, 1],
            n @ 3 .. 8 => ~[n-3, n-2, n-1],
            n => mpowers_of_two(n),
    })
)

impl Shrink for u8 {
    fn shrink(&self) -> Lazy<u8> { Lazy::new_from(shrink_uint!(*self)) }
}

impl Shrink for uint {
    fn shrink(&self) -> Lazy<uint> { Lazy::new_from(shrink_uint!(*self)) }
}

/* type out the (A, B) tuple case as we can save half the .clone() calls */
impl<A: Send + Clone + Shrink, B: Send + Clone + Shrink> Shrink for (A, B) {
    fn shrink(&self) -> Lazy<(A, B)> {
        match self {
            &(ref A, ref B) => {
                let mut L = Lazy::new();
                L.push_map_env(A.shrink(), B.clone(), |s, b| (s, b.clone()));
                L.push_map_env(B.shrink(), A.clone(), |s, a| (a.clone(), s));
                L
            }
        }
    }
}

macro_rules! shrink_tuple(
    ($P:pat : $($T:ident),+ -> $($S:expr),+) => (
    impl<$($T: Send + Clone + Shrink),+> Shrink for ($($T),+) {
        fn shrink(&self) -> Lazy<($($T),+)> {
            Lazy::create( |L| {
                match self {
                    &($(ref $T),+) => {
                        $(
                            L.push_map_env($T.shrink(), self.clone(), |s, t| {
                                let $P = t.clone();
                                $S
                                });
                        )+
                    }
                }
            })
        }
    }
    )
)

shrink_tuple!(
    (a, b, c) : A, B, C ->
    (s, b, c),
    (a, s, c),
    (a, b, s))

shrink_tuple!(
    (a, b, c, d) : A, B, C, D ->
    (s, b, c, d),
    (a, s, c, d),
    (a, b, s, d),
    (a, b, c, s))

shrink_tuple!(
    (a, b, c, d, e) : A, B, C, D, E ->
    (s, b, c, d, e),
    (a, s, c, d, e),
    (a, b, s, d, e),
    (a, b, c, s, e),
    (a, b, c, d, s))

shrink_tuple!(
    (a, b, c, d, e, f) : A, B, C, D, E, F ->
    (s, b, c, d, e, f),
    (a, s, c, d, e, f),
    (a, b, s, d, e, f),
    (a, b, c, s, e, f),
    (a, b, c, d, s, f),
    (a, b, c, d, e, s))

impl<T: Send + Clone + Shrink> Shrink for Option<T> {
    fn shrink(&self) -> Lazy<Option<T>> {
        Lazy::create( |L| {
            match *self {
                None => {}
                Some(ref x) => {
                    L.push(None);
                    L.push_map(x.shrink(), |y| Some(y));
                }
            }
        })
    }
}

impl<T: Send + Clone + Shrink, U: Send + Clone + Shrink> Shrink for Result<T, U> {
    fn shrink(&self) -> Lazy<Result<T, U>> {
        Lazy::create( |L| {
            match *self {
                Ok(ref x) => L.push_map(x.shrink(), |y| Ok(y)),
                Err(ref x) => L.push_map(x.shrink(), |y| Err(y)),
            }
        })
    }
}

impl<T: Send + Shrink> Shrink for ~T {
    fn shrink(&self) -> Lazy<~T> {
        Lazy::create( |L| {
            L.push_map((**self).shrink(), |u| ~u);
        })
    }
}

impl<T: 'static + Send + Shrink> Shrink for @T {
    fn shrink(&self) -> Lazy<@T> {
        Lazy::create( |L| {
            L.push_map((**self).shrink(), |u| @u);
        })
    }
}

impl Shrink for ~str {
    fn shrink(&self) -> Lazy<~str> {
        Lazy::create( |L| {
            if self.len() > 0 {
                let v = self.chars().collect::<~[char]>();
                L.push_map(v.shrink(), |v| std::str::from_chars(v));
            }
        })
    }
}

impl<T: Send + Clone + Shrink> Shrink for ~[T] {
    fn shrink(&self) -> Lazy<~[T]> {
        let mut L = Lazy::new();
        if self.len() == 0 {
            return L;
        }

        L.push(~[]);

        do L.push_thunk(self.clone()) |L, v| {
            if v.len() > 2 {
                let mid = v.len()/2;
                L.push(v.iter().take(mid).map(|x| x.clone()).collect());
                L.push(v.iter().skip(mid).map(|x| x.clone()).collect());
            }
            do L.push_thunk(v) |L, v| {
                for index in range(0, v.len()) {
                    /* remove one at a time */
                    do L.push_thunk((index, v.clone())) |L, (index, v)| {
                        let mut v1 = v.clone();
                        v1.remove(index);
                        L.push(v1);
                        /* shrink one at a time */
                        do L.push_thunk((index, v)) |L, (index, v)| {
                            L.push_map_env(v[index].shrink(), (index, v), |selt, &(ref index, ref v)| {
                                let mut v1 = v.clone();
                                v1[*index] = selt;
                                v1
                            });
                        }
                    }
                }
            }
        }
        L
    }
}


impl<K: Eq + Hash + Clone + Shrink + Send,
     V: Clone + Shrink + Send> Shrink for HashMap<K, V> {
    fn shrink(&self) -> Lazy<HashMap<K, V>> {
        Lazy::create( |L| {
            if self.len() > 0 {
                let v = self.clone().move_iter().collect::<~[(K, V)]>();
                L.push_map(v.shrink(), |v| v.move_iter().collect());
            }
        })
    }
}
