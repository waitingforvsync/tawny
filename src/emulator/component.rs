/// A hardware component that participates in clock-driven emulation.
///
/// Each component has opaque internal state and communicates via typed pin structs.
/// The system-level glue logic copies values between a shared bus and each
/// component's Input/Output pins.
///
/// Components are ticked at the 4 MHz base clock rate. Components that operate
/// at lower rates (2 MHz, 1 MHz) should use internal counters to divide the clock.
pub trait Component {
    /// The component's input pins — signals driven by the bus/glue logic.
    type Input;
    /// The component's output pins — signals produced by this component.
    type Output;

    /// Advance the component by one tick (4 MHz base).
    /// The component receives its current inputs and returns its outputs.
    fn tick(&mut self, input: &Self::Input) -> Self::Output;

    /// Reset the component to its initial power-on state.
    fn reset(&mut self);
}
