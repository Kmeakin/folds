#![feature(never_type)]

use std::convert::Infallible;
use std::marker::PhantomData;

pub trait Uninhabited {
    fn absurd<A>(self) -> A;
}

impl Uninhabited for Infallible {
    fn absurd<A>(self) -> A { match self {} }
}

impl Uninhabited for ! {
    fn absurd<A>(self) -> A { match self {} }
}

pub trait Fold {
    type Accumulator;
    type Error;
    type Element;

    fn init(&mut self) -> Self::Accumulator;

    fn try_step(
        &mut self,
        acc: Self::Accumulator,
        elem: Self::Element,
    ) -> Result<Self::Accumulator, Self::Error>;

    fn try_fold(
        &mut self,
        mut iter: impl Iterator<Item = Self::Element>,
    ) -> Result<Self::Accumulator, Self::Error> {
        iter.try_fold(self.init(), |acc, elem| self.try_step(acc, elem))
    }

    fn fold(&mut self, iter: impl Iterator<Item = Self::Element>) -> Self::Accumulator
    where
        Self::Error: Uninhabited,
    {
        match self.try_fold(iter) {
            Ok(acc) => acc,
            Err(err) => err.absurd(),
        }
    }

    fn zip<F2>(self, other: F2) -> Zip<Self, F2>
    where
        Self: Sized,
    {
        Zip {
            fold1: self,
            fold2: other,
        }
    }

    fn try_zip<F2>(self, other: F2) -> TryZip<Self, F2>
    where
        Self: Sized,
    {
        TryZip {
            fold1: self,
            fold2: other,
        }
    }
}

pub struct Zip<Fold1, Fold2> {
    fold1: Fold1,
    fold2: Fold2,
}

impl<Fold1, Fold2> Fold for Zip<Fold1, Fold2>
where
    Fold1: Fold,
    Fold2: Fold<Element = Fold1::Element>,
    Fold1::Element: Clone,
{
    type Accumulator = (
        Result<Fold1::Accumulator, Fold1::Error>,
        Result<Fold2::Accumulator, Fold2::Error>,
    );
    type Error = Infallible;
    type Element = Fold1::Element;

    fn init(&mut self) -> Self::Accumulator { (Ok(self.fold1.init()), Ok(self.fold2.init())) }

    fn try_step(
        &mut self,
        acc: Self::Accumulator,
        elem: Self::Element,
    ) -> Result<Self::Accumulator, Self::Error> {
        let (acc1, acc2) = acc;
        let res1 = acc1.and_then(|acc1| self.fold1.try_step(acc1, elem.clone()));
        let res2 = acc2.and_then(|acc2| self.fold2.try_step(acc2, elem.clone()));
        Ok((res1, res2))
    }
}

pub struct TryZip<Fold1, Fold2> {
    fold1: Fold1,
    fold2: Fold2,
}

impl<Fold1, Fold2> Fold for TryZip<Fold1, Fold2>
where
    Fold1: Fold,
    Fold2: Fold<Element = Fold1::Element, Error = Fold1::Error>,
    Fold1::Element: Clone,
{
    type Accumulator = (Fold1::Accumulator, Fold2::Accumulator);
    type Error = Fold1::Error;
    type Element = Fold1::Element;

    fn init(&mut self) -> Self::Accumulator { (self.fold1.init(), self.fold2.init()) }

    fn try_step(
        &mut self,
        acc: Self::Accumulator,
        elem: Self::Element,
    ) -> Result<Self::Accumulator, Self::Error> {
        let (acc1, acc2) = acc;
        let res1 = self.fold1.try_step(acc1, elem.clone())?;
        let res2 = self.fold2.try_step(acc2, elem)?;
        Ok((res1, res2))
    }
}

pub struct FromFn<A, E, F> {
    acc: A,
    fun: F,
    phantom: PhantomData<E>,
}

impl<Acc, Elem, F> Fold for FromFn<Acc, Elem, F>
where
    Acc: Clone,
    F: Fn(Acc, Elem) -> Acc,
{
    type Accumulator = Acc;
    type Error = Infallible;
    type Element = Elem;

    fn init(&mut self) -> Self::Accumulator { self.acc.clone() }

    fn try_step(
        &mut self,
        acc: Self::Accumulator,
        elem: Self::Element,
    ) -> Result<Self::Accumulator, Self::Error> {
        Ok((self.fun)(acc, elem))
    }
}

pub const fn from_fn<A, E, F>(acc: A, f: F) -> FromFn<A, E, F> {
    FromFn {
        acc,
        fun: f,
        phantom: PhantomData,
    }
}

pub struct FromTryFn<Acc, Elem, Fun> {
    acc: Acc,
    fun: Fun,
    phantom: PhantomData<Elem>,
}

impl<Acc, Error, Elem, Fun> Fold for FromTryFn<Acc, Elem, Fun>
where
    Acc: Clone,
    Fun: Fn(Acc, Elem) -> Result<Acc, Error>,
{
    type Accumulator = Acc;
    type Error = Error;
    type Element = Elem;

    fn init(&mut self) -> Self::Accumulator { self.acc.clone() }

    fn try_step(
        &mut self,
        acc: Self::Accumulator,
        elem: Self::Element,
    ) -> Result<Self::Accumulator, Self::Error> {
        (self.fun)(acc, elem)
    }
}

pub const fn from_try_fn<Acc, Elem, Fun>(acc: Acc, fun: Fun) -> FromTryFn<Acc, Elem, Fun> {
    FromTryFn {
        acc,
        fun,
        phantom: PhantomData,
    }
}

#[test]
fn sum_and_product() {
    let sum = from_fn(0_u32, u32::wrapping_add);
    let prod = from_fn(1_u32, u32::wrapping_mul);
    let mut fold = sum.try_zip(prod);
    let xs = &[1, 2, 3, 4];
    assert_eq!(fold.fold(xs.iter().copied()), (10, 24));
}

#[test]
fn try_sum_and_product() {
    let sum = from_try_fn(0_u32, |x: u32, y: u32| x.checked_add(y).ok_or(()));
    let prod = from_try_fn(1_u32, |x: u32, y: u32| x.checked_mul(y).ok_or(()));
    let mut fold = sum.try_zip(prod);
    let xs = &[1, 2, 3, 4];
    assert_eq!(fold.try_fold(xs.iter().copied()), Ok((10, 24)));
}
