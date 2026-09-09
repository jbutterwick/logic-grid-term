//! Constraint-propagation solver with backtracking to prove uniqueness (A2).

use crate::{Cat, Category, Clue, Entity, Item, Solution};

/// Outcome of [`solve`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SolveResult {
    /// Exactly one solution.
    Unique(Solution),
    /// Two or more solutions.
    Multiple,
    /// No solution.
    None,
}

/// Solve a puzzle from its categories and clues. Item order is the ordinal order in ordered
/// categories; `ordered` is accepted for API symmetry but the clues already name their category.
pub fn solve(categories: &[Category], ordered: &[bool], clues: &[Clue]) -> SolveResult {
    let _ = ordered;
    let n = categories.len();
    let m = categories.first().map_or(0, |c| c.items.len());
    if n < 2 || m < 1 || categories.iter().any(|c| c.items.len() != m) {
        return SolveResult::None;
    }
    if clues.iter().any(|c| !c.in_range(n, m)) {
        return SolveResult::None;
    }
    let Some(mut st) = State::new(n, m, clues) else {
        return SolveResult::None;
    };
    let mut found = None;
    match st.count(2, &mut found) {
        0 => SolveResult::None,
        1 => {
            let sol = found.expect("one solution recorded");
            debug_assert!(clues.iter().all(|c| c.holds(&sol)));
            SolveResult::Unique(sol)
        }
        _ => SolveResult::Multiple,
    }
}

/// Possibility matrix over all entity pairs: `p[a][b]` is true if `a` may pair with `b`.
#[derive(Clone)]
struct State<'a> {
    n: usize,
    m: usize,
    p: Vec<bool>,
    clues: &'a [Clue],
}

/// Contradiction marker.
struct Dead;

impl<'a> State<'a> {
    fn new(n: usize, m: usize, clues: &'a [Clue]) -> Option<Self> {
        let mut st = State {
            n,
            m,
            p: vec![true; (n * m) * (n * m)],
            clues,
        };
        for c in clues {
            let r = match *c {
                Clue::Is(a, b) => st.assign(a, b),
                Clue::IsNot(a, b) | Clue::NotSame(a, b) => st.eliminate(a, b).map(|_| ()),
                _ => Ok(()),
            };
            r.ok()?;
        }
        Some(st)
    }

    fn e(cat: Cat, item: Item) -> Entity {
        Entity { cat, item }
    }

    fn idx(&self, a: Entity, b: Entity) -> usize {
        (a.cat * self.m + a.item) * self.n * self.m + b.cat * self.m + b.item
    }

    fn get(&self, a: Entity, b: Entity) -> bool {
        if a.cat == b.cat {
            return a.item == b.item;
        }
        self.p[self.idx(a, b)]
    }

    /// Mark `a`/`b` impossible. `Ok(true)` if that changed anything.
    fn eliminate(&mut self, a: Entity, b: Entity) -> Result<bool, Dead> {
        if a.cat == b.cat {
            return if a.item == b.item {
                Err(Dead)
            } else {
                Ok(false)
            };
        }
        let (i, j) = (self.idx(a, b), self.idx(b, a));
        if !self.p[i] {
            return Ok(false);
        }
        self.p[i] = false;
        self.p[j] = false;
        Ok(true)
    }

    /// Force `a` to pair with `b` by eliminating every other item of `b.cat` for `a`.
    fn assign(&mut self, a: Entity, b: Entity) -> Result<(), Dead> {
        if !self.get(a, b) {
            return Err(Dead);
        }
        for j in 0..self.m {
            if j != b.item {
                self.eliminate(a, Self::e(b.cat, j))?;
            }
        }
        Ok(())
    }

    /// Items of `cat` still possible for `a`.
    fn options(&self, a: Entity, cat: Cat) -> Vec<Item> {
        (0..self.m)
            .filter(|&j| self.get(a, Self::e(cat, j)))
            .collect()
    }

    fn determined(&self, a: Entity, b: Entity) -> bool {
        self.get(a, b) && self.options(a, b.cat).len() == 1
    }

    /// Remove ordinal position `k` for `a` in category `ord`.
    fn eliminate_pos(&mut self, a: Entity, ord: Cat, k: Item) -> Result<bool, Dead> {
        self.eliminate(a, Self::e(ord, k))
    }

    /// One propagation pass. `Ok(true)` if anything changed.
    fn step(&mut self) -> Result<bool, Dead> {
        let mut changed = false;
        let (n, m) = (self.n, self.m);
        // Row rule: a row with one option pins it; zero options is a contradiction.
        for ca in 0..n {
            for cb in (0..n).filter(|&cb| cb != ca) {
                for i in 0..m {
                    let a = Self::e(ca, i);
                    match self.options(a, cb).as_slice() {
                        [] => return Err(Dead),
                        [j] => {
                            let b = Self::e(cb, *j);
                            for i2 in (0..m).filter(|&i2| i2 != i) {
                                changed |= self.eliminate(Self::e(ca, i2), b)?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // Path consistency (transitivity): a~c needs some b with a~b and b~c.
        for ca in 0..n {
            for cc in ca + 1..n {
                for cb in (0..n).filter(|&cb| cb != ca && cb != cc) {
                    for i in 0..m {
                        for k in 0..m {
                            let (a, c) = (Self::e(ca, i), Self::e(cc, k));
                            if self.get(a, c)
                                && !(0..m).any(|j| {
                                    let b = Self::e(cb, j);
                                    self.get(a, b) && self.get(b, c)
                                })
                            {
                                changed |= self.eliminate(a, c)?;
                            }
                        }
                    }
                }
            }
        }
        for c in self.clues {
            changed |= self.clue_step(c)?;
        }
        Ok(changed)
    }

    fn clue_step(&mut self, c: &Clue) -> Result<bool, Dead> {
        let mut ch = false;
        match *c {
            Clue::Is(..) | Clue::IsNot(..) | Clue::NotSame(..) => {}
            Clue::Either(a, b, c) => {
                if b.cat == c.cat {
                    for j in (0..self.m).filter(|&j| j != b.item && j != c.item) {
                        ch |= self.eliminate(a, Self::e(b.cat, j))?;
                    }
                } else {
                    ch |= self.eliminate(b, c)?;
                }
                match (self.get(a, b), self.get(a, c)) {
                    (false, false) => return Err(Dead),
                    (false, true) => {
                        ch |= !self.determined(a, c) && self.assign(a, c).map(|_| true)?
                    }
                    (true, false) => {
                        ch |= !self.determined(a, b) && self.assign(a, b).map(|_| true)?
                    }
                    (true, true) => {
                        if self.determined(a, b) {
                            ch |= self.eliminate(a, c)?;
                        } else if self.determined(a, c) {
                            ch |= self.eliminate(a, b)?;
                        }
                    }
                }
            }
            Clue::Before(a, b, o) | Clue::After(b, a, o) => {
                ch |= self.eliminate(a, b)?;
                let (la, lb) = (self.options(a, o), self.options(b, o));
                let (Some(&max_b), Some(&min_a)) = (lb.last(), la.first()) else {
                    return Err(Dead);
                };
                for k in la.into_iter().filter(|&k| k >= max_b) {
                    ch |= self.eliminate_pos(a, o, k)?;
                }
                for k in lb.into_iter().filter(|&k| k <= min_a) {
                    ch |= self.eliminate_pos(b, o, k)?;
                }
            }
            Clue::Adjacent(a, b, o) => {
                ch |= self.eliminate(a, b)?;
                let (la, lb) = (self.options(a, o), self.options(b, o));
                let near = |l: &[Item], k: Item| l.iter().any(|&x| x.abs_diff(k) == 1);
                for &k in la.iter().filter(|&&k| !near(&lb, k)) {
                    ch |= self.eliminate_pos(a, o, k)?;
                }
                for &k in lb.iter().filter(|&&k| !near(&la, k)) {
                    ch |= self.eliminate_pos(b, o, k)?;
                }
            }
        }
        Ok(ch)
    }

    fn propagate(&mut self) -> Result<(), Dead> {
        while self.step()? {}
        Ok(())
    }

    /// Anchor row with the fewest (>1) options, if any remain undetermined.
    fn pick(&self) -> Option<(Entity, Cat, Vec<Item>)> {
        (0..self.m)
            .flat_map(|i| (1..self.n).map(move |c| (Self::e(0, i), c)))
            .map(|(a, c)| (a, c, self.options(a, c)))
            .filter(|(_, _, o)| o.len() > 1)
            .min_by_key(|(_, _, o)| o.len())
    }

    fn extract(&self) -> Solution {
        Solution(
            (0..self.n)
                .map(|c| {
                    (0..self.m)
                        .map(|i| self.options(Self::e(0, i), c)[0])
                        .collect()
                })
                .collect(),
        )
    }

    /// Count solutions up to `limit`, recording the first one found.
    fn count(&mut self, limit: usize, found: &mut Option<Solution>) -> usize {
        if self.propagate().is_err() {
            return 0;
        }
        let Some((a, c, opts)) = self.pick() else {
            found.get_or_insert_with(|| self.extract());
            return 1;
        };
        let mut total = 0;
        for j in opts {
            let mut s = self.clone();
            if s.assign(a, Self::e(c, j)).is_ok() {
                total += s.count(limit - total, found);
            }
            if total >= limit {
                break;
            }
        }
        total
    }
}
