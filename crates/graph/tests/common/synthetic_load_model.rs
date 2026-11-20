//! Deterministic synthetic include graph + reduced model for the stage-5
//! load passes: the shared fixture of the dense-pass equivalence suite and
//! the scale bench.
//!
//! Shape: `headers` headers in a layered include DAG (each includes its
//! predecessor plus a few earlier headers inside a window), `units` source
//! files including a handful of headers each, and `symbols_per_file` symbols
//! per file. Every symbol uses a few symbols its file can see (type uses and
//! calls into included headers). `out_of_closure_percent` of the header
//! symbols additionally use symbols of an arbitrary header: a function
//! declared in a header and defined in a source file calls whatever that
//! source file sees, so its uses leave the header's own include closure.
//! Those edges are what make real semantic closures deep and cyclic.

use ariadne_graph::{
    IncludeEdge, IncludeGraph, IncludeKind, LanguageId, ReducedSemanticEdge, ReducedSemanticModel,
    ReducedSymbolRecord, SourceLocation,
};
use ariadne_index::types::{DependencyKind, SymbolId, SymbolKind};
use ariadne_repo::FileId;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Size and density knobs of the synthetic repository.
#[derive(Debug, Clone, Copy)]
pub struct SyntheticShape {
    pub headers: usize,
    pub units: usize,
    pub symbols_per_file: usize,
    pub uses_per_symbol: usize,
    pub out_of_closure_percent: u64,
    pub seed: u64,
}

/// Minimal deterministic generator (64-bit LCG, high bits).
struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound.max(1) as u64) as usize
    }
}

const INCLUDE_WINDOW: usize = 200;
const EXTRA_INCLUDES: usize = 3;
const UNIT_INCLUDES: usize = 8;

fn header_file(index: usize) -> FileId {
    FileId(index as u64 + 1)
}

fn unit_file(shape: &SyntheticShape, index: usize) -> FileId {
    FileId((shape.headers + index) as u64 + 1)
}

fn symbol_name(file: FileId, slot: usize) -> String {
    format!("ns{}::Type{}::member{}", file.0 % 97, file.0, slot)
}

fn include_edge(source: FileId, destination: FileId) -> IncludeEdge {
    IncludeEdge {
        source,
        destination: Some(destination),
        context_name: String::new(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: format!("h{}.h", destination.0),
        resolved_path: None,
        language: LanguageId::cpp(),
    }
}

/// Direct includes per file index (headers first, then units).
fn direct_includes(shape: &SyntheticShape, rng: &mut Lcg) -> Vec<Vec<FileId>> {
    let mut includes: Vec<Vec<FileId>> = Vec::with_capacity(shape.headers + shape.units);
    for index in 0..shape.headers {
        let mut targets = Vec::new();
        if index > 0 {
            targets.push(header_file(index - 1));
            let window_start = index.saturating_sub(INCLUDE_WINDOW);
            for _ in 0..EXTRA_INCLUDES {
                targets.push(header_file(window_start + rng.below(index - window_start)));
            }
        }
        targets.sort();
        targets.dedup();
        includes.push(targets);
    }
    for _ in 0..shape.units {
        let mut targets: Vec<FileId> =
            (0..UNIT_INCLUDES).map(|_| header_file(rng.below(shape.headers))).collect();
        targets.sort();
        targets.dedup();
        includes.push(targets);
    }
    includes
}

fn add_symbols(model: &mut ReducedSemanticModel, file: FileId, count: usize) {
    let mut ids = Vec::with_capacity(count);
    for slot in 0..count {
        let id = SymbolId::new(symbol_name(file, slot));
        let kind = if slot % 3 == 0 { SymbolKind::Class } else { SymbolKind::Function };
        model.symbols.insert(
            id.clone(),
            ReducedSymbolRecord {
                id: id.clone(),
                name: format!("member{slot}_{}", file.0),
                kind,
                defining_file: Some(file),
                declarations: vec![],
                definition: None,
                declaring_files: vec![],
            },
        );
        ids.push(id);
    }
    model.file_symbols.insert(file, ids);
}

fn add_uses(
    model: &mut ReducedSemanticModel,
    shape: &SyntheticShape,
    file: FileId,
    includes: &[FileId],
    is_header: bool,
    rng: &mut Lcg,
) {
    for slot in 0..shape.symbols_per_file {
        let source = SymbolId::new(symbol_name(file, slot));
        let mut targets: Vec<FileId> = Vec::new();
        if !includes.is_empty() {
            for _ in 0..shape.uses_per_symbol {
                targets.push(includes[rng.below(includes.len())]);
            }
        }
        if is_header && rng.next() % 100 < shape.out_of_closure_percent {
            targets.push(header_file(rng.below(shape.headers)));
        }
        for target_file in targets {
            let target = SymbolId::new(symbol_name(target_file, rng.below(shape.symbols_per_file)));
            model.edges.push(ReducedSemanticEdge {
                src_symbol: source.clone(),
                dst_symbol: target,
                kind: DependencyKind::Incomplete,
            });
        }
    }
}

/// Build the synthetic include graph and reduced model for `shape`.
pub fn build(shape: &SyntheticShape) -> (IncludeGraph, ReducedSemanticModel) {
    let mut rng = Lcg(shape.seed);
    let mut graph = IncludeGraph::new();
    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),
        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };
    let includes = direct_includes(shape, &mut rng);
    for (index, targets) in includes.iter().enumerate() {
        let is_header = index < shape.headers;
        let file = if is_header { header_file(index) } else { unit_file(shape, index - shape.headers) };
        let path = if is_header {
            format!("include/mod{}/h{}.h", index / 50, file.0)
        } else {
            format!("src/mod{}/unit{}.cpp", index / 50, file.0)
        };
        graph.register_file(file, PathBuf::from(path));
        for target in targets {
            graph.add_edge(include_edge(file, *target));
        }
        add_symbols(&mut model, file, shape.symbols_per_file);
    }
    for (index, targets) in includes.iter().enumerate() {
        let is_header = index < shape.headers;
        let file = if is_header { header_file(index) } else { unit_file(shape, index - shape.headers) };
        add_uses(&mut model, shape, file, targets, is_header, &mut rng);
    }
    (graph, model)
}
