#![deny(unsafe_code)]
#![deny(warnings)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use rtic::app;
use rtic::cyccnt::U32Ext;

use stm32l4xx_hal::{
    gpio::{gpiob::PB3, Output, PushPull, State},
    pac,
    prelude::*,
};

const BEATS_PER_MIN: u32 = 60;
const CLK_SPEED_MHZ: u32 = 72;

// Cycles per thousandth of beat
const MILLI_BEAT: u32 = CLK_SPEED_MHZ * 60_000 / BEATS_PER_MIN;

// Simple heart beat LED on/off sequence
const INTERVALS: [u32; 6] = [
    30,  // P Wave
    40,  // PR Segment
    120, // QRS Complex
    30,  // ST Segment
    60,  // T Wave
    720, // Rest
];

// We need to pass monotonic = rtic::cyccnt::CYCCNT to use schedule feature fo RTIC
#[app(device = crate::pac, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    // Global resources (global variables) are defined here and initialized with the 
    // `LateResources` struct in init
    struct Resources {
        led: PB3<Output<PushPull>>,
    }

    #[init(schedule = [blinker])]
    fn init(cx: init::Context) -> init::LateResources {
        // Enable cycle counter
        let mut core = cx.core;
        core.DWT.enable_cycle_counter();

        // Setup clocks
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let _clocks = rcc
            .cfgr
            .sysclk(CLK_SPEED_MHZ.mhz())
            .freeze(&mut flash.acr);

        // Setup LED
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.ahb2);
        let led = gpiob
            .pb3
            .into_push_pull_output_with_state(&mut gpiob.moder, &mut gpiob.otyper, State::Low);

        // Schedule the blinking task
        cx.schedule.blinker(cx.start, 0).unwrap();

        init::LateResources { led }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            core::sync::atomic::spin_loop_hint();
        }
    }

    #[task(schedule = [blinker], resources = [led])]
    fn blinker(cx: blinker::Context, state: usize) {
        let led = cx.resources.led;
        let duration = MILLI_BEAT * INTERVALS[state];
        let next_state = (state + 1) % INTERVALS.len();

        if state % 2 == 0 {
            led.set_high().unwrap();
        } else {
            led.set_low().unwrap();
        }

        cx.schedule.blinker(cx.scheduled + duration.cycles(), next_state).unwrap();
    }

    extern "C" {
        fn EXTI0();
    }
};
