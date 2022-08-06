use std::iter::Fuse;

pub struct LastRepeatIter<I>
where
    I: Iterator,
    I::Item: Clone,
{
    it: Fuse<I>,
    next: Option<I::Item>,
}

impl<I> LastRepeatIter<I>
where
    I: Iterator,
    I::Item: Clone,
{
    pub fn new<It>(it: It) -> Self
    where
        It: IntoIterator<IntoIter = I, Item = I::Item>,
    {
        let it = it.into_iter().fuse();
        LastRepeatIter { it, next: None }
    }
}

impl<I> Iterator for LastRepeatIter<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.it.next() {
            self.next = Some(next)
        }

        self.next.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        let mut it = LastRepeatIter::new(vec![1, 2, 3]);

        assert_eq!(it.next(), Some(1));
        assert_eq!(it.next(), Some(2));
        assert!(it.take(100).all(|n| n == 3));
    }

    #[test]
    fn it_breaks() {
        assert!(LastRepeatIter::new(None::<()>).next().is_none());
    }
}
