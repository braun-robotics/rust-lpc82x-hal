//! API for the CTimer peripheral
//!
//! Currently, only PWM output functionality is implemented.
//!
//! # Example
//!
//! ```no_run
//! use lpc8xx_hal::{
//!     delay::Delay,
//!     prelude::*,
//!     Peripherals,
//!     pac::CorePeripherals,
//! };
//!
//! let cp = CorePeripherals::take().unwrap();
//! let p = Peripherals::take().unwrap();
//!
//! let swm = p.SWM.split();
//! let mut delay = Delay::new(cp.SYST);
//! let mut syscon = p.SYSCON.split();
//!
//! let mut swm_handle = swm.handle.enable(&mut syscon.handle);
//!
//! // Use 8 bit pwm
//! let ctimer = p.CTIMER0.enable(256, 0, &mut syscon.handle);
//!
//! let pwm_output = p.pins.pio1_2.into_swm_pin();
//!
//! let (pwm_output, _) = swm.movable_functions.t0_mat0.assign(
//!     pwm_output,
//!     &mut swm_handle,
//! );
//!
//! let mut pwm_pin = ctimer.channels.channel1.attach(pwm_output);
//! loop {
//!     for i in 0..pwm_pin.get_max_duty() {
//!         delay.delay_ms(4_u8);
//!         pwm_pin.set_duty(i);
//!     }
//! }
//! ```

pub mod channels;

use crate::{
    init_state::{Disabled, Enabled},
    pac::CTIMER0,
    syscon,
};

use self::channels::{state::Detached, Channels};

/// Interface to a CTimer peripheral
///
/// Controls the CTimer.  Use [`Peripherals`] to gain access to an instance of
/// this struct.
///
/// Please refer to the [module documentation] for more information.
///
/// [`Peripherals`]: ../struct.Peripherals.html
/// [module documentation]: index.html
pub struct CTIMER<State, Channel1State, Channel2State, Channel3State> {
    /// The PWM channels of this CTIMER
    pub channels: Channels<State, Channel1State, Channel2State, Channel3State>,

    inner: CTIMER0,
    _state: State,
}

impl CTIMER<Disabled, Detached, Detached, Detached> {
    pub(crate) fn new(ct: CTIMER0) -> Self {
        Self {
            channels: Channels::new(),
            inner: ct,
            _state: Disabled,
        }
    }
}

impl<Channel1State, Channel2State, Channel3State>
    CTIMER<Disabled, Channel1State, Channel2State, Channel3State>
{
    /// Start the PWM timer, with a predefined period and prescaler
    ///
    /// The `period` sets resolution of the pwm and is returned with
    /// `get_max_duty`.
    pub fn enable(
        self,
        period: u32,
        prescaler: u32,
        syscon: &mut syscon::Handle,
    ) -> CTIMER<Enabled, Channel1State, Channel2State, Channel3State> {
        syscon.enable_clock(&self.inner);
        unsafe { self.inner.pr.write(|w| w.prval().bits(prescaler)) };
        // Use MAT3 to reset the counter
        unsafe { self.inner.mr[3].write(|w| w.match_().bits(period)) };
        self.inner.mcr.write(|w| {
            w.mr3r().set_bit();
            // Use shadow registers for the pwm output matches
            w.mr0rl().set_bit();
            w.mr1rl().set_bit();
            w.mr2rl().set_bit()
        });

        self.inner.pwmc.write(|w| {
            w.pwmen0().set_bit();
            w.pwmen1().set_bit();
            w.pwmen2().set_bit()
        });

        // Start the timer
        self.inner.tcr.write(|w| w.cen().set_bit());

        CTIMER {
            channels: Channels::new(),
            inner: self.inner,
            _state: Enabled(()),
        }
    }
}

impl<Channel1State, Channel2State, Channel3State>
    CTIMER<Enabled, Channel1State, Channel2State, Channel3State>
{
    /// Disable the CTIMER
    ///
    /// This method is only available, if `CTIMER` is in the [`Enabled`] state.
    /// Code that attempts to call this method when the peripheral is already
    /// disabled will not compile.
    ///
    /// Consumes this instance of `CTIMER` and returns another instance that has
    /// its `State` type parameter set to [`Disabled`].
    ///
    /// [`Enabled`]: ../init_state/struct.Enabled.html
    /// [`Disabled`]: ../init_state/struct.Disabled.html
    pub fn disable(
        self,
        syscon: &mut syscon::Handle,
    ) -> CTIMER<Disabled, Channel1State, Channel2State, Channel3State> {
        syscon.disable_clock(&self.inner);

        CTIMER {
            channels: Channels::new(),
            inner: self.inner,
            _state: Disabled,
        }
    }
}

impl<State, Channel1State, Channel2State, Channel3State>
    CTIMER<State, Channel1State, Channel2State, Channel3State>
{
    /// Return the raw peripheral
    ///
    /// This method serves as an escape hatch from the HAL API. It returns the
    /// raw peripheral, allowing you to do whatever you want with it, without
    /// limitations imposed by the API.
    ///
    /// If you are using this method because a feature you need is missing from
    /// the HAL API, please [open an issue] or, if an issue for your feature
    /// request already exists, comment on the existing issue, so we can
    /// prioritize it accordingly.
    ///
    /// [open an issue]: https://github.com/lpc-rs/lpc8xx-hal/issues
    pub fn free(self) -> CTIMER0 {
        self.inner
    }
}
