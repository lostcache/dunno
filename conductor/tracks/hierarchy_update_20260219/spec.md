# Track Specification: Hierarchy Update (Submodule and File)

## Overview
Expand the structural knowledge graph representation to better reflect code repositories. Previously it was `Project -> Module -> Task`. We are restructuring to explicitly capture files and optional sub-module groupings under the sequence: `Project -> Module -> Submodule(optional) -> File`.

## Objectives
1. **Submodule Node:** Add ability to group resources tightly into a `Submodule` that belongs to a standard `Module`.
2. **File Node:** Add structural mapping of specific files indicating which module (and submodule if applicable) they exist within.
3. **Data Completeness:** The graph remains explicitly structured to favor deterministic traversal over fuzzy natural language matching.

## User Stories
- **As an agent**, I need distinct resource handlers for submodules and files so that I can link bugs, tasks, or style rules specifically to the granular files instead of dumping everything at the module boundary.
- **As a human developer**, I can replicate standard source code layouts mapping modules and internal files more comfortably with my coding hierarchy.

## Constraints
- A Submodule must be strictly linked to a Module.
- A File must be directly linked to a Module, and optionally to a Submodule.
