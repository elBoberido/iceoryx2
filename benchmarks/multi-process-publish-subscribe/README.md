# Measuring latencies

https://btorpey.github.io/blog/2014/02/18/clock-sources-in-linux/

# Slow benchmark

- `clock_gettime` is slow
- VDSO is needed to get the time without a context switch
- it seems VDSO is deactivated
- `dmesg` prints `turning off TSC clock`
- `/sys/devices/system/clocksource/clocksource0/available_clocksource`
- `/sys/devices/system/clocksource/clocksource0/current_clocksource`
- clock needs to be `tsc` for the `VDSO` to work without fallback (or `pvclock/hvclock`, but those are only relevant to VM guests).
