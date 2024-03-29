#![no_std]

use core::mem::MaybeUninit;

// the api for Router and Route is heavily stolen from the heapless crate's Vec implementation

pub struct Router<'r, Handler, const CAP: usize, const SEG_CAP: usize> {
    len: usize,
    routes: [MaybeUninit<Route<'r, Handler, SEG_CAP>>; CAP],
}

impl<'r, Handler, const CAP: usize, const SEG_CAP: usize> Default for Router<'r, Handler, CAP, SEG_CAP> {
    fn default() -> Self { Self::new() }
}

impl<'r, Handler, const CAP: usize, const SEG_CAP: usize> core::ops::Deref for Router<'r, Handler, CAP, SEG_CAP> {
    type Target = [Route<'r, Handler, SEG_CAP>];
    fn deref(&self) -> &Self::Target { self.as_slice() }
}

impl<Handler: core::fmt::Debug, const CAP: usize, const SEG_CAP: usize> core::fmt::Debug for Router<'_, Handler, CAP, SEG_CAP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <[Route<'_, Handler, SEG_CAP>] as core::fmt::Debug>::fmt(self, f)
    }
}

impl<'r, Handler, const CAP: usize, const SEG_CAP: usize> Router<'r, Handler, CAP, SEG_CAP> {
    const UNINIT: MaybeUninit<Route<'r, Handler, SEG_CAP>> = MaybeUninit::uninit();
    const INIT: [MaybeUninit<Route<'r, Handler, SEG_CAP>>; CAP] = [Self::UNINIT; CAP];

    pub fn new() -> Self {
        Self { len: 0, routes: Self::INIT }
    }

    pub fn add_route(&mut self, route: Route<'r, Handler, SEG_CAP>) -> Result<(), Route<'_, Handler, SEG_CAP>> {
        if self.len == CAP { return Err(route); }
        self.routes[self.len].write(route);
        self.len += 1;
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

    pub const fn len(&self) -> usize { self.len }
    pub const fn is_empty(&self) -> bool { self.len == 0 }

    pub fn as_slice(&self) -> &[Route<'r, Handler, SEG_CAP>] {
        unsafe { core::slice::from_raw_parts(self.routes.as_ptr() as *const Route<'r, Handler, SEG_CAP>, self.len) }
    }
}

pub struct Route<'r, Handler, const CAP: usize> {
    len: usize,
    segments: [MaybeUninit<Segment<'r>>; CAP],
    handler: Handler
}

impl<'r, Handler, const CAP: usize> core::ops::Deref for Route<'r, Handler, CAP> {
    type Target = [Segment<'r>];
    fn deref(&self) -> &Self::Target { self.as_slice() }
}

impl<'r, Handler: core::fmt::Debug, const CAP: usize> core::fmt::Debug for Route<'r, Handler, CAP> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <[Segment<'r>] as core::fmt::Debug>::fmt(self, f)
    }
}

impl<'r, Handler, const CAP: usize> Route<'r, Handler, CAP> {
    pub fn new(route: &'r str, handler: Handler) -> Result<Self, RouteError> {
        let route = route.trim_matches('/');

        let mut segments: [MaybeUninit<Segment<'r>>; CAP] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let mut route_segments = route.split('/');
        let mut seg_cnt = 0;

        if route.is_empty() {
            segments[0].write(Segment::Constant("/"));
            return Ok(Self { len: 1, segments, handler })
        }


        for (idx, seg) in route_segments.clone().enumerate() {
            seg_cnt+=1;
            if idx == CAP { return Err(RouteError::TooManySegments); }

            let peeked = route_segments.nth(idx+1);
            if peeked.is_some() && parse_segment(seg) == Segment::Wildcard {
                return Err(RouteError::WildcardMustBeLast);
            }
            segments[idx].write(parse_segment(seg));
        }

        Ok(Self { len: seg_cnt, segments, handler })
    }
    fn full_match(&self, needle: &str) -> bool {
        // TODO: check if a match matched ALL the needle path segments
        needle
            .trim_matches('/')
            .split('/')
            .zip(&self.segments[..self.len])
            .all(|(needle, seg)| match_segment(needle, unsafe { seg.assume_init_ref() }))
    }

    pub fn handler(&self) -> &Handler { &self.handler }
    pub const fn len(&self) -> usize { self.len }
    pub const fn is_empty(&self) -> bool { self.len == 0 }

    pub fn as_slice(&self) -> &[Segment<'r>] {
        unsafe { core::slice::from_raw_parts(self.segments.as_ptr() as *const Segment<'r>, self.len) }
    }
}

#[derive(Debug)]
pub enum RouteError {
    TooManySegments,
    WildcardMustBeLast,
}

#[derive(Debug, PartialEq)]
pub enum Segment<'s> {
    Constant(&'s str),
    Named(&'s str),
    Wildcard,
    Slash,
}

impl Segment<'_> {
    const fn len(&self) -> usize {
        match self {
            Self::Constant(s) => s.len(),
            Self::Named(s) => s.len()+1,
            Self::Wildcard => 1,
            _ => 0,
        }
    }
}

fn parse_segment(input: &'_ str) -> Segment<'_> {
    // TODO: more complex parsing rules
    // |---- are more complex rules necessary?
    match &input[0..1] {
        ":" => Segment::Named(&input[1..]),
        "*" => Segment::Wildcard,
        _ => Segment::Constant(input)
    }
}
fn match_segment(needle: &str, seg: &Segment<'_>) -> bool {
    let needle_len = needle.len();
    #[cfg(test)] { extern crate std; std::println!("{needle}"); }

    match seg {
        Segment::Constant(s) => s.get(..needle_len).is_some_and(|s| needle.starts_with(s)),
        Segment::Named(_) => !needle.is_empty(),
        Segment::Wildcard => needle.starts_with('*') && !needle[1..].contains('*'),
        _ => false
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;

    #[test]
    fn router_test_route() {
        let mut router = Router::<(), 50, 4>::new();
        router.add_route(Route::new("/", ()).unwrap()).unwrap();
        router.add_route(Route::new("/:greeting", ()).unwrap()).unwrap();
        router.add_route(Route::new("/greetings", ()).unwrap()).unwrap();
        router.add_route(Route::new("/:greeting/hi", ()).unwrap()).unwrap();
        router.add_route(Route::new("/:greeting/hi/bye", ()).unwrap()).unwrap();
        router.add_route(Route::new("/:greeting/*", ()).unwrap()).unwrap();

        // std::println!("{router:#?}");
        // std::println!("{:#?}", router.find("gr"));
        // std::println!("{:#?}", router.find(":greeting"));

        router.filter("/g").for_each(|r| std::println!("{r:?}"));
        router.filter("/hello/hi").for_each(|r| std::println!("{r:?}"));
        panic!();
    }
}
