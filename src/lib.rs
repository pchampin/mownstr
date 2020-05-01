//! [`MownStr`](./enum.MownStr.html)
//! is either a borrowed reference to a `str` or an own `Box<str>`.

use std::borrow::Cow;
use std::fmt;
use std::hash;
use std::ops::Deref;
use std::slice;
use std::str;

/// "Maybe own str":Option
/// either a borrowed reference to a `str` or an own `Box<str>`.
///
/// It does not try to be mutable, nor generic,
/// which makes it lighter than, for example, `Cow<str>`.
///
/// # Panic
/// The drawback is that `MownStr`
/// does not support strings with a length > `usize::MAX/2`.
/// Trying to convert such a large string to a `MownStr` will panic.
pub struct MownStr<'a>(&'a str);

const LEN_MASK: usize = usize::MAX >> 1;
const OWN_FLAG: usize = !LEN_MASK;

impl<'a> MownStr<'a> {
    pub fn is_borrowed(&self) -> bool {
        (self.0.len() & OWN_FLAG) == 0
    }

    pub fn is_owned(&self) -> bool {
        (self.0.len() & OWN_FLAG) == OWN_FLAG
    }

    #[inline]
    fn real_len(&self) -> usize {
        self.0.len() & LEN_MASK
    }

    #[inline]
    unsafe fn into_ref(self) -> &'a str {
        debug_assert!(self.is_borrowed(), "into_ref() called on owned MownStr");
        let ptr = self.0.as_ptr();
        let len = self.real_len();
        let slice = slice::from_raw_parts(ptr, len);
        str::from_utf8_unchecked(slice)
    }

    #[inline]
    unsafe fn extract_box(&mut self) -> Box<str> {
        debug_assert!(self.is_owned(), "extract_box() called on borrowed MownStr");
        // extract data to make box
        #[allow(clippy::cast_ref_to_mut)]
        let mut_ref: &mut str = &mut *(self.0 as *const str as *mut str);
        let ptr = mut_ref.as_mut_ptr();
        let len = self.real_len();
        // turn to borrowed, to avoid double-free
        self.0 = "";
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
            MownStr(self.0)
        }
    }
}

// Construct a MownStr

impl<'a> From<&'a str> for MownStr<'a> {
    fn from(other: &'a str) -> MownStr<'a> {
        let len = other.len();
        assert!(len <= LEN_MASK);
        let ptr = other.as_ptr();

        let my_ref = unsafe {
            let slice = slice::from_raw_parts(ptr, len & LEN_MASK);
            str::from_utf8_unchecked(slice)
        };
        MownStr(my_ref)
    }
}

impl<'a> From<Box<str>> for MownStr<'a> {
    fn from(other: Box<str>) -> MownStr<'a> {
        let len = other.len();
        assert!(len <= LEN_MASK);
        let ptr = other.as_ptr();

        std::mem::forget(other);

        let my_ref = unsafe {
            let slice = slice::from_raw_parts(ptr, len | OWN_FLAG);
            str::from_utf8_unchecked(slice)
        };
        MownStr(my_ref)
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
        let ptr = self.0.as_ptr();
        let len = self.real_len();
        unsafe {
            let slice = slice::from_raw_parts(ptr, len);
            str::from_utf8_unchecked(slice)
        }
    }
}

impl<'a> AsRef<str> for MownStr<'a> {
    fn as_ref(&self) -> &str {
        &*self
    }
}

impl<'a> std::borrow::Borrow<str> for MownStr<'a> {
    fn borrow(&self) -> &str {
        &*self
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
            unsafe { other.into_ref() }.into()
        }
    }
}

impl<'a> MownStr<'a> {
    /// Convert this `MownStr` to ant type `T`
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
            unsafe { self.into_ref() }.into()
        }
    }
}

#[cfg(test)]
mod test {
    use super::MownStr;
    use std::borrow::Cow;
    use std::collections::HashSet;

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
}
