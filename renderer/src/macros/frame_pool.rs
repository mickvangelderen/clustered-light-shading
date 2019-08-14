macro_rules! impl_frame_pool {
    (
        $Pool: ident,
        $Item: ident,
        $Index: ident,
        $IndexIter: ident,
        ( $( $arg: ident: $ty: ty ),* ),
    ) => {
        pub struct $Pool {
            items: Vec<$Item>,
            used: usize,
        }

        impl $Pool {
            pub fn new() -> Self {
                Self {
                    items: Vec::new(),
                    used: 0,
                }
            }

            pub fn next_unused(&mut self, $($arg: $ty),*) -> $Index {
                let index = self.used;
                self.used += 1;

                if index < self.items.len() {
                    // Re-use.
                    self.items[index].reset($($arg),*);
                } else {
                    debug_assert_eq!(
                        index,
                        self.items.len(),
                        "Programming error, somehow more than one item needs to be created in frame pool."
                    );
                    self.items.push($Item::new($($arg),*));
                }

                $Index(index)
            }

            pub fn used_slice(&self) -> &[$Item] {
                &self.items[0..self.used]
            }

            pub fn used_index_iter(&self) -> $IndexIter {
                $IndexIter {
                    index: $Index(0),
                    count: self.used,
                }
            }

            pub fn used_count(&self) -> usize {
                self.used
            }

            pub fn reset(&mut self) {
                self.used = 0;
            }
        }

        #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $Index(usize);


        impl $Index {
            pub fn to_usize(&self) -> usize {
                self.0
            }
        }

        impl std::ops::Index<$Index> for $Pool {
            type Output = $Item;

            fn index(&self, index: $Index) -> &Self::Output {
                &self.items[index.0]
            }
        }

        impl std::ops::IndexMut<$Index> for $Pool {
            fn index_mut(&mut self, index: $Index) -> &mut Self::Output {
                &mut self.items[index.0]
            }
        }

        pub struct $IndexIter {
            index: $Index,
            count: usize,
        }

        impl Iterator for $IndexIter {
            type Item = $Index;

            fn next(&mut self) -> Option<Self::Item> {
                if self.index.0 < self.count {
                    let index = self.index;
                    self.index.0 += 1;
                    Some(index)
                } else {
                    None
                }
            }
        }
    };
}
