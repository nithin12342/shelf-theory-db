# Von Neumann physical-limits E2E gate (SPEC-057 / METHOD-052).
# Idempotent: re-running overwrites the expected output.
# Measures this host's real memory-hierarchy and storage limits, derives the
# physical floors from published constants, and asserts the physics-invariant
# ordering gates. The report (VON_NEUMANN_LIMITS.md) quotes these numbers.
set -e
cd "$(dirname "$0")/../../.."
mkdir -p tests/e2e/von_neumann/expected

cargo run --manifest-path Cargo.toml -p sheaf-engine --bin physical_limits_probe --release \
  | tee "tests/e2e/von_neumann/expected/physical_limits.txt"

# The probe must complete.
grep -q "PHYSICAL-LIMITS-PROBE: COMPLETE" "tests/e2e/von_neumann/expected/physical_limits.txt"

# Physics-invariant gates (ordering / identity, not performance targets).
grep -q "gate_hierarchy_ordered=PASS" "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "gate_dram_slower_than_l3=PASS" "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "gate_dram_above_ddr4_cas_floor=PASS" "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "gate_fsync_floor_exists=PASS" "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "gate_read_bw_significant=PASS" "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "gate_dram_slower_than_l1=PASS" "tests/e2e/von_neumann/expected/physical_limits.txt"

# Clock cross-checks: the 8-add vs 4-add chain ratio must be 2.0, and the
# 4x-imul vs 4-add ratio must be 3.0. The second validates the ABSOLUTE
# cycle count, which is what every "FLOP/cycle" claim rests on.
grep -q "C_clock_crosscheck_ok=PASS" "tests/e2e/von_neumann/expected/physical_limits.txt"

# The derived floors must be present (the report cites them by name).
grep -q "E_landauer_J_per_bit=" "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "E_dram_floor_ns " "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "E_machine_balance_1t_flop_per_byte=" "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "E_dram_only_estimate_ns=" "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "E_hdd_grain_nm=" "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "E_stlb_reach_MiB_at_1536_entries=" "tests/e2e/von_neumann/expected/physical_limits.txt"

# The same-window roofline pair (the machine balance must be measured in
# the same thermal window as the bandwidth it is divided by).
grep -q "G_same_window_machine_balance_flop_per_byte=" "tests/e2e/von_neumann/expected/physical_limits.txt"
grep -q "G_kernel_intensity_flop_per_byte d=4" "tests/e2e/von_neumann/expected/physical_limits.txt"

echo "VON NEUMANN LIMITS E2E: PASS"
