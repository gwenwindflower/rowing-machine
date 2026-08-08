# Themes — Spec

Themes adapt the baseline ecommerce simulation to a cohesive setting without changing its relational behavior. A theme owns its vocabulary, schema mappings, static catalog, and customer naming data. Bundled themes remain fully inspectable within the repository and add no runtime dependencies.

## Requirements

### Customer names

- **th-R001 Theme ownership.** Every customer name MUST be generated exclusively from the selected theme's declared name formats and components.
- **th-R002 Auditable inputs.** Bundled name generation MUST use native code and checked-in, reviewed data; it MUST NOT execute or load third-party generator code or data at runtime.
- **th-R003 Name determinism.** The same seed and theme configuration MUST produce the same customer name sequence.
- **th-R004 RNG isolation.** Customer names MUST use a dedicated PCG stream derived from `--seed` so name configuration changes do not perturb any non-name simulation field.
- **th-R005 Run-wide uniqueness.** A run MUST NOT repeat a full customer name until the selected theme's valid name combinations are exhausted, regardless of market boundaries.
- **th-R006 Default-scale capacity.** Every bundled theme MUST provide enough valid combinations to name the default run's addressable customer pool without repetition.
- **th-R007 Traceable composition.** Every generated name MUST be traceable to a declared theme format and reviewed whole-token components; the generator MUST NOT synthesize character sequences within a component.
- **th-R008 Actionable validation.** A theme with an invalid name format, unknown component reference, empty required component pool, or no valid combinations MUST be rejected before simulation with an error naming the theme and invalid field.
