/// Master clock running at 4 MHz.
///
/// The BBC Micro's 16 MHz crystal is divided to produce various clock signals.
/// We use 4 MHz as our base tick rate because it captures the CPU/video memory
/// interleaving: even ticks are video accesses, odd ticks are CPU accesses.
///
/// Components that run at 2 MHz (CPU, CRTC) tick every other 4 MHz edge.
/// Components that run at 1 MHz (cycle-stretched I/O) tick every fourth edge.
pub struct Clock {
    /// Total 4 MHz ticks elapsed since power-on.
    ticks: u64,
}

/// Which phase of the 4 MHz cycle we're in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Even ticks — video circuitry accesses memory.
    Video,
    /// Odd ticks — CPU accesses memory.
    Cpu,
}

impl Clock {
    pub fn new() -> Self {
        Self { ticks: 0 }
    }

    /// Advance the clock by one 4 MHz tick and return the new phase.
    pub fn tick(&mut self) -> Phase {
        self.ticks += 1;
        self.phase()
    }

    /// The current phase based on tick parity.
    pub fn phase(&self) -> Phase {
        if self.ticks % 2 == 0 {
            Phase::Video
        } else {
            Phase::Cpu
        }
    }

    /// Total ticks elapsed since power-on.
    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    /// Whether this tick is a 2 MHz boundary (every 2nd tick of the 4 MHz clock).
    /// The CPU and CRTC tick at this rate.
    pub fn is_2mhz_edge(&self) -> bool {
        self.ticks % 2 == 0
    }

    /// Whether this tick is a 1 MHz boundary (every 4th tick of the 4 MHz clock).
    /// Cycle-stretched I/O accesses operate at this rate.
    pub fn is_1mhz_edge(&self) -> bool {
        self.ticks % 4 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_alternates_phases() {
        let mut clock = Clock::new();
        // Tick 0 is video (even)
        assert_eq!(clock.phase(), Phase::Video);
        // Tick 1 is CPU (odd)
        assert_eq!(clock.tick(), Phase::Cpu);
        // Tick 2 is video again
        assert_eq!(clock.tick(), Phase::Video);
    }

    #[test]
    fn clock_2mhz_edge() {
        let mut clock = Clock::new();
        assert!(clock.is_2mhz_edge()); // tick 0
        clock.tick(); // tick 1
        assert!(!clock.is_2mhz_edge());
        clock.tick(); // tick 2
        assert!(clock.is_2mhz_edge());
    }

    #[test]
    fn clock_1mhz_edge() {
        let mut clock = Clock::new();
        assert!(clock.is_1mhz_edge()); // tick 0
        clock.tick(); // tick 1
        assert!(!clock.is_1mhz_edge());
        clock.tick(); // tick 2
        assert!(!clock.is_1mhz_edge());
        clock.tick(); // tick 3
        assert!(!clock.is_1mhz_edge());
        clock.tick(); // tick 4
        assert!(clock.is_1mhz_edge());
    }
}
