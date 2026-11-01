# 0238 — Step-mode acks wait for and print the next action

- **Status:** accepted (2026-11-01, T-3716); amends decision 0169 and SPEC-0027 section 4 (step mode)
- **Context:** in step mode (decision 0169) every GitHub write of a merger run is one action that
  the merger session performs and acks. `make merge-train-ack` only wrote the ack file and
  returned, so the session slept a fixed time and grepped the run's log for the next action. The
  09:35 merger run on 2026-11-01 (session cse_017265sSFvUhKEZ51G4oXa4m) spent about 45 s per write
  this way: an ack, `sleep 40`, then a grep. Merging one 5-PR batch took about 20 writes, or 11
  minutes of the session's 40-minute window. The same run missed an eject action printed at 09:57
  and performed it only at 10:07. It had ended its turn with "still waiting on CI".
- **Decision:** `make merge-train-ack` (`merge_step.py ack --next 90`) waits up to 90 s, polling
  once a second, for the run's next pending action and prints it in the run's own format.
  `make merge-train-next` (`merge_step.py next --wait 300`) blocks until an action is pending and
  prints it; it exits 3 when none appears in time. Both waits refuse a value that is not a finite,
  non-negative number of seconds, and neither prints an action whose ack is still on disk (the run
  clears it on its next poll), so an acked action is never shown again as the next one. The run and the ack file protocol are
  unchanged: the run still waits at most `ACK_TIMEOUT` for each ack and still reads every write
  back. The merger prompt in docs/process/routines.md says to use both instead of sleeping.
- **Consequences:** a write costs one GitHub call plus the run's read-back instead of a fixed
  40 s sleep, and a waiting session sees the next action within a second. The live merger
  routine follows the new text only once the owner updates it; until then the old prompt's
  sleeps still work, because the ack only returns later.
