pub mod suffix_table {
    use std::borrow::Cow;
    use std::fmt;
    use std::iter;
    use std::slice;
    use std::u32;

    use self::SuffixType::{Ascending, Descending, Valley};

    #[derive(Clone, Eq, PartialEq)]
    pub struct SuffixTable<'s, 't> {
        text: Cow<'s, str>,
        table: Cow<'t, [u32]>,
    }

    impl<'s, 't> SuffixTable<'s, 't> {
        /// Creates a new suffix table for `text` in `O(n)` time and `O(kn)`
        /// space, where `k` is the size of the alphabet in the text.
        ///
        /// The table stores either `S` or a `&S` and a lexicographically sorted
        /// list of suffixes. Each suffix is represented by a 32 bit integer and
        /// is a **byte index** into `text`.
        pub fn new<S>(text: S) -> SuffixTable<'s, 't>
        where
            S: Into<Cow<'s, str>>,
        {
            let text = text.into();
            let table = Cow::Owned(sais_table(&text));
            SuffixTable {
                text: text,
                table: table,
            }
        }

        /// Creates a new suffix table from an existing list of lexicographically
        /// sorted suffix indices.
        ///
        /// Note that the invariant that `table` must be a suffix table of `text`
        /// is not checked! If it isn't, this will cause other operations on a
        /// suffix table to fail in weird ways.
        ///
        /// This fails if the number of characters in `text` does not equal the
        /// number of suffixes in `table`.
        pub fn from_parts<S, T>(text: S, table: T) -> SuffixTable<'s, 't>
        where
            S: Into<Cow<'s, str>>,
            T: Into<Cow<'t, [u32]>>,
        {
            let (text, table) = (text.into(), table.into());
            assert_eq!(text.len(), table.len());
            SuffixTable {
                text: text,
                table: table,
            }
        }

        /// Extract the parts of a suffix table.
        ///
        /// This is useful to avoid copying when the suffix table is part of an
        /// intermediate computation.
        pub fn into_parts(self) -> (Cow<'s, str>, Cow<'t, [u32]>) {
            (self.text, self.table)
        }

        /// Computes the LCP array.
        pub fn lcp_lens(&self) -> Vec<u32> {
            let mut inverse = vec![0u32; self.text.len()];
            for (rank, &sufstart) in self.table().iter().enumerate() {
                inverse[sufstart as usize] = rank as u32;
            }
            lcp_lens_quadratic(self.text(), self.table())
            // Broken on Unicode text for now. ---AG
            // lcp_lens_linear(self.text(), self.table(), &inverse)
        }

        /// Return the suffix table.
        #[inline]
        pub fn table(&self) -> &[u32] {
            &self.table
        }

        /// Return the text.
        #[inline]
        pub fn text(&self) -> &str {
            &self.text
        }

        /// Returns the number of suffixes in the table.
        ///
        /// Alternatively, this is the number of *bytes* in the text.
        #[inline]
        pub fn len(&self) -> usize {
            self.table.len()
        }

        /// Returns `true` iff `self.len() == 0`.
        #[inline]
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }

        /// Returns the suffix at index `i`.
        #[inline]
        pub fn suffix(&self, i: usize) -> &str {
            &self.text[self.table[i] as usize..]
        }

        /// Returns the suffix bytes starting at index `i`.
        #[inline]
        pub fn suffix_bytes(&self, i: usize) -> &[u8] {
            &self.text.as_bytes()[self.table[i] as usize..]
        }

        /// Returns true if and only if `query` is in text.
        ///
        /// This runs in `O(mlogn)` time, where `m == query.len()` and
        /// `n == self.len()`. (As far as this author knows, this is the best known
        /// bound for a plain suffix table.)
        ///
        /// You should prefer this over `positions` when you only need to test
        /// existence (because it is faster).
        ///
        /// # Example
        ///
        /// Build a suffix array of some text and test existence of a substring:
        ///
        /// ```rust
        /// use suffix::SuffixTable;
        ///
        /// let sa = SuffixTable::new("The quick brown fox.");
        /// assert!(sa.contains("quick"));
        /// ```
        pub fn contains(&self, query: &str) -> bool {
            self.any_position(query).is_some()
        }

        /// Returns an unordered list of positions where `query` starts in `text`.
        ///
        /// This runs in `O(mlogn)` time, where `m == query.len()` and
        /// `n == self.len()`. (As far as this author knows, this is the best known
        /// bound for a plain suffix table.)
        ///
        /// Positions are byte indices into `text`.
        ///
        /// If you just need to test existence, then use `contains` since it is
        /// faster.
        ///
        /// # Example
        ///
        /// Build a suffix array of some text and find all occurrences of a
        /// substring:
        ///
        /// ```rust
        /// use suffix::SuffixTable;
        ///
        /// let sa = SuffixTable::new("The quick brown fox was very quick.");
        /// assert_eq!(sa.positions("quick"), &[4, 29]);
        /// ```
        pub fn positions(&self, query: &str) -> &[u32] {
            let (text, query) = (self.text.as_bytes(), query.as_bytes());

            // We can quickly decide whether the query won't match at all if
            // it's outside the range of suffixes.
            if text.len() == 0
                || query.len() == 0
                || (query < self.suffix_bytes(0) && !self.suffix_bytes(0).starts_with(query))
                || query > self.suffix_bytes(self.len() - 1)
            {
                return &[];
            }

            // The below is pretty close to the algorithm on Wikipedia:
            //
            //     http://en.wikipedia.org/wiki/Suffix_array#Applications
            //
            // The key difference is that after we find the start index, we look
            // for the end by finding the first occurrence that doesn't start
            // with `query`. That becomes our upper bound.
            let start = binary_search(&self.table, |&sufi| query <= &text[sufi as usize..]);
            let end = start
                + binary_search(&self.table[start..], |&sufi| {
                    !text[sufi as usize..].starts_with(query)
                });

            // Whoops. If start is somehow greater than end, then we've got
            // nothing.
            if start > end {
                &[]
            } else {
                &self.table[start..end]
            }
        }

        /// Returns an arbitrary one of the positions where `query` starts in
        /// `text`.
        ///
        /// This runs in `O(mlogn)` time just like [`contains`][Self::contains] and
        /// [`positions`][Self::positions], but the constant factor is that of
        /// `contains`, which is the more efficient of the two.
        ///
        /// The return value is a byte indices into `text`.
        ///
        /// # Example
        ///
        /// ```
        /// use suffix::SuffixTable;
        ///
        /// let sa = SuffixTable::new("The quick brown fox was very quick.");
        /// let position = sa.any_position("quick");
        /// assert!(position == Some(4) || position == Some(29));
        /// ```
        pub fn any_position(&self, query: &str) -> Option<u32> {
            let (text, query) = (self.text.as_bytes(), query.as_bytes());
            if query.len() == 0 {
                return None;
            }
            self.table
                .binary_search_by(|&sufi| {
                    text[sufi as usize..]
                        .iter()
                        .take(query.len())
                        .cmp(query.iter())
                })
                .ok()
                .map(|i| self.table[i])
        }
    }

    impl<'s, 't> fmt::Debug for SuffixTable<'s, 't> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            writeln!(f, "\n-----------------------------------------")?;
            writeln!(f, "SUFFIX TABLE")?;
            writeln!(f, "text: {}", self.text())?;
            for (rank, &sufstart) in self.table.iter().enumerate() {
                writeln!(f, "suffix[{}] {}, {}", rank, sufstart, self.suffix(rank))?;
            }
            writeln!(f, "-----------------------------------------")
        }
    }

    // #[allow(dead_code)]
    // fn lcp_lens_linear(text: &str, table: &[u32], inv: &[u32]) -> Vec<u32> {
    // // This algorithm is bunk because it doesn't work on Unicode. See comment
    // // in the code below.
    //
    // // This is a linear time construction algorithm taken from the first
    // // two slides of:
    // // http://www.cs.helsinki.fi/u/tpkarkka/opetus/11s/spa/lecture10.pdf
    // //
    // // It does require the use of the inverse suffix array, which makes this
    // // O(n) in space. The inverse suffix array gives us a special ordering
    // // with which to compute the LCPs.
    // let mut lcps = vec![0u32; table.len()];
    // let mut len = 0u32;
    // for (sufi2, &rank) in inv.iter().enumerate() {
    // if rank == 0 {
    // continue
    // }
    // let sufi1 = table[(rank - 1) as usize];
    // len += lcp_len(&text[(sufi1 + len) as usize..],
    // &text[(sufi2 as u32 + len) as usize..]);
    // lcps[rank as usize] = len;
    // if len > 0 {
    // // This is an illegal move because `len` is derived from `text`,
    // // which is a Unicode string. Subtracting `1` here assumes every
    // // character is a single byte in UTF-8, which is obviously wrong.
    // // TODO: Figure out how to get LCP lengths in linear time on
    // // UTF-8 encoded strings.
    // len -= 1;
    // }
    // }
    // lcps
    // }

    fn lcp_lens_quadratic(text: &str, table: &[u32]) -> Vec<u32> {
        // This is quadratic because there are N comparisons for each LCP.
        // But it is done in constant space.

        // The first LCP is always 0 because of the definition:
        //   LCP_LENS[i] = lcp_len(suf[i-1], suf[i])
        let mut lcps = vec![0u32; table.len()];
        let text = text.as_bytes();
        for (i, win) in table.windows(2).enumerate() {
            lcps[i + 1] = lcp_len(&text[win[0] as usize..], &text[win[1] as usize..]);
        }
        lcps
    }

    fn lcp_len(a: &[u8], b: &[u8]) -> u32 {
        a.iter()
            .zip(b.iter())
            .take_while(|(ca, cb)| ca == cb)
            .count() as u32
    }

    fn sais_table<'s>(text: &'s str) -> Vec<u32> {
        let text = text.as_bytes();
        assert!(text.len() <= u32::MAX as usize);
        let mut sa = vec![0u32; text.len()];
        let mut stypes = SuffixTypes::new(text.len() as u32);
        let mut bins = Bins::new();
        sais(&mut *sa, &mut stypes, &mut bins, &Utf8(text));
        sa
    }

    fn sais<T>(sa: &mut [u32], stypes: &mut SuffixTypes, bins: &mut Bins, text: &T)
    where
        T: Text,
        <<T as Text>::IdxChars as Iterator>::Item: IdxChar,
    {
        // Instead of working out edge cases in the code below, just allow them
        // to assume >=2 characters.
        match text.len() {
            0 => return,
            1 => {
                sa[0] = 0;
                return;
            }
            _ => {}
        }

        for v in sa.iter_mut() {
            *v = 0;
        }
        stypes.compute(text);
        bins.find_sizes(text.char_indices().map(|c| c.idx_char().1));
        bins.find_tail_pointers();

        // Insert the valley suffixes.
        for (i, c) in text.char_indices().map(|v| v.idx_char()) {
            if stypes.is_valley(i as u32) {
                bins.tail_insert(sa, i as u32, c);
            }
        }

        // Now find the start of each bin.
        bins.find_head_pointers();

        // Insert the descending suffixes.
        let (lasti, lastc) = text.prev(text.len());
        if stypes.is_desc(lasti) {
            bins.head_insert(sa, lasti, lastc);
        }
        for i in 0..sa.len() {
            let sufi = sa[i];
            if sufi > 0 {
                let (lasti, lastc) = text.prev(sufi);
                if stypes.is_desc(lasti) {
                    bins.head_insert(sa, lasti, lastc);
                }
            }
        }

        // ... and the find the end of each bin.
        bins.find_tail_pointers();

        // Insert the ascending suffixes.
        for i in (0..sa.len()).rev() {
            let sufi = sa[i];
            if sufi > 0 {
                let (lasti, lastc) = text.prev(sufi);
                if stypes.is_asc(lasti) {
                    bins.tail_insert(sa, lasti, lastc);
                }
            }
        }

        // Find and move all wstrings to the beginning of `sa`.
        let mut num_wstrs = 0u32;
        for i in 0..sa.len() {
            let sufi = sa[i];
            if stypes.is_valley(sufi) {
                sa[num_wstrs as usize] = sufi;
                num_wstrs += 1;
            }
        }
        // This check is necessary because we don't have a sentinel, which would
        // normally guarantee at least one wstring.
        if num_wstrs == 0 {
            num_wstrs = 1;
        }

        let mut prev_sufi = 0u32; // the first suffix can never be a valley
        let mut name = 0u32;
        // We set our "name buffer" to be max u32 values. Since there are at
        // most n/2 wstrings, a name can never be greater than n/2.
        for i in num_wstrs..(sa.len() as u32) {
            sa[i as usize] = u32::MAX;
        }
        for i in 0..num_wstrs {
            let cur_sufi = sa[i as usize];
            if prev_sufi == 0 || !text.wstring_equal(stypes, cur_sufi, prev_sufi) {
                name += 1;
                prev_sufi = cur_sufi;
            }
            // This divide-by-2 trick only works because it's impossible to have
            // two wstrings start at adjacent locations (they must at least be
            // separated by a single descending character).
            sa[(num_wstrs + (cur_sufi / 2)) as usize] = name - 1;
        }

        // We've inserted the lexical names into the latter half of the suffix
        // array, but it's sparse. so let's smush them all up to the end.
        let mut j = sa.len() as u32 - 1;
        for i in (num_wstrs..(sa.len() as u32)).rev() {
            if sa[i as usize] != u32::MAX {
                sa[j as usize] = sa[i as usize];
                j -= 1;
            }
        }

        // If we have fewer names than wstrings, then there are at least 2
        // equivalent wstrings, which means we need to recurse and sort them.
        if name < num_wstrs {
            let split_at = sa.len() - (num_wstrs as usize);
            let (r_sa, r_text) = sa.split_at_mut(split_at);
            sais(
                &mut r_sa[..num_wstrs as usize],
                stypes,
                bins,
                &LexNames(r_text),
            );
            stypes.compute(text);
        } else {
            for i in 0..num_wstrs {
                let reducedi = sa[((sa.len() as u32) - num_wstrs + i) as usize];
                sa[reducedi as usize] = i;
            }
        }

        // Re-calibrate the bins by finding their sizes and the end of each bin.
        bins.find_sizes(text.char_indices().map(|c| c.idx_char().1));
        bins.find_tail_pointers();

        // Replace the lexical names with their corresponding suffix index in the
        // original text.
        let mut j = sa.len() - (num_wstrs as usize);
        for (i, _) in text.char_indices().map(|v| v.idx_char()) {
            if stypes.is_valley(i as u32) {
                sa[j] = i as u32;
                j += 1;
            }
        }
        // And now map the suffix indices from the reduced text to suffix
        // indices in the original text. Remember, `sa[i]` yields a lexical name.
        // So all we have to do is get the suffix index of the original text for
        // that lexical name (which was made possible in the loop above).
        //
        // In other words, this sets the suffix indices of only the wstrings.
        for i in 0..num_wstrs {
            let sufi = sa[i as usize];
            sa[i as usize] = sa[(sa.len() as u32 - num_wstrs + sufi) as usize];
        }
        // Now zero out everything after the wstrs.
        for i in num_wstrs..(sa.len() as u32) {
            sa[i as usize] = 0;
        }

        // Insert the valley suffixes and zero out everything else..
        for i in (0..num_wstrs).rev() {
            let sufi = sa[i as usize];
            sa[i as usize] = 0;
            bins.tail_insert(sa, sufi, text.char_at(sufi));
        }

        // Now find the start of each bin.
        bins.find_head_pointers();

        // Insert the descending suffixes.
        let (lasti, lastc) = text.prev(text.len());
        if stypes.is_desc(lasti) {
            bins.head_insert(sa, lasti, lastc);
        }
        for i in 0..sa.len() {
            let sufi = sa[i];
            if sufi > 0 {
                let (lasti, lastc) = text.prev(sufi);
                if stypes.is_desc(lasti) {
                    bins.head_insert(sa, lasti, lastc);
                }
            }
        }

        // ... and find the end of each bin again.
        bins.find_tail_pointers();

        // Insert the ascending suffixes.
        for i in (0..sa.len()).rev() {
            let sufi = sa[i];
            if sufi > 0 {
                let (lasti, lastc) = text.prev(sufi);
                if stypes.is_asc(lasti) {
                    bins.tail_insert(sa, lasti, lastc);
                }
            }
        }
    }

    struct SuffixTypes {
        types: Vec<SuffixType>,
    }

    #[derive(Clone, Copy, Debug, Eq)]
    enum SuffixType {
        Ascending,
        Descending,
        Valley,
    }

    impl SuffixTypes {
        fn new(num_bytes: u32) -> SuffixTypes {
            SuffixTypes {
                types: vec![SuffixType::Ascending; num_bytes as usize],
            }
        }

        fn compute<'a, T>(&mut self, text: &T)
        where
            T: Text,
            <<T as Text>::IdxChars as Iterator>::Item: IdxChar,
        {
            let mut chars = text.char_indices().map(|v| v.idx_char()).rev();
            let (mut lasti, mut lastc) = match chars.next() {
                None => return,
                Some(t) => t,
            };
            self.types[lasti] = Descending;
            for (i, c) in chars {
                if c < lastc {
                    self.types[i] = Ascending;
                } else if c > lastc {
                    self.types[i] = Descending;
                } else {
                    self.types[i] = self.types[lasti].inherit();
                }
                if self.types[i].is_desc() && self.types[lasti].is_asc() {
                    self.types[lasti] = Valley;
                }
                lastc = c;
                lasti = i;
            }
        }

        #[inline]
        fn ty(&self, i: u32) -> SuffixType {
            self.types[i as usize]
        }
        #[inline]
        fn is_asc(&self, i: u32) -> bool {
            self.ty(i).is_asc()
        }
        #[inline]
        fn is_desc(&self, i: u32) -> bool {
            self.ty(i).is_desc()
        }
        #[inline]
        fn is_valley(&self, i: u32) -> bool {
            self.ty(i).is_valley()
        }
        #[inline]
        fn equal(&self, i: u32, j: u32) -> bool {
            self.ty(i) == self.ty(j)
        }
    }

    impl SuffixType {
        #[inline]
        fn is_asc(&self) -> bool {
            match *self {
                Ascending | Valley => true,
                _ => false,
            }
        }

        #[inline]
        fn is_desc(&self) -> bool {
            if let Descending = *self {
                true
            } else {
                false
            }
        }

        #[inline]
        fn is_valley(&self) -> bool {
            if let Valley = *self {
                true
            } else {
                false
            }
        }

        fn inherit(&self) -> SuffixType {
            match *self {
                Valley => Ascending,
                _ => *self,
            }
        }
    }

    impl PartialEq for SuffixType {
        #[inline]
        fn eq(&self, other: &SuffixType) -> bool {
            (self.is_asc() && other.is_asc()) || (self.is_desc() && other.is_desc())
        }
    }

    struct Bins {
        alphas: Vec<u32>,
        sizes: Vec<u32>,
        ptrs: Vec<u32>,
    }

    impl Bins {
        fn new() -> Bins {
            Bins {
                alphas: Vec::with_capacity(10_000),
                sizes: Vec::with_capacity(10_000),
                ptrs: Vec::new(), // re-allocated later, no worries
            }
        }

        fn find_sizes<I>(&mut self, chars: I)
        where
            I: Iterator<Item = u32>,
        {
            self.alphas.clear();
            for size in self.sizes.iter_mut() {
                *size = 0;
            }
            for c in chars {
                self.inc_size(c);
                if self.size(c) == 1 {
                    self.alphas.push(c);
                }
            }
            self.alphas.sort();

            let ptrs_len = self.alphas[self.alphas.len() - 1] + 1;
            self.ptrs = vec![0u32; ptrs_len as usize];
        }

        fn find_head_pointers(&mut self) {
            let mut sum = 0u32;
            for &c in self.alphas.iter() {
                self.ptrs[c as usize] = sum;
                sum += self.size(c);
            }
        }

        fn find_tail_pointers(&mut self) {
            let mut sum = 0u32;
            for &c in self.alphas.iter() {
                sum += self.size(c);
                self.ptrs[c as usize] = sum - 1;
            }
        }

        #[inline]
        fn head_insert(&mut self, sa: &mut [u32], i: u32, c: u32) {
            let ptr = &mut self.ptrs[c as usize];
            sa[*ptr as usize] = i;
            *ptr += 1;
        }

        #[inline]
        fn tail_insert(&mut self, sa: &mut [u32], i: u32, c: u32) {
            let ptr = &mut self.ptrs[c as usize];
            sa[*ptr as usize] = i;
            if *ptr > 0 {
                *ptr -= 1;
            }
        }

        #[inline]
        fn inc_size(&mut self, c: u32) {
            if c as usize >= self.sizes.len() {
                self.sizes.resize(1 + (c as usize), 0);
            }
            self.sizes[c as usize] += 1;
        }

        #[inline]
        fn size(&self, c: u32) -> u32 {
            self.sizes[c as usize]
        }
    }

    /// Encapsulates iteration and indexing over text.
    ///
    /// This enables us to expose a common interface between a `String` and
    /// a `Vec<u32>`. Specifically, a `Vec<u32>` is used for lexical renaming.
    trait Text {
        /// An iterator over characters.
        ///
        /// Must be reversible.
        type IdxChars: Iterator + DoubleEndedIterator;

        /// The length of the text.
        fn len(&self) -> u32;

        /// The character previous to the byte index `i`.
        fn prev(&self, i: u32) -> (u32, u32);

        /// The character at byte index `i`.
        fn char_at(&self, i: u32) -> u32;

        /// An iterator over characters tagged with their byte offsets.
        fn char_indices(&self) -> Self::IdxChars;

        /// Compare two strings at byte indices `w1` and `w2`.
        fn wstring_equal(&self, stypes: &SuffixTypes, w1: u32, w2: u32) -> bool;
    }

    struct Utf8<'s>(&'s [u8]);

    impl<'s> Text for Utf8<'s> {
        type IdxChars = iter::Enumerate<slice::Iter<'s, u8>>;

        #[inline]
        fn len(&self) -> u32 {
            self.0.len() as u32
        }

        #[inline]
        fn prev(&self, i: u32) -> (u32, u32) {
            (i - 1, self.0[i as usize - 1] as u32)
        }

        #[inline]
        fn char_at(&self, i: u32) -> u32 {
            self.0[i as usize] as u32
        }

        fn char_indices(&self) -> iter::Enumerate<slice::Iter<'s, u8>> {
            self.0.iter().enumerate()
        }

        fn wstring_equal(&self, stypes: &SuffixTypes, w1: u32, w2: u32) -> bool {
            let w1chars = self.0[w1 as usize..].iter().enumerate();
            let w2chars = self.0[w2 as usize..].iter().enumerate();
            for ((i1, c1), (i2, c2)) in w1chars.zip(w2chars) {
                let (i1, i2) = (w1 + i1 as u32, w2 + i2 as u32);
                if c1 != c2 || !stypes.equal(i1, i2) {
                    return false;
                }
                if i1 > w1 && (stypes.is_valley(i1) || stypes.is_valley(i2)) {
                    return true;
                }
            }
            // At this point, we've exhausted either `w1` or `w2`, which means the
            // next character for one of them should be the sentinel. Since
            // `w1 != w2`, only one string can be exhausted. The sentinel is never
            // equal to another character, so we can conclude that the wstrings
            // are not equal.
            false
        }
    }

    struct LexNames<'s>(&'s [u32]);

    impl<'s> Text for LexNames<'s> {
        type IdxChars = iter::Enumerate<slice::Iter<'s, u32>>;

        #[inline]
        fn len(&self) -> u32 {
            self.0.len() as u32
        }

        #[inline]
        fn prev(&self, i: u32) -> (u32, u32) {
            (i - 1, self.0[i as usize - 1])
        }

        #[inline]
        fn char_at(&self, i: u32) -> u32 {
            self.0[i as usize]
        }

        fn char_indices(&self) -> iter::Enumerate<slice::Iter<'s, u32>> {
            self.0.iter().enumerate()
        }

        fn wstring_equal(&self, stypes: &SuffixTypes, w1: u32, w2: u32) -> bool {
            let w1chars = self.0[w1 as usize..].iter().enumerate();
            let w2chars = self.0[w2 as usize..].iter().enumerate();
            for ((i1, c1), (i2, c2)) in w1chars.zip(w2chars) {
                let (i1, i2) = (w1 + i1 as u32, w2 + i2 as u32);
                if c1 != c2 || !stypes.equal(i1, i2) {
                    return false;
                }
                if i1 > w1 && (stypes.is_valley(i1) || stypes.is_valley(i2)) {
                    return true;
                }
            }
            // At this point, we've exhausted either `w1` or `w2`, which means the
            // next character for one of them should be the sentinel. Since
            // `w1 != w2`, only one string can be exhausted. The sentinel is never
            // equal to another character, so we can conclude that the wstrings
            // are not equal.
            false
        }
    }

    /// A trait for converting indexed characters into a uniform representation.
    trait IdxChar {
        /// Convert `Self` to a `(usize, u32)`.
        fn idx_char(self) -> (usize, u32);
    }

    impl<'a> IdxChar for (usize, &'a u8) {
        #[inline]
        fn idx_char(self) -> (usize, u32) {
            (self.0, *self.1 as u32)
        }
    }

    impl<'a> IdxChar for (usize, &'a u32) {
        #[inline]
        fn idx_char(self) -> (usize, u32) {
            (self.0, *self.1)
        }
    }

    impl IdxChar for (usize, char) {
        #[inline]
        fn idx_char(self) -> (usize, u32) {
            (self.0, self.1 as u32)
        }
    }

    /// Binary search to find first element such that `pred(T) == true`.
    ///
    /// Assumes that if `pred(xs[i]) == true` then `pred(xs[i+1]) == true`.
    ///
    /// If all elements yield `pred(T) == false`, then `xs.len()` is returned.
    fn binary_search<T, F>(xs: &[T], mut pred: F) -> usize
    where
        F: FnMut(&T) -> bool,
    {
        let (mut left, mut right) = (0, xs.len());
        while left < right {
            let mid = (left + right) / 2;
            if pred(&xs[mid]) {
                right = mid;
            } else {
                left = mid + 1;
            }
        }
        left
    }
}
