use alloc::vec::Vec;

use super::msg::Msg;
use super::serv::Serv;
use super::unit::Unit;
use super::unit::UnitParseErr;

use super::user::Usr;

use crate::vnix::serv::{io, etc, gfx};

use crate::driver::{CLIErr, DispErr, TimeErr, CLI, Disp, Time};

#[derive(Debug)]
pub enum KernErr {
    MemoryOut,
    EncodeFault,
    UsrNotFound,
    ServNotFound,
    ParseErr(UnitParseErr),
    CLIErr(CLIErr),
    DispErr(DispErr),
    TimeErr(TimeErr)
}

pub struct Kern<'a> {
    // drivers
    pub cli: &'a mut dyn CLI,
    pub disp: &'a mut dyn Disp,
    pub time: &'a mut dyn Time,

    // vnix
    users: Vec<Usr>
}

impl<'a> Kern<'a> {
    pub fn new(cli: &'a mut dyn CLI, disp: &'a mut dyn Disp, time: &'a mut dyn Time) -> Self {
        let kern = Kern {
            cli,
            disp,
            time,
            users: Vec::new(),
        };

        kern
    }

    pub fn reg_usr(&mut self, usr: Usr) -> Result<(), KernErr> {
        self.users.push(usr);
        Ok(())
    }

    pub fn msg(&self, ath: &str, u: Unit) -> Result<Msg, KernErr> {
        let usr = self.users.iter().find(|usr| usr.name == ath).ok_or(KernErr::UsrNotFound).cloned()?;
        Msg::new(usr, u)
    }

    pub fn task(&mut self, msg: Msg) -> Result<Option<Msg>, KernErr> {
        if let Unit::Map(ref m) = msg.msg {
            let serv = m.iter().filter_map(|p| Some((p.0.as_str()?, p.1.as_str()?))).find(|(u, _)| u == "task").map(|(_, s)| s);

            if let Some(serv) = serv {
                return self.send(serv.as_str(), msg);
            }
        }

        Ok(None)
    }

    pub fn send(&mut self, serv: &str, msg: Msg) -> Result<Option<Msg>, KernErr> {
        match serv {
            "io.term" => {
                let (inst, msg) = io::Term::inst(msg, self)?;
                inst.handle(msg, self)
            },
            "etc.chrono" => {
                let (inst, msg) = etc::Chrono::inst(msg, self)?;
                inst.handle(msg, self)
            },
            "gfx.2d" => {
                let (inst, msg) = gfx::GFX2D::inst(msg, self)?;
                inst.handle(msg, self)
            }
            _ => Err(KernErr::ServNotFound)
        }
    }
}
