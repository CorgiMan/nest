// types for jobs and input data such as parts, sheets, etc.


pub use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Status {
    Pending,
    Running,
    Done,
    Failed,
}

#[derive(Serialize, Deserialize)]
pub enum ErrorType {
    Timeout,
    InvalidInput,
    PartDoesNotFit,
    Cancelled,
    TooBusy,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub nesting_job_ulid: String,
    pub parts: Vec<Part>,
    pub sheets: Vec<Sheet>,
    pub tool_diameter: f64,
    pub timeout: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Part {
    pub quantity: i32,
    pub contour: Vec<geo::Coord>,
    pub rotations: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sheet {
    pub length: f32,
    pub width: f32,
    pub cost: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Update {
    pub status: Status,
    pub nesting_solution: Option<GenerationResult>,
    pub error: Option<Error>,
}

#[derive(Serialize, Deserialize)]
pub struct Error {
    pub error_type: ErrorType,
    pub message: String,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct GenerationResult {
    pub sheet_count: i32,
    pub last_sheet_left_over: i32,
    pub cut_loss_ratio: f32,
    pub placements_and_location: Vec<(Placement, geo::Coord)>,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Placement {
    pub part_index: usize,
    pub nth_part: u32,
    pub angle: i32,
}
