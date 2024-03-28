#![no_std]

use core::mem::MaybeUninit;

pub struct Router<'r, Handler, const CAP: usize = 50, const SEG_CAP: usize = 4> {
    len: usize,
    routes: [MaybeUninit<Route<'r, Handler, SEG_CAP>>; CAP],
}

impl<'r, Handler, const CAP: usize, const SEG_CAP: usize> Default for Router<'r, Handler, CAP, SEG_CAP> {
    fn default() -> Self { Self::new() }
}

impl<'r, Handler, const CAP: usize, const SEG_CAP: usize> Router<'r, Handler, CAP, SEG_CAP> {
    const UNINIT: MaybeUninit<Route<'r, Handler, SEG_CAP>> = MaybeUninit::uninit();
    const INIT: [MaybeUninit<Route<'r, Handler, SEG_CAP>>; CAP] = [Self::UNINIT; CAP];

    pub fn new() -> Self {
        Self { len: 0, routes: Self::INIT }
    }

    pub fn add_route(&mut self, route: Route<'r, Handler, SEG_CAP>) -> Result<(), Route<'_, Handler, SEG_CAP>> {
        if self.len == CAP { return Err(route); }
        self.routes[self.len].write(route); self.len += 1;
        Ok(())
    }

    pub fn find(&self, needle: &'r str) -> Option<&Route<'_, Handler, SEG_CAP>> {
        self.filter(needle).next()
    }
    pub fn filter(&'r self, needle: &'r str) -> impl Iterator<Item = &Route<'r, Handler, SEG_CAP>> {
        self.routes[..self.len].iter().filter(|r| {
            unsafe { r.assume_init_ref() }.full_match(needle)
        }).map(move |r| unsafe { r.assume_init_ref() })

    }
}

pub struct Route<'r, Handler, const CAP: usize> {
    len: usize,
    segments: [MaybeUninit<Segment<'r>>; CAP],
    handler: Handler
}

impl<'r, Handler, const CAP: usize> Route<'r, Handler, CAP> {
    pub fn new(route: &'r str, handler: Handler) -> Self {
        let route = route.trim_matches('/');
        let mut segments: [MaybeUninit<Segment<'r>>; CAP] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let route_segments = route.split('/');
        let mut seg_cnt = 0;

        for (idx, seg) in route_segments.enumerate() {
            seg_cnt+=1;
            if idx == CAP { break; }
            segments[idx].write(parse_segment(seg));
        }

        Self { len: seg_cnt, segments, handler }
    }
    fn full_match(&self, needle: &str) -> bool {
        let mut offset = 0;

        for seg in &self.segments[..self.len] {
            let seg = unsafe { seg.assume_init_ref() };
            if offset >= needle.len() { return true; }
            if match_segment(needle, seg, offset) { offset+=seg.len(); continue; } else { return false; }
        }
        true
    }

    pub fn handler(&self) -> &Handler { &self.handler }
}

pub enum Segment<'s> {
    Constant(&'s str),
    Named(&'s str),
    Wildcard,
}

impl Segment<'_> {
    const fn len(&self) -> usize {
        match self {
            Self::Constant(s) => s.len(),
            Self::Named(s) => s.len()+1,
            Self::Wildcard => 1,
        }
    }
}

fn parse_segment(input: &'_ str) -> Segment<'_> {
    // TODO: do better than this
    match &input[0..1] {
        ":" => Segment::Named(&input[1..]),
        "*" => Segment::Wildcard,
        _ => Segment::Constant(input)
    }
}
fn match_segment(needle: &str, seg: &Segment<'_>, offset: usize) -> bool {
    match seg {
        Segment::Constant(s) => needle[offset..].starts_with(s),
        Segment::Named(s) => needle[offset+1..].starts_with(s),
        Segment::Wildcard => needle[offset..].starts_with('*'),
    }
}
