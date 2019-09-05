# GDB

`to-wait` returns controlt o GDB

## Startup

* `gdb_main`
* `gdb_init`
* `initialize_all_files`
* `_initialize_arch_tdep`
* `_initialize_arch_remote`
* `initialize_current_architecture`
* `arch_gdbarch_init`
* `interp_pre_command_loop`

## Target

* `interp_pre_command_loop`
- `target_open`
- `start_remote` (Remote Target Only)
- `wait_for_inferio`
- `target_wait`
- `handle_inferior_event`
- `normal_stop`

## handle_inferior_event

* `handle_inferior_event`
- `read_pc_pid`
- `gdbarch_register_type`
- `target_fetch_registers`
- `watchpoints_triggered`
- `target_stopped_by_watchpoint`

## load

Dynamically load file into the running program and record it's symbols

* `interp_pre_command_loop`
- `target_load`
- `target_xfer_partial`

## break

* `interp_pre_command_loop`
- `break_command`
- `parse_breakpoint_sals`
- `gdbarch_skip_prologue`

## run

* `interp_pre_command_loop`
- `run_command`
- `target_create_inferior`
- `target_find_description`
- `proceed`
- `gdbarch_single_step_through_delay`
- `target_insert_breakpoint`
- `target_resume`
- `wait_for_inferior`
- `normal_stop`

## Continue

* `interp_pre_command_loop`
* `continue_command`
- `proceed`
- `wait_for_inferior`
- `target_wait`
- `handle_inferior_event`
- `target_resume`
- `normal_stop`

# Resources

https://www.embecosm.com/appnotes/ean3/html/ch02s11s05.html
