use core::fmt::{Write, Display};

use alloc::vec::Vec;

use async_trait::async_trait;
use crate::vnix::utils::Maybe;


#[derive(Debug)]
pub enum CLIErr {
    Clear,
    Write,
    GetKey,
    GetResolution,
    SetResolution
}

#[derive(Debug)]
pub enum DispErr {
    GetResolution,
    SetResolution,
    SetPixel,
    GetMouseState,
    Flush
}

#[derive(Debug)]
pub enum TimeErr {
    Wait,
    StartTimer,
}

#[derive(Debug)]
pub enum RndErr {
    GetBytes
}

#[derive(Debug)]
pub enum MemErr {
    NotEnough
}

#[derive(Debug)]
pub enum DrvErr {
    DriverFault,
    CLI(CLIErr),
    Disp(DispErr),
    Time(TimeErr),
    Rnd(RndErr),
    Mem(MemErr),
}

#[derive(Debug, PartialEq)]
pub enum TermKey {
    Esc,
    Up,
    Down,
    Left,
    Right,
    Unknown,
    Char(char)
}

#[derive(Debug)]
pub struct Mouse {
    pub dpos: (i32, i32),
    pub res: (usize, usize),
    pub click: (bool, bool)
}

// pub type TimeAsync = Box<dyn Coroutine<Yield = (), Return = Result<(), TimeErr>>>;

#[derive(Debug, Clone, Copy)]
pub enum Duration {
    Micro(usize),
    Milli(usize),
    Seconds(usize)
}

#[derive(Debug, Clone, Copy)]
pub enum TimeUnit {
    Micro,
    Milli,
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year
}

#[async_trait(?Send)]
pub trait Time {
    fn start(&mut self) -> Result<(), TimeErr>;
    fn wait(&mut self, dur: Duration) -> Result<(), TimeErr>;
    async fn wait_async(&self, dur: Duration) -> Result<(), TimeErr>;
    fn uptime(&self, units: TimeUnit) -> Result<u128, TimeErr>;
}

pub trait CLI: Write {
    fn res(&self) -> Result<(usize, usize), CLIErr>;
    fn res_list(&self) -> Result<Vec<(usize, usize)>, CLIErr>;
    fn set_res(&mut self, res: (usize, usize)) -> Result<(), CLIErr>;

    fn glyth(&mut self, ch: char, pos: (usize, usize)) -> Result<(), CLIErr>;
    fn get_key(&mut self, block: bool) -> Maybe<TermKey, CLIErr>;
    fn clear(&mut self) -> Result<(), CLIErr>;
}

pub trait Rnd {
    fn get_bytes(&mut self, buf: &mut [u8]) -> Result<(), RndErr>;
}

pub trait Disp {
    fn res(&self) -> Result<(usize, usize), DispErr>;
    fn res_list(&self) -> Result<Vec<(usize, usize)>, DispErr>;
    fn set_res(&mut self, res: (usize, usize)) -> Result<(), DispErr>; 

    fn mouse(&mut self, block: bool) -> Maybe<Mouse, DispErr>;

    fn px(&mut self, px: u32, x: usize, y: usize) -> Result<(), DispErr>;
    fn blk(&mut self, pos: (i32, i32), img_size: (usize, usize), src: u32, img: &[u32]) -> Result<(), DispErr>;
    fn fill(&mut self, f: &dyn Fn(usize, usize) -> u32) -> Result<(), DispErr>;
    fn flush(&mut self) -> Result<(), DispErr>;
    fn flush_blk(&mut self, pos: (i32, i32), size: (usize, usize)) -> Result<(), DispErr>;
}

#[derive(Debug, Clone, Copy)]
pub enum MemSizeUnits {
    Bytes,
    Kilo,
    Mega,
    Giga
}

pub trait Mem {
    fn free(&self, units: MemSizeUnits) -> Result<usize, MemErr>;
}

impl Display for TermKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TermKey::Char(c) => write!(f, "{}", c),
            TermKey::Esc => write!(f, "ESC"),
            TermKey::Up => write!(f, "UP"),
            TermKey::Down => write!(f, "DOWN"),
            TermKey::Left => write!(f, "LEFT"),
            TermKey::Right => write!(f, "RIGHT"),
            TermKey::Unknown => write!(f, "UNKNOWN")
        }
    }
}
