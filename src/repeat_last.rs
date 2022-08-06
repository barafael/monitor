pub struct LastRepeatIter<I>
where
    I: Iterator,
    I::Item: Clone,
{
    it: I,
    next: Option<I::Item>,
    last: bool,
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
        LastRepeatIter {
            it: it.into_iter(),
            next: None,
            last: false,
        }
    }
}

impl<I> Iterator for LastRepeatIter<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.last {
            match self.it.next() {
                None => self.last = true,
                Some(next) => self.next = Some(next),
            }
        }

        self.next.clone()
    }
}

#[test]
fn it_works() {
    let mut it = LastRepeatIter::new(vec![1, 2, 3]);

    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(2));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), Some(3));
}
