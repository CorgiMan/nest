use rand::{random, Rng};
use std::collections::HashSet;

use rayon::prelude::*;

use crate::job;
use crate::nesting_runner::NestPart;
use crate::packing::{PackingResult, PlacementSequence};

const NO_PROGRESS_LIMIT: usize = 6;
const POPULATION_SIZE: usize = 2;
const MUTATION_RATE: f64 = 0.1;

#[derive(Clone)]
pub struct Population {
    parts: Vec<NestPart>,
    pub individuals: Vec<PlacementSequence>,
    pub generation: usize,
    pub last_improvement: usize,
    pub last_fitness: f64,
}

impl Population {
    pub fn new(parts: Vec<NestPart>) -> Self {
        assert!(parts.len() >= 2);

        use crate::p;
        p!(parts.len());

        let parts_copy = parts.to_owned();
        let enumerated_parts_sorted = parts_copy.iter().enumerate().collect::<Vec<_>>();
        // todo: sort by area instead: sorted_parts_with_index.sort_by(|x, y| x.1.area.cmp(&y.1.area));

        let mut individuals = Vec::<PlacementSequence>::new();

        // create initial population. we rotate the first part by all possible angles, then the
        // second part, etc. until we have POPULATION_SIZE individuals. We use some math to easily
        // iterate over the required permutations. The variable j encodes the rotation of each part.
        // To get the rotation index of the first part we take j % part.rotations.len(), we then set
        // j /= part.rotations.len() and repeat this process to get the rotations of the remaining
        // parts. If for example each part has 2 rotations, j's binary encoding specifies which
        // parts are rotated: a one in digit k would mean the kth part is rotated.
        for i in 0..POPULATION_SIZE {
            let mut j = i;
            let mut placements = Vec::<job::Placement>::new();

            for (part_index, part) in &enumerated_parts_sorted {
                for nth_part in 0..part.quantity {
                    let angle = part.rotations[j % part.rotations.len()];
                    j /= part.rotations.len();

                    placements.push(job::Placement {
                        part_index: *part_index,
                        nth_part,
                        angle,
                    })
                }
            }

            individuals.push(PlacementSequence { placements })
        }

        Population {
            parts,
            individuals,
            generation: 0,
            last_improvement: 0,
            last_fitness: 0.,
        }
    }
}

impl Iterator for Population {
    type Item = job::GenerationResult; // todo job::GenerationResult

    fn next(&mut self) -> Option<Self::Item> {
        // Calculate next population in parallel

        let packing_results = self.par_packing_results();
        self.individuals = self.next_population(&packing_results);
        let (fittest_result, fittest_sequence) = &packing_results[0];

        // Track improvements and stop if no improvement for a few generations

        self.generation += 1;

        if self.last_fitness < fittest_result.fitness {
            self.last_fitness = fittest_result.fitness;
            self.last_improvement = 0;
        } else {
            self.last_improvement += 1;
        }

        if self.last_improvement >= NO_PROGRESS_LIMIT {
            return None;
        }

        // todo: stop if queue is too big, (maybe sent signal from manager)

        Some(job::GenerationResult {
            last_sheet_left_over: 0, // todo
            sheet_count: 1,          // todo
            cut_loss_ratio: 0.7,     // todo
            placements_and_location: fittest_sequence
                .placements
                .iter()
                .copied()
                .zip(fittest_result.placed_at.iter().copied())
                .collect::<Vec<_>>(),
        })
    }
}

/// Some helper functions to determine next generation.
impl Population {
    fn par_packing_results(&self) -> Vec<(PackingResult, PlacementSequence)> {
        let mut packing_results = self
            .individuals
            .par_iter()
            .map(|individual| individual.pack())
            .zip(self.individuals.to_owned())
            .collect::<Vec<_>>();

        packing_results
            .sort_unstable_by(|(r1, _), (r2, _)| r1.fitness.partial_cmp(&r2.fitness).unwrap());

        packing_results
    }

    fn next_population(
        &self,
        packing_results: &Vec<(PackingResult, PlacementSequence)>,
    ) -> Vec<PlacementSequence> {
        let mut next_population = Vec::<PlacementSequence>::new();

        loop {
            let male_ix = random_weighted_index(packing_results.len());
            let mut female_ix = random_weighted_index(packing_results.len() - 1);
            if female_ix == male_ix {
                female_ix += 1;
            }

            let male = &packing_results[male_ix].1;
            let female = &packing_results[female_ix].1;

            let (child1, child2) = &self.mate(male, female);

            next_population.push(self.mutate(child1));
            if next_population.len() == self.individuals.len() {
                break;
            }

            next_population.push(self.mutate(child2));
            if next_population.len() == self.individuals.len() {
                break;
            }
        }

        next_population
    }

    fn mate(
        &self,
        male: &PlacementSequence,
        female: &PlacementSequence,
    ) -> (PlacementSequence, PlacementSequence) {
        assert_eq!(male.placements.len(), female.placements.len());

        // calc crossover point
        let ignore_count = male.placements.len() / 10;
        let start_index = ignore_count;
        let end_index = male.placements.len() - ignore_count;
        let cross_ix = rand::thread_rng().gen_range(start_index..=end_index);

        let child1 = {
            let cut_gene = &male.placements[..cross_ix];

            // collect all male genes
            let set: HashSet<(usize, u32)> = cut_gene
                .iter()
                .map(|p| (p.part_index, p.nth_part))
                .collect();

            // skip all male genes
            let complement = female
                .placements
                .iter()
                .filter(|p| !set.contains(&(p.part_index, p.nth_part)));

            PlacementSequence {
                placements: cut_gene
                    .iter()
                    .chain(complement)
                    .map(|p| p.to_owned())
                    .collect(),
            }
        };

        assert_eq!(male.placements.len(), child1.placements.len());

        let child2 = {
            let cut_gene = &female.placements[..cross_ix];

            // collect all male genes
            let set: HashSet<(usize, u32)> = cut_gene
                .iter()
                .map(|p| (p.part_index, p.nth_part))
                .collect();

            // skip all male genes
            let complement = female
                .placements
                .iter()
                .filter(|p| !set.contains(&(p.part_index, p.nth_part)));

            PlacementSequence {
                placements: cut_gene
                    .iter()
                    .chain(complement)
                    .map(|p| p.to_owned())
                    .collect(),
            }
        };

        assert_eq!(male.placements.len(), child2.placements.len());

        (child1, child2)
    }

    fn mutate(&self, individual: &PlacementSequence) -> PlacementSequence {
        let mut placements = individual.placements.to_owned();
        let len = placements.len();

        // swap once in a while
        // todo: tune with swapping random parts, not just with i+1
        for i in 0..len - 1 {
            if random::<f64>() > MUTATION_RATE {
                continue;
            }
            placements.swap(i, i + 1)
        }

        // pick a different rotation once in a while
        for placement in placements.iter_mut() {
            if random::<f64>() > MUTATION_RATE {
                continue;
            }

            let rotations = &self.parts[placement.part_index].rotations;
            let mut index = rotations
                .iter()
                .position(|&r| r == placement.angle)
                .unwrap();

            index += rand::thread_rng().gen_range(0..rotations.len() - 1);
            index %= rotations.len();
            placement.angle = rotations[index];
        }

        PlacementSequence { placements }
    }
}

fn random_weighted_index(len: usize) -> usize {
    let n = len as f64;
    let y = rand::random::<f64>() * n;

    // I don't really know why this works. It could be simplified but I'm
    // afraid it might detune the algorithm.
    let x = -0.5 * (4.0 * n * n - 4.0 * n * (y - 2.0) + 1.0).sqrt() + n + 1.5;

    x.floor() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genetic_algorithm::Population;
    use crate::nest_polygon::NestPolygon;

    use geo::Coord;

    #[test]
    fn mutates() {
        let parts = vec![NestPart {
            quantity: 4,
            polygon: NestPolygon::new(vec![
                Coord { x: 70.0, y: 10.0 },
                Coord { x: 80.0, y: 20.0 },
                Coord { x: 90.0, y: 40.0 },
            ]),
            rotations: vec![0, 90, 180, 270],
        }];

        let population = Population::new(parts);
        let x = &population.individuals[0];
        let _y = &population.individuals[0];
        let _m = population.mutate(x);
    }
}
