#[derive(Debug, Clone)]
pub struct Pool<T> {
    pub used: usize,
    pub items: Vec<T>,
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Pool {
            used: 0,
            items: Default::default(),
        }
    }
}

impl<T> Pool<T> {
    pub fn next(&mut self) -> Option<usize> {
        let index = self.used;
        if index < self.items.len() {
            self.used += 1;
            Some(index)
        } else {
            None
        }
    }

    pub fn push(&mut self, item: T) -> usize {
        // Ensure we only push when we've used up all unused items.
        assert_eq!(self.used, self.items.len());

        let index = self.used;
        self.used += 1;
        self.items.push(item);
        index
    }

    pub fn reset(&mut self) {
        self.used = 0;
    }

    pub fn len(&self) -> usize {
        self.used
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, T> {
        self.items[..self.used].iter()
    }

    pub fn iter_mut<'a>(&'a mut self) -> std::slice::IterMut<'a, T> {
        self.items[..self.used].iter_mut()
    }
}

impl<T> std::ops::Index<usize> for Pool<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl<T> std::ops::IndexMut<usize> for Pool<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}

impl<'a, T> IntoIterator for &'a Pool<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Pool<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
