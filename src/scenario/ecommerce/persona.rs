use crate::engine::{calendar::Season, stream::Stream};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Persona {
    Courier,
    Artificer,
    FeastReveler,
    Apprentice,
    Wanderer,
    Herbalist,
}

impl Persona {
    pub(super) fn block(rng: &mut Stream) -> [Self; 20] {
        let mut block = [
            Self::Courier,
            Self::Courier,
            Self::Courier,
            Self::Courier,
            Self::Courier,
            Self::Artificer,
            Self::Artificer,
            Self::Artificer,
            Self::Artificer,
            Self::Artificer,
            Self::FeastReveler,
            Self::FeastReveler,
            Self::Apprentice,
            Self::Apprentice,
            Self::Apprentice,
            Self::Apprentice,
            Self::Wanderer,
            Self::Wanderer,
            Self::Herbalist,
            Self::Herbalist,
        ];
        for index in (1..block.len()).rev() {
            let other = rng.index(index + 1);
            block.swap(index, other);
        }
        block
    }

    pub(super) fn probability(self, weekend: bool, season: Season, favorite: u32) -> f64 {
        let fraction = f64::from(favorite) / 100.0;
        match self {
            Self::Courier => {
                if weekend {
                    0.001
                } else {
                    0.5 + fraction * 0.3
                }
            }
            Self::Artificer => {
                if weekend {
                    0.001
                } else {
                    fraction * 0.4
                }
            }
            Self::FeastReveler => {
                if weekend {
                    0.2 + fraction * 0.2
                } else {
                    0.0
                }
            }
            Self::Apprentice => {
                if season == Season::Summer {
                    0.0
                } else {
                    0.1 + fraction * 0.4
                }
            }
            Self::Wanderer => 0.1,
            Self::Herbalist => {
                if season == Season::Summer {
                    0.1 + fraction * 0.4
                } else {
                    0.2
                }
            }
        }
    }

    pub(super) fn sparrow_probability(self) -> f64 {
        match self {
            Self::Courier => 0.2,
            Self::Artificer => 0.01,
            Self::FeastReveler | Self::Apprentice => 0.8,
            Self::Wanderer => 0.1,
            Self::Herbalist => 0.6,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn minute(self, rng: &mut Stream, favorite: u32) -> i32 {
        let (mean, deviation) = match self {
            Self::Courier => (450.0, 30.0),
            Self::Artificer => (420.0, 180.0),
            Self::FeastReveler => (300.0 + (f64::from(favorite) - 50.0) / 50.0 * 120.0, 120.0),
            Self::Apprentice => (540.0, 120.0),
            Self::Wanderer | Self::Herbalist => (300.0, 120.0),
        };
        rng.normal(mean, deviation).round().max(0.0) as i32
    }

    pub(super) fn items(self, rng: &mut Stream, favorite: u32) -> Vec<usize> {
        let mut items = Vec::new();
        match self {
            Self::Courier | Self::Herbalist => add(&mut items, rng, 10, 1),
            Self::Artificer => {
                add(&mut items, rng, 10, 1);
                if rng.uniform() < 0.3 {
                    add(&mut items, rng, 10, 1);
                }
                if rng.uniform() < 0.3 {
                    let kind = solid(rng);
                    add(&mut items, rng, kind, 1);
                }
            }
            Self::FeastReveler => {
                let count = 1 + favorite as usize / 20;
                let kind = solid(rng);
                add(&mut items, rng, kind, count);
                add(&mut items, rng, 10, count);
            }
            Self::Apprentice => {
                add(&mut items, rng, 10, 1);
                if rng.uniform() < 0.5 {
                    let kind = solid(rng);
                    add(&mut items, rng, kind, 1);
                }
            }
            Self::Wanderer => {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let elixirs = (rng.uniform() * 10.0 / 3.0) as usize;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let solids = (rng.uniform() * 10.0 / 3.0) as usize;
                add(&mut items, rng, 10, elixirs);
                if solids > 0 {
                    let kind = solid(rng);
                    add(&mut items, rng, kind, solids);
                }
            }
        }
        items
    }
}

fn solid(rng: &mut Stream) -> usize {
    if rng.uniform() < 0.5 { 5 } else { 0 }
}
fn add(items: &mut Vec<usize>, rng: &mut Stream, kind: usize, count: usize) {
    items.extend((0..count).map(|_| kind + rng.index(5)));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[allow(clippy::float_cmp)]
    fn seasonal_and_weekend_dispositions_match_personas() {
        assert_eq!(
            Persona::Apprentice.probability(false, Season::Summer, 70),
            0.0
        );
        assert_eq!(
            Persona::FeastReveler.probability(false, Season::Winter, 70),
            0.0
        );
        assert!(
            Persona::Herbalist.probability(true, Season::Summer, 100)
                > Persona::Herbalist.probability(true, Season::Winter, 100)
        );
        assert_eq!(
            Persona::Courier.probability(true, Season::Winter, 100),
            0.001
        );
        let mut stream = Stream::derive(42, "persona-test", &[]);
        for _ in 0..100 {
            let items = Persona::Courier.items(&mut stream, 50);
            assert_eq!(items.len(), 1);
            assert!(items[0] >= 10);
        }
    }
}
