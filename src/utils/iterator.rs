use std::fmt::Debug;

pub trait PartitionResult<T, E> {
    type Item;
    fn partition_result(self) -> (Vec<T>, Vec<E>);
}

impl<T, E, I> PartitionResult<T, E> for I
where
    I: Iterator<Item = Result<T, E>>,
    T: Debug,
    E: Debug,
{
    type Item = I::Item;
    fn partition_result(self) -> (Vec<T>, Vec<E>) {
        let (oks, errs): (Vec<_>, Vec<_>) = self.partition(Result::is_ok);
        let oks = oks.into_iter().map(Result::unwrap).collect();
        let errs = errs.into_iter().map(Result::unwrap_err).collect();
        (oks, errs)
    }
}
