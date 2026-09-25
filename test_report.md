# Cargo Test Report

- ? discovery::tests::test_infer_context_name: **ok**
- ? discovery::tests::test_parse_args_msvc: **ok**
- ? discovery::tests::test_parse_args_gcc_clang: **ok**
- ? discovery::tests::test_parse_args_quote_handling_in_command: **ok**
- ? types::tests::test_compilation_context_normalization: **ok**
- ? measure::tests::test_format_report: **ok**
- ? types::tests::test_build_discovery_result_determinism: **ok**
- ? discovery::tests::test_build_discoverer_infer_default: **ok**
- ? discovery::tests::test_build_discoverer_compile_commands_with_args: **ok**
- ? measure::tests::test_build_measurer: **ok**

**Summary**: test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- ? commands::progress::tests::test_format_progress: **ok**
- ? commands::ui::tests::test_progress_calls_present: **ok**
- ? optimize::tests::test_format_m14_output: **ok**
- ? tests::test_cli_migrate_preview: **ok**
- ? tests::test_cli_compare: **ok**
- ? tests::test_cli_plan: **ok**
- ? tests::test_cli_apply: **ok**
- ? tests::test_cli_analyze_load: **ok**
- ? tests::test_cli_plan_show: **ok**
- ? tests::test_cli_analyze_workers_flag: **ok**
- ? tests::test_cli_measure: **ok**
- ? tests::test_cli_optimize: **ok**
- ? tests::test_cli_transform_extract: **ok**
- ? tests::test_cli_migrate_no_preview_flag: **ok**
- ? tests::test_cli_explain_load: **ok**
- ? tests::test_cli_analyze_repo: **ok**
- ? tests::test_handle_compare: **ok**
- ? tests::test_handle_analyze: **ok**
- ? tests::test_handle_measure: **ok**

**Summary**: test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- ? checker::tests::test_rejection_cycle_creation_scc: **ok**
- ? checker::tests::test_rejection_frozen_file: **ok**
- ? checker::tests::test_rejection_non_movable: **ok**
- ? checker::tests::test_rejection_contract_violation: **ok**
- ? scc::tests::test_acyclic_sccs: **ok**
- ? checker::tests::test_rejection_unresolved_dep: **ok**
- ? checker::tests::test_valid_extraction: **ok**
- ? frontier::tests::test_frontier_analyzer: **ok**
- ? plan::tests::test_format_detail: **ok**
- ? checker::tests::test_rejection_cycle_creation_include: **ok**
- ? scc::tests::test_connected_multi_scc_graph: **ok**
- ? planner::tests::test_planner_basic: **ok**
- ? scc::tests::test_cyclic_sccs: **ok**
- ? scc::tests::test_empty_input: **ok**
- ? scc::tests::test_disconnected_components: **ok**
- ? scc::tests::test_self_loop: **ok**
- ? scc::tests::test_multiple_disjoint_sccs: **ok**

**Summary**: test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- ? reduced::tests::test_query_helpers: **ok**
- ? callgraph::tests::test_callgraph_resolution_priority: **ok**
- ? callgraph::tests::test_callgraph_build_qualified_names: **ok**
- ? reduced::tests::test_model_determinism: **ok**
- ? callgraph::tests::test_callgraph_build: **ok**
- ? callgraph::tests::test_shortest_call_path: **ok**
- ? callgraph::tests::test_callers_and_callees: **ok**
- ? load::tests::test_analyze_file_load: **ok**
- ? callgraph::tests::test_recursion_cycles: **ok**
- ? load::tests::test_analyze_symbol_load: **ok**
- ? reduced::tests::test_dependency_deduplication_and_precedence: **ok**
- ? graph::tests::test_cycle_detection: **ok**
- ? load::tests::test_format_summary: **ok**
- ? load::tests::test_explain_file_load: **ok**
- ? load::tests::test_explain_symbol_load: **ok**
- ? graph::tests::test_include_graph_closure_and_explain: **ok**
- ? reduced::tests::test_symbol_identity_resolution_and_aggregation: **ok**
- ? resolver::tests::test_normalize_path: **ok**
- ? reduced::tests::test_tu_usage_aggregation: **ok**
- ? resolver::tests::test_parse_include_directives: **ok**
- ? traversal::tests::test_include_graph_traversal: **ok**
- ? types::tests::test_include_closure_normalization: **ok**
- ? types::tests::test_include_edge_serialization: **ok**
- ? resolver::tests::test_resolve_file_includes: **ok**
- ? resolver::tests::test_resolve_directive: **ok**

**Summary**: test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- ? analyzer::tests::test_forward_declaration_incomplete: **ok**
- ? analyzer::tests::test_call_site_extraction: **ok**
- ? analyzer::tests::test_by_value_member_complete: **ok**
- ? analyzer::tests::test_function_scope_dependency_edges: **ok**
- ? analyzer::tests::test_inheritance_complete: **ok**
- ? analyzer::tests::test_namespace_and_qualified_function_definition: **ok**
- ? analyzer::tests::test_function_declaration_return_incomplete: **ok**
- ? analyzer::tests::test_member_methods_in_class_bodies: **ok**
- ? cache::tests::test_incremental_stats: **ok**
- ? analyzer::tests::test_tokenizer_ignores_comments_and_strings: **ok**
- ? types::tests::test_types_and_index_result_aggregation: **ok**
- ? cache::tests::test_fingerprint_determinism: **ok**
- ? analyzer::tests::test_sizeof_complete: **ok**
- ? cache::tests::test_entry_validation: **ok**

**Summary**: test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- ? test_m0_fixture_location_and_artifacts: **ok**

**Summary**: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- ? test_m10_format_output_rolled_back: **ok**
- ? test_m10_format_output_committed: **ok**
- ? test_m10_tier_validation_structural: **ok**
- ? test_m10_rollback_identity: **ok**
- ? test_m10_apply_success_path: **ok**

**Summary**: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
- ? test_m11_metrics_attributed_to_correct_tus: **ok**
- ? test_m11_measurement_collection_succeeds: **ok**
- ? test_m11_before_after_comparison_calculated_correctly: **ok**
- ? test_m11_structural_and_measured_metrics_correlated_in_report: **ok**

**Summary**: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
- ? test_m12_incremental_analysis_cold_run: **ok**
- ? test_m12_incremental_analysis_version_mismatch: **ok**
- ? test_m12_incremental_analysis_header_modification: **ok**
- ? test_m12_incremental_analysis_warm_run: **ok**
- ? test_m12_incremental_analysis_context_invalidation: **ok**

**Summary**: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
- ? test_m13_straggler_scheduling: **ok**
- ? test_m13_failure_recovery: **ok**
- ? test_m13_memory_bounding_and_cli_output: **ok**
- ? test_m13_determinism_workers_comparison: **ok**

**Summary**: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
- ? test_m14_rollback_on_failed_validation: **ok**
- ? test_m14_end_to_end_optimization_flow: **ok**
- ? test_m14_cli_optimize_output: **ok**

**Summary**: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
- ? test_m15_graph_ui_command: **ok**

**Summary**: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s
- ? test_clang_and_default_backends_produce_consistent_results: **ok**
- ? test_clang_worker_backend_fallback_succeeds: **ok**
- ? test_clang_worker_processes_multiple_translation_units: **ok**

**Summary**: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
- ? test_e2e_standalone_html_export_traversal: **ok**
- ? test_e2e_ui_traversal_and_pathfinding: **ok**

**Summary**: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.53s
- ? test_m17_dir_nav_markers_in_export: **ok**

**Summary**: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
- ? test_m1_discovery_determinism: **ok**
- ? test_m1_repository_discovery_on_basic_fixture: **ok**

**Summary**: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
- ? test_m2_cli_handle_build_discover: **ok**
- ? test_m2_build_discovery_determinism_and_contexts: **ok**

**Summary**: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
- ? test_m3_basic_fixture_include_graph: **ok**
- ? test_m3_cli_commands: **ok**
- ? test_m3_edge_cases_duplicates_system_missing_cycles: **ok**

**Summary**: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
- ? test_m4_semantic_indexing: **ok**
- ? test_m4_semantic_critical_patterns: **ok**

**Summary**: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
- ? test_m5_reduced_model_aggregation_and_queries: **ok**
- ? test_m5_reduced_model_determinism: **ok**

**Summary**: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
- ? test_m6_explain_provenance: **ok**
- ? test_m6_file_load_analysis: **ok**
- ? test_m6_symbol_load_and_collateral_quantification: **ok**
- ? test_m6_load_artifacts: **ok**

**Summary**: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
- ? test_m7_rejection_cases: **ok**
- ? test_m7_plan_show_formatting: **ok**
- ? test_m7_plan_generation_basic: **ok**
- ? test_m7_plan_artifacts_and_determinism: **ok**

**Summary**: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
- ? detector::tests::test_source_file_kind_is_analyzable: **ok**
- ? registry::tests::empty_registry_returns_none: **ok**
- ? registry::tests::plugin_name_accessible_via_detector: **ok**
- ? registry::tests::registered_plugin_not_found_for_unknown_extension: **ok**
- ? resolver::tests::test_raw_dep_edge_fields: **ok**
- ? registry::tests::registered_plugin_is_found_for_matching_extension: **ok**
- ? rewriter::tests::test_rewrite_spec_fields: **ok**

**Summary**: test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- ? discovery::tests::test_build_discovery_result: **ok**
- ? policy::tests::test_ownership_classification: **ok**
- ? policy::tests::test_policy_precedence: **ok**
- ? policy::tests::test_generated_directory_detection: **ok**
- ? tests::test_stable_file_ids: **ok**
- ? tests::test_repository_locate: **ok**
- ? discovery::tests::test_compute_file_hash: **ok**
- ? discovery::tests::test_discover_repository_files: **ok**
- ? tests::test_path_canonicalization_and_discovery: **ok**

**Summary**: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
- ? manager::tests::test_artifact_manager_creation: **ok**
- ? manager::tests::test_save_comparison_artifacts: **ok**
- ? manager::tests::test_save_migration_artifacts: **ok**
- ? manager::tests::test_save_load_callgraph_artifacts: **ok**
- ? manager::tests::test_save_build_context_artifacts: **ok**
- ? manager::tests::test_load_migration_artifacts_roundtrip: **ok**
- ? manager::tests::test_migration_manifest_content: **ok**
- ? manager::tests::test_save_extraction_artifacts: **ok**
- ? manager::tests::test_save_include_artifacts: **ok**
- ? manager::tests::test_save_index_artifacts: **ok**
- ? manager::tests::test_save_and_load_load_artifacts: **ok**
- ? manager::tests::test_save_and_load_cache_artifacts: **ok**
- ? manager::tests::test_save_and_load_plan_artifacts: **ok**
- ? manager::tests::test_save_reduced_artifacts: **ok**
- ? manager::tests::test_save_and_load_measurement_artifacts: **ok**
- ? manager::tests::test_save_repository_artifacts: **ok**
- ? manager::tests::test_load_reduced_artifacts: **ok**
- ? manager::tests::test_save_transaction_artifacts: **ok**

**Summary**: test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
- ? engine::tests::test_format_stats_m13: **ok**
- ? queue::tests::test_queue_ordering: **ok**
- ? queue::tests::test_from_translation_units: **ok**
- ? queue::tests::test_requeue: **ok**
- ? shard::tests::test_hash_partitioner_determinism: **ok**
- ? tests::test_scheduler_init: **ok**
- ? shard::tests::test_sharded_reducer_equivalence: **ok**
- ? queue::tests::test_estimate_tu_cost: **ok**
- ? worker::tests::test_worker_pool_memory_tracking: **ok**
- ? worker::tests::test_worker_pool_failure_recovery: **ok**
- ? worker::tests::test_worker_pool_success: **ok**
- ? worker::tests::test_worker_pool_max_retries_exceeded: **ok**
- ? incremental::tests::test_header_modification: **ok**
- ? incremental::tests::test_cold_run: **ok**
- ? incremental::tests::test_warm_run: **ok**
- ? incremental::tests::test_analysis_version_mismatch: **ok**
- ? engine::tests::test_resumability_and_checkpointing: **ok**

**Summary**: test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
- ? tests::test_consumer_rewrite_format_diff: **ok**
- ? tests::test_migration_preview_empty: **ok**
- ? tests::test_include_rewriter_predecessor_includes: **ok**
- ? tests::test_include_rewriter_build_preview: **ok**
- ? tests::test_acyclic_includes: **ok**
- ? tests::test_include_rewriter_multiple_symbols_to_multiple_headers: **ok**
- ? tests::test_include_rewriter_side_effect_guard_no_symbols: **ok**
- ? tests::test_include_rewriter_single_symbol_to_single_header: **ok**
- ? tests::test_consumer_rewrite_is_noop: **ok**
- ? tests::test_migration_preview_format_diff: **ok**
- ? tests::test_extract_basic: **ok**
- ? tests::test_migration_preview_side_effect_preservation: **ok**
- ? tests::test_no_symbol_lost_invariant: **ok**
- ? tests::test_generated_header_fields: **ok**
- ? tests::test_transform_engine_init: **ok**
- ? tests::test_write_to_disk: **ok**

**Summary**: test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- ? test_build_symbol_elements_filters_unindexed: **ok**
- ? test_file_collateral_dto_serialization: **ok**
- ? test_dto_builders: **ok**
- ? test_server_collateral_endpoint: **ok**
- ? test_server_index_route: **ok**
- ? test_async_ui_state_loading: **ok**
- ? test_server_traversal_routes: **ok**
- ? test_server_routes: **ok**
- ? test_export_html: **ok**
- ? test_ui_noticeable_fading_elements: **ok**
- ? test_selected_node_edge_highlighting: **ok**
- ? test_index_html_content: **ok**
- ? test_ui_inspector_collateral_breakdown_and_path_illumination: **ok**
- ? test_ui_heat_map_coloring: **ok**
- ? test_ui_loading_overlay_and_progress: **ok**
- ? test_tour_overlay_dismissal_and_positioning: **ok**
- ? test_ui_selected_node_focal_priority: **ok**
- ? test_ui_dependency_tree_direction_preserving: **ok**
- ? test_ui_layout_and_interactive_traversal: **ok**
- ? test_ui_filter_and_degree_stepper_elements: **ok**
- ? test_onboarding_and_help_content: **ok**
- ? test_search_advanced_queries_and_dto_paths: **ok**
- ? test_ui_subgraph_filter_and_expansion_engine: **ok**
- ? test_export_html_empty_template_uses_index: **ok**

**Summary**: test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
- ? tests::test_format_output: **ok**
- ? tests::test_file_snapshot: **ok**
- ? tests::test_transaction_apply_and_rollback: **ok**

**Summary**: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
- ?? crates\lang\src\registry.rs - registry::LanguageRegistry (line 31): **ignored**

**Summary**: test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

**Summary**: test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s