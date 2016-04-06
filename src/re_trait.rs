// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// Slot is a single saved capture location. Note that there are two slots for
/// every capture in a regular expression (one slot each for the start and end
/// of the capture).
pub type Slot = Option<usize>;

/// RegularExpression describes types that can implement regex searching.
///
/// This trait is my attempt at reducing code duplication and to standardize
/// the internal API. Specific duplication that is avoided are the `find`
/// and `capture` iterators, which are slightly tricky.
///
/// It's not clear whether this trait is worth it, and it also isn't
/// clear whether it's useful as a public trait or not. Methods like
/// `next_after_empty` reak of bad design, but the rest of the methods seem
/// somewhat reasonable. One particular thing this trait would expose would be
/// the ability to start the search of a regex anywhere in a haystack, which
/// isn't possible in the current public API.
pub trait RegularExpression: Sized {
    type Text: ?Sized;

    fn slots_len(&self) -> usize;

    fn next_after_empty(&self, text: &Self::Text, i: usize) -> usize;

    fn shortest_match_at(
        &self,
        text: &Self::Text,
        start: usize,
    ) -> Option<usize>;

    fn is_match_at(
        &self,
        text: &Self::Text,
        start: usize,
    ) -> bool;

    fn find_at(
        &self,
        text: &Self::Text,
        start: usize,
    ) -> Option<(usize, usize)>;

    fn captures_at(
        &self,
        slots: &mut [Slot],
        text: &Self::Text,
        start: usize,
    ) -> Option<(usize, usize)>;

    fn find_iter<'t>(
        self,
        text: &'t Self::Text,
    ) -> FindMatches<'t, Self> {
        FindMatches {
            re: self,
            text: text,
            last_end: 0,
            last_match: None,
        }
    }

    fn captures_iter<'t>(
        self,
        text: &'t Self::Text,
    ) -> FindCaptures<'t, Self> {
        FindCaptures(self.find_iter(text))
    }
}

pub struct FindMatches<'t, R> where R: RegularExpression, R::Text: 't {
    re: R,
    text: &'t R::Text,
    last_end: usize,
    last_match: Option<usize>,
}

impl<'t, R> FindMatches<'t, R> where R: RegularExpression, R::Text: 't {
    pub fn text(&self) -> &'t R::Text {
        self.text
    }

    pub fn regex(&self) -> &R {
        &self.re
    }
}

impl<'t, R> Iterator for FindMatches<'t, R>
        where R: RegularExpression, R::Text: 't + AsRef<[u8]> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        let text_len = self.text.as_ref().len();
        if self.last_end > text_len {
            return None
        }
        let (s, e) = match self.re.find_at(self.text, self.last_end) {
            None => return None,
            Some((s, e)) => (s, e),
        };
        // Don't accept empty matches immediately following a match.
        // i.e., no infinite loops please.
        if e == s && Some(self.last_end) == self.last_match {
            if self.last_end >= text_len {
                return None;
            }
            self.last_end = self.re.next_after_empty(
                &self.text, self.last_end);
            return self.next();
        }
        self.last_end = e;
        self.last_match = Some(self.last_end);
        Some((s, e))
    }
}

pub struct FindCaptures<'t, R>(FindMatches<'t, R>)
    where R: RegularExpression, R::Text: 't;

impl<'t, R> FindCaptures<'t, R> where R: RegularExpression, R::Text: 't {
    pub fn text(&self) -> &'t R::Text {
        self.0.text()
    }

    pub fn regex(&self) -> &R {
        self.0.regex()
    }
}

impl<'t, R> Iterator for FindCaptures<'t, R>
        where R: RegularExpression, R::Text: 't + AsRef<[u8]> {
    type Item = Vec<Slot>;

    fn next(&mut self) -> Option<Vec<Slot>> {
        let text_len = self.0.text.as_ref().len();
        if self.0.last_end > text_len {
            return None
        }

        let mut slots = vec![None; self.0.re.slots_len()];
        let (s, e) = match self.0.re.captures_at(
            &mut slots,
            self.0.text,
            self.0.last_end,
        ) {
            None => return None,
            Some((s, e)) => (s, e),
        };

        // Don't accept empty matches immediately following a match.
        // i.e., no infinite loops please.
        if e == s && Some(self.0.last_end) == self.0.last_match {
            if self.0.last_end >= text_len {
                return None;
            }
            self.0.last_end = self.0.re.next_after_empty(
                &self.0.text, self.0.last_end);
            return self.next();
        }
        self.0.last_end = e;
        self.0.last_match = Some(self.0.last_end);
        Some(slots)
    }
}
