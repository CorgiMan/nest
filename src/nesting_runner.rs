use crate::genetic_algorithm::Population;
use crate::job;
use crate::nest_polygon::NestPolygon;

#[derive(Clone, Debug)]
pub struct NestPart {
    pub quantity: u32,
    pub polygon: NestPolygon,
    pub rotations: Vec<i32>,
}

pub struct NestingRunner {
    job: job::Input,
    update_callback: Box<dyn Fn(job::Update)>,
    // nfp_cache: NFPCache,
    population: Box<Population>,
    best_solution: Option<job::GenerationResult>,
}

impl NestingRunner {
    pub fn new(job: job::Input, update_callback: Box<dyn Fn(job::Update)>) -> NestingRunner {
        let parts = job
            .parts
            .iter()
            .map(|part| NestPart {
                quantity: part.quantity as u32,
                polygon: NestPolygon::new(part.contour.to_owned()),
                rotations: part.rotations.to_owned(),
            })
            .collect();

        let population = Box::new(Population::new(parts));

        // todo:
        // - setup bin
        // - apply tool_diameter / 2 offsets to parts and other calculations
        // - remove rotations for which parts don't fit
        // - respond with Err if a part doesn't fit
        // - setup NFPCache

        NestingRunner {
            job,
            population,
            best_solution: None,
            update_callback,
        }
    }

    pub fn start(&self) {
        // iterate over generations
        for results in self.population.clone() {
            let update = job::Update {
                status: job::Status::Running,
                nesting_solution: Some(results),
                error: None,
            };
            (self.update_callback)(update);
        }
        (self.update_callback)(job::Update {
            status: job::Status::Done,
            nesting_solution: self.best_solution.to_owned(),
            error: None,
        })
    }
}
