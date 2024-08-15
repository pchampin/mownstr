//! [`MownStr`]
//! is either a borrowed reference to a `str` or an own `Box<str>`.

use std::borrow::Cow;
use std::fmt;
use std::hash;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::slice;
use std::str;

/// "Maybe own str":
/// either a borrowed reference to a `str` or an owned `Box<str>`.
///
/// It does not try to be mutable, nor generic,
/// which makes it lighter than, for example, `Cow<str>`.
///
/// # Panic
/// The drawback is that `MownStr`
/// does not support strings with a length > `usize::MAX/2`.
/// Trying to convert such a large string to a `MownStr` will panic.
pub struct MownStr<'a> {
    addr: NonNull<u8>,
    xlen: usize,
    _phd: PhantomData<&'a str>,
}

// MownStr does not implement `Sync` and `Send` by default,
// because NonNull<u8> does not.
// However, it is safe to declare it as Sync and Send,
// because MownStr is basically nothing more than a `&str`,
// or a `Box<str>`, and both are `Sync` and `Send`.
unsafe impl Sync for MownStr<'_> {}
unsafe impl Send for MownStr<'_> {}

const LEN_MASK: usize = usize::MAX >> 1;
const OWN_FLAG: usize = !LEN_MASK;

impl<'a> MownStr<'a> {
    pub const fn from_str(other: &'a str) -> MownStr<'a> {
        assert!(other.len() <= LEN_MASK);
        // NB: The only 'const' constuctor for NonNull is new_unchecked
        // so we need an unsafe block.

        // SAFETY: we need a *mut u8 for new_unchecked,
        //         but MownStr will never mutate its content
        let ptr = other.as_ptr() as *mut u8;
        let addr = unsafe {
            // SAFETY: ptr can not be null,
            NonNull::new_unchecked(ptr)
        };
        MownStr {
            addr,
            xlen: other.len(),
            _phd: PhantomData,
        }
    }

    pub const fn is_borrowed(&self) -> bool {
        (self.xlen & OWN_FLAG) == 0
    }

    pub const fn is_owned(&self) -> bool {
        (self.xlen & OWN_FLAG) == OWN_FLAG
    }

    pub const fn borrowed(&self) -> MownStr {
        MownStr {
            addr: self.addr,
            xlen: self.xlen & LEN_MASK,
            _phd: PhantomData,
        }
    }

    #[inline]
    fn real_len(&self) -> usize {
        self.xlen & LEN_MASK
    }

    #[inline]
    unsafe fn make_ref(&self) -> &'a str {
        debug_assert!(self.is_borrowed(), "make_ref() called on owned MownStr");
        let ptr = self.addr.as_ptr();
        let slice = slice::from_raw_parts(ptr, self.xlen);
        str::from_utf8_unchecked(slice)
    }

    /// Convert an *owned* MownStr to a box.
    //
    // NB: conceptually this method consumes the Mownstr.
    // The reason why self is a mutable ref instead of a move is purely technical
    // (to make it usable in Drop::drop()).
    #[inline]
    unsafe fn extract_box(&mut self) -> Box<str> {
        debug_assert!(self.is_owned(), "extract_box() called on borrowed MownStr");
        // extract data to make box
        let ptr = self.addr.as_ptr();
        let len = self.real_len();
        // turn to borrowed, to avoid double-free
        self.xlen = 0;
        debug_assert!(self.is_borrowed());
        // make box
        let slice = slice::from_raw_parts_mut(ptr, len);
        let raw = str::from_utf8_unchecked_mut(slice) as *mut str;
        Box::from_raw(raw)
    }
}

impl<'a> Drop for MownStr<'a> {
    fn drop(&mut self) {
        if self.is_owned() {
            unsafe {
                std::mem::drop(self.extract_box());
            }
        }
    }
}

impl<'a> Clone for MownStr<'a> {
    fn clone(&self) -> MownStr<'a> {
        if self.is_owned() {
            Box::<str>::from(self.deref()).into()
        } else {
            MownStr {
                addr: self.addr,
                xlen: self.xlen,
                _phd: self._phd,
            }
        }
    }
}

// Construct a MownStr

impl<'a> From<&'a str> for MownStr<'a> {
    fn from(other: &'a str) -> MownStr<'a> {
        Self::from_str(other)
    }
}

impl<'a> From<Box<str>> for MownStr<'a> {
    fn from(other: Box<str>) -> MownStr<'a> {
        let len = other.len();
        assert!(len <= LEN_MASK);
        let addr = Box::leak(other).as_mut_ptr();
        let addr = unsafe {
            // SAFETY: ptr can not be null,
            NonNull::new_unchecked(addr)
        };

        let xlen = len | OWN_FLAG;
        let _phd = PhantomData;
        MownStr { addr, xlen, _phd }
    }
}

impl<'a> From<String> for MownStr<'a> {
    fn from(other: String) -> MownStr<'a> {
        other.into_boxed_str().into()
    }
}

impl<'a> From<Cow<'a, str>> for MownStr<'a> {
    fn from(other: Cow<'a, str>) -> MownStr<'a> {
        match other {
            Cow::Borrowed(r) => r.into(),
            Cow::Owned(s) => s.into(),
        }
    }
}

// Using a MownStr as a str

impl<'a> Deref for MownStr<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        let ptr = self.addr.as_ptr();
        let len = self.real_len();
        unsafe {
            let slice = slice::from_raw_parts(ptr, len);
            str::from_utf8_unchecked(slice)
        }
    }
}

impl<'a> AsRef<str> for MownStr<'a> {
    fn as_ref(&self) -> &str {
        self.deref()
    }
}

impl<'a> std::borrow::Borrow<str> for MownStr<'a> {
    fn borrow(&self) -> &str {
        self.deref()
    }
}

// Comparing between MownStr

impl<'a> hash::Hash for MownStr<'a> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<'a> PartialEq for MownStr<'a> {
    fn eq(&self, other: &MownStr<'a>) -> bool {
        self.deref() == other.deref()
    }
}

impl<'a> Eq for MownStr<'a> {}

impl<'a> PartialOrd for MownStr<'a> {
    fn partial_cmp(&self, other: &MownStr<'a>) -> Option<std::cmp::Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl<'a> Ord for MownStr<'a> {
    fn cmp(&self, other: &MownStr<'a>) -> std::cmp::Ordering {
        self.deref().cmp(other.deref())
    }
}

// Comparing MownStr with str

impl<'a> PartialEq<&'a str> for MownStr<'a> {
    fn eq(&self, other: &&'a str) -> bool {
        self.deref() == *other
    }
}

impl<'a> PartialOrd<&'a str> for MownStr<'a> {
    fn partial_cmp(&self, other: &&'a str) -> Option<std::cmp::Ordering> {
        self.deref().partial_cmp(*other)
    }
}

impl<'a> PartialEq<MownStr<'a>> for &'a str {
    fn eq(&self, other: &MownStr<'a>) -> bool {
        self == &other.deref()
    }
}

impl<'a> PartialOrd<MownStr<'a>> for &'a str {
    fn partial_cmp(&self, other: &MownStr<'a>) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.deref())
    }
}

// Formatting

impl<'a> fmt::Debug for MownStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.deref(), f)
    }
}

impl<'a> fmt::Display for MownStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.deref(), f)
    }
}

// Converting

impl<'a> From<MownStr<'a>> for Box<str> {
    fn from(other: MownStr<'a>) -> Box<str> {
        other.to()
    }
}

impl<'a> From<MownStr<'a>> for String {
    fn from(other: MownStr<'a>) -> String {
        other.to()
    }
}

impl<'a> From<MownStr<'a>> for Cow<'a, str> {
    fn from(other: MownStr<'a>) -> Cow<'a, str> {
        if other.is_owned() {
            other.to::<String>().into()
        } else {
            unsafe { other.make_ref() }.into()
        }
    }
}

impl<'a> MownStr<'a> {
    /// Convert this `MownStr` to any type `T`
    /// that can be created from either a `&str` or a `Box<str>`.
    ///
    /// This can not be implemented with the `From` trait,
    /// because this would conflict with `From<MownStr<'a>>`.
    ///
    /// # Usage
    /// ```
    /// # use mownstr::MownStr;
    /// # use std::rc::Rc;
    /// let ms = MownStr::from("hello world");
    /// let rc = ms.to::<Rc<str>>();
    ///
    /// let o1 = Some(MownStr::from("hi there"));
    /// let o2 = o1.map(MownStr::to::<Rc<str>>);
    /// ```
    pub fn to<T>(mut self) -> T
    where
        T: From<&'a str> + From<Box<str>>,
    {
        if self.is_owned() {
            unsafe { self.extract_box() }.into()
        } else {
            unsafe { self.make_ref() }.into()
        }
    }
}

#[cfg(test)]
#[allow(clippy::eq_op)]
mod test {
    use super::MownStr;
    use std::borrow::Cow;
    use std::collections::HashSet;
    use std::fs;
    use std::str::FromStr;

    #[test]
    fn size() {
        assert_eq!(
            std::mem::size_of::<MownStr<'static>>(),
            std::mem::size_of::<&'static str>(),
        );
    }

    #[test]
    fn niche() {
        assert_eq!(
            std::mem::size_of::<MownStr<'static>>(),
            std::mem::size_of::<Option<MownStr<'static>>>(),
        );
    }

    #[test]
    fn test_build_borrowed_empty() {
        let mown: MownStr = "".into();
        assert!(mown.is_borrowed());
        assert_eq!(mown, "");
    }

    #[test]
    fn test_build_borrowed() {
        let mown: MownStr = "hello".into();
        assert!(mown.is_borrowed());
    }

    #[test]
    fn test_build_owned_from_box() {
        let bx: Box<str> = "hello".into();
        let mown: MownStr = bx.into();
        assert!(mown.is_owned());
    }

    #[test]
    fn test_build_owned_from_string() {
        let mown: MownStr = "hello".to_string().into();
        assert!(mown.is_owned());
    }

    #[test]
    fn test_build_borrowed_from_cow() {
        let mown: MownStr = Cow::Borrowed("hello").into();
        assert!(mown.is_borrowed());
    }

    #[test]
    fn test_build_owned_from_cow() {
        let mown: MownStr = Cow::<str>::Owned("hello".to_string()).into();
        assert!(mown.is_owned());
    }

    #[test]
    fn test_borrowed() {
        let mown1: MownStr = "hello".to_string().into();
        let mown2 = mown1.borrowed();
        assert!(mown2.is_borrowed());
        assert_eq!(mown1, mown2);
    }

    #[test]
    fn test_deref() {
        let txt = "hello";
        let mown1: MownStr = txt.into();
        assert_eq!(&*mown1, txt);
        assert_eq!(&mown1[..], txt);
        let mown2: MownStr = txt.to_string().into();
        assert_eq!(&*mown2, txt);
        assert_eq!(&mown2[..], txt);
    }

    #[test]
    fn test_hash() {
        let txt = "hello";
        let mown1: MownStr = txt.into();
        let mown2: MownStr = txt.to_string().into();

        let mut set = HashSet::new();
        set.insert(mown1.clone());
        assert!(set.contains(&mown1));
        assert!(set.contains(&mown2));
        assert!(set.contains(txt));

        let mut set = HashSet::new();
        set.insert(mown2.clone());
        assert!(set.contains(&mown1));
        assert!(set.contains(&mown2));
        assert!(set.contains(txt));
    }

    #[test]
    fn test_eq() {
        let txt = "hello";
        let mown1: MownStr = txt.into();
        let mown2: MownStr = txt.to_string().into();

        assert_eq!(mown1, txt);
        assert_eq!(mown1, mown1);
        assert_eq!(mown1, mown2);
        assert_eq!(mown2, txt);
        assert_eq!(mown2, mown1);
        assert_eq!(mown2, mown2);
        assert_eq!(txt, mown1);
        assert_eq!(txt, mown2);
    }

    #[test]
    fn test_order() {
        let txt = "hello";
        let mown1: MownStr = txt[..4].into();
        let mown2: MownStr = txt[..3].to_string().into();

        assert!(mown1 <= txt);
        assert!(mown1 <= mown1);
        assert!(mown1 >= mown2);
        assert!(mown2 <= txt);
        assert!(mown2 <= mown1);
        assert!(mown2 >= mown2);
        assert!(txt >= mown1);
        assert!(txt >= mown2);
    }

    #[test]
    fn test_display() {
        let mown1: MownStr = "hello".into();
        let mown2: MownStr = "hello".to_string().into();
        assert_eq!(format!("{:?}", mown1), "\"hello\"");
        assert_eq!(format!("{:?}", mown2), "\"hello\"");
        assert_eq!(format!("{}", mown1), "hello");
        assert_eq!(format!("{}", mown2), "hello");
    }

    #[test]
    fn no_double_free() {
        let bx = {
            let mown = MownStr::from("hello world".to_string());
            assert_eq!(&mown[..4], "hell");
            mown.to::<Box<str>>()
        };
        assert_eq!(&bx[..4], "hell");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn no_memory_leak() {
        // performs several MownStr allocation in sequence,
        // droping each one before allocating the next one
        // (unless the v.pop() line below is commented out).
        //
        // If there is no memory leak,
        // the increase in memory should be roughly 1 time the allocated size;
        // otherwise, it should be roghly 10 times that size.

        let m0 = get_rss_anon();
        println!("memory = {} kB", m0);
        let mut v = vec![];
        for i in 0..10 {
            v.pop(); // COMMENT THIS LINE OUT to simulate a memory leak
            let s = unsafe { String::from_utf8_unchecked(vec![b'a' + i; CAP]) };
            v.push(MownStr::from(s));
            println!(
                "{} MownStr(s) in the Vec, of len {}, starting with {:?}",
                v.len(),
                v[v.len() - 1].len(),
                &v[v.len() - 1][..2]
            );
        }
        let m1 = get_rss_anon();
        println!("memory = {} kB", m1);
        assert!(!v.is_empty()); // ensure that v is not optimized away to soon
        let increase = (m1 - m0) as f64 / (CAP / 1000) as f64;
        println!("increase = {}", increase);
        assert!(increase < 1.5);
    }

    #[test]
    fn empty_string() {
        let empty = "".to_string();
        let _ = MownStr::from(empty);
    }

    const CAP: usize = 100_000_000;

    fn get_rss_anon() -> usize {
        let txt = fs::read_to_string("/proc/self/status").expect("read proc status");
        let txt = txt.split("RssAnon:").nth(1).unwrap();
        let txt = txt.split(" kB").next().unwrap();
        let txt = txt.trim();
        usize::from_str(txt).unwrap()
    }
}
