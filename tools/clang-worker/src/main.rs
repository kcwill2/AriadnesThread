use serde::{Deserialize, Serialize};

/// The worker boundary protocol expects the sidecar to read JSON requests 
/// from stdin and write JSON responses to stdout.
///
/// Request Format:
/// ```json
/// {
///   "action": "analyze",
///   "source_file": "path/to/file.cpp",
///   "context": ["-Iinclude", "-DDEBUG"]
/// }
/// ```
///
/// Response Format:
/// ```json
/// {
///   "status": "success",
///   "analysis": {
///       // TuAnalysis JSON structure matching ariadne-index
///   }
/// }
/// ```

#[allow(dead_code)]
#[derive(Deserialize)]
struct WorkerRequest {
    action: String,
    source_file: String,
    context: Vec<String>,
}

#[allow(dead_code)]
#[derive(Serialize)]
struct WorkerResponse {
    status: String,
    // Add analysis fields later
}

fn main() {
    println!("clang-worker 0.1.0");
    // Placeholder: In real implementation, read from stdin line-by-line 
    // and process requests using libclang.
}
