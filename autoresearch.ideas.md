# Autoresearch Ideas

## Deferred Optimizations

- **u128 fast path for add**: When both operands fit in 96 bits and have the same scale, could use u128 addition directly instead of lo64/hi split. Would eliminate the carry check logic.
- **SIMD for Dec64 operations**: ARM NEON could potentially do the 96-bit add in fewer instructions using 128-bit vector ops.
- **Specialize fold operations**: The fold benchmarks dominate. Could we provide a specialized batch-add API that amortizes overhead? (Probably out of scope since we can't change benchmarks.)
- **Reduce is_zero checks**: The is_zero check loads 3 fields and ORs them. For the common non-zero case, this is wasted work. Could use a "definitely not zero" flag bit in the flags field (requires API change).
- **Optimize unaligned_add for common scale differences**: For small scale differences (1-9), could use a single multiplication instead of the general loop.
- **Division: reduce branches in div32**: The div32 method has multiple branches for different mantissa sizes. Could restructure to reduce branching.
- **rem_impl algorithmic improvements**: The rem_10k fold mostly processes zeros after the first iteration. Could add early-exit for repeated zero-on-zero.
