use core::marker::PhantomData;

use crate::syscon::{self, clock_source::PeripheralClockSelector};

/// A struct containing the clock configuration for a peripheral
pub struct Clock<Clock> {
    pub(crate) divval: u16,
    pub(crate) mstsclhigh: u8,
    pub(crate) mstscllow: u8,
    pub(crate) _clock: PhantomData<Clock>,
}

impl<C> Clock<C>
where
    C: ClockSource,
{
    /// Create the clock config for the I2C peripheral
    ///
    /// `mstclhigh` and `mstcllow` have to be between 2-9.
    pub fn new(_: &C, divval: u16, mstsclhigh: u8, mstscllow: u8) -> Self {
        assert!(mstsclhigh > 1 && mstsclhigh < 10);
        assert!(mstscllow > 1 && mstscllow < 10);
        Self {
            divval,
            mstsclhigh: mstsclhigh - 2,
            mstscllow: mstscllow - 2,
            _clock: PhantomData,
        }
    }
}

/// Implemented for I2C clock sources
pub trait ClockSource: private::Sealed {
    /// Select the clock source
    ///
    /// This method is used by the I2C API internally. It should not be relevant
    /// to most users.
    ///
    /// The `selector` argument should not be required to implement this trait,
    /// but it makes sure that the caller has access to the peripheral they are
    /// selecting the clock for.
    fn select<S>(selector: &S, handle: &mut syscon::Handle)
    where
        S: PeripheralClockSelector;
}

#[cfg(feature = "82x")]
mod target {
    use crate::syscon;

    use super::ClockSource;

    impl super::private::Sealed for () {}

    impl ClockSource for () {
        fn select<S>(_: &S, _: &mut syscon::Handle) {
            // nothing to do; `()` represents the clock that is selected by
            // default
        }
    }
}

#[cfg(feature = "845")]
mod target {
    use crate::syscon::{
        self,
        clock_source::{PeripheralClock, PeripheralClockSelector},
    };

    use super::ClockSource;

    impl<T> super::private::Sealed for T where T: PeripheralClock {}
    impl<T> ClockSource for T
    where
        T: PeripheralClock,
    {
        fn select<S>(selector: &S, handle: &mut syscon::Handle)
        where
            S: PeripheralClockSelector,
        {
            T::select(selector, handle);
        }
    }
}

mod private {
    pub trait Sealed {}
}
