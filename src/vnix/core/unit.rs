use core::ops::Deref;
use core::str::Chars;
use core::fmt::{Display, Formatter};

use alloc::format;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::{String, ToString};


#[derive(Debug)]
pub enum UnitParseErr {
    NotNone,
    NotBool,
    NotByte,
    NotInt,
    NotDec,
    NotStr,
    NotRef,
    NotPair,
    NotList,
    NotMap,
    NotUnit,
    NotClosedBrackets,
    NotClosedQuotes,
    MissedSeparator,
    UnexpectedEnd,
    MissedDot,
    MissedPartAfterDot,
    RefNotString,
    RefInvalidPath
}

#[derive(Debug, PartialEq, Clone)]
pub enum Unit {
    None,
    Bool(bool),
    Byte(u8),
    Int(i32),
    Dec(f32),
    Str(String),
    Ref(Vec<String>),
    Pair((Box<Unit>, Box<Unit>)),
    Lst(Vec<Unit>),
    Map(Vec<(Unit, Unit)>)
}

#[derive(Debug)]
pub enum SchemaUnit<'a> {
    None(&'a mut Option<()>),
    Bool(&'a mut Option<bool>),
    Byte(&'a mut Option<u8>),
    Int(&'a mut Option<i32>),
    Dec(&'a mut Option<f32>),
    Str(&'a mut Option<String>),
    // Ref,
    Pair((Box<Schema<'a>>, Box<Schema<'a>>)),
    Lst(Vec<Schema<'a>>),
    Map(Vec<(Schema<'a>, Schema<'a>)>),
    Unit(&'a mut Option<Unit>),
}

#[derive(Debug)]
pub enum Schema<'a> {
    Value(Unit),
    Unit(SchemaUnit<'a>),
    Or((Box<Schema<'a>>, Box<Schema<'a>>))
}

pub trait FromUnit: Sized {
    fn from_unit(u: &Unit) -> Option<Self>;
}

impl Eq for Unit {}

impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Unit::None => write!(f, "-"),
            Unit::Bool(b) => {
                if *b {
                    write!(f, "t")
                } else {
                    write!(f, "f")
                }
            },
            Unit::Byte(b) => write!(f, "{:#02x}", b),
            Unit::Int(i) => write!(f, "{}", i),
            Unit::Dec(d) => write!(f, "{}", d),
            Unit::Str(s) => {
                if s.as_str().chars().all(|c| c.is_alphanumeric() || c == '.' || c == '#') {
                    write!(f, "{}", s)
                } else {
                    write!(f, "`{}`", s)
                }
            },
            Unit::Ref(path) => write!(f, "@{}", path.join(".")),
            Unit::Pair(p) => write!(f, "({} {})", p.0, p.1),
            Unit::Lst(lst) => {
                write!(f, "[")?;

                for (i, u) in lst.iter().enumerate() {
                    if i == lst.len() - 1 {
                        write!(f, "{}", u)?;
                    } else {
                        write!(f, "{} ", u)?;
                    }
                }

                write!(f, "]")
            },
            Unit::Map(map) => {
                write!(f, "{{")?;

                for (i, (u0, u1)) in map.iter().enumerate() {
                    if i == map.len() - 1 {
                        write!(f, "{}:{}", u0, u1)?;
                    } else {
                        write!(f, "{}:{} ", u0, u1)?;
                    }
                }

                write!(f, "}}")
            }
        }
    }
}

impl Unit {
    fn parse_ch<'a>(ch: char, it: Chars<'a>) -> (bool, Chars<'a>) {
        let mut tmp = it.clone();

        if let Some(c) = tmp.next() {
            if c == ch {
                return (true, tmp)
            }
        }
        (false, it)
    }

    fn parse_ws<'a>(it: Chars<'a>) -> (bool, Chars<'a>) {
        let mut tmp = it.clone();

        if let Some(c) = tmp.next() {
            let mut it = tmp;

            if c.is_ascii_whitespace() {
                tmp = it.clone();

                while let Some(c) = it.next() {
                    if !c.is_ascii_whitespace() {
                        break;
                    }
                    tmp = it.clone();
                }

                return (true, tmp);
            }
        }

        (false, it)
    }

    fn parse_none<'a>(it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        let (ok, tmp) = Unit::parse_ch('-', it);

        if ok {
            return Ok((Unit::None, tmp));
        }

       Err(UnitParseErr::NotNone)
    }

    fn parse_bool<'a>(it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        let (ok_t, tmp_t) = Unit::parse_ch('t', it.clone());
        let (ok_f, tmp_f) = Unit::parse_ch('f', it);

        let mut tmp = if ok_t {tmp_t.clone()} else {tmp_f.clone()};

        if let Some(c) = tmp.next() {
            if c.is_alphanumeric() {
                return Err(UnitParseErr::NotBool);
            }
        }

        if ok_t {
            return Ok((Unit::Bool(true), tmp_t))
        }

        if ok_f {
            return Ok((Unit::Bool(false), tmp_f))
        }

        Err(UnitParseErr::NotBool)
    }

    fn parse_byte<'a>(mut it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        if let Some(s) = it.as_str().get(0..4) {
            it.next().unwrap();
            it.next().unwrap();
            it.next().unwrap();
            it.next().unwrap();

            if let Ok(v) = u8::from_str_radix(s.trim_start_matches("0x"), 16) {
                return Ok((Unit::Byte(v), it))
            }
        }

        Err(UnitParseErr::NotByte)
    }

    fn parse_int<'a>(mut it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        let mut s = String::new();
        let mut tmp = it.clone();

        while let Some(c) = it.next() {
            if !(c.is_numeric() || c == '-') {
                break;
            }

            s.push(c);
            tmp = it.clone();
        }

        if let Ok(v) = s.parse::<i32>() {
            return Ok((Unit::Int(v), tmp));
        }

        Err(UnitParseErr::NotInt)
    }

    fn parse_dec<'a>(it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        if let Ok((fst, mut it)) = Unit::parse_int(it) {
            let (ok, tmp) = Unit::parse_ch('.', it);

            if !ok {
                return Err(UnitParseErr::MissedDot);
            }

            it = tmp;

            if let Ok((scd, it)) = Unit::parse_int(it) {
                let s = format!("{}.{}", fst, scd);
                let out = s.parse::<f32>().map_err(|_| UnitParseErr::NotDec)?;

                return Ok((Unit::Dec(out), it));
            } else {
                return Err(UnitParseErr::MissedPartAfterDot);
            }
        }
        Err(UnitParseErr::NotDec)
    }

    fn parse_str<'a>(mut it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        if let Some(c) = it.next() {
            // `complex string`
            if c == '`' {
                let mut s = String::new();
                let mut tmp = it.clone();

                while let Some(c) = it.next() {
                    if c == '`' {
                        break;
                    }

                    s.push(c);
                    tmp = it.clone();
                }

                if let Some(c) = tmp.next() {
                    if c == '`' {
                        return Ok((Unit::Str(s), tmp));
                    } else {
                        return Err(UnitParseErr::NotClosedQuotes);
                    }
                } else {
                    return Err(UnitParseErr::NotClosedQuotes);
                }
            }

            // abc.123#
            if c.is_alphanumeric() || c == '.' || c == '#' {
                let mut s = String::new();
                let mut tmp = it.clone();

                s.push(c);

                while let Some(c) = it.next() {
                    if !(c.is_alphanumeric() || c == '.' || c == '#') {
                        break;
                    }

                    s.push(c);
                    tmp = it.clone();
                }

                return Ok((Unit::Str(s), tmp));
            }
        }
        Err(UnitParseErr::NotStr)
    }

    fn parse_ref<'a>(mut it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        let (ok, tmp) = Unit::parse_ch('@', it);

        if !ok {
            return Err(UnitParseErr::NotRef);
        }

        it = tmp;

        let tmp = Unit::parse_str(it)?;

        if let Unit::Str(path) = tmp.0 {
            let path = path.split(".").map(|s| s.to_string()).collect::<Vec<_>>();

            for p in &path {
                if !p.chars().all(|c| c.is_alphanumeric()) {
                    return Err(UnitParseErr::RefInvalidPath);
                }
            }

            return Ok((Unit::Ref(path), tmp.1));
        }
        return Err(UnitParseErr::RefNotString);
    }

    fn parse_pair<'a>(mut it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        let (ok, tmp) = Unit::parse_ch('(', it);

        if !ok {
            return Err(UnitParseErr::NotPair)
        }

        it = tmp;

        let u0 = Unit::parse(it)?;
        it = u0.1;

        let (ok, tmp) = Unit::parse_ws(it);

        if !ok {
            return Err(UnitParseErr::MissedSeparator);
        }

        it = tmp;

        let u1 = Unit::parse(it)?;
        it = u1.1;

        let (ok, tmp) = Unit::parse_ch(')', it);

        if !ok {
            return Err(UnitParseErr::NotClosedBrackets);
        }

        it = tmp;

        return Ok((
            Unit::Pair((
                Box::new(u0.0),
                Box::new(u1.0)
            )),
            it
        ));
    }

    fn parse_list<'a>(mut it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        let (ok, tmp) = Unit::parse_ch('[', it);

        if !ok {
            return Err(UnitParseErr::NotList);
        }

        it = tmp;

        let mut lst = Vec::new();

        loop {
            let (_, tmp) = Unit::parse_ws(it);
            it = tmp;

            let u = Unit::parse(it)?;
            lst.push(u.0);
            it = u.1;

            let (ok_ws, tmp) = Unit::parse_ws(it);
            it = tmp;

            let (ok, tmp) = Unit::parse_ch(']', it);

            if ok {
                it = tmp;
                return Ok((Unit::Lst(lst), it))
            } else if !ok_ws {
                return Err(UnitParseErr::MissedSeparator);
            }

            it = tmp;
        }
    }

    fn parse_map<'a>(mut it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        let (ok, tmp) = Unit::parse_ch('{', it);

        if !ok {
            return Err(UnitParseErr::NotMap);
        }

        it = tmp;

        let mut map = Vec::new();

        loop {
            let (_, tmp) = Unit::parse_ws(it);
            it = tmp;

            let u0 = Unit::parse(it)?;
            it = u0.1;

            let (_, tmp) = Unit::parse_ws(it);
            it = tmp;

            let (ok, tmp) = Unit::parse_ch(':', it);

            if !ok {
                return Err(UnitParseErr::MissedSeparator);
            }

            it = tmp;

            let (_, tmp) = Unit::parse_ws(it);
            it = tmp;

            let u1 = Unit::parse(it)?;
            it = u1.1;

            map.push((u0.0, u1.0));

            let (ok_ws, tmp) = Unit::parse_ws(it);
            it = tmp;

            let (ok, tmp) = Unit::parse_ch('}', it);

            if ok {
                it = tmp;
                return Ok((Unit::Map(map), it))
            } else if !ok_ws {
                return Err(UnitParseErr::MissedSeparator);
            }

            it = tmp;
        }
    }

    pub fn parse<'a>(it: Chars<'a>) -> Result<(Self, Chars<'a>), UnitParseErr> {
        // bool
        if let Ok((u, it)) = Unit::parse_bool(it.clone()) {
            return Ok((u, it));
        }

        // byte
        if let Ok((u, it)) = Unit::parse_byte(it.clone()) {
            return Ok((u, it));
        }

        // dec
        if let Ok((u, it)) = Unit::parse_dec(it.clone()) {
            return Ok((u, it));
        }

        // int
        if let Ok((u, it)) = Unit::parse_int(it.clone()) {
            return Ok((u, it));
        }

        // none
        if let Ok((u, it)) = Unit::parse_none(it.clone()) {
            return Ok((u, it));
        }

        // str
        if let Ok((u, it)) = Unit::parse_str(it.clone()) {
            return Ok((u, it));
        }

        // pair
        if let Ok((u, it)) = Unit::parse_pair(it.clone()) {
            return Ok((u, it));
        }

        if let Ok((u, it)) = Unit::parse_ref(it.clone()) {
            return Ok((u, it));
        }

        // list
        if let Ok((u, it)) = Unit::parse_list(it.clone()) {
            return Ok((u, it));
        }

        // map
        if let Ok((u, it)) = Unit::parse_map(it.clone()) {
            return Ok((u, it));
        }

        Err(UnitParseErr::NotUnit)
    }

    fn find_unit_loc<'a, I>(&self, glob: &Unit, path: &mut I) -> Option<Unit> where I: Iterator<Item = &'a String> {
        if let Some(curr) = path.next() {
            if let Unit::Ref(n_path) = self {
                return glob.find_unit(&mut n_path.iter());
            }

            if let Unit::Pair(p) = self {
                if curr == "0" {
                    return p.0.deref().find_unit_loc(glob, path);
                } else if curr == "1" {
                    return p.1.deref().find_unit_loc(glob, path);
                }
            }

            if let Unit::Lst(lst) = self {
                let idx = curr.parse::<usize>().ok()?;
                if let Some(u) = lst.get(idx) {
                    return u.find_unit_loc(glob, path);
                }
            }

            if let Unit::Map(m) = self {
                return m.iter().filter_map(|(u0, u1)| Some((u0.as_str()?, u1)))
                        .find(|(s, _)| *s == *curr)
                        .map(|(_, u)| u.find_unit_loc(glob, path)).flatten();
            }

            return None;
        } else {
            if let Unit::Ref(n_path) = self {
                return glob.find_unit(&mut n_path.iter());
            }

            return Some(self.clone());
        }
    }

    pub fn find_unit<'a, I>(&self, path: &mut I) -> Option<Unit> where I: Iterator<Item = &'a String> {
        self.find_unit_loc(self, path)
    }

    pub fn find_bool<'a, I>(&self, path: &mut I) -> Option<bool> where I: Iterator<Item = &'a String> {
        self.find_unit(path).map(|u| u.as_bool()).flatten()
    }

    pub fn find_int<'a, I>(&self, path: &mut I) -> Option<i32> where I: Iterator<Item = &'a String> {
        self.find_unit(path).map(|u| u.as_int()).flatten()
    }

    pub fn find_str<'a, I>(&self, path: &mut I) -> Option<String> where I: Iterator<Item = &'a String> {
        self.find_unit(path).map(|u| u.as_str()).flatten()
    }

    pub fn find_pair<'a, I>(&self, path: &mut I) -> Option<(Unit, Unit)> where I: Iterator<Item = &'a String> {
        self.find_unit(path).map(|u| u.as_pair()).flatten().map(|p| (p.0.deref().clone(), p.1.deref().clone()))
    }

    pub fn find_list<'a, I>(&self, path: &mut I) -> Option<Vec<Unit>> where I: Iterator<Item = &'a String> {
        self.find_unit(path).map(|u| u.as_vec()).flatten()
    }

    pub fn find_map<'a, I>(&self, path: &mut I) -> Option<Vec<(Unit, Unit)>> where I: Iterator<Item = &'a String> {
        self.find_unit(path).map(|u| u.as_map()).flatten()
    }

    pub fn as_none(&self) -> Option<()> {
        if let Unit::None = self {
            return Some(())
        }
        None
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Unit::Bool(v) = self {
            return Some(*v)
        }
        None
    }

    pub fn as_byte(&self) -> Option<u8> {
        if let Unit::Byte(v) = self {
            return Some(*v)
        }
        None
    }

    pub fn as_int(&self) -> Option<i32> {
        if let Unit::Int(v) = self {
            return Some(*v)
        }
        None
    }

    pub fn as_dec(&self) -> Option<f32> {
        if let Unit::Dec(v) = self {
            return Some(*v)
        }
        None
    }

    pub fn as_str(&self) -> Option<String> {
        if let Unit::Str(s) = self {
            return Some(s.clone())
        }
        None
    }

    pub fn as_ref(&self) -> Option<Vec<String>> {
        if let Unit::Ref(path) = self {
            return Some(path.clone());
        }
        None
    }

    pub fn as_pair(&self) -> Option<(Box<Unit>, Box<Unit>)> {
        if let Unit::Pair((u0, u1)) = self {
            return Some((u0.clone(), u1.clone()))
        }
        None
    }

    pub fn as_vec(&self) -> Option<Vec<Unit>> {
        if let Unit::Lst(lst) = self {
            return Some(lst.clone());
        }
        None
    }

    pub fn as_vec_typed<A, B>(&self, f: B) -> Option<Vec<A>> where A: Clone, B: Fn(&Self) -> Option<A> {
        if let Unit::Lst(lst) = self {
            return Some(lst.iter().filter_map(|u| f(u)).collect());
        }
        None
    }

    pub fn as_map(&self) -> Option<Vec<(Unit, Unit)>> {
        if let Unit::Map(m) = self {
            return Some(m.clone());
        }
        None
    }

    pub fn as_map_find(&self, sch: &str) -> Option<Unit> {
        if let Unit::Map(m) = self {
            return m.iter()
                .filter_map(|(u0, u1)| Some((u0.as_str()?, u1)))
                .map(|(s, u)| {
                    if s == sch {
                        return Some(u.clone());
                    }
                    None
                }).next()?;
        }
        None
    }

    pub fn merge(self, u: Unit) -> Unit {
        if let Unit::Map(m) = u {
            if let Some(mut tmp) = self.as_map() {
                tmp.retain(|(u, _)| {
                    m.iter().find(|(n, _)| u == n).is_none()
                });

                tmp.extend(m);

                return Unit::Map(tmp);
            }
        }
        self
    }
}


impl<'a> SchemaUnit<'a> {
    fn find(&mut self, glob:&Unit, u: &Unit) -> bool {
        if let Unit::Ref(path) = u {
            if let Some(u) = glob.find_unit(&mut path.iter()) {
                return self.find(glob, &u);
            }
        }

        match self {
            SchemaUnit::None(..) => {
                if let Unit::None = u {
                    return true;
                }
            }
            SchemaUnit::Bool(ref mut b) => {
                if let Unit::Bool(v) = u {
                    b.replace(*v);
                    return true;
                }
            },
            SchemaUnit::Byte(ref mut b) => {
                if let Unit::Byte(v) = u {
                    b.replace(*v);
                    return true;
                }
            },
            SchemaUnit::Int(ref mut i) => {
                if let Unit::Int(v) = u {
                    i.replace(*v);
                    return true;
                }
            },
            SchemaUnit::Dec(ref mut f) => {
                if let Unit::Dec(v) = u {
                    f.replace(*v);
                    return true;
                }
            },
            SchemaUnit::Str(ref mut s) => {
                if let Unit::Str(v) = u {
                    s.replace(v.clone());
                    return true;
                }
            },
            SchemaUnit::Pair(ref mut p) => {
                if let Unit::Pair((u0, u1)) = u {
                    return p.0.find_loc(glob, u0) && p.1.find_loc(glob, u1);
                }
            },
            SchemaUnit::Lst(ref mut l) => {
                if let Unit::Lst(u_lst) = u {
                    return u_lst.iter().zip(l.iter_mut()).map(|(u, s)| {
                        s.find_loc(glob, &u)
                    }).fold(true, |a, b| a && b);
                }
            },
            SchemaUnit::Map(ref mut m) => {
                if let Unit::Map(u_m) = u {
                    return u_m.iter().map(|u_p| {
                        m.iter_mut().map(|s_p| {
                            if s_p.0.find_loc(glob, &u_p.0) {
                                return s_p.1.find_loc(glob, &u_p.1);
                            }
                            false
                        }).fold(false, |a, b| a || b)
                    }).fold(false, |a, b| a || b)
                }
            },
            SchemaUnit::Unit(_u) => {
                _u.replace(u.clone());
                return true;
            }
        }
        return false;
    }
}

impl<'a> Schema<'a> {
    pub fn find_loc(&mut self, glob:&Unit, u: &Unit) -> bool {
        match self {
            Schema::Unit(_u) => _u.find(glob, u),
            Schema::Value(ref _u) => _u.clone() == u.clone(),
            Schema::Or((ref mut schm0, ref mut schm1)) => schm0.find_loc(glob, u) || schm1.find_loc(glob, u)
        }
    }

    pub fn find(&mut self, u: &Unit) -> bool {
        self.find_loc(u, u)
    }
}
