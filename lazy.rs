// vim: sts=4 sw=4 et

/*!
 Lazy is a Lazily generated sequence, only traversable once, implementing Iterator.

 It allows lazy generation by allowing generators to tack on thunks of closures
 that are not called until the list is traversed to that point.

 Only has list structure if all thunks are nested inside each other. Otherwise
 it is more like a tree.

 Uses a custom construction with ~Thunk and ~Eval to allow moving a value into
 a once-callable Thunk, and mutating/moving that value when evaluating it.


 This library was first implemented using ~fn but I switched to extern fn.
 Update: Now switched to proc.

 */

/// Lazily generated sequence, only traversable once
pub struct Lazy<T> {
    head: ~[T],
    thunks: ~[~Eval<Lazy<T>>],
}

trait Eval<L> {
    fn eval(~self, &mut L);
}

/// A frozen computation that can be resolved in the context of an L value (a Lazy)
struct Thunk<L, Up> {
    upvar: Up,
    f: proc(&mut L, Up),
}

impl<L, Up> Eval<L> for Thunk<L, Up> {
    fn eval(~self, x: &mut L) {
        let Thunk{ upvar: v, f: f } = *self;
        f(x, v)
    }
}

impl<T> Lazy<T> {
    pub fn new() -> Lazy<T> {
        Lazy::new_from(~[])
    }

    pub fn new_from(v: ~[T]) -> Lazy<T> {
        Lazy{head: v, thunks: ~[]}
    }

    pub fn create(f: |&mut Lazy<T>|) -> Lazy<T> {
        let mut L = Lazy::new();
        f(&mut L);
        L
    }

    pub fn next(&mut self) -> Option<T> {
        while self.head.len() == 0 && self.thunks.len() > 0 {
            let next = self.thunks.shift().unwrap();
            next.eval(self);
        }
        self.head.shift()
    }

    /// push a value to the end of the Lazy.
    pub fn push(&mut self, x: T) {
        self.head.push(x);
    }

    /// push a thunk to the end of the thunk list of lazy.
    /// ordered after all immediate push values.
    pub fn push_thunk<Up: Send>(&mut self, x: Up,
                                f: proc(&mut Lazy<T>, Up)) {
        let t = ~Thunk { upvar: x, f: f };
        self.thunks.push(t as ~Eval<Lazy<T>>)
    }

    /// lazily map from the iterator `a` using function `f`, appending the results to self.
    /// Static function without environment.
    pub fn push_map<A, J: Send + Iterator<A>>(&mut self, it: J,
                                              f: 'static |A|: Send -> T) {
        do self.push_thunk((f, it)) |L, (f, mut it)| {
            match it.next() {
                None => {}
                Some(x) => {
                    L.push(f(x));
                    L.push_map(it, f);
                }
            }
        }
    }

    /// Static function with ref to supplied environment.
    pub fn push_map_env<A, J: Send + Iterator<A>, Env: Send>
        (&mut self, it: J, env: Env,
         f: 'static |A, &mut Env|: Send -> T) {
        do self.push_thunk((f, it, env)) |L, (f, mut it, mut env)| {
            match it.next() {
                None => {}
                Some(x) => {
                    L.push(f(x, &mut env));
                    L.push_map_env(it, env, f);
                }
            }
        }
    }
}

impl<T> Iterator<T> for Lazy<T> {
    fn next(&mut self) -> Option<T> { self.next() }
}

#[test]
fn test_lazy_list() {
    let mut L = Lazy::create( |L| {
        L.push(3);
        do L.push_thunk(~[4, 5]) |L, mut v| {
            L.push(v.shift().unwrap());
            do L.push_thunk(v) |L, mut v| {
                L.push(v.shift().unwrap());
            }
        }
    });

    assert_eq!(L.next(), Some(3));
    assert_eq!(L.next(), Some(4));
    assert_eq!(L.next(), Some(5));
    assert_eq!(L.next(), None);

    let mut M = Lazy::new();
    M.push_map(Lazy::new_from(~[3,4,5]), |x| (x, 1));
    assert_eq!(M.next(), Some((3,1)));
    assert_eq!(M.next(), Some((4,1)));
    assert_eq!(M.next(), Some((5,1)));
    assert_eq!(M.next(), None);
}
