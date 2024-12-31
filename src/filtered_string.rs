use std::fmt;

use bevy::utils::HashMap;

/// A wrapper type for filtering characters from a string based on a character
/// map.
///
/// This type allows you to filter out characters from a string that do not
/// exist in the given `atlas_character_map`. It provides a reusable abstraction
/// for working with filtered strings efficiently, avoiding unnecessary
/// allocations.
#[derive(Debug)]
pub(crate) struct FilteredString<'s, S: AsRef<str>> {
    string: S,
    atlas_character_map: &'s HashMap<char, usize>,
}

impl<'s, S: AsRef<str>> FilteredString<'s, S> {
    /// Creates a new `FilteredString` instance.
    ///
    /// # Parameters
    /// - `string`: The input string to be filtered.
    /// - `atlas_character_map`: A reference to a character map that determines
    ///   which characters are retained.
    ///
    /// # Returns
    /// A `FilteredString` instance that can produce iterators over the filtered
    /// characters.
    pub(crate) fn new(string: S, atlas_character_map: &'s HashMap<char, usize>) -> Self {
        Self {
            string,
            atlas_character_map,
        }
    }

    /// Returns an iterator over the filtered characters.
    ///
    /// This method filters the input string to include only characters that
    /// exist in the `atlas_character_map`.
    ///
    /// # Returns
    /// An iterator that yields characters retained by the filter.
    pub(crate) fn filtered_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.string
            .as_ref()
            .chars()
            .filter(|c| self.atlas_character_map.contains_key(c))
    }

    /// Checks if the filtered string is empty.
    ///
    /// # Returns
    /// `true` if there are no characters in the filtered string; otherwise,
    /// `false`.
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.filtered_chars().next().is_none()
    }
}

impl<S: AsRef<str>> fmt::Display for FilteredString<'_, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.filtered_chars() {
            write!(f, "{c}")?;
        }
        Ok(())
    }
}
