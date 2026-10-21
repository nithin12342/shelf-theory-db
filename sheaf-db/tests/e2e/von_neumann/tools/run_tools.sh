# Rebuild and run the two standalone projection tools.
# They are NOT cargo targets: they measure the cost of work the engine does,
# so they live here as plain rustc programs.
#
# Before you begin: a rustc toolchain on the PATH.
# Verify: the output shows HASHBUILD, HASHPROBE, SEQSCAN, ANSWERONLY,
# FIBER_TOP10_SCAN and FIBER_TOP10_PRESORTED lines.
set -e
cd "$(dirname "$0")"

rustc -O cpu_floor.rs -o cpu_floor.exe
./cpu_floor.exe

rustc -O q2_projection.rs -o q2_projection.exe
./q2_projection.exe

echo "PROJECTION TOOLS: COMPLETE"
