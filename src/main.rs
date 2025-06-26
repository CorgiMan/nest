// external functions that allow to add_job and init the worker thread
// the jobs are picked up from the job queue one by one in a separate thread

// NestingManager
//  |- Part
//  |   - NestPolygon
//  |      - geo::Polygon
//  |- Population
//  |   - PlacementSequence
//  |       - Placement (refs Part)
//  |- NFPCache (refs Part)
mod genetic_algorithm;
mod job;
mod nest_polygon;
mod nesting_runner;
mod packing;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

// print some variables with p!(a, b, c)
macro_rules! p(
    ($($a:expr),*) => {
        println!(concat!($(stringify!($a), " = {:#?}, "),*), $($a),*);
    }
);
pub(crate) use p;

use crate::nesting_runner::NestingRunner;

type UpdateCallback = extern "C" fn(update: *const c_char);

static mut JOB_CHANNEL: Option<Sender<(job::Input, UpdateCallback)>> = None;
static mut JOB_RECEIVER: Option<Receiver<(job::Input, UpdateCallback)>> = None;

fn main() {}

#[no_mangle]
pub extern "C" fn init() {
    // Create the channel for jobs
    let (tx, rx) = channel();
    unsafe {
        JOB_CHANNEL = Some(tx);
        JOB_RECEIVER = Some(rx);
    }

    // Spawn a worker thread to process jobs
    thread::spawn(|| {
        loop {
            // Wait for a job to be received
            let (job, update_callback) = unsafe {
                match &JOB_RECEIVER {
                    Some(receiver) => receiver.recv().unwrap(),
                    None => break,
                }
            };

            // todo: update with pending if queued

            // Create a NestingRunner for the job and start it
            let runner = NestingRunner::new(
                job,
                Box::new(move |update| {
                    let json_string = serde_json::to_string(&update).unwrap();
                    let cstring = CString::new(json_string).unwrap().into_raw();
                    update_callback(cstring);
                }),
            );
            runner.start();
        }
    });
}

/// # Safety
///
/// input has to point to a valid C string. Its JSON contents will be deserialized into a job::Input
#[no_mangle]
pub unsafe extern "C" fn add_job(input: *const c_char, update_callback: UpdateCallback) {
    // Deserialize the input
    let c_str = unsafe { CStr::from_ptr(input) };
    let bytes = c_str.to_bytes();
    let byte_slice = unsafe { std::slice::from_raw_parts(bytes.as_ptr(), bytes.len()) };
    let job: job::Input = serde_json::from_slice(byte_slice).unwrap();

    // Send the job to the worker thread
    if let Some(channel) = unsafe { &JOB_CHANNEL } {
        channel.send((job, update_callback)).unwrap();
    }
}
