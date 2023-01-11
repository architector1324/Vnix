use core::ops::Deref;

use alloc::vec::Vec;
use alloc::vec;

use crate::vnix::core::msg::Msg;
use crate::vnix::core::unit::Unit;

use crate::vnix::core::serv::Serv;
use crate::vnix::core::kern::{KernErr, Kern};


pub struct GFX2D {
    fill: Option<u32>
}

impl Default for GFX2D {
    fn default() -> Self {
        GFX2D {
            fill: None
        }
    }
}

impl Serv for GFX2D {
    fn inst(msg: Msg, kern: &mut Kern) -> Result<(Self, Msg), KernErr> {
        let mut inst = GFX2D::default();

        // config instance
        if let Unit::Map(ref m) = msg.msg {
            let mut it = m.iter().filter_map(|p| Some((p.0.as_str()?, p.1.as_int()?)));
            it.find(|(s, _)| s == "fill").map(|(_, col)| inst.fill.replace(col as u32));
        }

        Ok((inst, msg))
    }

    fn handle(&self, msg: Msg, kern: &mut Kern) -> Result<Option<Msg>, KernErr> {
        if let Some(col) = self.fill {
            let img: Vec::<Unit> = (0..1920*1080).map(|_| Unit::Int(col as i32)).collect();
            let m = vec![
                (Unit::Str("img".into()), Unit::Lst(img)),
                (Unit::Str("task".into()), Unit::Str("io.term".into())) // FIXME: remove it!
            ];

            return Ok(Some(kern.msg(&msg.ath.name, Unit::Map(m))?))
        }

        Ok(None)
    }
}
