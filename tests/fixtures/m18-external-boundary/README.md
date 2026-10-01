# m18-external-boundary fixture

Locks the M18.5 divergence-(i) reconciliation end-to-end: ThirdParty and
search-path headers must stay in the plugin TU closure exactly as the
pre-M18 legacy resolver visited them (`ResolutionContext.search_paths` +
on-disk fallback in `scheduler::tu_store` and `graph::builder`).
Consumed by `tests/integration/tests/m18_cpp_plugin_parity/external_boundary_parity.rs`;
no golden-baseline or other suite reads this directory.

## Files

- `compile_commands.json` — one unit for `src/main.cpp`; supplies `-I third_party` and `-I ext_sdk` so angled includes resolve only through search paths.
- `src/main.cpp` — the TU: one quoted include, two angled includes reachable only via the `-I` dirs, one angled include that exists nowhere (unresolved-edge lock).
- `src/core.h` — quoted source-directory hit, with its own nested quoted include (transitive-lock carrier).
- `src/core_defs.h` — transitive quoted leaf reached through `core.h`.
- `third_party/external_lib/api.h` — ThirdParty-owned header reached via angled `<external_lib/api.h>`; legacy classified the edge External yet still visited it (divergence-(i) subject).
- `third_party/external_lib/api_impl.h` — quoted child of `api.h`; locks that scanning continues THROUGH External-classified headers.
- `ext_sdk/sdk_bridge.h` — Repository-owned header reached via angled `<sdk_bridge.h>` through `-I ext_sdk` (search-path feeding without ThirdParty ownership).
- `ext_sdk/bridge_defs.h` — quoted child of `sdk_bridge.h`, transitive leaf of the ext_sdk chain.
