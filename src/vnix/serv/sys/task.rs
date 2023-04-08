use core::pin::Pin;
use core::ops::{Generator, GeneratorState};

use spin::Mutex;

use alloc::rc::Rc;
use alloc::boxed::Box;
use alloc::string::String;

use crate::vnix::utils::Maybe;
use crate::{thread, thread_await, read_async, as_map_find_async, maybe, as_map_find_as_async, as_async, maybe_ok, task_result};

use crate::vnix::core::msg::Msg;
use crate::vnix::core::kern::{Kern, KernErr};
use crate::vnix::core::task::{ThreadAsync, TaskRun, TaskSig};
use crate::vnix::core::serv::{ServHlrAsync, ServInfo};
use crate::vnix::core::unit::{Unit, UnitReadAsyncI, UnitModify, UnitAs, UnitNew, UnitReadAsync, UnitTypeReadAsync};


pub const SERV_PATH: &'static str = "sys.task";
pub const SERV_HELP: &'static str = "Service for run task from message\nExample: (load @task.hello)@io.store@sys.task";


fn stream(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> UnitReadAsync {
    thread!({
        maybe_ok!(msg.clone().as_stream());

        let (msg, ath) = maybe!(read_async!(msg, ath, orig, kern));
        Ok(Some((msg, ath)))
    })
}

fn _loop(mut ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<Rc<String>, KernErr>> {
    thread!({
        let msg = if let Some(msg) = msg.clone().as_map_find("task.loop") {
            msg
        } else if let Some((s, msg)) = msg.clone().as_pair() {
            let (s, _ath) = maybe!(as_async!(s, as_str, ath, orig, kern));
            ath = _ath;

            if s.as_str() != "task.loop" {
                return Ok(None)
            }
            msg
        } else {
            return Ok(None)
        };

        // loop count
        if let Some((cnt, msg)) = msg.clone().as_pair() {
            let (cnt, mut ath) = maybe!(as_async!(cnt, as_uint, ath, orig, kern));

            for _ in 0..cnt {
                if let Some((_, _ath)) = read_async!(msg, ath, orig, kern)? {
                    ath = _ath;
                }
            }
            return Ok(Some(ath))
        }

        // infinite
        loop {
            read_async!(msg, ath, orig, kern)?;
        }
    })
}

fn separate(mut ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<Rc<String>, KernErr>> {
    thread!({
        let msg = if let Some(msg) = msg.clone().as_map_find("task.sep") {
            msg
        } else if let Some((s, msg)) = msg.clone().as_pair() {
            let (s, _ath) = maybe!(as_async!(s, as_str, ath, orig, kern));
            ath = _ath;

            if s.as_str() != "task.sep" {
                return Ok(None)
            }
            msg
        } else {
            return Ok(None)
        };

        // infinite
        if let Some((_msg, serv, _)) = msg.as_stream() {
            let run = TaskRun(_msg, serv);
            kern.lock().reg_task(&ath, "sys.task", run)?;
        }

        Ok(Some(ath))
    })
}

fn chain(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> UnitReadAsync {
    thread!({
        let (lst, mut ath) = maybe!(as_map_find_as_async!(msg, "task", as_list, ath, orig, kern));
        let mut _msg = if let Some((_msg, _ath)) = as_map_find_async!(msg, "msg", ath, orig, kern)? {
            ath = _ath;
            _msg
        } else {
            msg.clone()
        };

        for p in Rc::unwrap_or_clone(lst) {
            let (serv, _ath) = maybe!(as_async!(p, as_str, ath, orig, kern));
            let prev = _msg.clone();

            let run = TaskRun(_msg, Rc::unwrap_or_clone(serv));
            let id = kern.lock().reg_task(&_ath, "sys.task", run)?;

            let __msg = maybe_ok!(task_result!(id, kern)?);

            _msg = prev.merge_with(__msg.msg);
            ath = Rc::new(__msg.ath);
        }
        return Ok(Some((_msg, ath)))
    })
}

fn queue(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<Rc<String>, KernErr>> {
    thread!({
        let (lst, mut ath) = if let Some((lst, ath)) =  as_map_find_as_async!(msg, "task.que", as_list, ath, orig, kern)? {
            (lst, ath)
        } else if let Some((s, lst)) = msg.as_pair() {
            let (s, ath) = maybe!(as_async!(s, as_str, ath, orig, kern));

            if s.as_str() != "task.que" {
                return Ok(None)
            }

            let (lst, ath) = maybe!(as_async!(lst, as_list, ath, orig, kern));
            (lst, ath)
        } else {
            return Ok(None)
        };

        for p in Rc::unwrap_or_clone(lst) {
            if let Some((_, _ath)) = read_async!(p, ath, orig, kern)? {
                ath = _ath;
            }
        }
        Ok(Some(ath))
    })
}

fn sim(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<(), KernErr>> {
    thread!({
        let lst = if let Some((lst, _)) =  as_map_find_as_async!(msg, "task.sim", as_list, ath, orig, kern)? {
            lst
        } else if let Some((s, lst)) = msg.as_pair() {
            let (s, ath) = maybe!(as_async!(s, as_str, ath, orig, kern));

            if s.as_str() != "task.sim" {
                return Ok(None)
            }

            let (lst, _) = maybe!(as_async!(lst, as_list, ath, orig, kern));
            lst
        } else {
            return Ok(None)
        };

        for p in lst.iter() {
            if let Some((_msg, serv, _)) = p.clone().as_stream() {
                let run = TaskRun(_msg, serv);
                kern.lock().reg_task(&ath, "sys.task", run)?;
            }
        }
        Ok(None)
    })
}

fn stack(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<Rc<String>, KernErr>> {
    thread!({
        // let (u, serv, _) = maybe_ok!(msg.as_map_find("task.stk").and_then(|u| u.as_stream()));
        let (u, serv, _) = if let Some((u, serv, addr)) = msg.clone().as_map_find("task.stk").and_then(|u| u.as_stream()) {
            (u, serv, addr)
        } else if let Some((s, msg)) = msg.clone().as_pair() {
            let (s, _) = maybe!(as_async!(s, as_str, ath, orig, kern));

            if s.as_str() != "task.stk" {
                return Ok(None)
            }
            maybe_ok!(msg.as_stream())
        } else {
            return Ok(None)
        };

        let (lst, mut ath) = maybe!(as_async!(u, as_list, ath, orig, kern));

        for p in Rc::unwrap_or_clone(lst) {
            let (msg, _ath) = maybe!(read_async!(p, ath, orig, kern));
            ath = _ath;

            let run = TaskRun(msg, serv.clone());
            let id = kern.lock().reg_task(&ath, "sys.task", run)?;

            if let Some(msg) = task_result!(id, kern)? {
                ath = Rc::new(msg.ath);
            }
        }
        Ok(Some(ath))
    })
}

fn run(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> UnitTypeReadAsync<Option<Unit>> {
    thread!({
        // loop
        if let Some(_ath) = thread_await!(_loop(ath.clone(), msg.clone(), orig.clone(), kern))? {
            if _ath != ath {
                return Ok(Some((Some(msg), ath)))
            }
            return Ok(Some((None, ath)))
        }

        // separate
        if let Some(_ath) = thread_await!(separate(ath.clone(), msg.clone(), orig.clone(), kern))? {
            if _ath != ath {
                return Ok(Some((Some(msg), ath)))
            }
            return Ok(Some((None, ath)))
        }

        // chain
        if let Some((msg, ath)) = thread_await!(chain(ath.clone(), msg.clone(), orig.clone(), kern))? {
            let msg = Unit::map(&[
                (Unit::str("msg"), msg)]
            );
            return Ok(Some((Some(msg), ath)))
        }
    
        // sim
        thread_await!(sim(ath.clone(), msg.clone(), orig.clone(), kern))?;
    
        // queue
        if let Some(_ath) = thread_await!(queue(ath.clone(), msg.clone(), orig.clone(), kern))? {
            if _ath != ath {
                return Ok(Some((Some(msg), ath)))
            }
            return Ok(Some((None, ath)))
        }

        // stack
        if let Some(_ath) = thread_await!(stack(ath.clone(), msg.clone(), orig.clone(), kern))? {
            if _ath != ath {
                return Ok(Some((Some(msg), ath)))
            }
            return Ok(Some((None, ath)))
        }

        // stream
        if let Some((msg, ath)) = thread_await!(stream(ath.clone(), msg.clone(), orig.clone(), kern))? {
            let msg = Unit::map(&[
                (Unit::str("msg"), msg)]
            );
            return Ok(Some((Some(msg), ath)))
        }

        Ok(None)
    })
}

pub fn signal(ath: Rc<String>, orig: Unit, msg: Unit, kern: &Mutex<Kern>) -> ThreadAsync<Maybe<Rc<String>, KernErr>> {
    thread!({
        let (sig, id) = maybe_ok!(msg.as_pair());

        let (sig, ath) = maybe!(as_async!(sig, as_str, ath, orig, kern));
        let (id, ath) = maybe!(as_async!(id, as_uint, ath, orig, kern));

        match sig.as_str() {
            "kill" => kern.lock().task_sig(id as usize, TaskSig::Kill)?,
            _ => return Ok(None)
        }

        Ok(Some(ath))
    })
}

pub fn task_hlr(mut msg: Msg, _serv: ServInfo, kern: &Mutex<Kern>) -> ServHlrAsync {
    thread!({
        let ath = Rc::new(msg.ath.clone());
        let (_msg, mut ath) = maybe!(read_async!(msg.msg.clone(), ath, msg.msg.clone(), kern));

        // task
        if let Some((__msg, ath)) = thread_await!(run(ath.clone(), _msg.clone(), _msg.clone(), kern))? {
            let msg = _msg.clone().merge_with(maybe_ok!(__msg));
            return kern.lock().msg(&ath, msg).map(|msg| Some(msg))
        }

        // signal
        if let Some(_ath) = thread_await!(signal(ath.clone(), _msg.clone(), _msg.clone(), kern))? {
            if _ath != ath {
                ath = _ath;
                msg = kern.lock().msg(&ath, _msg.clone())?;
            }
            return Ok(Some(msg))
        }

        Ok(Some(msg))
    })
}
