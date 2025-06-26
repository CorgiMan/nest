// todo: on new assert quantity > 0
// #[derive(Clone, Debug)]
// pub struct Placement {
//     pub part_index: usize,
//     pub nth_part: u32,
//     pub angle: f64,
// }

use crate::job::Placement;

#[derive(Clone)]
pub struct PlacementSequence {
    pub placements: Vec<Placement>,
}

pub struct PackingResult {
    pub fitness: f64,
    pub placed_at: Vec<geo::Coord>,
}

impl PlacementSequence {
    pub fn pack(&self) -> PackingResult {
        PackingResult {
            fitness: 123.,
            placed_at: vec![
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 0., y: 0. },
                geo::Coord { x: 0., y: 0. },
            ],
        }
    }
}
