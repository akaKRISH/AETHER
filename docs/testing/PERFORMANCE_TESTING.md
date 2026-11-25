# Performance testing

Record CPU model, cores, OS/build, build mode, thread cap, warmup, input size and repetition count. Measure startup, evaluation and replay separately. Report latency distributions and peak RSS without combining mixed hardware. Time process termination on controlled supported runners. For R2 metadata reads use five local clients, record query and data distribution, and test the proposed p95 target in T24.

Run a pilot to calibrate repetitions and noise before setting a regression threshold. Compare immutable workload definitions. [Test plan](TEST_PLAN.md)
