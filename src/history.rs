use std::{array, iter::Take, mem};

// NOTE maybe at some point it makes sense to again work with String or some adapted version of it

/// A structure storing a command history
#[derive(Debug, PartialEq, PartialOrd)]
pub struct History<const N: usize> {
    len: usize,
    /// Last read value, counted from the end, none if on a clean line
    cur: Option<usize>,
    stored_commands: [Vec<char>; N],
}

impl<const N: usize> History<N> {
    /// The capacity of this History
    pub const CAPACITY: usize = N;

    /// Create an empty history
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a history containing the first [Self::CAPACITY] elements of the given iterator.
    pub fn with_initial(initial: impl IntoIterator<Item = Vec<char>>) -> Self {
        let mut me = Self::new();
        let iter = initial.into_iter();
        for (slot, s) in me.stored_commands.iter_mut().zip(iter) {
            *slot = s;
        }

        me
    }

    /// Push entry to the end of the history, removing the oldest entry if the capacity is reached
    ///
    /// This resets the current index which means that the next current returns the newest entry.
    pub fn push(&mut self, command: Vec<char>) {
        if command.len() == 0 || command.iter().copied().all(char::is_whitespace) {
            return;
        }

        self.cur = None;
        if self.len == N {
            self.stored_commands.rotate_left(1);
            self.stored_commands[N - 1] = command;
        } else if self.len < N {
            self.stored_commands[self.len] = command;
            self.len += 1;
        } else {
            unreachable!()
        }
    }

    /// Pop the newest entry of the history
    pub fn pop(&mut self) -> Option<Vec<char>> {
        self.cur = None;
        // NOTE Right side of or should not happen, prevents compiler from generatingpanicing path
        if self.len == 0 || self.len > N {
            None
        } else {
            self.len -= 1;
            let val = mem::take(&mut self.stored_commands[self.len]);
            Some(val)
        }
    }

    pub fn get(&self, idx: usize) -> Option<&[char]> {
        if idx >= self.len {
            None
        } else {
            self.stored_commands.get(idx).map(|s| s.as_ref())
        }
    }

    /// Get a reference to the newest element
    pub fn newest(&self) -> Option<&[char]> {
        if self.len == 0 {
            None
        } else {
            self.get(self.len - 1)
        }
    }

    /// Get a reference to the current element
    pub fn current(&self) -> Option<&[char]> {
        if self.len == 0 {
            return None;
        }

        if let Some(cur) = self.cur {
            let idx = self.len.saturating_sub(cur + 1);
            self.get(idx)
        } else {
            None
        }
    }

    /// Get the previous element and move the current pointer to the previous element
    pub fn prev(&mut self) -> Option<&[char]> {
        if let Some(cur) = self.cur.as_mut() {
            if *cur + 1 < self.len {
                *cur += 1;
            }
        } else {
            self.cur = Some(0);
        }

        self.current()
    }

    /// Get the previous element and move the current pointer to the previous element
    pub fn next(&mut self) -> Option<&[char]> {
        if let Some(0) = self.cur {
            self.cur = None;
            return None;
        }

        if let Some(cur) = self.cur.as_mut() {
            *cur -= 1;
            self.current()
        } else {
            Some(&[])
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &[char]> {
        self.stored_commands
            .iter()
            .map(|s| s.as_slice())
            .take(self.len)
    }
}

impl<const N: usize> IntoIterator for History<N> {
    type Item = Vec<char>;
    type IntoIter = Take<array::IntoIter<Vec<char>, N>>;

    fn into_iter(self) -> Self::IntoIter {
        self.stored_commands.into_iter().take(self.len)
    }
}

impl<const N: usize> Default for History<N> {
    fn default() -> Self {
        Self {
            cur: None,
            len: 0,
            stored_commands: [(); N].map(|_| Vec::new()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    trait ToCharVec {
        fn to_char_vec(self) -> Vec<char>;
    }

    impl ToCharVec for &str {
        fn to_char_vec(self) -> Vec<char> {
            self.chars().collect()
        }
    }

    #[test]
    fn push_pop0() {
        let mut history = History::<32>::new();
        history.push("Hello".to_char_vec());
        assert_eq!(history.pop(), Some("Hello".to_char_vec()))
    }

    #[test]
    fn push_pop1() {
        let mut history = History::<32>::new();
        history.push("Hello".to_char_vec());
        history.push("World".to_char_vec());
        assert_eq!(history.pop(), Some("World".to_char_vec()));
        assert_eq!(history.pop(), Some("Hello".to_char_vec()));
    }

    #[test]
    fn contains_all() {
        let mut history = History::<32>::new();
        history.push("Hello".to_char_vec());
        history.push("World".to_char_vec());
        history.push("!".to_char_vec());
        assert_eq!(
            &history.into_iter().collect::<Vec<_>>(),
            &["Hello".to_char_vec(), "World".to_char_vec(), "!".to_char_vec()]
        )
    }

    #[test]
    fn contains_all_reached_cap() {
        let mut history = History::<2>::new();
        history.push("Hello".to_char_vec());
        history.push("World".to_char_vec());
        history.push("!".to_char_vec());
        assert_eq!(
            &history.into_iter().collect::<Vec<_>>(),
            &["World".to_char_vec(), "!".to_char_vec()]
        )
    }

    #[test]
    fn navigating0() {
        let mut history = History::<32>::new();
        history.push("Hello".to_char_vec());
        assert_eq!(history.current(), None);
        assert_eq!(history.prev(), Some("Hello".to_char_vec().as_slice()));
        assert_eq!(history.current(), Some("Hello".to_char_vec().as_slice()));
        history.push("World".to_char_vec());
        assert_eq!(history.current(), None);
        assert_eq!(history.prev(), Some("World".to_char_vec().as_slice()));
        assert_eq!(history.current(), Some("World".to_char_vec().as_slice()));
        assert_eq!(history.prev(), Some("Hello".to_char_vec().as_slice()));
        assert_eq!(history.current(), Some("Hello".to_char_vec().as_slice()));
        assert_eq!(history.next(), Some("World".to_char_vec().as_slice()));
        assert_eq!(history.next(), None);
    }

    #[test]
    fn navigating1() {
        let mut history = History::<32>::new();
        history.push("Hello".to_char_vec());
        assert_eq!(history.prev(), Some("Hello".to_char_vec().as_slice()));
        assert_eq!(history.next(), None);
    }
}
