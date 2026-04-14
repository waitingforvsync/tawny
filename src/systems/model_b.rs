use crate::emulator::clock::{Clock, Phase};
use crate::emulator::component::Component;
use crate::emulator::hd6845s::{Hd6845s, Hd6845sInput};
use crate::emulator::mos6502::{Mos6502, Mos6502Input, Mos6502Output};
use crate::emulator::r6522::{R6522, R6522Input};
use crate::emulator::ram::{Ram, RamInput};
use crate::emulator::rom::{Ram16k, Rom, RomInput, RomOutput};
use crate::emulator::vidproc::{Vidproc, VidprocInput};

// ======================================================================
// Address decoding
// ======================================================================

/// The chip selected by address decoding — exactly one device responds
/// to any given address, mirroring the hardware's active-high select lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipSelect {
    Ram,        // 0x0000–0x7FFF
    Paged,      // 0x8000–0xBFFF
    OsRom,      // 0xC000–0xFBFF, 0xFF00–0xFFFF
    Crtc,       // 0xFE00–0xFE07
    Acia,       // 0xFE08–0xFE0F
    SerialUla,  // 0xFE10–0xFE17
    Vidproc,    // 0xFE20–0xFE2F
    RomSelect,  // 0xFE30–0xFE3F
    SystemVia,  // 0xFE40–0xFE5F
    UserVia,    // 0xFE60–0xFE7F
    Fdc,        // 0xFE80–0xFE9F
    Econet,     // 0xFEA0–0xFEBF
    Adc,        // 0xFEC0–0xFEDF
    Tube,       // 0xFEE0–0xFEFF
    ExternalFc, // 0xFC00–0xFCFF
    ExternalFd, // 0xFD00–0xFDFF
}

/// Decode a 16-bit address to the selected chip.
pub fn decode_address(address: u16) -> ChipSelect {
    match address {
        0x0000..=0x7FFF => ChipSelect::Ram,
        0x8000..=0xBFFF => ChipSelect::Paged,
        0xC000..=0xFBFF => ChipSelect::OsRom,
        0xFC00..=0xFCFF => ChipSelect::ExternalFc,
        0xFD00..=0xFDFF => ChipSelect::ExternalFd,
        0xFE00..=0xFEFF => decode_fe_page(address),
        0xFF00..=0xFFFF => ChipSelect::OsRom,
    }
}

/// Decode within the 0xFE00–0xFEFF I/O page.
fn decode_fe_page(address: u16) -> ChipSelect {
    let lo = address as u8;
    match lo {
        0x00..=0x07 => ChipSelect::Crtc,
        0x08..=0x0F => ChipSelect::Acia,
        0x10..=0x17 => ChipSelect::SerialUla,
        // 0x18..=0x1F is unused in the decode, but the hardware would
        // not assert any select here. Map to Crtc as the 74LS138 would.
        0x18..=0x1F => ChipSelect::Crtc,
        0x20..=0x2F => ChipSelect::Vidproc,
        0x30..=0x3F => ChipSelect::RomSelect,
        0x40..=0x5F => ChipSelect::SystemVia,
        0x60..=0x7F => ChipSelect::UserVia,
        0x80..=0x9F => ChipSelect::Fdc,
        0xA0..=0xBF => ChipSelect::Econet,
        0xC0..=0xDF => ChipSelect::Adc,
        0xE0..=0xFF => ChipSelect::Tube,
    }
}

/// Whether an address is on the 1 MHz bus (requires cycle stretching).
///
/// Pages 0xFC and 0xFD are always slow. Within 0xFE, bits 7:5 of the low
/// byte index into an 8-entry table matching the hardware's IC23 (74LS30)
/// slow-access detection.
pub fn is_slow(address: u16) -> bool {
    let page = (address >> 8) as u8;
    match page {
        0xFC | 0xFD => true,
        0xFE => {
            // Bits 7:5 of the low byte select 32-byte chunks.
            let chunk = (address as u8) >> 5;
            const SLOW: [bool; 8] = [
                true,  // 0: 0xFE00–0xFE1F (CRTC, ACIA, Serial ULA)
                false, // 1: 0xFE20–0xFE3F (Vidproc, ROM select)
                true,  // 2: 0xFE40–0xFE5F (System VIA)
                true,  // 3: 0xFE60–0xFE7F (User VIA)
                false, // 4: 0xFE80–0xFE9F (FDC)
                true,  // 5: 0xFEA0–0xFEBF (Econet)
                false, // 6: 0xFEC0–0xFEDF (ADC)
                false, // 7: 0xFEE0–0xFEFF (Tube)
            ];
            SLOW[chunk as usize]
        }
        _ => false,
    }
}

// ======================================================================
// Paged ROM/RAM banks
// ======================================================================

/// A sideways ROM or RAM bank occupying the 0x8000–0xBFFF window.
pub enum PagedBank {
    Rom(Rom),
    Ram(Ram16k),
}

impl PagedBank {
    fn tick(&mut self, input: &RomInput) -> RomOutput {
        match self {
            PagedBank::Rom(rom) => rom.tick(input),
            PagedBank::Ram(ram) => ram.tick(input),
        }
    }
}

// ======================================================================
// BBC Model B system
// ======================================================================

/// BBC Model B system — components, bus, and glue logic.
pub struct ModelB {
    clock: Clock,

    // Active components (ticked at their native rate).
    cpu: Mos6502,
    pub crtc: Hd6845s,
    pub vidproc: Vidproc,
    pub system_via: R6522,
    pub user_via: R6522,

    // Passive storage (ticked only when selected).
    ram: Ram,
    os_rom: Rom,
    paged_banks: [Option<PagedBank>; 16],
    paged_select: u8,

    // CPU bus state carried between ticks. After each cpu.tick(), the system
    // routes the bus and stores the result here for the next tick's input.
    cpu_bus_data: u8,

    // Cycle stretching state.
    /// The CPU's most recent output, held during a stretch so we can route
    /// the bus when the stretch completes.
    cpu_output: Mos6502Output,
    /// The chip selected by the stretched address.
    stretched_cs: ChipSelect,
    /// Extra 2 MHz ticks remaining in the current stretch (0 = no stretch).
    stretch_remaining: u8,

    // Interrupt aggregation.
    irq: bool,
    nmi: bool,
}

const OS_ROM: &[u8; 0x4000] = include_bytes!("../../roms/os12.rom");
const BASIC_ROM: &[u8; 0x4000] = include_bytes!("../../roms/basic2.rom");

impl ModelB {
    pub fn new() -> Self {
        let mut paged_banks: [Option<PagedBank>; 16] = Default::default();
        paged_banks[15] = Some(PagedBank::Rom(Rom::new(Box::new(*BASIC_ROM))));
        paged_banks[14] = Some(PagedBank::Ram(Ram16k::new()));

        Self {
            clock: Clock::new(),
            cpu: Mos6502::new(),
            crtc: Hd6845s::new(),
            vidproc: Vidproc::new(),
            system_via: R6522::new(),
            user_via: R6522::new(),
            ram: Ram::new(),
            os_rom: Rom::new(Box::new(*OS_ROM)),
            paged_banks,
            paged_select: 15, // BASIC selected by default
            cpu_bus_data: 0,
            cpu_output: Mos6502Output { address: 0, data: 0, rw: true, sync: false },
            stretched_cs: ChipSelect::Ram,
            stretch_remaining: 0,
            irq: false,
            nmi: false,
        }
    }

    /// Insert a ROM into a sideways bank (0–15).
    pub fn insert_rom(&mut self, slot: usize, data: Box<[u8; 0x4000]>) {
        assert!(slot < 16);
        self.paged_banks[slot] = Some(PagedBank::Rom(Rom::new(data)));
    }

    /// Insert sideways RAM into a bank (0–15).
    pub fn insert_sideways_ram(&mut self, slot: usize) {
        assert!(slot < 16);
        self.paged_banks[slot] = Some(PagedBank::Ram(Ram16k::new()));
    }

    /// Provide read access to the CPU (for tests/debugging).
    pub fn cpu(&self) -> &Mos6502 {
        &self.cpu
    }

    /// Provide mutable access to the CPU (for tests/debugging).
    pub fn cpu_mut(&mut self) -> &mut Mos6502 {
        &mut self.cpu
    }

    /// Update the system for `n` 2 MHz cycles.
    ///
    /// Each 2 MHz cycle = 2 ticks at 4 MHz (one video phase + one CPU phase).
    /// Cycle stretching may cause a CPU cycle to span multiple 2 MHz cycles.
    pub fn update(&mut self, cycles_2mhz: u32) {
        for _ in 0..cycles_2mhz {
            self.tick_4mhz(); // video phase
            self.tick_4mhz(); // cpu phase
        }
    }

    /// Advance the system by one 4 MHz tick.
    fn tick_4mhz(&mut self) {
        let phase = self.clock.tick();

        match phase {
            Phase::Video => self.tick_video(),
            Phase::Cpu => self.tick_cpu(),
        }
    }

    /// Video phase: CRTC generates address, RAM read, feed to Vidproc.
    fn tick_video(&mut self) {
        let crtc_out = self.crtc.tick(&Hd6845sInput {
            rs: false,
            cs: false,
            rw: true,
            data: 0,
        });

        let ram_out = self.ram.tick(&RamInput {
            address: crtc_out.ma & 0x7FFF,
            data: 0,
            rw: true,
            ce: true,
        });

        let _vidproc_out = self.vidproc.tick(&VidprocInput {
            data: 0,
            cs: false,
            video_data: ram_out.data,
            de: crtc_out.de,
        });
    }

    /// CPU phase: tick active components (always) and run CPU bus cycle (if not stretching).
    fn tick_cpu(&mut self) {
        // During a stretch, the CPU is frozen — active components still tick
        // but with ce=false. When the stretch ends, we route the bus using
        // the cached CPU output before ticking the CPU.
        if self.stretch_remaining > 0 {
            self.stretch_remaining -= 1;
            if self.stretch_remaining == 0 {
                // Stretch complete: route the cached address now.
                self.route_cpu_bus(self.stretched_cs);
            }
            self.tick_active_components(None, 0, 0, true);
            return;
        }

        // Normal cycle: tick the CPU, then route the bus.
        let cpu_out = self.cpu.tick(&Mos6502Input {
            data: self.cpu_bus_data,
            irq: self.irq,
            nmi: self.nmi,
        });
        self.cpu_output = cpu_out;
        let cs = decode_address(cpu_out.address);

        // Check for cycle stretching on 1 MHz devices.
        if is_slow(cpu_out.address) {
            let extra = self.calc_stretch_ticks();
            if extra > 0 {
                self.stretched_cs = cs;
                self.stretch_remaining = extra;
                self.tick_active_components(None, 0, 0, true);
                return;
            }
        }

        // Route bus and tick active components.
        self.route_cpu_bus(cs);
        self.tick_active_components(Some(cs), cpu_out.address, cpu_out.data, cpu_out.rw);
    }

    /// Route the CPU's bus access to the selected device, storing the result
    /// in `cpu_bus_data` for the next tick.
    fn route_cpu_bus(&mut self, cs: ChipSelect) {
        let cpu_out = self.cpu_output;
        self.cpu_bus_data = match cs {
            ChipSelect::Crtc | ChipSelect::SystemVia | ChipSelect::UserVia
            | ChipSelect::Vidproc => {
                // Active components — will be ticked with ce=true by tick_active_components.
                // Data resolved there.
                0xFF // placeholder, overwritten by tick_active_components
            }
            _ => self.route_passive(cpu_out.address, cpu_out.data, cpu_out.rw, cs),
        };
    }

    /// Tick active components every CPU phase. `cs` is Some when the CPU is
    /// addressing one of them (ce=true for that device).
    fn tick_active_components(&mut self, cs: Option<ChipSelect>, addr: u16, data: u8, rw: bool) {
        let crtc_out = self.crtc.tick(&Hd6845sInput {
            rs: addr & 1 != 0,
            cs: cs == Some(ChipSelect::Crtc),
            rw,
            data,
        });

        let sys_via_out = self.system_via.tick(&R6522Input {
            rs: (addr & 0x0F) as u8,
            cs: cs == Some(ChipSelect::SystemVia),
            rw,
            data,
        });

        let usr_via_out = self.user_via.tick(&R6522Input {
            rs: (addr & 0x0F) as u8,
            cs: cs == Some(ChipSelect::UserVia),
            rw,
            data,
        });

        let _vidproc_out = self.vidproc.tick(&VidprocInput {
            data,
            cs: cs == Some(ChipSelect::Vidproc),
            video_data: 0,
            de: false,
        });

        // Aggregate interrupts from VIAs.
        self.irq = sys_via_out.irq || usr_via_out.irq;

        // If an active component was addressed, store its data for the CPU.
        if let Some(cs) = cs {
            self.cpu_bus_data = match cs {
                ChipSelect::Crtc => crtc_out.data,
                ChipSelect::SystemVia => sys_via_out.data,
                ChipSelect::UserVia => usr_via_out.data,
                ChipSelect::Vidproc => 0xFF, // write-only
                _ => self.cpu_bus_data, // already set by route_cpu_bus
            };
        }
    }

    /// Calculate extra 2 MHz ticks to defer for cycle stretching.
    ///
    /// The 1 MHzE phase at the time of access determines the stretch:
    /// - 1MHzE low  → 1 extra 2 MHz tick  (phi0 high held for 3 extra half-cycles)
    /// - 1MHzE high → 2 extra 2 MHz ticks (phi0 low+high both extended)
    fn calc_stretch_ticks(&self) -> u8 {
        // 1MHzE repeats every 4 ticks: high for ticks 0-1, low for ticks 2-3.
        // CPU phase ticks are 1, 3, 5, 7, ...
        // tick % 4 == 1 → 1MHzE is high → 2 extra
        // tick % 4 == 3 → 1MHzE is low  → 1 extra
        let phase_in_1mhz = self.clock.ticks() % 4;
        if phase_in_1mhz <= 1 {
            2 // 1MHzE high: longer stretch
        } else {
            1 // 1MHzE low: shorter stretch
        }
    }

    /// Route a CPU access to passive components (RAM, ROM, paged banks,
    /// write-only registers, unimplemented devices).
    fn route_passive(&mut self, address: u16, data: u8, rw: bool, cs: ChipSelect) -> u8 {
        match cs {
            ChipSelect::Ram => {
                let out = self.ram.tick(&RamInput {
                    address: address & 0x7FFF,
                    data,
                    rw,
                    ce: true,
                });
                out.data
            }
            ChipSelect::OsRom => {
                let out = self.os_rom.tick(&RomInput {
                    address: address & 0x3FFF,
                    data,
                    rw,
                    ce: true,
                });
                out.data
            }
            ChipSelect::Paged => self.route_paged(address, data, rw),
            ChipSelect::RomSelect => {
                if !rw {
                    self.paged_select = data & 0x0F;
                }
                0xFF // write-only
            }
            // Unimplemented devices: reads return 0xFF, writes ignored.
            _ => 0xFF,
        }
    }

    /// Route access to the currently selected paged ROM/RAM bank.
    fn route_paged(&mut self, address: u16, data: u8, rw: bool) -> u8 {
        let slot = self.paged_select as usize;
        match &mut self.paged_banks[slot] {
            Some(bank) => {
                let out = bank.tick(&RomInput {
                    address: address & 0x3FFF,
                    data,
                    rw,
                    ce: true,
                });
                out.data
            }
            None => 0xFF, // empty slot — undriven bus
        }
    }

    /// Reset the entire system.
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.crtc.reset();
        self.vidproc.reset();
        self.system_via.reset();
        self.user_via.reset();
        self.cpu_bus_data = 0;
        self.stretch_remaining = 0;
        self.irq = false;
        self.nmi = false;
    }
}

// ======================================================================
// Tests
// ======================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- decode_address ---

    #[test]
    fn decode_ram() {
        assert_eq!(decode_address(0x0000), ChipSelect::Ram);
        assert_eq!(decode_address(0x7FFF), ChipSelect::Ram);
        assert_eq!(decode_address(0x4000), ChipSelect::Ram);
    }

    #[test]
    fn decode_paged() {
        assert_eq!(decode_address(0x8000), ChipSelect::Paged);
        assert_eq!(decode_address(0xBFFF), ChipSelect::Paged);
    }

    #[test]
    fn decode_os_rom() {
        assert_eq!(decode_address(0xC000), ChipSelect::OsRom);
        assert_eq!(decode_address(0xFBFF), ChipSelect::OsRom);
        assert_eq!(decode_address(0xFF00), ChipSelect::OsRom);
        assert_eq!(decode_address(0xFFFC), ChipSelect::OsRom);
        assert_eq!(decode_address(0xFFFF), ChipSelect::OsRom);
    }

    #[test]
    fn decode_external_pages() {
        assert_eq!(decode_address(0xFC00), ChipSelect::ExternalFc);
        assert_eq!(decode_address(0xFCFF), ChipSelect::ExternalFc);
        assert_eq!(decode_address(0xFD00), ChipSelect::ExternalFd);
        assert_eq!(decode_address(0xFDFF), ChipSelect::ExternalFd);
    }

    #[test]
    fn decode_fe_page_devices() {
        assert_eq!(decode_address(0xFE00), ChipSelect::Crtc);
        assert_eq!(decode_address(0xFE07), ChipSelect::Crtc);
        assert_eq!(decode_address(0xFE08), ChipSelect::Acia);
        assert_eq!(decode_address(0xFE0F), ChipSelect::Acia);
        assert_eq!(decode_address(0xFE10), ChipSelect::SerialUla);
        assert_eq!(decode_address(0xFE17), ChipSelect::SerialUla);
        assert_eq!(decode_address(0xFE20), ChipSelect::Vidproc);
        assert_eq!(decode_address(0xFE2F), ChipSelect::Vidproc);
        assert_eq!(decode_address(0xFE30), ChipSelect::RomSelect);
        assert_eq!(decode_address(0xFE3F), ChipSelect::RomSelect);
        assert_eq!(decode_address(0xFE40), ChipSelect::SystemVia);
        assert_eq!(decode_address(0xFE5F), ChipSelect::SystemVia);
        assert_eq!(decode_address(0xFE60), ChipSelect::UserVia);
        assert_eq!(decode_address(0xFE7F), ChipSelect::UserVia);
        assert_eq!(decode_address(0xFE80), ChipSelect::Fdc);
        assert_eq!(decode_address(0xFE9F), ChipSelect::Fdc);
        assert_eq!(decode_address(0xFEA0), ChipSelect::Econet);
        assert_eq!(decode_address(0xFEBF), ChipSelect::Econet);
        assert_eq!(decode_address(0xFEC0), ChipSelect::Adc);
        assert_eq!(decode_address(0xFEDF), ChipSelect::Adc);
        assert_eq!(decode_address(0xFEE0), ChipSelect::Tube);
        assert_eq!(decode_address(0xFEFF), ChipSelect::Tube);
    }

    // --- is_slow ---

    #[test]
    fn slow_fc_fd_pages() {
        assert!(is_slow(0xFC00));
        assert!(is_slow(0xFCFF));
        assert!(is_slow(0xFD00));
        assert!(is_slow(0xFDFF));
    }

    #[test]
    fn slow_fe_page() {
        // Slow: CRTC/ACIA/Serial ULA (0xFE00–0xFE1F)
        assert!(is_slow(0xFE00));
        assert!(is_slow(0xFE1F));
        // Fast: Vidproc/RomSelect (0xFE20–0xFE3F)
        assert!(!is_slow(0xFE20));
        assert!(!is_slow(0xFE3F));
        // Slow: VIAs (0xFE40–0xFE7F)
        assert!(is_slow(0xFE40));
        assert!(is_slow(0xFE7F));
        // Fast: FDC (0xFE80–0xFE9F)
        assert!(!is_slow(0xFE80));
        // Slow: Econet (0xFEA0–0xFEBF)
        assert!(is_slow(0xFEA0));
        // Fast: ADC, Tube (0xFEC0–0xFEFF)
        assert!(!is_slow(0xFEC0));
        assert!(!is_slow(0xFEE0));
    }

    #[test]
    fn non_io_not_slow() {
        assert!(!is_slow(0x0000));
        assert!(!is_slow(0x7FFF));
        assert!(!is_slow(0x8000));
        assert!(!is_slow(0xC000));
        assert!(!is_slow(0xFF00));
    }

    // --- Integration: reset vector fetch ---

    #[test]
    fn cpu_fetches_reset_vector_from_os_rom() {
        let mut system = ModelB::new();
        system.reset();

        // The 6502 reset sequence is 7 cycles (BRK microcode with reset
        // vector). Cycle 7 is fetch_opcode, which reads the opcode at
        // the reset vector target and increments PC past it.
        // OS 1.20 reset vector at 0xFFFC–0xFFFD = 0xD9CD.
        system.update(7);
        assert_eq!(system.cpu().pc, 0xD9CE); // entry + 1 (opcode fetched)
    }
}
