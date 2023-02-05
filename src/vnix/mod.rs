pub mod core;
pub mod serv;
pub mod utils;

use alloc::string::String;
use alloc::vec;

use crate::driver::CLIErr;

use self::core::unit::Unit;
use self::core::user::Usr;
use self::core::kern::{Kern, KernErr};
use self::core::serv::{Serv, ServKind};


pub fn vnix_entry(mut kern: Kern) -> Result<(), KernErr> {
    // register service
    let services = vec![
        ("io.term", ServKind::IOTerm),
        ("io.store", ServKind::IOStore),
        ("etc.chrono", ServKind::EtcChrono),
        ("etc.fsm", ServKind::EtcFSM),
        ("gfx.2d", ServKind::GFX2D),
        ("math.int", ServKind::MathInt),
        ("sys.task", ServKind::SysTask),
        ("sys.usr", ServKind::SysUsr),
    ];

    for (name, kind) in services {
        let serv = Serv::new(name, kind);
        kern.reg_serv(serv)?;

        writeln!(kern.cli, "INFO vnix:kern: service `{}` registered", name).map_err(|_| KernErr::CLIErr(CLIErr::Write))?;
    }

    // register user
    let _super = Usr::new("super", &mut kern)?.0;
    kern.reg_usr(_super.clone())?;

    writeln!(kern.cli, "INFO vnix:kern: user `{}` registered", _super).map_err(|_| KernErr::CLIErr(CLIErr::Write))?;

    let s = "{store:(load @img.minecraft.grass) term.gfx:[@msg key] task:[io.store io.term]}";
    let u = Unit::parse(s.chars()).map_err(|e| KernErr::ParseErr(e))?.0;

    let msg = kern.msg("super", u)?;
    kern.task(msg)?;

    Ok(())

    // // login task
    // let mut ath: String = "super".into();

    // 'login: loop {
    //     let path = Unit::parse("@task.gfx.login".chars()).map_err(|e| KernErr::ParseErr(e))?.0;

    //     let u = kern.ram_store.load(path).ok_or(KernErr::DbLoadFault)?;
    //     let msg = kern.msg("super", u)?;
    
    //     let go = kern.task(msg);

    //     match go {
    //         Err(e) => writeln!(kern.cli, "ERR vnix:kern: failed to login {:?}", e).map_err(|_| KernErr::CLIErr(CLIErr::Write))?,
    //         Ok(msg) => {
    //             if let Some(msg) = msg {
    //                 ath = msg.ath;
    //                 break 'login;
    //             }
    //         }
    //     }
    // }

    // // zen
    // let path = Unit::parse("@task.gfx.desk.zen".chars()).map_err(|e| KernErr::ParseErr(e))?.0;

    // let u = kern.ram_store.load(path).ok_or(KernErr::DbLoadFault)?;
    // let msg = kern.msg(&ath, u)?;

    // kern.task(msg)?;

    // // λ
    // loop {
    //     let path = Unit::parse("@task.gfx.lambda".chars()).map_err(|e| KernErr::ParseErr(e))?.0;

    //     let u = kern.ram_store.load(path).ok_or(KernErr::DbLoadFault)?;
    //     let msg = kern.msg(&ath, u)?;

    //     // run
    //     if let Err(e) = kern.task(msg) {
    //         writeln!(kern.cli, "ERR vnix:kern: {:?}", e).map_err(|_| KernErr::CLIErr(CLIErr::Write))?;
    //     }
    // }
}
