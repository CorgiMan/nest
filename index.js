const ffi = require('ffi-napi');
const ref = require('ref-napi');

// Load the Rust library

const mylib = ffi.Library('./target/debug/libmylib.dylib', {
    'init': ['void', []],
    add_job: ['void', ['string', 'pointer']]
});

mylib.init();

function updateCallback({ status, nesting_solution, error }) {
    console.log('node update', status, nesting_solution, error);
    console.log(nesting_solution?.placements_and_location)
}

const updateCallbackPtr = ffi.Callback('void',
    ['string'],
    (jobUpdateJSON) => {
        const jobUpdate = JSON.parse(jobUpdateJSON);
        updateCallback(jobUpdate);
    });

// Make an extra reference to the callback pointer to avoid GC, a common practice in
// ffi-napi: https://www.sobyte.net/post/2022-02/communicate-with-cpp-code-in-node/
process.on('exit', function () {
    updateCallbackPtr
})

setInterval(function () {
    console.log("timer that keeps nodejs processing running");
}, 1000 * 60 * 60);

// Call the Rust function

const input = JSON.stringify({
    nesting_job_ulid: '01EYQZJZJZJZJZJZJZJZJZJZJZ',
    tool_diameter: 19,
    timeout: 60 * 60 * 1000, // 1 hour
    parts: [
        {
            quantity: 5,
            contour: [{ x: 0, y: 0 }, { x: 1, y: 1 }, { x: 1, y: 1 }],
            rotations: [0, 180]
        },
        {
            quantity: 5,
            contour: [{ x: 0, y: 0 }, { x: 1, y: 1 }, { x: 1, y: 1 }],
            rotations: [0, 180]
        },
    ],
    sheets: [
        { length: 10.0, width: 20.0, cost: 5.0 },
        { length: 15.0, width: 25.0, cost: 8.0 },
        { length: 30.0, width: 40.0, cost: 12.0 }
    ],
});

console.log('input', input)
mylib.add_job(input, updateCallbackPtr);
