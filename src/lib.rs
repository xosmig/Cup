
// If it will have only alias to the Item type, it may be better to use IntoIterator trait instead.
pub trait Container { // or Collection 
    type Item: Sized;
}

// pub trait ForwardCup
// pub trait ForwardCupMut
// etc

pub trait BidirCup: Clone + PartialEq {
    type Cont: Container + ?Sized;
    type Item;

    fn step_next(&mut self, cont: &Self::Cont);
    fn step_prev(&mut self, cont: &Self::Cont);

    fn as_ref<'b>(&'b self, cont: &'b Self::Cont) -> &Self::Item;
}

pub trait BidirCupMut: BidirCup {
    fn as_mut_ref<'b>(&'b self, cont: &'b mut Self::Cont) -> &mut Self::Item;
    fn swap(&self, other: &Self, cont: &mut Self::Cont);
}

#[cfg(test)]
mod tests {
    use super::Container;
    use super::BidirCup;
    use super::BidirCupMut;
    use std::marker::PhantomData;
    use std::default::Default;

    impl<T> Container for [T] {
        type Item = T;
    }

    #[derive(Copy)]
    struct SliceCupMut<T> {
        idx: usize,
        phantom: PhantomData<T>,
    }

    impl<T> SliceCupMut<T> {
        fn new(idx: usize) -> SliceCupMut<T> {
            SliceCupMut { idx: idx, phantom: Default::default() }
        }
    }

    impl<'a, T> Clone for SliceCupMut<T> {
        fn clone(&self) -> Self {
            SliceCupMut::new(self.idx)
        }
    }

    impl<'a, T> PartialEq for SliceCupMut<T> {
        fn eq(&self, other: &Self) -> bool {
            self.idx == other.idx
        }
    }

    impl<T> BidirCup for SliceCupMut<T> {
        type Cont = [T];
        type Item = T;

        fn step_next(&mut self, cont: &Self::Cont) {
            self.idx += 1;
            assert!(self.idx <= cont.len());
        }

        fn step_prev(&mut self, _: &Self::Cont) {
            assert!(self.idx > 0);
            self.idx -= 1;
        }

        fn as_ref<'b>(&'b self, cont: &'b Self::Cont) -> &Self::Item {
            &cont[self.idx]
        }
    }

    impl<T> BidirCupMut for SliceCupMut<T> {
        fn as_mut_ref<'b>(&'b self, cont: &'b mut Self::Cont) -> &mut Self::Item {
            &mut cont[self.idx]
        }

        fn swap(&self, other: &Self, cont: &mut [T]) {
            cont.swap(self.idx, other.idx);
        }

    }

    trait Crutch<T> { // this methods should be without any trait
        fn begin(&self) -> SliceCupMut<T>;
        fn end(&self) -> SliceCupMut<T>;
    }

    impl<T> Crutch<T> for [T] {
        fn begin(&self) -> SliceCupMut<T> {
            SliceCupMut::new(0)
        }

        fn end(&self) -> SliceCupMut<T> {
            SliceCupMut::new(self.len())
        }
    }

    #[test]
    fn test() {
        let mut s = [1, 2, 3, 4];

        let mut it1 = s.begin();
        let it2 = s.end();
        let mut it3 = it2.clone();

        let mut i = 0;
        while it1 != it2 { 
            assert_eq!(*it1.as_ref(&s), s[i]);
            it1.step_next(&s);
            i += 1;
        }

        it3.step_prev(&s);
        *it3.as_mut_ref(&mut s) = 2;

        assert_eq!(*it3.as_ref(&s), 2);
        it3.step_prev(&s);
        assert_eq!(*it3.as_ref(&s), 3);
    }

    trait Partition: Container {
        fn partition<I, P>(&mut self, mut first: I, last: I, mut predicate: P) -> I
            where I: BidirCupMut<Cont = Self>,
                  P: FnMut(&<I as BidirCup>::Item) -> bool,
        {
            let mut mid = first.clone();
            while first != last {
                if predicate(first.as_ref(self)) {
                    mid.swap(&first, self);
                    mid.step_next(self);
                }
                first.step_next(self);
            }
            mid       
        }
    }

    impl<T> Partition for [T] {}

    #[test]
    fn test_partition() {
        let mut s = [1, 7, 12, 2, 44, 5];
        
        let begin = s.begin();
        let end = s.end();

        let mut it1 = begin;
        it1.step_next(&s);

        let mut it2 = end;
        it2.step_prev(&s);

        let mid = s.partition(it1, it2, |x| *x < 12);

        assert_eq!(s[0], 1);
        assert_eq!(s[5], 5);

        while it1 != mid {
            assert!(*it1.as_ref(&s) < 12);
            it1.step_next(&s);
        }
        
        while it1 != it2 {
            assert!(*it1.as_ref(&s) >= 12);
            it1.step_next(&s);
        }
    }
}
